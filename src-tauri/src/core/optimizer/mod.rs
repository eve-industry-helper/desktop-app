pub mod bonuses;
pub mod dag;
pub mod job_queue;
pub mod reprocess_lp;
pub mod stock;

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::commands::market::{get_esi_adjusted_prices_internal, get_market_prices_internal};
use crate::core::models::industry::{
    BuildConfig,
    IndustryPlan,
    IndustrySettings,
    IndustryStructure,
    PlanEntry,
    PlanEntryKind,
    ReservedResource,
};
use crate::core::models::sde::SdeIndex;
use crate::core::optimizer::bonuses::{
    effective_sell_unit_revenue,
    is_base_mineral,
    BASE_MINERALS,
};
use crate::core::optimizer::dag::{
    collect_reachable_types,
    evaluate_material_node,
    evaluate_root_blueprint,
    EvalContext,
    NodeDecision,
    OptimizedNode,
};
pub use crate::core::optimizer::job_queue::JobSummary;
use crate::core::optimizer::job_queue::build_ordered_job_queue;
use crate::core::optimizer::reprocess_lp::{
    find_candidate_compressed_ores,
    solve_mineral_reprocessing,
    ReprocessInput,
    ReprocessingPlan,
};
use crate::core::optimizer::stock::{
    build_stock_inventory,
    fetch_cached_character_assets,
    haulable_map_from,
    HaulableStockEntry,
    StockInventory,
};
use crate::core::sde_service::ensure_sde_loaded;
use crate::core::state::AppState;
use crate::core::storage::read_json_or_default;

const SETTINGS_FILE: &str = "industry_settings.json";
const PLANS_FILE: &str = "plans.json";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OptimizeScope {
    #[default]
    Entry,
    Plan,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptimizePlanRequest {
    pub character_id: Option<u32>,
    #[serde(default)]
    pub plan_id: Option<String>,
    #[serde(default)]
    pub scope: OptimizeScope,
    #[serde(default)]
    pub plan_config: BuildConfig,
    pub entries: Vec<PlanEntry>,
    #[serde(default)]
    pub bp_me_overrides: HashMap<u32, u8>,
    #[serde(default)]
    pub bp_te_overrides: HashMap<u32, u8>,
    #[serde(default)]
    pub force_price_refresh: bool,
    #[serde(default)]
    pub force_asset_refresh: bool,
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
    pub used_local_resources: Vec<ReservedResource>,
    pub reserved_by_other_plans_count: usize,
    pub warnings: Vec<String>,
}

fn sde_type_name(sde: &SdeIndex, type_id: u32) -> String {
    sde.types
        .get(&type_id)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| format!("Type {}", type_id))
}

fn resolve_structure<'a>(
    structures: &'a [IndustryStructure],
    id: Option<&str>,
) -> Option<&'a IndustryStructure> {
    let target_id = id?;
    structures.iter().find(|s| s.id == target_id)
}

