use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_me() -> u8 {
    10
}
fn default_te() -> u8 {
    20
}
fn default_one() -> u64 {
    1
}
fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSkillCache {
    pub character_name: String,
    #[serde(default)]
    pub skills: HashMap<u32, u8>,
    #[serde(default)]
    pub reprocessing_implant_bonus: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndustrySettings {
    #[serde(default)]
    pub character_skills: HashMap<u32, CharacterSkillCache>,
    #[serde(default)]
    pub structures: Vec<IndustryStructure>,
    #[serde(default)]
    pub pricing: PriceSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanMaterial {
    pub type_id: u32,
    pub name: String,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawMaterial {
    pub material_type_id: u32,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DecisionOverride {
    #[default]
    Auto,
    Buy,
    Craft,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildConfig {
    #[serde(default)]
    pub manufacturing_structure_id: Option<String>,
    #[serde(default)]
    pub reaction_structure_id: Option<String>,
    #[serde(default)]
    pub reprocessing_structure_id: Option<String>,
    #[serde(default = "default_me")]
    pub default_me: u8,
    #[serde(default = "default_te")]
    pub default_te: u8,
    #[serde(default = "default_one")]
    pub quantity: u64,
    #[serde(default = "default_true")]
    pub use_character_assets: bool,
    #[serde(default = "default_true")]
    pub enable_reprocessing_lp: bool,
    #[serde(default)]
    pub credit_surplus_minerals: bool,
    #[serde(default)]
    pub overrides: HashMap<u32, DecisionOverride>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            manufacturing_structure_id: None,
            reaction_structure_id: None,
            reprocessing_structure_id: None,
            default_me: 10,
            default_te: 20,
            quantity: 1,
            use_character_assets: true,
            enable_reprocessing_lp: true,
            credit_surplus_minerals: false,
            overrides: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReservedResource {
    pub location_id: u64,
    pub location_name: String,
    pub type_id: u32,
    pub name: String,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanBlueprint {
    pub item_name: String,
    pub item_type_id: u32,
    pub blueprint_name: String,
    pub blueprint_type_id: u32,
    pub runs: u32,
    #[serde(default)]
    pub materials: Vec<PlanMaterial>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanBuyItem {
    pub item_name: String,
    pub item_type_id: u32,
    pub quantity: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlanEntryKind {
    Fit,
    Blueprint,
    Reaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlanEntry {
    pub id: String,
    pub name: String,
    pub kind: PlanEntryKind,
    #[serde(default)]
    pub blueprints: Vec<PlanBlueprint>,
    #[serde(default)]
    pub buy_only_items: Vec<PlanBuyItem>,
    #[serde(default)]
    pub config: BuildConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndustryPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub entries: Vec<PlanEntry>,
    #[serde(default)]
    pub config: BuildConfig,
    #[serde(default)]
    pub validated: bool,
    #[serde(default)]
    pub validated_at: Option<u64>,
    #[serde(default)]
    pub reserved_resources: Vec<ReservedResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityStatus {
    HighSec,
    LowSec,
    NullSec,
    Wormhole,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StructureType {
    NpcStation,
    Raitaru,
    Azbel,
    Sotiyo,
    Athanor,
    Tatara,
    Astrahus,
    Fortizar,
    Keepstar,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndustryStructure {
    pub id: String,
    pub name: String,
    pub structure_type: StructureType,
    pub security_status: SecurityStatus,
    pub manufacturing_cost_index: f64,
    #[serde(default)]
    pub reaction_cost_index: f64,
    pub facility_tax: f64,
    #[serde(default)]
    pub rigs: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub location_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceProvider {
    #[default]
    Fuzzwork,
    EveTycoon,
    Janice,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PriceBasis {
    #[default]
    Sell,
    Buy,
    Split,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceSettings {
    #[serde(default)]
    pub provider: PriceProvider,
    #[serde(default = "default_region_id")]
    pub buy_region_id: u32,
    #[serde(default = "default_region_id")]
    pub sell_region_id: u32,
    #[serde(default)]
    pub material_price_basis: PriceBasis,
    #[serde(default)]
    pub product_price_basis: PriceBasis,
    #[serde(default)]
    pub janice_api_key: String,
}

fn default_region_id() -> u32 {
    10000002 // The Forge (Jita)
}

impl Default for PriceSettings {
    fn default() -> Self {
        Self {
            provider: PriceProvider::Fuzzwork,
            buy_region_id: default_region_id(),
            sell_region_id: default_region_id(),
            material_price_basis: PriceBasis::Sell,
            product_price_basis: PriceBasis::Sell,
            janice_api_key: String::new(),
        }
    }
}