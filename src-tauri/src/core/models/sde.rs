use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::models::industry::RawMaterial;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintActivity {
    Manufacturing,
    Reaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdeBlueprintEntry {
    pub blueprint_type_id: u32,
    pub activity: BlueprintActivity,
    pub product_type_id: u32,
    pub product_quantity: u32,
    pub time: u32,
    pub materials: Vec<RawMaterial>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdeTypeInfo {
    pub name: String,
    pub group_id: u32,
    pub portion_size: u32,
    pub volume: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_group_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdeGroupInfo {
    pub name: String,
    pub category_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SdeIndex {
    pub release: String,
    #[serde(default)]
    pub etag: Option<String>,
    pub blueprints: HashMap<u32, SdeBlueprintEntry>,
    pub product_to_blueprint: HashMap<u32, u32>,
    pub reprocessing: HashMap<u32, Vec<RawMaterial>>,
    pub types: HashMap<u32, SdeTypeInfo>,
    pub groups: HashMap<u32, SdeGroupInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdeStatus {
    pub loaded: bool,
    pub release: String,
    pub blueprint_count: usize,
    pub type_count: usize,
    pub reprocessing_count: usize,
}

impl From<&SdeIndex> for SdeStatus {
    fn from(idx: &SdeIndex) -> Self {
        Self {
            loaded: !idx.blueprints.is_empty(),
            release: idx.release.clone(),
            blueprint_count: idx.blueprints.len(),
            type_count: idx.types.len(),
            reprocessing_count: idx.reprocessing.len(),
        }
    }
}