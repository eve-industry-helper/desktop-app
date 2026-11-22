use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::core::models::industry::{IndustryStructure, PriceBasis};
use crate::core::models::market::{EsiMarketPrice, MarketPrice};
use crate::core::models::sde::{BlueprintActivity, SdeBlueprintEntry, SdeIndex};
use crate::core::optimizer::bonuses::{
    blueprint_job_cost,
    blueprint_job_duration_secs,
    blueprint_me_multiplier,
    compute_material_qty,
    effective_buy_unit_price,
};
use crate::core::optimizer::stock::HaulableStockEntry;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DecisionOverride {
    #[default]
    Auto,
    Buy,
    Craft,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeDecision {
    Buy,
    Craft,
    InStock,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptimizedNode {
    pub type_id: u32,
    pub name: String,
    pub required_qty: u64,
    pub from_local_stock: u64,
    pub net_qty: u64,
    pub decision: NodeDecision,
    pub is_craftable: bool,
    pub activity: Option<BlueprintActivity>,
    pub blueprint_type_id: Option<u32>,
    pub runs: u64,
    pub output_qty: u64,
    pub me_level: u8,
    pub te_level: u8,
    pub unit_buy_price: f64,
    pub buy_cost: f64,
    pub craft_cost: Option<f64>,
    pub job_cost: f64,
    pub job_duration_secs: u64,
    pub chosen_cost: f64,
    pub haulable_stock: Vec<HaulableStockEntry>,
    pub children: Vec<OptimizedNode>,
}

pub struct EvalContext<'a> {
    pub sde: &'a SdeIndex,
    pub buy_prices: &'a HashMap<u32, MarketPrice>,
    pub esi_prices: &'a HashMap<u32, EsiMarketPrice>,
    pub material_basis: &'a PriceBasis,
    pub skills: &'a HashMap<u32, u8>,
    pub mfg_structure: Option<&'a IndustryStructure>,
    pub rxn_structure: Option<&'a IndustryStructure>,
    pub overrides: &'a HashMap<u32, DecisionOverride>,
    pub bp_me_overrides: &'a HashMap<u32, u8>,
    pub bp_te_overrides: &'a HashMap<u32, u8>,
    pub default_me: u8,
    pub default_te: u8,
    pub haulable: &'a HashMap<u32, Vec<HaulableStockEntry>>,
}

impl<'a> EvalContext<'a> {
    fn structure_for(&self, activity: BlueprintActivity) -> Option<&'a IndustryStructure> {
        match activity {
            BlueprintActivity::Manufacturing => self.mfg_structure,
            BlueprintActivity::Reaction => self.rxn_structure.or(self.mfg_structure),
        }
    }

    fn me_te_for(&self, bp: &SdeBlueprintEntry) -> (u8, u8) {
        if bp.activity == BlueprintActivity::Reaction {
            return (0, 0);
        }
        let is_t2 = self
            .sde
            .types
            .get(&bp.product_type_id)
            .and_then(|t| t.meta_group_id)
            .map(|mg| mg == 2)
            .unwrap_or(false);

        let default_me = if is_t2 { self.default_me.min(2) } else { self.default_me.min(10) };
        let default_te = if is_t2 { self.default_te.min(4) } else { self.default_te.min(20) };

        let me = self.bp_me_overrides.get(&bp.blueprint_type_id).copied().unwrap_or(default_me);
        let te = self.bp_te_overrides.get(&bp.blueprint_type_id).copied().unwrap_or(default_te);
        (me, te)
    }

    pub fn type_name(&self, type_id: u32) -> String {
        self.sde
            .types
            .get(&type_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| format!("Type {}", type_id))
    }
}

/// Collects every reachable `type_id` in the production tree of `type_id`.
pub fn collect_reachable_types(sde: &SdeIndex, type_id: u32, out: &mut HashSet<u32>) {
    if !out.insert(type_id) {
        return;
    }
    if let Some(&bp_id) = sde.product_to_blueprint.get(&type_id)
        && let Some(bp) = sde.blueprints.get(&bp_id)
    {
        for mat in &bp.materials {
            collect_reachable_types(sde, mat.material_type_id, out);
        }
    }
}

