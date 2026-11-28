use std::collections::HashMap;

use crate::core::models::industry::{
    IndustryStructure,
    PriceBasis,
    SecurityStatus,
    StructureType,
};
use crate::core::models::market::{EsiMarketPrice, MarketPrice};
use crate::core::models::sde::{BlueprintActivity, SdeBlueprintEntry, SdeIndex};

pub const SCC_SURCHARGE: f64 = 0.04;

// Base mineral type IDs
pub const BASE_MINERALS: [u32; 8] = [
    34,    // Tritanium
    35,    // Pyerite
    36,    // Mexallon
    37,    // Isogen
    38,    // Nocxium
    39,    // Zydrine
    40,    // Megacyte
    11399, // Morphite
];

pub fn is_base_mineral(type_id: u32) -> bool {
    BASE_MINERALS.contains(&type_id)
}

pub fn skill_level(skills: &HashMap<u32, u8>, skill_id: u32) -> f64 {
    skills.get(&skill_id).copied().unwrap_or(0) as f64
}

pub fn security_industry_mult(sec: &SecurityStatus) -> f64 {
    match sec {
        SecurityStatus::HighSec => 1.0,
        SecurityStatus::LowSec => 1.9,
        SecurityStatus::NullSec | SecurityStatus::Wormhole => 2.1,
    }
}

pub fn security_reprocessing_mult(sec: &SecurityStatus) -> f64 {
    match sec {
        SecurityStatus::HighSec => 1.0,
        SecurityStatus::LowSec => 1.06,
        SecurityStatus::NullSec | SecurityStatus::Wormhole => 1.12,
    }
}

pub fn structure_mfg_role_bonuses(st: &StructureType) -> (f64, f64, f64) {
    // Returns (me_mult, te_mult, job_cost_mult)
    match st {
        StructureType::Raitaru => (0.99, 0.85, 0.97),
        StructureType::Azbel => (0.99, 0.80, 0.96),
        StructureType::Sotiyo => (0.99, 0.70, 0.95),
        _ => (1.0, 1.0, 1.0),
    }
}

pub fn structure_rxn_te_mult(st: &StructureType) -> f64 {
    match st {
        StructureType::Tatara => 0.75,
        _ => 1.0,
    }
}

pub fn structure_reprocessing_role_bonus(st: &StructureType) -> f64 {
    match st {
        StructureType::Athanor => 1.02,
        StructureType::Tatara => 1.055,
        _ => 1.0,
    }
}

