use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::core::models::eve::Blueprint;
use crate::core::models::industry::{IndustryPlan, IndustrySettings, IndustryStructure};
use crate::core::models::sde::{BlueprintActivity, BlueprintSearchMatch, SdeBlueprintEntry, SdeIndex};
use crate::core::optimizer::bonuses::{
    blueprint_job_duration_secs,
    blueprint_me_multiplier,
    compute_material_qty,
};
use crate::core::optimizer::dag::collect_reachable_types;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintPreviewRequest {
    pub character_id: Option<u32>,
    pub blueprint: Blueprint,
    #[serde(default)]
    pub location_name: Option<String>,
    #[serde(default)]
    pub owned_bp_me: HashMap<u32, u8>,
    #[serde(default)]
    pub include_all_locations: bool,
    #[serde(default)]
    pub force_asset_refresh: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreviewNodeStatus {
    InStock,
    Craftable,
    Partial,
    Missing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMaterialNode {
    pub type_id: u32,
    pub name: String,
    pub required_qty: u64,
    pub available_qty: u64,
    pub missing_qty: u64,
    pub status: PreviewNodeStatus,
    pub is_craftable: bool,
    pub activity: Option<BlueprintActivity>,
    pub blueprint_type_id: Option<u32>,
    pub runs: u64,
    pub output_qty: u64,
    pub me_level: u8,
    pub haulable_stock: Vec<HaulableStockEntry>,
    pub children: Vec<PreviewMaterialNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSummaryItem {
    pub type_id: u32,
    pub name: String,
    pub required_qty: u64,
    pub available_qty: u64,
    pub missing_qty: u64,
    pub haulable_stock: Vec<HaulableStockEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintPreviewResult {
    pub blueprint: Blueprint,
    pub info: BlueprintSearchMatch,
    pub output_qty: u64,
    pub job_duration_secs: u64,
    pub location_name: String,
    pub matched_structure_name: Option<String>,
    pub include_all_locations: bool,
    pub can_craft_all: bool,
    pub has_sub_crafts: bool,
    pub materials: Vec<PreviewMaterialNode>,
    pub missing_summary: Vec<PreviewSummaryItem>,
    pub available_summary: Vec<PreviewSummaryItem>,
    pub reserved_by_plans_count: usize,
    pub warnings: Vec<String>,
}

struct PreviewEvalContext<'a> {
    sde: &'a SdeIndex,
    structure: Option<&'a IndustryStructure>,
    owned_bp_me: &'a HashMap<u32, u8>,
    haulable: &'a HashMap<u32, Vec<HaulableStockEntry>>,
}

impl<'a> PreviewEvalContext<'a> {
    fn type_name(&self, type_id: u32) -> String {
        self.sde
            .types
            .get(&type_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| format!("Type {}", type_id))
    }

    fn sub_bp_me(&self, bp: &SdeBlueprintEntry) -> u8 {
        if bp.activity == BlueprintActivity::Reaction {
            return 0;
        }
        self.owned_bp_me
            .get(&bp.blueprint_type_id)
            .copied()
            .unwrap_or(0)
            .min(10)
    }
}

struct PreviewTreeEvaluation {
    materials: Vec<PreviewMaterialNode>,
    missing_summary: Vec<PreviewSummaryItem>,
    available_summary: Vec<PreviewSummaryItem>,
    can_craft_all: bool,
    has_sub_crafts: bool,
}

fn evaluate_preview_material(
    ctx: &PreviewEvalContext<'_>,
    type_id: u32,
    required_qty: u64,
    available_pool: &mut HashMap<u32, u64>,
    visiting: &mut HashSet<u32>,
) -> PreviewMaterialNode {
    let available = available_pool.get(&type_id).copied().unwrap_or(0);
    let from_stock = available.min(required_qty);
    if from_stock > 0
        && let Some(slot) = available_pool.get_mut(&type_id)
    {
        *slot -= from_stock;
    }

    let missing_qty = required_qty - from_stock;
    let haulable_stock = ctx.haulable.get(&type_id).cloned().unwrap_or_default();

    let sub_bp = ctx
        .sde
        .product_to_blueprint
        .get(&type_id)
        .and_then(|bp_id| ctx.sde.blueprints.get(bp_id));

    if missing_qty == 0 {
        return PreviewMaterialNode {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            available_qty: from_stock,
            missing_qty: 0,
            status: PreviewNodeStatus::InStock,
            is_craftable: sub_bp.is_some(),
            activity: sub_bp.map(|b| b.activity),
            blueprint_type_id: sub_bp.map(|b| b.blueprint_type_id),
            runs: 0,
            output_qty: 0,
            me_level: 0,
            haulable_stock,
            children: Vec::new(),
        };
    }

    let can_recurse = sub_bp.is_some() && !visiting.contains(&type_id);
    if !can_recurse {
        let status = if from_stock > 0 {
            PreviewNodeStatus::Partial
        } else {
            PreviewNodeStatus::Missing
        };
        return PreviewMaterialNode {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            available_qty: from_stock,
            missing_qty,
            status,
            is_craftable: false,
            activity: None,
            blueprint_type_id: None,
            runs: 0,
            output_qty: 0,
            me_level: 0,
            haulable_stock,
            children: Vec::new(),
        };
    }

    let bp = sub_bp.unwrap();
    visiting.insert(type_id);

    let out_per_run = bp.product_quantity.max(1) as u64;
    let runs = missing_qty.div_ceil(out_per_run);
    let output_qty = runs * out_per_run;
    let me_level = ctx.sub_bp_me(bp);
    let me_mult = blueprint_me_multiplier(bp, me_level, ctx.structure, ctx.sde);

    let mut children = Vec::with_capacity(bp.materials.len());
    for mat in &bp.materials {
        let child_req = compute_material_qty(mat.quantity, runs, me_mult);
        let child = evaluate_preview_material(
            ctx,
            mat.material_type_id,
            child_req,
            available_pool,
            visiting,
        );
        children.push(child);
    }

    visiting.remove(&type_id);

    let all_children_covered = !children.is_empty()
        && children
            .iter()
            .all(|c| matches!(c.status, PreviewNodeStatus::InStock | PreviewNodeStatus::Craftable));
    let any_child_progress = children
        .iter()
        .any(|c| !matches!(c.status, PreviewNodeStatus::Missing));

    let status = if all_children_covered {
        PreviewNodeStatus::Craftable
    } else if from_stock > 0 || any_child_progress {
        PreviewNodeStatus::Partial
    } else {
        PreviewNodeStatus::Missing
    };

    PreviewMaterialNode {
        type_id,
        name: ctx.type_name(type_id),
        required_qty,
        available_qty: from_stock,
        missing_qty,
        status,
        is_craftable: true,
        activity: Some(bp.activity),
        blueprint_type_id: Some(bp.blueprint_type_id),
        runs,
        output_qty,
        me_level,
        haulable_stock,
        children,
    }
}

fn accumulate_preview_summaries(
    node: &PreviewMaterialNode,
    missing_totals: &mut HashMap<u32, (u64, u64, u64)>,
    available_totals: &mut HashMap<u32, (u64, u64)>,
) {
    if node.available_qty > 0 {
        let entry = available_totals.entry(node.type_id).or_insert((0, 0));
        entry.0 += node.required_qty;
        entry.1 += node.available_qty;
    }

    if node.children.is_empty() {
        if node.missing_qty > 0 {
            let entry = missing_totals.entry(node.type_id).or_insert((0, 0, 0));
            entry.0 += node.required_qty;
            entry.1 += node.available_qty;
            entry.2 += node.missing_qty;
        }
    } else {
        for child in &node.children {
            accumulate_preview_summaries(child, missing_totals, available_totals);
        }
    }
}

fn build_pool_from_inventory(
    stock_inventory: &StockInventory,
    location_id: u64,
    include_all_locations: bool,
) -> (HashMap<u32, u64>, HashMap<u32, Vec<HaulableStockEntry>>) {
    if include_all_locations {
        let mut combined: HashMap<u32, u64> = HashMap::new();
        for items in stock_inventory.by_location.values() {
            for (&type_id, &qty) in items {
                *combined.entry(type_id).or_insert(0) += qty;
            }
        }
        (combined, HashMap::new())
    } else {
        let local = stock_inventory
            .by_location
            .get(&location_id)
            .cloned()
            .unwrap_or_default();
        let haulable = haulable_map_from(
            &stock_inventory.by_location,
            &stock_inventory.location_names,
            Some(location_id),
        );
        (local, haulable)
    }
}

fn evaluate_blueprint_tree(
    ctx: &PreviewEvalContext<'_>,
    bp: &SdeBlueprintEntry,
    runs: u64,
    me_level: u8,
    available_pool: &mut HashMap<u32, u64>,
) -> PreviewTreeEvaluation {
    let me_mult = blueprint_me_multiplier(bp, me_level, ctx.structure, ctx.sde);
    let mut visiting = HashSet::new();
    visiting.insert(bp.product_type_id);

    let mut materials = Vec::with_capacity(bp.materials.len());
    for mat in &bp.materials {
        let req = compute_material_qty(mat.quantity, runs, me_mult);
        let node = evaluate_preview_material(
            ctx,
            mat.material_type_id,
            req,
            available_pool,
            &mut visiting,
        );
        materials.push(node);
    }

    let mut missing_totals: HashMap<u32, (u64, u64, u64)> = HashMap::new();
    let mut available_totals: HashMap<u32, (u64, u64)> = HashMap::new();
    for node in &materials {
        accumulate_preview_summaries(node, &mut missing_totals, &mut available_totals);
    }

    let mut missing_summary: Vec<PreviewSummaryItem> = missing_totals
        .into_iter()
        .map(|(type_id, (required_qty, available_qty, missing_qty))| PreviewSummaryItem {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            available_qty,
            missing_qty,
            haulable_stock: ctx.haulable.get(&type_id).cloned().unwrap_or_default(),
        })
        .collect();
    missing_summary.sort_by(|a, b| {
        b.missing_qty
            .cmp(&a.missing_qty)
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut available_summary: Vec<PreviewSummaryItem> = available_totals
        .into_iter()
        .map(|(type_id, (required_qty, available_qty))| PreviewSummaryItem {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            available_qty,
            missing_qty: required_qty.saturating_sub(available_qty),
            haulable_stock: Vec::new(),
        })
        .collect();
    available_summary.sort_by(|a, b| {
        b.available_qty
            .cmp(&a.available_qty)
            .then_with(|| a.name.cmp(&b.name))
    });

    let can_craft_all = materials
        .iter()
        .all(|m| matches!(m.status, PreviewNodeStatus::InStock | PreviewNodeStatus::Craftable));
    let has_sub_crafts = materials.iter().any(|m| !m.children.is_empty());

    PreviewTreeEvaluation {
        materials,
        missing_summary,
        available_summary,
        can_craft_all,
        has_sub_crafts,
    }
}

pub async fn run_blueprint_preview(
    app: &AppHandle,
    state: &AppState,
    req: BlueprintPreviewRequest,
) -> Result<BlueprintPreviewResult, String> {
    let sde = ensure_sde_loaded(app, state).await?;
    let settings: IndustrySettings = read_json_or_default(app, SETTINGS_FILE).unwrap_or_default();
    let saved_plans: Vec<IndustryPlan> = read_json_or_default(app, PLANS_FILE).unwrap_or_default();
    let mut warnings = Vec::new();

    let mut blueprint = req.blueprint;
    let bp = sde
        .blueprints
        .get(&blueprint.type_id)
        .ok_or_else(|| format!("Blueprint {} not found in local SDE.", blueprint.type_id))?;

    let runs = blueprint.runs.max(1) as u64;
    let out_per_run = bp.product_quantity.max(1) as u64;
    let output_qty = runs.saturating_mul(out_per_run);

    let (me_level, te_level) = if bp.activity == BlueprintActivity::Reaction {
        (0u8, 0u8)
    } else {
        (
            blueprint.material_efficiency.min(10) as u8,
            blueprint.time_efficiency.min(20) as u8,
        )
    };
    blueprint.runs = runs.min(i32::MAX as u64) as i32;
    blueprint.material_efficiency = me_level as u32;
    blueprint.time_efficiency = te_level as u32;

    let location_id = blueprint.location_id;

    let skills: HashMap<u32, u8> = req
        .character_id
        .and_then(|cid| settings.character_skills.get(&cid))
        .map(|c| c.skills.clone())
        .unwrap_or_default();

    let matched_structure = settings
        .structures
        .iter()
        .find(|s| s.location_id == Some(location_id));

    // Aggregate reservations from all validated plans
    let mut reserved_by_plans: HashMap<(u64, u32), u64> = HashMap::new();
    for plan in &saved_plans {
        if !plan.validated {
            continue;
        }
        for res in &plan.reserved_resources {
            *reserved_by_plans
                .entry((res.location_id, res.type_id))
                .or_insert(0) += res.quantity;
        }
    }

    // Collect all reachable material types for this blueprint
    let mut relevant_asset_types: HashSet<u32> = HashSet::new();
    for mat in &bp.materials {
        collect_reachable_types(&sde, mat.material_type_id, &mut relevant_asset_types);
    }

    let stock_inventory = if let Some(char_id) = req.character_id {
        match fetch_cached_character_assets(app, state, char_id, req.force_asset_refresh).await {
            Ok(assets) => {
                build_stock_inventory(
                    app,
                    state,
                    char_id,
                    &assets,
                    &settings.structures,
                    &relevant_asset_types,
                    &reserved_by_plans,
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

    let reserved_by_plans_count = stock_inventory.reserved_elsewhere.len();
    let (mut available_pool, haulable_map) =
        build_pool_from_inventory(&stock_inventory, location_id, req.include_all_locations);

    let ctx = PreviewEvalContext {
        sde: &sde,
        structure: matched_structure,
        owned_bp_me: &req.owned_bp_me,
        haulable: &haulable_map,
    };

    let eval = evaluate_blueprint_tree(&ctx, bp, runs, me_level, &mut available_pool);

    let job_duration_secs =
        blueprint_job_duration_secs(bp, te_level, runs, matched_structure, &skills, &sde);

    let product_name = ctx.type_name(bp.product_type_id);
    let blueprint_name = sde
        .types
        .get(&bp.blueprint_type_id)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| match bp.activity {
            BlueprintActivity::Reaction => format!("{} Reaction Formula", product_name),
            BlueprintActivity::Manufacturing => format!("{} Blueprint", product_name),
        });

    let info = BlueprintSearchMatch {
        blueprint_type_id: bp.blueprint_type_id,
        blueprint_name,
        product_type_id: bp.product_type_id,
        product_name,
        activity: bp.activity,
    };

    let location_name = req
        .location_name
        .filter(|n| !n.trim().is_empty())
        .or_else(|| stock_inventory.location_names.get(&location_id).cloned())
        .or_else(|| matched_structure.map(|s| s.name.clone()))
        .unwrap_or_else(|| format!("Location {}", location_id));

    Ok(BlueprintPreviewResult {
        blueprint,
        info,
        output_qty,
        job_duration_secs,
        location_name,
        matched_structure_name: matched_structure.map(|s| s.name.clone()),
        include_all_locations: req.include_all_locations,
        can_craft_all: eval.can_craft_all,
        has_sub_crafts: eval.has_sub_crafts,
        materials: eval.materials,
        missing_summary: eval.missing_summary,
        available_summary: eval.available_summary,
        reserved_by_plans_count,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::industry::RawMaterial;
    use crate::core::models::sde::SdeTypeInfo;

    fn make_type(name: &str) -> SdeTypeInfo {
        SdeTypeInfo {
            name: name.to_string(),
            group_id: 1,
            portion_size: 1,
            volume: 1.0,
            meta_group_id: None,
        }
    }

    fn make_test_sde() -> SdeIndex {
        let mut sde = SdeIndex::default();

        sde.types.insert(100, make_type("Test Ship Blueprint"));
        sde.types.insert(101, make_type("Test Ship"));
        sde.types.insert(200, make_type("Component A Blueprint"));
        sde.types.insert(201, make_type("Component A"));
        sde.types.insert(210, make_type("Component B Blueprint"));
        sde.types.insert(202, make_type("Component B"));
        sde.types.insert(34, make_type("Tritanium"));
        sde.types.insert(35, make_type("Pyerite"));

        sde.blueprints.insert(
            100,
            SdeBlueprintEntry {
                blueprint_type_id: 100,
                activity: BlueprintActivity::Manufacturing,
                product_type_id: 101,
                product_quantity: 1,
                time: 3600,
                materials: vec![
                    RawMaterial { material_type_id: 201, quantity: 2 },
                    RawMaterial { material_type_id: 202, quantity: 2 },
                ],
            },
        );
        sde.blueprints.insert(
            200,
            SdeBlueprintEntry {
                blueprint_type_id: 200,
                activity: BlueprintActivity::Manufacturing,
                product_type_id: 201,
                product_quantity: 1,
                time: 600,
                materials: vec![RawMaterial { material_type_id: 34, quantity: 10 }],
            },
        );
        sde.blueprints.insert(
            210,
            SdeBlueprintEntry {
                blueprint_type_id: 210,
                activity: BlueprintActivity::Manufacturing,
                product_type_id: 202,
                product_quantity: 1,
                time: 600,
                materials: vec![
                    RawMaterial { material_type_id: 34, quantity: 10 },
                    RawMaterial { material_type_id: 35, quantity: 5 },
                ],
            },
        );

        sde.product_to_blueprint.insert(101, 100);
        sde.product_to_blueprint.insert(201, 200);
        sde.product_to_blueprint.insert(202, 210);

        sde
    }

    #[test]
    fn direct_stock_marks_in_stock_without_subcraft() {
        let sde = make_test_sde();
        let bp = sde.blueprints.get(&100).unwrap();
        let mut pool = HashMap::from([(201, 2), (202, 2)]);
        let haulable = HashMap::new();
        let owned_me = HashMap::new();
        let ctx = PreviewEvalContext {
            sde: &sde,
            structure: None,
            owned_bp_me: &owned_me,
            haulable: &haulable,
        };

        let eval = evaluate_blueprint_tree(&ctx, bp, 1, 0, &mut pool);

        assert!(eval.can_craft_all);
        assert!(!eval.has_sub_crafts);
        assert!(eval.missing_summary.is_empty());
        assert_eq!(eval.available_summary.len(), 2);
        assert!(eval.materials.iter().all(|m| m.status == PreviewNodeStatus::InStock));
        assert!(eval.materials.iter().all(|m| m.children.is_empty()));
    }

    #[test]
    fn recursive_subcraft_marks_craftable_when_raw_stock_covers_all() {
        let sde = make_test_sde();
        let bp = sde.blueprints.get(&100).unwrap();
        let mut pool = HashMap::from([(34, 40), (35, 10)]);
        let haulable = HashMap::new();
        let owned_me = HashMap::new();
        let ctx = PreviewEvalContext {
            sde: &sde,
            structure: None,
            owned_bp_me: &owned_me,
            haulable: &haulable,
        };

        let eval = evaluate_blueprint_tree(&ctx, bp, 1, 0, &mut pool);

        assert!(eval.can_craft_all);
        assert!(eval.has_sub_crafts);
        assert!(eval.missing_summary.is_empty());
        assert_eq!(eval.materials[0].status, PreviewNodeStatus::Craftable);
        assert_eq!(eval.materials[1].status, PreviewNodeStatus::Craftable);
        assert_eq!(eval.materials[0].children.len(), 1);
        assert_eq!(eval.materials[1].children.len(), 2);
    }

    #[test]
    fn shared_raw_stock_is_not_double_counted_across_siblings() {
        let sde = make_test_sde();
        let bp = sde.blueprints.get(&100).unwrap();
        let mut pool = HashMap::from([(201, 1), (34, 25), (35, 10)]);
        let haulable = HashMap::new();
        let owned_me = HashMap::new();
        let ctx = PreviewEvalContext {
            sde: &sde,
            structure: None,
            owned_bp_me: &owned_me,
            haulable: &haulable,
        };

        let eval = evaluate_blueprint_tree(&ctx, bp, 1, 0, &mut pool);

        assert!(!eval.can_craft_all);
        assert!(eval.has_sub_crafts);
        assert_eq!(eval.materials[0].status, PreviewNodeStatus::Craftable);
        assert_eq!(eval.materials[0].runs, 1);
        assert_eq!(eval.materials[1].status, PreviewNodeStatus::Partial);
        assert_eq!(eval.materials[1].runs, 2);
        assert_eq!(eval.missing_summary.len(), 1);
        assert_eq!(eval.missing_summary[0].type_id, 34);
        assert_eq!(eval.missing_summary[0].missing_qty, 5);
    }

    #[test]
    fn local_vs_all_locations_pooling_and_haulable() {
        let mut inv = StockInventory::default();
        inv.by_location.insert(60003760, HashMap::from([(34, 15)]));
        inv.by_location.insert(60008494, HashMap::from([(34, 25)]));
        inv.location_names.insert(60003760, "Jita".to_string());
        inv.location_names.insert(60008494, "Amarr".to_string());

        let (local_pool, haulable) = build_pool_from_inventory(&inv, 60003760, false);
        assert_eq!(local_pool.get(&34), Some(&15));
        assert_eq!(haulable.get(&34).unwrap().len(), 1);
        assert_eq!(haulable.get(&34).unwrap()[0].location_name, "Amarr");
        assert_eq!(haulable.get(&34).unwrap()[0].quantity, 25);

        let (all_pool, all_haulable) = build_pool_from_inventory(&inv, 60003760, true);
        assert_eq!(all_pool.get(&34), Some(&40));
        assert!(all_haulable.is_empty());
    }

    #[test]
    fn cycle_guard_prevents_infinite_recursion() {
        let mut sde = SdeIndex::default();
        sde.types.insert(1, make_type("Loop A BP"));
        sde.types.insert(2, make_type("Loop A"));
        sde.types.insert(3, make_type("Loop B BP"));
        sde.types.insert(4, make_type("Loop B"));

        sde.blueprints.insert(
            1,
            SdeBlueprintEntry {
                blueprint_type_id: 1,
                activity: BlueprintActivity::Manufacturing,
                product_type_id: 2,
                product_quantity: 1,
                time: 60,
                materials: vec![RawMaterial { material_type_id: 4, quantity: 1 }],
            },
        );
        sde.blueprints.insert(
            3,
            SdeBlueprintEntry {
                blueprint_type_id: 3,
                activity: BlueprintActivity::Manufacturing,
                product_type_id: 4,
                product_quantity: 1,
                time: 60,
                materials: vec![RawMaterial { material_type_id: 2, quantity: 1 }],
            },
        );
        sde.product_to_blueprint.insert(2, 1);
        sde.product_to_blueprint.insert(4, 3);

        let bp = sde.blueprints.get(&1).unwrap();
        let mut pool = HashMap::new();
        let haulable = HashMap::new();
        let owned_me = HashMap::new();
        let ctx = PreviewEvalContext {
            sde: &sde,
            structure: None,
            owned_bp_me: &owned_me,
            haulable: &haulable,
        };

        let eval = evaluate_blueprint_tree(&ctx, bp, 1, 0, &mut pool);

        assert!(!eval.can_craft_all);
        assert_eq!(eval.materials.len(), 1);
        assert_eq!(eval.materials[0].children.len(), 1);
        assert!(eval.materials[0].children[0].children.is_empty());
        assert_eq!(eval.missing_summary.len(), 1);
        assert_eq!(eval.missing_summary[0].type_id, 2);
    }
}