/// Evaluates a root blueprint job directly (forcing Craft unless overridden to Buy).
/// Evaluates a root blueprint target for `target_qty` finished units.
pub fn evaluate_root_blueprint(
    ctx: &EvalContext<'_>,
    bp_id: u32,
    target_qty: u64,
    local_stock: &mut HashMap<u32, u64>,
) -> Option<OptimizedNode> {
    let bp = ctx.sde.blueprints.get(&bp_id)?;
    let out_per_run = bp.product_quantity.max(1) as u64;
    let required_qty = target_qty.max(1);
    let runs = required_qty.div_ceil(out_per_run);
    let output_qty = runs * out_per_run;

    let mut visiting = HashSet::new();
    visiting.insert(bp.product_type_id);

    let (me, te) = ctx.me_te_for(bp);
    let structure = ctx.structure_for(bp.activity);
    let me_mult = blueprint_me_multiplier(bp, me, structure, ctx.sde);
    let job_cost = blueprint_job_cost(bp, runs, structure, ctx.esi_prices);
    let job_duration_secs = blueprint_job_duration_secs(bp, te, runs, structure, ctx.skills, ctx.sde);

    let unit_buy = effective_buy_unit_price(
        ctx.buy_prices.get(&bp.product_type_id),
        ctx.material_basis,
        ctx.skills,
    );
    let buy_cost = unit_buy * (required_qty as f64);

    let mut craft_stock = local_stock.clone();
    let mut children = Vec::with_capacity(bp.materials.len());
    let mut materials_cost = 0.0;

    for mat in &bp.materials {
        let req = compute_material_qty(mat.quantity, runs, me_mult);
        let child = evaluate_material_node(ctx, mat.material_type_id, req, &mut craft_stock, &mut visiting);
        materials_cost += child.chosen_cost;
        children.push(child);
    }

    let craft_cost = materials_cost + job_cost;
    let override_choice = ctx
        .overrides
        .get(&bp.product_type_id)
        .copied()
        .unwrap_or(DecisionOverride::Auto);

    let decision = match override_choice {
        DecisionOverride::Buy => NodeDecision::Buy,
        DecisionOverride::Craft => NodeDecision::Craft,
        DecisionOverride::Auto => {
            if unit_buy > 0.0 && buy_cost < craft_cost {
                NodeDecision::Buy
            } else {
                NodeDecision::Craft
            }
        }
    };

    let (chosen_cost, final_children, final_job_cost, final_duration, final_runs, final_output) =
        match decision {
            NodeDecision::Craft => {
                *local_stock = craft_stock;
                (craft_cost, children, job_cost, job_duration_secs, runs, output_qty)
            }
            NodeDecision::Buy | NodeDecision::InStock => (buy_cost, Vec::new(), 0.0, 0, 0, 0),
        };

    Some(OptimizedNode {
        type_id: bp.product_type_id,
        name: ctx.type_name(bp.product_type_id),
        required_qty,
        from_local_stock: 0,
        net_qty: required_qty,
        decision,
        is_craftable: true,
        activity: Some(bp.activity),
        blueprint_type_id: Some(bp.blueprint_type_id),
        runs: final_runs,
        output_qty: final_output,
        me_level: me,
        te_level: te,
        unit_buy_price: unit_buy,
        buy_cost,
        craft_cost: Some(craft_cost),
        job_cost: final_job_cost,
        job_duration_secs: final_duration,
        chosen_cost,
        haulable_stock: ctx.haulable.get(&bp.product_type_id).cloned().unwrap_or_default(),
        children: final_children,
    })
}