fn accumulate_tree(
    node: &OptimizedNode,
    buy_totals: &mut HashMap<u32, (u64, f64)>,
    stock_totals: &mut HashMap<u32, (u64, f64)>,
) {
    if node.from_local_stock > 0 {
        let entry = stock_totals
            .entry(node.type_id)
            .or_insert((0, node.unit_buy_price));
        entry.0 += node.from_local_stock;
    }

    match node.decision {
        NodeDecision::Buy if node.net_qty > 0 => {
            let entry = buy_totals
                .entry(node.type_id)
                .or_insert((0, node.unit_buy_price));
            entry.0 += node.net_qty;
        }
        NodeDecision::Craft => {
            for child in &node.children {
                accumulate_tree(child, buy_totals, stock_totals);
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
    let saved_plans: Vec<IndustryPlan> = read_json_or_default(app, PLANS_FILE).unwrap_or_default();
    let mut warnings = Vec::new();

    let character_skill_cache = req
        .character_id
        .and_then(|cid| settings.character_skills.get(&cid));
    let skills: HashMap<u32, u8> = character_skill_cache
        .map(|c| c.skills.clone())
        .unwrap_or_default();
    let reprocessing_implant_bonus = character_skill_cache
        .map(|c| c.reprocessing_implant_bonus)
        .unwrap_or(0.0);

    let reprocessing_enabled = match req.scope {
        OptimizeScope::Entry => req
            .entries
            .first()
            .map(|e| e.config.enable_reprocessing_lp)
            .unwrap_or(req.plan_config.enable_reprocessing_lp),
        OptimizeScope::Plan => {
            if req.entries.is_empty() {
                req.plan_config.enable_reprocessing_lp
            } else {
                req.entries.iter().any(|e| e.config.enable_reprocessing_lp)
            }
        }
    };

    let credit_surplus = match req.scope {
        OptimizeScope::Entry => req
            .entries
            .first()
            .map(|e| e.config.credit_surplus_minerals)
            .unwrap_or(req.plan_config.credit_surplus_minerals),
        OptimizeScope::Plan => {
            if req.entries.is_empty() {
                req.plan_config.credit_surplus_minerals
            } else {
                req.entries.iter().any(|e| e.config.credit_surplus_minerals)
            }
        }
    };

    let reprocessing_structure_id = match req.scope {
        OptimizeScope::Entry => req
            .entries
            .first()
            .and_then(|e| e.config.reprocessing_structure_id.as_deref())
            .or(req.plan_config.reprocessing_structure_id.as_deref()),
        OptimizeScope::Plan => req
            .entries
            .iter()
            .find_map(|e| e.config.reprocessing_structure_id.as_deref())
            .or(req.plan_config.reprocessing_structure_id.as_deref()),
    };

    // Aggregate reservations from all OTHER validated plans
    let mut reserved_by_other_plans: HashMap<(u64, u32), u64> = HashMap::new();
    for plan in &saved_plans {
        if !plan.validated || req.plan_id.as_deref() == Some(plan.id.as_str()) {
            continue;
        }
        for res in &plan.reserved_resources {
            *reserved_by_other_plans
                .entry((res.location_id, res.type_id))
                .or_insert(0) += res.quantity;
        }
    }

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

    if reprocessing_enabled {
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

    // 3. Build per-location Assets inventory (minus other validated plans' reservations)
    let any_uses_assets = req.entries.iter().any(|e| e.config.use_character_assets);

    let stock_inventory = if any_uses_assets
        && let Some(char_id) = req.character_id
    {
        match fetch_cached_character_assets(app, state, char_id, req.force_asset_refresh).await {
            Ok(assets) => {
                build_stock_inventory(
                    app,
                    state,
                    char_id,
                    &assets,
                    &settings.structures,
                    &relevant_asset_types,
                    &reserved_by_other_plans,
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

    let reserved_by_other_plans_count = stock_inventory.reserved_elsewhere.len();
    if reserved_by_other_plans_count > 0 {
        warnings.push(format!(
            "{} material type(s) had available stock reduced by other validated plans.",
            reserved_by_other_plans_count
        ));
    }

    let mut remaining_by_loc = stock_inventory.by_location.clone();
    let mut consumed_by_loc: HashMap<(u64, u32), u64> = HashMap::new();
    let mut unlinked_warned: HashSet<String> = HashSet::new();

    // 4. Evaluate Buy-vs-Craft DAG per entry using each entry's own config
    let mut nodes = Vec::new();
    let mut total_revenue_net = 0.0;

    for entry in &req.entries {
        let entry_cfg = &entry.config;
        let entry_multiplier = entry_cfg.quantity.max(1);

        let mfg_structure = resolve_structure(
            &settings.structures,
            entry_cfg.manufacturing_structure_id.as_deref(),
        );
        let rxn_structure = resolve_structure(
            &settings.structures,
            entry_cfg
                .reaction_structure_id
                .as_deref()
                .or(entry_cfg.reprocessing_structure_id.as_deref()),
        );

        let use_assets = entry_cfg.use_character_assets;
        let selected_loc = mfg_structure.and_then(|s| s.location_id);

        if use_assets
            && let Some(s) = mfg_structure
            && selected_loc.is_none()
            && unlinked_warned.insert(s.name.clone())
        {
            warnings.push(format!(
                "Structure '{}' has no linked EVE location ID; local stock deduction skipped (all stock shown as haulable).",
                s.name
            ));
        }

        let haulable_for_entry = haulable_map_from(
            &remaining_by_loc,
            &stock_inventory.location_names,
            if use_assets { selected_loc } else { None },
        );

        let mut entry_local = match (use_assets, selected_loc) {
            (true, Some(loc_id)) => remaining_by_loc.get(&loc_id).cloned().unwrap_or_default(),
            _ => HashMap::new(),
        };
        let before_local = entry_local.clone();

        let ctx = EvalContext {
            sde: &sde,
            buy_prices: &buy_prices,
            esi_prices: &esi_prices,
            material_basis: &pricing.material_price_basis,
            skills: &skills,
            mfg_structure,
            rxn_structure,
            overrides: &entry_cfg.overrides,
            bp_me_overrides: &req.bp_me_overrides,
            bp_te_overrides: &req.bp_te_overrides,
            default_me: entry_cfg.default_me,
            default_te: entry_cfg.default_te,
            haulable: &haulable_for_entry,
        };

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
            let target_qty = base_target_qty.saturating_mul(entry_multiplier);

            if let Some(node) =
                evaluate_root_blueprint(&ctx, bp.blueprint_type_id, target_qty, &mut entry_local)
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
            let target_qty = (item.quantity.max(1) as u64).saturating_mul(entry_multiplier);
            let node = evaluate_material_node(
                &ctx,
                item.item_type_id,
                target_qty,
                &mut entry_local,
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

        if use_assets
            && let Some(loc_id) = selected_loc
        {
            for (&type_id, &before_qty) in &before_local {
                let after_qty = entry_local.get(&type_id).copied().unwrap_or(0);
                let used = before_qty.saturating_sub(after_qty);
                if used > 0 {
                    *consumed_by_loc.entry((loc_id, type_id)).or_insert(0) += used;
                }
            }
            entry_local.retain(|_, q| *q > 0);
            if entry_local.is_empty() {
                remaining_by_loc.remove(&loc_id);
            } else {
                remaining_by_loc.insert(loc_id, entry_local);
            }
        }
    }

    // 5. Consolidate Shopping List, Job Queue, and Local Stock Savings
    let mut buy_totals: HashMap<u32, (u64, f64)> = HashMap::new();
    let mut stock_totals: HashMap<u32, (u64, f64)> = HashMap::new();

    for node in &nodes {
        accumulate_tree(node, &mut buy_totals, &mut stock_totals);
    }

    let job_queue = build_ordered_job_queue(&nodes);

    let local_stock_value_saved: f64 = stock_totals
        .values()
        .map(|(qty, price)| (*qty as f64) * price)
        .sum();

    let total_job_cost: f64 = job_queue.iter().map(|j| j.job_cost).sum();
    let total_job_duration_secs: u64 = job_queue.iter().map(|j| j.duration_secs).sum();
    let total_buy_materials_cost: f64 = buy_totals
        .values()
        .map(|(qty, price)| (*qty as f64) * price)
        .sum();

    // 6. Run Mineral-Basket Reprocessing LP on consolidated base minerals
    let reprocessing_structure = resolve_structure(&settings.structures, reprocessing_structure_id);

    let mineral_basket: HashMap<u32, u64> = buy_totals
        .iter()
        .filter(|(id, (qty, _))| is_base_mineral(**id) && *qty > 0)
        .map(|(&id, &(qty, _))| (id, qty))
        .collect();

    let reprocessing = if reprocessing_enabled && !mineral_basket.is_empty() {
        let lp_plan = solve_mineral_reprocessing(&ReprocessInput {
            sde: &sde,
            needed_minerals: &mineral_basket,
            buy_prices: &buy_prices,
            sell_prices: &sell_prices,
            material_basis: &pricing.material_price_basis,
            product_basis: &pricing.product_price_basis,
            structure: reprocessing_structure,
            skills: &skills,
            reprocessing_implant_bonus,
            credit_surplus,
        });
        Some(lp_plan)
    } else {
        None
    };

    // Any stock remaining in `remaining_by_loc` for items still on the shopping list is haulable
    let final_haulable = haulable_map_from(&remaining_by_loc, &stock_inventory.location_names, None);

    let mut shopping_list: Vec<ShoppingItem> = Vec::new();
    let reprocessing_active = reprocessing
        .as_ref()
        .map(|r| r.used_reprocessing)
        .unwrap_or(false);

    for (&type_id, &(qty, unit_price)) in &buy_totals {
        if reprocessing_active && is_base_mineral(type_id) {
            continue;
        }
        let from_local = stock_totals.get(&type_id).map(|(q, _)| *q).unwrap_or(0);
        shopping_list.push(ShoppingItem {
            type_id,
            name: sde_type_name(&sde, type_id),
            quantity: qty,
            unit_price,
            total_cost: unit_price * (qty as f64),
            from_local_stock: from_local,
            haulable_stock: final_haulable.get(&type_id).cloned().unwrap_or_default(),
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
                haulable_stock: final_haulable.get(&ore.type_id).cloned().unwrap_or_default(),
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
                haulable_stock: final_haulable.get(&min.type_id).cloned().unwrap_or_default(),
            });
        }
    }

    shopping_list.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));

    let mut used_local_resources: Vec<ReservedResource> = consumed_by_loc
        .into_iter()
        .map(|((location_id, type_id), quantity)| ReservedResource {
            location_id,
            location_name: stock_inventory
                .location_names
                .get(&location_id)
                .cloned()
                .unwrap_or_else(|| format!("Location {}", location_id)),
            type_id,
            name: sde_type_name(&sde, type_id),
            quantity,
        })
        .collect();
    used_local_resources.sort_by_key(|r| std::cmp::Reverse(r.quantity));

    let total_build_cost_before_reproc = total_buy_materials_cost + total_job_cost;
    let reprocessing_savings = reprocessing.as_ref().map(|r| r.savings).unwrap_or(0.0);
    let total_build_cost_final = (total_build_cost_before_reproc - reprocessing_savings).max(0.0);
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
        used_local_resources,
        reserved_by_other_plans_count,
        warnings,
    })
}