/// Checks whether a fitted rig applies to a given product type and returns (me_bonus, te_bonus)
/// as base fractions before security scaling (e.g. (0.02, 0.20) for a T1 L-Set rig).
fn rig_bonus_for_product(
    rig_name: &str,
    activity: BlueprintActivity,
    category_id: u32,
    group_id: u32,
    is_advanced: bool,
) -> (f64, f64) {
    let lower = rig_name.to_lowercase();
    let is_t2 = lower.ends_with(" ii");
    let me_base = if is_t2 { 0.024 } else { 0.02 };
    let te_base = if is_t2 { 0.24 } else { 0.20 };

    if activity == BlueprintActivity::Reaction {
        if !lower.contains("reactor") {
            return (0.0, 0.0);
        }
        // Specific M-Set reactor rigs vs general Reactor rigs
        let matches = if lower.contains("biochemical") {
            group_id == 712 || group_id == 4097
        } else if lower.contains("hybrid") {
            group_id == 974
        } else if lower.contains("composite") {
            group_id == 429 || group_id == 428
        } else {
            true
        };
        if !matches {
            return (0.0, 0.0);
        }
        if lower.contains("material efficiency") {
            return (me_base, 0.0);
        }
        if lower.contains("time efficiency") {
            return (0.0, te_base);
        }
        // L-Set Reactor Efficiency gives both ME & TE
        return (me_base, te_base);
    }

    // Manufacturing rigs
    if !lower.contains("manufacturing") {
        return (0.0, 0.0);
    }

    let is_ship = category_id == 6;
    let is_equipment = matches!(category_id, 7 | 20 | 22 | 32 | 66);
    let is_ammo = category_id == 8;
    let is_drone = matches!(category_id, 18 | 87);
    let is_structure = matches!(category_id, 65) || group_id == 536;
    let is_component = matches!(group_id, 332 | 334 | 536 | 873 | 913 | 964 | 1136);

    let is_small_ship = matches!(group_id, 25 | 31 | 237 | 324 | 420 | 541 | 830 | 831 | 834 | 893 | 1283 | 1305 | 1527 | 1534);
    let is_medium_ship = matches!(group_id, 26 | 28 | 358 | 380 | 419 | 463 | 540 | 543 | 832 | 833 | 894 | 906 | 963 | 1201 | 1202 | 1972);
    let is_large_ship = matches!(group_id, 27 | 381 | 898 | 900 | 941);
    let is_capital_ship = matches!(group_id, 30 | 485 | 513 | 547 | 659 | 883 | 902 | 1538 | 4594);

    let applies = if lower.contains("xl-set equipment and consumable") {
        is_equipment || is_ammo || is_drone
    } else if lower.contains("xl-set ship") {
        is_ship
    } else if lower.contains("xl-set structure and component") {
        is_structure || is_component
    } else if lower.contains("equipment") {
        is_equipment
    } else if lower.contains("ammunition") {
        is_ammo
    } else if lower.contains("drone and fighter") {
        is_drone
    } else if lower.contains("basic small ship") {
        is_ship && is_small_ship && !is_advanced
    } else if lower.contains("advanced small ship") {
        is_ship && is_small_ship && is_advanced
    } else if lower.contains("basic medium ship") {
        is_ship && is_medium_ship && !is_advanced
    } else if lower.contains("advanced medium ship") {
        is_ship && is_medium_ship && is_advanced
    } else if lower.contains("basic large ship") {
        is_ship && is_large_ship && !is_advanced
    } else if lower.contains("advanced large ship") {
        is_ship && is_large_ship && is_advanced
    } else if lower.contains("capital ship") {
        is_ship && is_capital_ship
    } else if lower.contains("advanced component") {
        is_component
    } else if lower.contains("structure manufacturing") {
        is_structure
    } else {
        false
    };

    if !applies {
        return (0.0, 0.0);
    }

    if lower.contains("material efficiency") {
        (me_base, 0.0)
    } else if lower.contains("time efficiency") {
        (0.0, te_base)
    } else {
        // L-Set and XL-Set combined Efficiency rigs apply both ME and TE
        (me_base, te_base)
    }
}

pub fn effective_rig_multipliers(
    structure: Option<&IndustryStructure>,
    sde: &SdeIndex,
    bp: &SdeBlueprintEntry,
) -> (f64, f64) {
    let Some(s) = structure else {
        return (1.0, 1.0);
    };

    let type_info = sde.types.get(&bp.product_type_id);
    let group_id = type_info.map(|t| t.group_id).unwrap_or(0);
    let category_id = sde
        .groups
        .get(&group_id)
        .map(|g| g.category_id)
        .unwrap_or(0);
    let is_advanced = type_info
        .and_then(|t| t.meta_group_id)
        .map(|mg| mg == 2 || mg == 14)
        .unwrap_or(false);

    let sec_mult = security_industry_mult(&s.security_status);
    let mut best_me = 0.0_f64;
    let mut best_te = 0.0_f64;

    for rig in &s.rigs {
        let (me, te) = rig_bonus_for_product(rig, bp.activity, category_id, group_id, is_advanced);
        best_me = best_me.max(me);
        best_te = best_te.max(te);
    }

    (1.0 - best_me * sec_mult, 1.0 - best_te * sec_mult)
}

/// Computes exact EVE required material quantity for `runs` of a blueprint.
pub fn compute_material_qty(base_qty: u64, runs: u64, me_mult: f64) -> u64 {
    if base_qty <= 1 {
        return base_qty * runs;
    }
    let scaled = (runs as f64) * (base_qty as f64) * me_mult;
    let rounded = (scaled - 1e-9).ceil().max(0.0) as u64;
    rounded.max(runs)
}

