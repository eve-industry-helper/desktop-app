use std::collections::{HashMap, HashSet};
use microlp::{ComparisonOp, OptimizationDirection, Problem, Variable};
use serde::{Deserialize, Serialize};

use crate::core::models::industry::{IndustryStructure, PriceBasis};
use crate::core::models::market::MarketPrice;
use crate::core::models::sde::SdeIndex;
use crate::core::optimizer::bonuses::{
    asteroid_reprocessing_yield,
    effective_buy_unit_price,
    effective_sell_unit_revenue,
    is_base_mineral,
};

const ASTEROID_CATEGORY_ID: u32 = 25;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrePurchasePlan {
    pub type_id: u32,
    pub name: String,
    pub batches: u64,
    pub portion_size: u32,
    pub units_to_buy: u64,
    pub unit_price: f64,
    pub total_cost: f64,
    pub yield_percent: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MineralShortfall {
    pub type_id: u32,
    pub name: String,
    pub quantity: u64,
    pub unit_price: f64,
    pub total_cost: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SurplusMineral {
    pub type_id: u32,
    pub name: String,
    pub quantity: u64,
    pub unit_sell_value: f64,
    pub total_sell_value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReprocessingPlan {
    pub used_reprocessing: bool,
    pub direct_minerals_cost: f64,
    pub ore_purchase_cost: f64,
    pub remaining_minerals_cost: f64,
    pub gross_outlay: f64,
    pub surplus_value: f64,
    pub net_effective_cost: f64,
    pub savings: f64,
    pub ores: Vec<OrePurchasePlan>,
    pub direct_minerals: Vec<MineralShortfall>,
    pub surplus_minerals: Vec<SurplusMineral>,
}

/// Finds all Compressed Asteroid Ores in `SdeIndex` that yield at least one needed mineral.
pub fn find_candidate_compressed_ores(
    sde: &SdeIndex,
    needed_minerals: &HashSet<u32>,
) -> Vec<u32> {
    if needed_minerals.is_empty() {
        return Vec::new();
    }

    let mut ore_ids = Vec::new();
    for (&type_id, info) in &sde.types {
        if !info.name.starts_with("Compressed ") {
            continue;
        }
        let is_asteroid = sde
            .groups
            .get(&info.group_id)
            .map(|g| g.category_id == ASTEROID_CATEGORY_ID)
            .unwrap_or(false);
        if !is_asteroid {
            continue;
        }
        let Some(reproc_mats) = sde.reprocessing.get(&type_id) else {
            continue;
        };
        if reproc_mats
            .iter()
            .any(|m| needed_minerals.contains(&m.material_type_id))
        {
            ore_ids.push(type_id);
        }
    }
    ore_ids
}

struct OreCandidate {
    type_id: u32,
    name: String,
    portion_size: u32,
    unit_price: f64,
    batch_cost: f64,
    yield_rate: f64,
    yields_per_batch: Vec<(u32, u64)>,
    max_useful_batches: f64,
}

pub struct ReprocessInput<'a> {
    pub sde: &'a SdeIndex,
    pub needed_minerals: &'a HashMap<u32, u64>,
    pub buy_prices: &'a HashMap<u32, MarketPrice>,
    pub sell_prices: &'a HashMap<u32, MarketPrice>,
    pub material_basis: &'a PriceBasis,
    pub product_basis: &'a PriceBasis,
    pub structure: Option<&'a IndustryStructure>,
    pub skills: &'a HashMap<u32, u8>,
    pub credit_surplus: bool,
}

pub fn solve_mineral_reprocessing(input: &ReprocessInput<'_>) -> ReprocessingPlan {
    let needed: HashMap<u32, u64> = input
        .needed_minerals
        .iter()
        .filter(|(id, qty)| is_base_mineral(**id) && **qty > 0)
        .map(|(&id, &qty)| (id, qty))
        .collect();

    let mut direct_minerals_cost = 0.0;
    let mut direct_fallback = Vec::new();

    for (&min_id, &qty) in &needed {
        let unit_price = effective_buy_unit_price(
            input.buy_prices.get(&min_id),
            input.material_basis,
            input.skills,
        );
        let cost = unit_price * (qty as f64);
        direct_minerals_cost += cost;
        direct_fallback.push(MineralShortfall {
            type_id: min_id,
            name: input
                .sde
                .types
                .get(&min_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("Type {}", min_id)),
            quantity: qty,
            unit_price,
            total_cost: cost,
        });
    }
    direct_fallback.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));

    let make_direct_plan = || ReprocessingPlan {
        used_reprocessing: false,
        direct_minerals_cost,
        ore_purchase_cost: 0.0,
        remaining_minerals_cost: direct_minerals_cost,
        gross_outlay: direct_minerals_cost,
        surplus_value: 0.0,
        net_effective_cost: direct_minerals_cost,
        savings: 0.0,
        ores: Vec::new(),
        direct_minerals: direct_fallback.clone(),
        surplus_minerals: Vec::new(),
    };

    if needed.is_empty() {
        return make_direct_plan();
    }

    let needed_set: HashSet<u32> = needed.keys().copied().collect();
    let candidate_ids = find_candidate_compressed_ores(input.sde, &needed_set);

    let mut candidates = Vec::new();
    for ore_id in candidate_ids {
        let Some(info) = input.sde.types.get(&ore_id) else {
            continue;
        };
        let Some(reproc_mats) = input.sde.reprocessing.get(&ore_id) else {
            continue;
        };
        let unit_price = effective_buy_unit_price(
            input.buy_prices.get(&ore_id),
            input.material_basis,
            input.skills,
        );
        if unit_price <= 0.0 {
            continue;
        }

        let yield_rate = asteroid_reprocessing_yield(info.group_id, input.structure, input.skills);
        let yields_per_batch: Vec<(u32, u64)> = reproc_mats
            .iter()
            .filter_map(|m| {
                let y = ((m.quantity as f64) * yield_rate).floor() as u64;
                (y > 0).then_some((m.material_type_id, y))
            })
            .collect();

        let mut max_useful = 0.0_f64;
        for &(min_id, y) in &yields_per_batch {
            if let Some(&req) = needed.get(&min_id) {
                let batches_for_min = (req as f64 / y as f64).ceil();
                max_useful = max_useful.max(batches_for_min);
            }
        }
        if max_useful <= 0.0 {
            continue;
        }

        let batch_cost = unit_price * (info.portion_size.max(1) as f64);
        candidates.push(OreCandidate {
            type_id: ore_id,
            name: info.name.clone(),
            portion_size: info.portion_size.max(1),
            unit_price,
            batch_cost,
            yield_rate,
            yields_per_batch,
            max_useful_batches: max_useful,
        });
    }

    if candidates.is_empty() {
        return make_direct_plan();
    }

    // Build LP: minimise total ore + direct mineral cost (minus bounded surplus credit if enabled)
    let mut problem = Problem::new(OptimizationDirection::Minimize);

    let ore_vars: Vec<Variable> = candidates
        .iter()
        .map(|c| {
            let mut obj_cost = c.batch_cost;
            if input.credit_surplus {
                // Credit byproduct minerals not in `needed` directly on the ore batch
                for &(min_id, y) in &c.yields_per_batch {
                    if !needed.contains_key(&min_id) {
                        let rev = effective_sell_unit_revenue(
                            input.sell_prices.get(&min_id),
                            input.product_basis,
                            input.skills,
                        );
                        obj_cost -= rev * (y as f64);
                    }
                }
            }
            problem.add_var(obj_cost, (0.0, c.max_useful_batches))
        })
        .collect();

    let mut direct_vars: HashMap<u32, Variable> = HashMap::new();
    let mut surplus_vars: HashMap<u32, Variable> = HashMap::new();

    for (&min_id, &req_qty) in &needed {
        let buy_u = effective_buy_unit_price(
            input.buy_prices.get(&min_id),
            input.material_basis,
            input.skills,
        );
        // If a mineral has no market price, give it a large penalty so the LP satisfies it via ore
        let cost_coeff = if buy_u > 0.0 { buy_u } else { 1e9 };
        let b_var = problem.add_var(cost_coeff, (0.0, req_qty as f64));
        direct_vars.insert(min_id, b_var);

        if input.credit_surplus {
            let rev = effective_sell_unit_revenue(
                input.sell_prices.get(&min_id),
                input.product_basis,
                input.skills,
            );
            let s_var = problem.add_var(-rev, (0.0, f64::INFINITY));
            surplus_vars.insert(min_id, s_var);
        }
    }

    for (&min_id, &req_qty) in &needed {
        let mut terms: Vec<(Variable, f64)> = Vec::new();
        for (idx, cand) in candidates.iter().enumerate() {
            if let Some(&(_, y)) = cand.yields_per_batch.iter().find(|&&(id, _)| id == min_id) {
                terms.push((ore_vars[idx], y as f64));
            }
        }
        terms.push((direct_vars[&min_id], 1.0));

        if let Some(&s_var) = surplus_vars.get(&min_id) {
            terms.push((s_var, -1.0));
            problem.add_constraint(&terms, ComparisonOp::Eq, req_qty as f64);
        } else {
            problem.add_constraint(&terms, ComparisonOp::Ge, req_qty as f64);
        }
    }

    let Ok(outcome) = problem.solve() else {
        return make_direct_plan();
    };
    let Ok(solution) = outcome.into_solution() else {
        return make_direct_plan();
    };

    // Discretise LP ore batches to exact integers and compute exact shortfalls & surpluses
    let mut ores = Vec::new();
    let mut ore_purchase_cost = 0.0;
    let mut yielded_totals: HashMap<u32, u64> = HashMap::new();

    for (idx, cand) in candidates.iter().enumerate() {
        let raw_batches = solution.var_value(ore_vars[idx]);
        if raw_batches < 0.5 {
            continue;
        }
        let batches = raw_batches.round() as u64;
        if batches == 0 {
            continue;
        }

        let units_to_buy = batches * (cand.portion_size as u64);
        let total_cost = cand.unit_price * (units_to_buy as f64);
        ore_purchase_cost += total_cost;

        for &(min_id, y) in &cand.yields_per_batch {
            *yielded_totals.entry(min_id).or_insert(0) += y * batches;
        }

        ores.push(OrePurchasePlan {
            type_id: cand.type_id,
            name: cand.name.clone(),
            batches,
            portion_size: cand.portion_size,
            units_to_buy,
            unit_price: cand.unit_price,
            total_cost,
            yield_percent: cand.yield_rate * 100.0,
        });
    }

    if ores.is_empty() {
        return make_direct_plan();
    }

    let mut direct_minerals = Vec::new();
    let mut remaining_minerals_cost = 0.0;

    for (&min_id, &req_qty) in &needed {
        let got = yielded_totals.get(&min_id).copied().unwrap_or(0);
        let deficit = req_qty.saturating_sub(got);
        if deficit > 0 {
            let unit_price = effective_buy_unit_price(
                input.buy_prices.get(&min_id),
                input.material_basis,
                input.skills,
            );
            let total_cost = unit_price * (deficit as f64);
            remaining_minerals_cost += total_cost;
            direct_minerals.push(MineralShortfall {
                type_id: min_id,
                name: input
                    .sde
                    .types
                    .get(&min_id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("Type {}", min_id)),
                quantity: deficit,
                unit_price,
                total_cost,
            });
        }
    }

    let mut surplus_minerals = Vec::new();
    let mut surplus_value = 0.0;

    for (&min_id, &got) in &yielded_totals {
        let req = needed.get(&min_id).copied().unwrap_or(0);
        let extra = got.saturating_sub(req);
        if extra > 0 {
            let unit_sell = effective_sell_unit_revenue(
                input.sell_prices.get(&min_id),
                input.product_basis,
                input.skills,
            );
            let total_sell = unit_sell * (extra as f64);
            surplus_value += total_sell;
            surplus_minerals.push(SurplusMineral {
                type_id: min_id,
                name: input
                    .sde
                    .types
                    .get(&min_id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("Type {}", min_id)),
                quantity: extra,
                unit_sell_value: unit_sell,
                total_sell_value: total_sell,
            });
        }
    }

    let gross_outlay = ore_purchase_cost + remaining_minerals_cost;
    let net_effective_cost = if input.credit_surplus {
        (gross_outlay - surplus_value).max(0.0)
    } else {
        gross_outlay
    };

    // Only recommend reprocessing if it actually beats direct buying under the active mode
    let comparison_cost = if input.credit_surplus {
        net_effective_cost
    } else {
        gross_outlay
    };

    if comparison_cost >= direct_minerals_cost {
        return make_direct_plan();
    }

    ores.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));
    direct_minerals.sort_by(|a, b| b.total_cost.total_cmp(&a.total_cost));
    surplus_minerals.sort_by(|a, b| b.total_sell_value.total_cmp(&a.total_sell_value));

    ReprocessingPlan {
        used_reprocessing: true,
        direct_minerals_cost,
        ore_purchase_cost,
        remaining_minerals_cost,
        gross_outlay,
        surplus_value,
        net_effective_cost,
        savings: direct_minerals_cost - comparison_cost,
        ores,
        direct_minerals,
        surplus_minerals,
    }
}