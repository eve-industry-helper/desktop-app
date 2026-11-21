use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndustryPlan {
    pub id: String,
    pub name: String,
    pub blueprints: Vec<PlanBlueprint>,
    pub buy_only_items: Vec<PlanBuyItem>,
}

#[derive(Deserialize)]
pub struct SdeMaterial {
    #[serde(rename = "typeID")]
    pub type_id: u32,
    pub quantity: u64,
}

#[derive(Deserialize)]
pub struct SdeActivity {
    #[serde(default)]
    pub materials: Vec<SdeMaterial>,
}

#[derive(Deserialize)]
pub struct SdeActivities {
    pub manufacturing: Option<SdeActivity>,
    pub reaction: Option<SdeActivity>,
}

#[derive(Deserialize)]
pub struct SdeBlueprint {
    pub activities: SdeActivities,
}