/// Computes total material efficiency multiplier `(1 - bp_me) * struct_me * rig_me`.
pub fn blueprint_me_multiplier(
    bp: &SdeBlueprintEntry,
    bp_me: u8,
    structure: Option<&IndustryStructure>,
    sde: &SdeIndex,
) -> f64 {
    let bp_mult = match bp.activity {
        BlueprintActivity::Manufacturing => 1.0 - (bp_me.min(10) as f64 / 100.0),
        BlueprintActivity::Reaction => 1.0,
    };
    let struct_me = match (bp.activity, structure) {
        (BlueprintActivity::Manufacturing, Some(s)) => {
            structure_mfg_role_bonuses(&s.structure_type).0
        }
        _ => 1.0,
    };
    let (rig_me, _) = effective_rig_multipliers(structure, sde, bp);
    bp_mult * struct_me * rig_me
}

/// Computes total job duration in seconds for `runs`.
pub fn blueprint_job_duration_secs(
    bp: &SdeBlueprintEntry,
    bp_te: u8,
    runs: u64,
    structure: Option<&IndustryStructure>,
    skills: &HashMap<u32, u8>,
    sde: &SdeIndex,
) -> u64 {
    let (_, rig_te) = effective_rig_multipliers(structure, sde, bp);

    let te_mult = match bp.activity {
        BlueprintActivity::Manufacturing => {
            let bp_mult = 1.0 - (bp_te.min(20) as f64 / 100.0);
            let skill_mult = (1.0 - 0.04 * skill_level(skills, 3380))
                * (1.0 - 0.03 * skill_level(skills, 3388));
            let struct_te = structure
                .map(|s| structure_mfg_role_bonuses(&s.structure_type).1)
                .unwrap_or(1.0);
            bp_mult * skill_mult * struct_te * rig_te
        }
        BlueprintActivity::Reaction => {
            let skill_mult = 1.0 - 0.04 * skill_level(skills, 45746);
            let struct_te = structure
                .map(|s| structure_rxn_te_mult(&s.structure_type))
                .unwrap_or(1.0);
            skill_mult * struct_te * rig_te
        }
    };

    ((runs as f64) * (bp.time as f64) * te_mult).ceil().max(1.0) as u64
}

/// Computes job installation cost using CCP's EIV (base ME0 materials * ESI adjusted_price).
pub fn blueprint_job_cost(
    bp: &SdeBlueprintEntry,
    runs: u64,
    structure: Option<&IndustryStructure>,
    esi_prices: &HashMap<u32, EsiMarketPrice>,
) -> f64 {
    let eiv: f64 = bp
        .materials
        .iter()
        .map(|m| {
            let adj = esi_prices
                .get(&m.material_type_id)
                .map(|p| p.adjusted_price)
                .unwrap_or(0.0);
            (m.quantity as f64) * (runs as f64) * adj
        })
        .sum();

    let (sci, tax, struct_cost_mult) = match (bp.activity, structure) {
        (BlueprintActivity::Manufacturing, Some(s)) => (
            s.manufacturing_cost_index / 100.0,
            s.facility_tax / 100.0,
            structure_mfg_role_bonuses(&s.structure_type).2,
        ),
        (BlueprintActivity::Reaction, Some(s)) => (
            s.reaction_cost_index / 100.0,
            s.facility_tax / 100.0,
            1.0,
        ),
        (_, None) => (0.02, 0.0025, 1.0),
    };

    eiv * (sci * struct_cost_mult + tax + SCC_SURCHARGE)
}

