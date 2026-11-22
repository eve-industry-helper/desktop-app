pub mod bonuses;
pub mod dag;
pub mod reprocess_lp;
pub mod stock;

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::commands::market::{get_esi_adjusted_prices_internal, get_market_prices_internal};
use crate::core::models::industry::{
    IndustrySettings,
    IndustryStructure,
    PlanEntry,
    PlanEntryKind,
    StructureType,
};
use crate::core::models::sde::BlueprintActivity;
use crate::core::optimizer::bonuses::{
    effective_sell_unit_revenue,
    is_base_mineral,
    BASE_MINERALS,
};
use crate::core::optimizer::dag::{
    collect_reachable_types,
    evaluate_material_node,
    evaluate_root_blueprint,
    DecisionOverride,
    EvalContext,
    NodeDecision,
    OptimizedNode,
};
use crate::core::optimizer::reprocess_lp::{
    find_candidate_compressed_ores,
    solve_mineral_reprocessing,
    ReprocessInput,
    ReprocessingPlan,
};
use crate::core::optimizer::stock::{
    build_stock_inventory,
    fetch_cached_character_assets,
    HaulableStockEntry,
    StockInventory,
};
use crate::core::sde_service::ensure_sde_loaded;
use crate::core::state::AppState;
use crate::core::storage::read_json_or_default;

const SETTINGS_FILE: &str = "industry_settings.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptimizePlanRequest {
    pub character_id: Option<u32>,
    pub entries: Vec<PlanEntry>,
    pub manufacturing_structure_id: Option<String>,
    pub reaction_structure_id: Option<String>,
    pub reprocessing_structure_id: Option<String>,
    #[serde(default)]
    pub overrides: HashMap<u32, DecisionOverride>,
    #[serde(default)]
    pub bp_me_overrides: HashMap<u32, u8>,
    #[serde(default)]
    pub bp_te_overrides: HashMap<u32, u8>,
    #[serde(default = "default_me")]
    pub default_me: u8,
    #[serde(default = "default_te")]
    pub default_te: u8,
    #[serde(default = "default_one")]
    pub quantity_multiplier: u64,
    #[serde(default = "default_true")]
    pub use_character_assets: bool,
    #[serde(default = "default_true")]
    pub enable_reprocessing_lp: bool,
    #[serde(default)]
    pub credit_surplus_minerals: bool,
    #[serde(default)]
    pub force_price_refresh: bool,
    #[serde(default)]
    pub force_asset_refresh: bool,
}

fn default_me() -> u8 {
    10
}
fn default_te() -> u8 {
    20
}
fn default_true() -> bool {
    true
}
fn default_one() -> u64 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingItem {
    pub type_id: u32,
    pub name: String,
    pub quantity: u64,
    pub unit_price: f64,
    pub total_cost: f64,
    pub from_local_stock: u64,
    pub haulable_stock: Vec<HaulableStockEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub blueprint_type_id: u32,
    pub product_type_id: u32,
    pub product_name: String,
    pub activity: BlueprintActivity,
    pub runs: u64,
    pub output_qty: u64,
    pub job_cost: f64,
    pub duration_secs: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationResult {
    pub total_revenue_net: f64,
    pub total_buy_materials_cost: f64,
    pub total_job_cost: f64,
    pub total_build_cost_before_reproc: f64,
    pub total_build_cost_final: f64,
    pub net_profit: f64,
    pub profit_margin_percent: f64,
    pub total_job_duration_secs: u64,
    pub local_stock_value_saved: f64,
    pub nodes: Vec<OptimizedNode>,
    pub shopping_list: Vec<ShoppingItem>,
    pub job_queue: Vec<JobSummary>,
    pub reprocessing: Option<ReprocessingPlan>,
    pub warnings: Vec<String>,
}

fn resolve_structure<'a>(
    structures: &'a [IndustryStructure],
    id: Option<&str>,
    fallback_refinery: bool,
) -> Option<&'a IndustryStructure> {
    if let Some(target_id) = id
        && let Some(found) = structures.iter().find(|s| s.id == target_id)
    {
        return Some(found);
    }
    if fallback_refinery {
        return structures.iter().find(|s| {
            matches!(
                s.structure_type,
                StructureType::Tatara | StructureType::Athanor
            )
        });
    }
    None
}

