use std::collections::HashMap;
use tauri::{AppHandle, State};

use crate::core::models::industry::{IndustryPlan, IndustrySettings, RawMaterial, SdeBlueprint};
use crate::core::state::AppState;
use crate::core::storage::{read_json_or_default, write_json_atomic};

const SETTINGS_FILE: &str = "industry_settings.json";
const PLANS_FILE: &str = "plans.json";
const BLUEPRINTS_CACHE_FILE: &str = "blueprints_cache.json";

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

    let mut cache: HashMap<u32, Vec<RawMaterial>> =
        read_json_or_default(&app, BLUEPRINTS_CACHE_FILE).unwrap_or_default();

    let mut cache_updated = false;
    let mut result: HashMap<u32, Vec<RawMaterial>> = HashMap::new();

    for bp_id in blueprint_ids {
        if let Some(cached) = cache.get(&bp_id) {
            result.insert(bp_id, cached.clone());
            continue;
        }

        let url = format!(
            "https://sde.evetech.net/tranquility/latest/blueprints/{}.json",
            bp_id
        );

        let res = state
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error fetching blueprint {}: {}", bp_id, e))?;

        if res.status().is_success() {
            let sde_bp: SdeBlueprint = res
                .json()
                .await
                .map_err(|e| format!("Failed to parse blueprint {}: {}", bp_id, e))?;

            let mats: Vec<RawMaterial> = sde_bp
                .activities
                .manufacturing
                .or(sde_bp.activities.reaction)
                .map(|act| act.materials)
                .unwrap_or_default()
                .into_iter()
                .map(|m| RawMaterial {
                    material_type_id: m.type_id,
                    quantity: m.quantity,
                })
                .collect();

            cache.insert(bp_id, mats.clone());
            result.insert(bp_id, mats);
            cache_updated = true;
        } else {
            result.insert(bp_id, vec![]);
        }
    }

    if cache_updated {
        let _ = write_json_atomic(&app, BLUEPRINTS_CACHE_FILE, &cache);
    }

    Ok(result)
}