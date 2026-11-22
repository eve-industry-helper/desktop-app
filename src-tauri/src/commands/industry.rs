use std::collections::HashMap;
use tauri::{AppHandle, State};

use crate::core::models::industry::{IndustryPlan, IndustrySettings, RawMaterial};
use crate::core::state::AppState;
use crate::core::storage::{read_json_or_default, write_json_atomic};
use crate::core::optimizer::{run_optimizer, OptimizationResult, OptimizePlanRequest};
use crate::core::optimizer::stock::{list_asset_locations, AssetLocationOption};
const SETTINGS_FILE: &str = "industry_settings.json";
const PLANS_FILE: &str = "plans.json";

#[tauri::command]
pub fn save_industry_settings(app: AppHandle, settings: IndustrySettings) -> Result<(), String> {
    write_json_atomic(&app, SETTINGS_FILE, &settings)
}

#[tauri::command]
pub fn load_industry_settings(app: AppHandle) -> Result<IndustrySettings, String> {
    read_json_or_default(&app, SETTINGS_FILE)
}

#[tauri::command]
pub fn save_plans(app: AppHandle, plans: Vec<IndustryPlan>) -> Result<(), String> {
    write_json_atomic(&app, PLANS_FILE, &plans)
}

#[tauri::command]
pub fn load_plans(app: AppHandle) -> Result<Vec<IndustryPlan>, String> {
    read_json_or_default(&app, PLANS_FILE)
}

#[tauri::command]
pub async fn get_blueprint_materials(
    app: AppHandle,
    state: State<'_, AppState>,
    blueprint_ids: Vec<u32>,
) -> Result<HashMap<u32, Vec<RawMaterial>>, String> {
    if blueprint_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let sde = crate::core::sde_service::ensure_sde_loaded(&app, &state).await?;
    let mut result = HashMap::with_capacity(blueprint_ids.len());

    for bp_id in blueprint_ids {
        let mats = sde
            .blueprints
            .get(&bp_id)
            .map(|bp| bp.materials.clone())
            .unwrap_or_default();
        result.insert(bp_id, mats);
    }

    Ok(result)
}

#[tauri::command]
pub async fn optimize_plan(
    app: AppHandle,
    state: State<'_, AppState>,
    request: OptimizePlanRequest,
) -> Result<OptimizationResult, String> {
    run_optimizer(&app, &state, request).await
}

#[tauri::command]
pub async fn list_character_asset_locations(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
    force_refresh: Option<bool>,
) -> Result<Vec<AssetLocationOption>, String> {
    list_asset_locations(&app, &state, character_id, force_refresh.unwrap_or(false)).await
}