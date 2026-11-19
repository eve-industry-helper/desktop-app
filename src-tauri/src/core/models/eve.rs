use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct JwtPayload {
    pub name: Option<String>,
    pub sub: Option<String>,
}

#[derive(Serialize)]
pub struct CharacterProfile {
    pub character_id: u32,
    pub character_name: String,
    pub portrait_url: String,
}

impl Default for CharacterProfile {
    fn default() -> Self {
        Self {
            character_id: 1,
            character_name: "Unknown Character".to_string(),
            portrait_url: "https://images.evetech.net/characters/1/portrait?size=128".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndustryJob {
    pub job_id: u32,
    pub installer_id: u32,
    pub facility_id: u64,
    pub station_id: u64,
    pub activity_id: u32,
    pub blueprint_id: u64,
    pub blueprint_type_id: u32,
    pub blueprint_location_id: u64,
    pub output_location_id: u64,
    pub runs: u32,
    pub cost: Option<f64>,
    pub licensed_runs: Option<u32>,
    pub probability: Option<f32>,
    pub product_type_id: Option<u32>,
    pub status: String,
    pub duration: u32,
    pub start_date: String,
    pub end_date: String,
    pub pause_date: Option<String>,
    pub completed_date: Option<String>,
    pub completed_character_id: Option<u32>,
    pub successful_runs: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UniverseName {
    pub id: u64,
    pub name: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blueprint {
    pub item_id: u64,
    pub type_id: u32,
    pub location_id: u64,
    pub location_flag: String,
    pub quantity: i32,
    pub time_efficiency: u32,
    pub material_efficiency: u32,
    pub runs: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    pub item_id: u64,
    pub type_id: u32,
    pub location_id: u64,
    pub location_flag: String,
    pub location_type: String,
    pub quantity: i32,
    pub is_singleton: bool,
    pub is_blueprint_copy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructureInfo {
    pub name: String,
    pub owner_id: u32,
    pub position: Option<Position>,
    pub solar_system_id: u32,
    pub type_id: Option<u32>,
}