fn accumulate_tree(
    node: &OptimizedNode,
    buy_totals: &mut HashMap<u32, (u64, f64)>,
    stock_totals: &mut HashMap<u32, (u64, f64)>,
    jobs: &mut HashMap<u32, JobSummary>,
) {
    if node.from_local_stock > 0 {
        let entry = stock_totals.entry(node.type_id).or_insert((0, node.unit_buy_price));
        entry.0 += node.from_local_stock;
    }

    match node.decision {
        NodeDecision::Buy if node.net_qty > 0 => {
            let entry = buy_totals.entry(node.type_id).or_insert((0, node.unit_buy_price));
            entry.0 += node.net_qty;
        }
        NodeDecision::Craft => {
            if let (Some(bp_id), Some(activity)) = (node.blueprint_type_id, node.activity) {
                let job = jobs.entry(bp_id).or_insert_with(|| JobSummary {
                    blueprint_type_id: bp_id,
                    product_type_id: node.type_id,
                    product_name: node.name.clone(),
                    activity,
                    runs: 0,
                    output_qty: 0,
                    job_cost: 0.0,
                    duration_secs: 0,
                });
                job.runs += node.runs;
                job.output_qty += node.output_qty;
                job.job_cost += node.job_cost;
                job.duration_secs += node.job_duration_secs;
            }
            for child in &node.children {
                accumulate_tree(child, buy_totals, stock_totals, jobs);
            }
        }
        _ => {}
    }
}

