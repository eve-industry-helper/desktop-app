use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSkillCache {
    pub character_name: String,
    #[serde(default)]
    pub skills: HashMap<u32, u8>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndustryPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub entries: Vec<PlanEntry>,
}

// --- CCP SDE Deserialization Models ---

#[derive(Deserialize, Debug)]
pub struct SdeMaterial {
    #[serde(rename = "typeID")]
    pub type_id: u32,
    pub quantity: u64,
}

#[derive(Deserialize, Debug)]
pub struct SdeActivity {
    #[serde(default)]
    pub materials: Vec<SdeMaterial>,
}

#[derive(Deserialize, Debug)]
pub struct SdeActivities {
    pub manufacturing: Option<SdeActivity>,
    pub reaction: Option<SdeActivity>,
}

#[derive(Deserialize, Debug)]
pub struct SdeBlueprint {
    pub activities: SdeActivities,
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
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