/// Maps an Asteroid/Moon ore or Ice `group_id` to its EVE Processing Skill ID.
pub fn ore_processing_skill_id(group_id: u32) -> Option<u32> {
    match group_id {
        // Ice Processing (18025)
        465 => Some(18025),
        // Simple Ore Processing (60377): Veldspar, Scordite, Pyroxeres, Plagioclase
        462 | 460 | 459 | 458 => Some(60377),
        // Coherent Ore Processing (60378): Omber, Kernite, Jaspet, Hemorphite, Hedbergite
        469 | 457 | 456 | 455 | 454 => Some(60378),
        // Variegated Ore Processing (60379): Crokite, Dark Ochre, Gneiss
        450 | 453 | 467 => Some(60379),
        // Complex Ore Processing (60380): Arkonor, Bistot, Spodumain
        451 | 452 | 461 => Some(60380),
        // Abyssal Ore Processing (60381): Bezdnacine, Rakovene, Talassonite
        1996 | 2000 | 2002 => Some(60381),
        // Mercoxit Ore Processing (12189)
        468 => Some(12189),
        // Moon Ore Processing
        1884 => Some(46152), // Ubiquitous
        1920 => Some(46153), // Common
        1921 => Some(46154), // Uncommon
        1922 => Some(46155), // Rare
        1923 => Some(46156), // Exceptional
        _ => None,
    }
}

/// Computes exact reprocessing yield (0.0..=1.0) for an asteroid/moon ore or ice at `structure`:
/// `((50 + Rm) / 100 * (1 + Sec)) * (1 + Sm) * (1 + 0.03 * R) * (1 + 0.02 * Re) * (1 + 0.02 * Op) * (1 + Im)`
/// where `Sec` only applies when a matching reprocessing rig (`Rm > 0`) is fitted.
pub fn asteroid_reprocessing_yield(
    group_id: u32,
    structure: Option<&IndustryStructure>,
    skills: &HashMap<u32, u8>,
    implant_bonus: f64,
) -> f64 {
    let is_moon = matches!(group_id, 1884 | 1920 | 1921 | 1922 | 1923);
    let is_ice = group_id == 465;

    let (rig_modifier, sec_mult, struct_bonus) = match structure {
        Some(s) if matches!(s.structure_type, StructureType::Athanor | StructureType::Tatara) => {
            let mut best_rm = 0.0_f64;
            for rig in &s.rigs {
                let lower = rig.trim().to_lowercase();
                let applies = lower.contains("reprocessing efficiency")
                    || (is_moon && lower.contains("moon ore grading"))
                    || (is_ice && lower.contains("ice grading"))
                    || (!is_moon && !is_ice && lower.contains("asteroid ore grading"));
                if applies {
                    let rm = if lower.ends_with(" ii") { 3.0 } else { 1.0 };
                    best_rm = best_rm.max(rm);
                }
            }
            let sec_mult = if best_rm > 0.0 {
                security_reprocessing_mult(&s.security_status)
            } else {
                1.0
            };
            (
                best_rm,
                sec_mult,
                structure_reprocessing_role_bonus(&s.structure_type),
            )
        }
        _ => (0.0, 1.0, 1.0),
    };

    let base_with_rig = (50.0 + rig_modifier) / 100.0;
    let reproc = 1.0 + 0.03 * skill_level(skills, 3385).min(5.0);
    let reproc_eff = 1.0 + 0.02 * skill_level(skills, 3389).min(5.0);
    let spec = ore_processing_skill_id(group_id)
        .map(|sid| 1.0 + 0.02 * skill_level(skills, sid).min(5.0))
        .unwrap_or(1.0);
    let implant_mult = 1.0 + implant_bonus.clamp(0.0, 0.04);

    ((base_with_rig * sec_mult) * struct_bonus * reproc * reproc_eff * spec * implant_mult).min(1.0)
}

/// Computes exact reprocessing yield (0.50..=0.55) for modules, ships, and other non-ore items:
/// `50% * (1 + Scrapmetal Processing * 0.02)`, unaffected by structures, rigs, security, or implants.
pub fn scrapmetal_reprocessing_yield(skills: &HashMap<u32, u8>) -> f64 {
    let lvl = skill_level(skills, 12196).min(5.0);
    (0.50 * (1.0 + 0.02 * lvl)).min(0.55)
}