pub async fn run_optimizer(
    app: &AppHandle,
    state: &AppState,
    req: OptimizePlanRequest,
) -> Result<OptimizationResult, String> {
    let sde = ensure_sde_loaded(app, state).await?;
    let settings: IndustrySettings = read_json_or_default(app, SETTINGS_FILE).unwrap_or_default();
    let mut warnings = Vec::new();

    let skills: HashMap<u32, u8> = req
        .character_id
        .and_then(|cid| settings.character_skills.get(&cid))
        .map(|c| c.skills.clone())
        .unwrap_or_default();

    let mfg_structure = resolve_structure(
        &settings.structures,
        req.manufacturing_structure_id.as_deref(),
        false,
    );
    let rxn_structure = resolve_structure(
        &settings.structures,
        req.reaction_structure_id.as_deref(),
        true,
    );
    let reproc_structure = resolve_structure(
        &settings.structures,
        req.reprocessing_structure_id.as_deref(),
        true,
    );

    // 1. Walk plan entries to collect all reachable type_ids for pricing and stock filtering
    let mut buy_type_ids: HashSet<u32> = HashSet::new();
    let mut sell_type_ids: HashSet<u32> = HashSet::new();
    let mut relevant_asset_types: HashSet<u32> = HashSet::new();

    for entry in &req.entries {
        for bp in &entry.blueprints {
            sell_type_ids.insert(bp.item_type_id);
            collect_reachable_types(&sde, bp.item_type_id, &mut buy_type_ids);
            if let Some(sde_bp) = sde.blueprints.get(&bp.blueprint_type_id) {
                for m in &sde_bp.materials {
                    collect_reachable_types(&sde, m.material_type_id, &mut buy_type_ids);
                }
            }
        }
        for item in &entry.buy_only_items {
            buy_type_ids.insert(item.item_type_id);
            sell_type_ids.insert(item.item_type_id);
        }
    }

    relevant_asset_types.extend(&buy_type_ids);

    // Add candidate compressed ores for LP if any base mineral appears in the tree
    if req.enable_reprocessing_lp {
        let needed_base: HashSet<u32> = buy_type_ids
            .iter()
            .copied()
            .filter(|id| is_base_mineral(*id))
            .collect();
        let ore_ids = find_candidate_compressed_ores(&sde, &needed_base);
        buy_type_ids.extend(ore_ids);
        sell_type_ids.extend(BASE_MINERALS);
    }

    // 2. Fetch single-provider market prices + ESI adjusted prices concurrently
    let pricing = &settings.pricing;
    let janice_key = (!pricing.janice_api_key.is_empty()).then(|| pricing.janice_api_key.clone());

    let (buy_prices, sell_prices, esi_prices) = if pricing.buy_region_id == pricing.sell_region_id {
        let mut combined: HashSet<u32> = buy_type_ids.clone();
        combined.extend(&sell_type_ids);

        let (hub, esi) = tokio::try_join!(
            get_market_prices_internal(
                state,
                pricing.provider,
                pricing.buy_region_id,
                combined.into_iter().collect(),
                janice_key,
                req.force_price_refresh,
            ),
            get_esi_adjusted_prices_internal(state, req.force_price_refresh),
        )?;
        (hub.clone(), hub, esi)
    } else {
        tokio::try_join!(
            get_market_prices_internal(
                state,
                pricing.provider,
                pricing.buy_region_id,
                buy_type_ids.into_iter().collect(),
                janice_key.clone(),
                req.force_price_refresh,
            ),
            get_market_prices_internal(
                state,
                pricing.provider,
                pricing.sell_region_id,
                sell_type_ids.into_iter().collect(),
                janice_key,
                req.force_price_refresh,
            ),
            get_esi_adjusted_prices_internal(state, req.force_price_refresh),
        )?
    };

    // 3. Build Assets inventory (local stock in selected structure vs haulable elsewhere)
    let stock_inventory = if req.use_character_assets
        && let Some(char_id) = req.character_id
    {
        match fetch_cached_character_assets(app, state, char_id, req.force_asset_refresh).await {
            Ok(assets) => {
                let selected_loc = mfg_structure.and_then(|s| s.location_id);
                if mfg_structure.is_some() && selected_loc.is_none() {
                    warnings.push(
                        "Selected structure has no linked EVE location ID; local stock deduction skipped (all stock shown as haulable)."
                            .to_string(),
                    );
                }
                build_stock_inventory(
                    app,
                    state,
                    char_id,
                    &assets,
                    selected_loc,
                    &settings.structures,
                    &relevant_asset_types,
                )
                    .await
            }
            Err(e) => {
                warnings.push(format!("Could not load character assets: {}", e));
                StockInventory::default()
            }
        }
    } else {
        StockInventory::default()
    };

    let mut remaining_local = stock_inventory.local.clone();

    let ctx = EvalContext {
        sde: &sde,
        buy_prices: &buy_prices,
        esi_prices: &esi_prices,
        material_basis: &pricing.material_price_basis,
        skills: &skills,
        mfg_structure,
        rxn_structure,
        overrides: &req.overrides,
        bp_me_overrides: &req.bp_me_overrides,
        bp_te_overrides: &req.bp_te_overrides,
        default_me: req.default_me,
        default_te: req.default_te,
        haulable: &stock_inventory.haulable,
    };

    // 4. Evaluate Buy-vs-Craft DAG for every entry
    let multiplier = req.quantity_multiplier.max(1);
    let mut nodes = Vec::new();
    let mut total_revenue_net = 0.0;

    for entry in &req.entries {
        for bp in &entry.blueprints {
            let out_per_run = sde
                .blueprints
                .get(&bp.blueprint_type_id)
                .map(|b| b.product_quantity.max(1) as u64)
                .unwrap_or(1);

            let base_target_qty = match entry.kind {
                PlanEntryKind::Fit => bp.runs.max(1) as u64,
                PlanEntryKind::Blueprint | PlanEntryKind::Reaction => {
                    (bp.runs.max(1) as u64) * out_per_run
                }
            };
            let target_qty = base_target_qty.saturating_mul(multiplier);

            if let Some(node) =
                evaluate_root_blueprint(&ctx, bp.blueprint_type_id, target_qty, &mut remaining_local)
            {
                let unit_rev = effective_sell_unit_revenue(
                    sell_prices.get(&node.type_id),
                    &pricing.product_price_basis,
                    &skills,
                );
                total_revenue_net += unit_rev * (node.required_qty as f64);
                nodes.push(node);
            } else {
                warnings.push(format!(
                    "Blueprint {} ({}) not found in local SDE.",
                    bp.blueprint_name, bp.blueprint_type_id
                ));
            }
        }

        for item in &entry.buy_only_items {
            let mut visiting = HashSet::new();
            let target_qty = (item.quantity.max(1) as u64).saturating_mul(multiplier);
            let node = evaluate_material_node(
                &ctx,
                item.item_type_id,
                target_qty,
                &mut remaining_local,
                &mut visiting,
            );
            let unit_rev = effective_sell_unit_revenue(
                sell_prices.get(&item.item_type_id),
                &pricing.product_price_basis,
                &skills,
            );
            total_revenue_net += unit_rev * (target_qty as f64);
            nodes.push(node);
        }
    }

    // 5. Consolidate Shopping List, Job Queue, and Local Stock Savings
    let mut buy_totals: HashMap<u32, (u64, f64)> = HashMap::new();
    let mut stock_totals: HashMap<u32, (u64, f64)> = HashMap::new();
    let mut jobs_map: HashMap<u32, JobSummary> = HashMap::new();

    for node in &nodes {
        accumulate_tree(node, &mut buy_totals, &mut stock_totals, &mut jobs_map);
    }

    let local_stock_value_saved: f64 = stock_totals
        .values()
        .map(|(qty, price)| (*qty as f64) * price)
        .sum();

    let total_job_cost: f64 = jobs_map.values().map(|j| j.job_cost).sum();
    let total_job_duration_secs: u64 = jobs_map.values().map(|j| j.duration_secs).sum();
    let total_buy_materials_cost: f64 = buy_totals
        .values()
        .map(|(qty, price)| (*qty as f64) * price)
        .sum();

    // 6. Run Mineral-Basket Reprocessing LP on consolidated base minerals
    let mineral_basket: HashMap<u32, u64> = buy_totals
        .iter()
        .filter(|(id, (qty, _))| is_base_mineral(**id) && *qty > 0)
        .map(|(&id, &(qty, _))| (id, qty))
        .collect();

    let reprocessing = if req.enable_reprocessing_lp && !mineral_basket.is_empty() {
        let lp_plan = solve_mineral_reprocessing(&ReprocessInput {
            sde: &sde,
            needed_minerals: &mineral_basket,
            buy_prices: &buy_prices,
            sell_prices: &sell_prices,
            material_basis: &pricing.material_price_basis,
            product_basis: &pricing.product_price_basis,
            structure: reproc_structure,
            skills: &skills,
            credit_surplus: req.credit_surplus_minerals,
        });
        Some(lp_plan)
    } else {
        None
    };

    // Build final shopping list (if LP used reprocessing, swap raw minerals for ores + shortfall)
    let mut shopping_list: Vec<ShoppingItem> = Vec::new();
    let reproc_active = reprocessing
        .as_ref()
        .map(|r| r.used_reprocessing)
        .unwrap_or(false);

    for (&type_id, &(qty, unit_price)) in &buy_totals {
        if reproc_active && is_base_mineral(type_id) {
            continue;
        }
        let from_local = stock_totals.get(&type_id).map(|(q, _)| *q).unwrap_or(0);
        shopping_list.push(ShoppingItem {
            type_id,
            name: ctx.type_name(type_id),
            quantity: qty,
            unit_price,
            total_cost: unit_price * (qty as f64),
            from_local_stock: from_local,
            haulable_stock: stock_inventory.haulable.get(&type_id).cloned().unwrap_or_default(),
        });
    }

    if let Some(ref lp) = reprocessing
        && lp.used_reprocessing
    {
        for ore in &lp.ores {
            shopping_list.push(ShoppingItem {
                type_id: ore.type_id,
                name: ore.name.clone(),
                quantity: ore.units_to_buy,
                unit_price: ore.unit_price,
                total_cost: ore.total_cost,
                from_local_stock: 0,
                haulable_stock: stock_inventory
                    .haulable
                    .get(&ore.type_id)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
        for min in &lp.direct_minerals {
            let from_local = stock_totals.get(&min.type_id).map(|(q, _)| *q).unwrap_or(0);
            shopping_list.push(ShoppingItem {
                type_id: min.type_id,
                name: min.name.clone(),
                quantity: min.quantity,
                unit_price: min.unit_price,
                total_cost: min.total_cost,
                from_local_stock: from_local,
                haulable_stock: stock_inventory
                    .haulable
                    .get(&min.type_id)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
    }

    shopping_list.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));

    let mut job_queue: Vec<JobSummary> = jobs_map.into_values().collect();
    job_queue.sort_by_key(|a| std::cmp::Reverse(a.duration_secs));

    let total_build_cost_before_reproc = total_buy_materials_cost + total_job_cost;
    let reproc_savings = reprocessing
        .as_ref()
        .map(|r| r.savings)
        .unwrap_or(0.0);
    let total_build_cost_final = (total_build_cost_before_reproc - reproc_savings).max(0.0);
    let net_profit = total_revenue_net - total_build_cost_final;
    let profit_margin_percent = if total_build_cost_final > 0.0 {
        (net_profit / total_build_cost_final) * 100.0
    } else {
        0.0
    };

    Ok(OptimizationResult {
        total_revenue_net,
        total_buy_materials_cost,
        total_job_cost,
        total_build_cost_before_reproc,
        total_build_cost_final,
        net_profit,
        profit_margin_percent,
        total_job_duration_secs,
        local_stock_value_saved,
        nodes,
        shopping_list,
        job_queue,
        reprocessing,
        warnings,
    })
}