/// Evaluates an intermediate or raw material node with local stock deduction and Buy-vs-Craft.
pub fn evaluate_material_node(
    ctx: &EvalContext<'_>,
    type_id: u32,
    required_qty: u64,
    local_stock: &mut HashMap<u32, u64>,
    visiting: &mut HashSet<u32>,
) -> OptimizedNode {
    let available = local_stock.get(&type_id).copied().unwrap_or(0);
    let from_local_stock = available.min(required_qty);
    if from_local_stock > 0
        && let Some(slot) = local_stock.get_mut(&type_id)
    {
        *slot -= from_local_stock;
    }

    let net_qty = required_qty - from_local_stock;
    let unit_buy = effective_buy_unit_price(ctx.buy_prices.get(&type_id), ctx.material_basis, ctx.skills);
    let buy_cost = unit_buy * (net_qty as f64);
    let haulable_stock = ctx.haulable.get(&type_id).cloned().unwrap_or_default();

    let bp = ctx
        .sde
        .product_to_blueprint
        .get(&type_id)
        .and_then(|bp_id| ctx.sde.blueprints.get(bp_id));

    if net_qty == 0 {
        return OptimizedNode {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            from_local_stock,
            net_qty: 0,
            decision: NodeDecision::InStock,
            is_craftable: bp.is_some(),
            activity: bp.map(|b| b.activity),
            blueprint_type_id: bp.map(|b| b.blueprint_type_id),
            runs: 0,
            output_qty: 0,
            me_level: 0,
            te_level: 0,
            unit_buy_price: unit_buy,
            buy_cost: 0.0,
            craft_cost: None,
            job_cost: 0.0,
            job_duration_secs: 0,
            chosen_cost: 0.0,
            haulable_stock,
            children: Vec::new(),
        };
    }

    let can_recurse = bp.is_some() && !visiting.contains(&type_id);
    if !can_recurse {
        return OptimizedNode {
            type_id,
            name: ctx.type_name(type_id),
            required_qty,
            from_local_stock,
            net_qty,
            decision: NodeDecision::Buy,
            is_craftable: false,
            activity: None,
            blueprint_type_id: None,
            runs: 0,
            output_qty: 0,
            me_level: 0,
            te_level: 0,
            unit_buy_price: unit_buy,
            buy_cost,
            craft_cost: None,
            job_cost: 0.0,
            job_duration_secs: 0,
            chosen_cost: buy_cost,
            haulable_stock,
            children: Vec::new(),
        };
    }

    let bp = bp.unwrap();
    visiting.insert(type_id);

    let out_per_run = bp.product_quantity.max(1) as u64;
    let runs = net_qty.div_ceil(out_per_run);
    let output_qty = runs * out_per_run;

    let (me, te) = ctx.me_te_for(bp);
    let structure = ctx.structure_for(bp.activity);
    let me_mult = blueprint_me_multiplier(bp, me, structure, ctx.sde);
    let job_cost = blueprint_job_cost(bp, runs, structure, ctx.esi_prices);
    let job_duration_secs = blueprint_job_duration_secs(bp, te, runs, structure, ctx.skills, ctx.sde);

    let mut craft_stock = local_stock.clone();
    let mut children = Vec::with_capacity(bp.materials.len());
    let mut materials_cost = 0.0;

    for mat in &bp.materials {
        let req = compute_material_qty(mat.quantity, runs, me_mult);
        let child = evaluate_material_node(ctx, mat.material_type_id, req, &mut craft_stock, visiting);
        materials_cost += child.chosen_cost;
        children.push(child);
    }

    visiting.remove(&type_id);

    let craft_cost = materials_cost + job_cost;
    let override_choice = ctx
        .overrides
        .get(&type_id)
        .copied()
        .unwrap_or(DecisionOverride::Auto);

    let decision = match override_choice {
        DecisionOverride::Buy => NodeDecision::Buy,
        DecisionOverride::Craft => NodeDecision::Craft,
        DecisionOverride::Auto => {
            if unit_buy > 0.0 && buy_cost <= craft_cost {
                NodeDecision::Buy
            } else {
                NodeDecision::Craft
            }
        }
    };

    let (chosen_cost, final_children, final_job_cost, final_duration, final_runs, final_output) =
        match decision {
            NodeDecision::Craft => {
                *local_stock = craft_stock;
                (craft_cost, children, job_cost, job_duration_secs, runs, output_qty)
            }
            NodeDecision::Buy | NodeDecision::InStock => (buy_cost, Vec::new(), 0.0, 0, 0, 0),
        };

    OptimizedNode {
        type_id,
        name: ctx.type_name(type_id),
        required_qty,
        from_local_stock,
        net_qty,
        decision,
        is_craftable: true,
        activity: Some(bp.activity),
        blueprint_type_id: Some(bp.blueprint_type_id),
        runs: final_runs,
        output_qty: final_output,
        me_level: me,
        te_level: te,
        unit_buy_price: unit_buy,
        buy_cost,
        craft_cost: Some(craft_cost),
        job_cost: final_job_cost,
        job_duration_secs: final_duration,
        chosen_cost,
        haulable_stock,
        children: final_children,
    }
}