/// Computes reprocessing yield for any EVE item, dispatching ores/ice vs. modules/items (scrapmetal).
pub fn item_reprocessing_yield(
    category_id: u32,
    group_id: u32,
    structure: Option<&IndustryStructure>,
    skills: &HashMap<u32, u8>,
    implant_bonus: f64,
) -> f64 {
    const ASTEROID_CATEGORY_ID: u32 = 25;
    if category_id == ASTEROID_CATEGORY_ID || group_id == 465 {
        asteroid_reprocessing_yield(group_id, structure, skills, implant_bonus)
    } else {
        scrapmetal_reprocessing_yield(skills)
    }
}

pub fn sales_tax_rate(skills: &HashMap<u32, u8>) -> f64 {
    0.075 * (1.0 - 0.11 * skill_level(skills, 16622))
}

pub fn broker_fee_rate(skills: &HashMap<u32, u8>) -> f64 {
    (0.03 - 0.003 * skill_level(skills, 3446)).max(0.0)
}

pub fn pick_price(price: Option<&MarketPrice>, basis: &PriceBasis) -> f64 {
    let Some(p) = price else {
        return 0.0;
    };
    match basis {
        PriceBasis::Buy => p.buy,
        PriceBasis::Sell => p.sell,
        PriceBasis::Split => p.split,
    }
}

/// Effective unit acquisition cost (adds broker fee only when placing Buy Orders).
pub fn effective_buy_unit_price(
    price: Option<&MarketPrice>,
    basis: &PriceBasis,
    skills: &HashMap<u32, u8>,
) -> f64 {
    let raw = pick_price(price, basis);
    if raw <= 0.0 {
        return 0.0;
    }
    match basis {
        PriceBasis::Buy => raw * (1.0 + broker_fee_rate(skills)),
        PriceBasis::Sell | PriceBasis::Split => raw,
    }
}

/// Effective unit net revenue after sales tax (and broker fee when listing Sell Orders).
pub fn effective_sell_unit_revenue(
    price: Option<&MarketPrice>,
    basis: &PriceBasis,
    skills: &HashMap<u32, u8>,
) -> f64 {
    let raw = pick_price(price, basis);
    if raw <= 0.0 {
        return 0.0;
    }
    let tax = sales_tax_rate(skills);
    let broker = match basis {
        PriceBasis::Sell => broker_fee_rate(skills),
        PriceBasis::Buy | PriceBasis::Split => 0.0,
    };
    raw * (1.0 - tax - broker).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_structure(
        structure_type: StructureType,
        security_status: SecurityStatus,
        rigs: Vec<&str>,
    ) -> IndustryStructure {
        IndustryStructure {
            id: "test-struct".to_string(),
            name: "Test Refinery".to_string(),
            structure_type,
            security_status,
            manufacturing_cost_index: 0.0,
            reaction_cost_index: 0.0,
            facility_tax: 0.0,
            rigs: rigs.into_iter().map(String::from).collect(),
            services: vec!["Reprocessing".to_string()],
            location_id: None,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {:.10}, got {:.10}",
            expected,
            actual
        );
    }

    #[test]
    fn max_nullsec_tatara_t2_with_rx804_matches_eve_formula() {
        let tatara = make_structure(
            StructureType::Tatara,
            SecurityStatus::NullSec,
            vec!["Standup L-Set Reprocessing Efficiency II"],
        );
        let skills = HashMap::from([(3385, 5), (3389, 5), (60377, 5)]);
        let y = asteroid_reprocessing_yield(462, Some(&tatara), &skills, 0.04);
        // ((50 + 3) / 100 * 1.12) * 1.055 * 1.15 * 1.10 * 1.10 * 1.04 = 0.90628105568
        assert_close(y, 0.53 * 1.12 * 1.055 * 1.15 * 1.10 * 1.10 * 1.04);
    }

    #[test]
    fn highsec_tatara_t2_with_rx804_matches_eve_formula() {
        let tatara = make_structure(
            StructureType::Tatara,
            SecurityStatus::HighSec,
            vec!["Standup L-Set Reprocessing Efficiency II"],
        );
        let skills = HashMap::from([(3385, 5), (3389, 5), (60377, 5)]);
        let y = asteroid_reprocessing_yield(462, Some(&tatara), &skills, 0.04);
        assert_close(y, 0.53 * 1.00 * 1.055 * 1.15 * 1.10 * 1.10 * 1.04);
    }

    #[test]
    fn unrigged_refineries_do_not_receive_security_bonus() {
        let skills = HashMap::new();

        let athanor = make_structure(StructureType::Athanor, SecurityStatus::NullSec, vec![]);
        assert_close(asteroid_reprocessing_yield(462, Some(&athanor), &skills, 0.0), 0.50 * 1.02);

        let tatara = make_structure(StructureType::Tatara, SecurityStatus::Wormhole, vec![]);
        assert_close(asteroid_reprocessing_yield(462, Some(&tatara), &skills, 0.0), 0.50 * 1.055);
    }

    #[test]
    fn lowsec_t1_athanor_and_non_matching_rig_behavior() {
        let skills = HashMap::new();

        let athanor_matching = make_structure(
            StructureType::Athanor,
            SecurityStatus::LowSec,
            vec!["Standup M-Set Asteroid Ore Grading Processor I"],
        );
        assert_close(
            asteroid_reprocessing_yield(462, Some(&athanor_matching), &skills, 0.0),
            0.51 * 1.06 * 1.02,
        );

        // Moon rig does not apply to asteroid ore -> Rm = 0, Sec = 0
        let athanor_moon_only = make_structure(
            StructureType::Athanor,
            SecurityStatus::NullSec,
            vec!["Standup M-Set Moon Ore Grading Processor II"],
        );
        assert_close(
            asteroid_reprocessing_yield(462, Some(&athanor_moon_only), &skills, 0.0),
            0.50 * 1.02,
        );
    }

    #[test]
    fn other_upwell_and_npc_have_zero_structure_modifier() {
        let fortizar = make_structure(StructureType::Fortizar, SecurityStatus::NullSec, vec![]);
        let skills = HashMap::from([(3385, 5), (3389, 5), (60377, 5)]);
        let expected = 0.50 * 1.15 * 1.10 * 1.10 * 1.02;

        assert_close(asteroid_reprocessing_yield(462, Some(&fortizar), &skills, 0.02), expected);
        assert_close(asteroid_reprocessing_yield(462, None, &skills, 0.02), expected);
    }

    #[test]
    fn ice_processing_uses_ice_rig_and_skill() {
        let athanor = make_structure(
            StructureType::Athanor,
            SecurityStatus::Wormhole,
            vec!["Standup M-Set Ice Grading Processor II"],
        );
        let skills = HashMap::from([(3385, 5), (3389, 5), (18025, 5)]);
        let y = item_reprocessing_yield(25, 465, Some(&athanor), &skills, 0.04);
        assert_close(y, 0.53 * 1.12 * 1.02 * 1.15 * 1.10 * 1.10 * 1.04);
    }

    #[test]
    fn scrapmetal_formula_ignores_structure_rig_sec_and_implant_and_caps_at_55_percent() {
        let tatara = make_structure(
            StructureType::Tatara,
            SecurityStatus::NullSec,
            vec!["Standup L-Set Reprocessing Efficiency II"],
        );

        let lvl0 = HashMap::new();
        assert_close(scrapmetal_reprocessing_yield(&lvl0), 0.50);
        assert_close(item_reprocessing_yield(7, 53, Some(&tatara), &lvl0, 0.04), 0.50);

        let lvl3 = HashMap::from([(12196, 3), (3385, 5), (3389, 5)]);
        assert_close(scrapmetal_reprocessing_yield(&lvl3), 0.53);
        assert_close(item_reprocessing_yield(7, 53, Some(&tatara), &lvl3, 0.04), 0.53);

        let lvl5 = HashMap::from([(12196, 5), (3385, 5), (3389, 5)]);
        assert_close(scrapmetal_reprocessing_yield(&lvl5), 0.55);
        assert_close(item_reprocessing_yield(7, 53, Some(&tatara), &lvl5, 0.04), 0.55);
    }
}