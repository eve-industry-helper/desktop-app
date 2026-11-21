use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use reqwest::Client;
use tauri::{AppHandle, Manager};
use crate::core::models::industry::{
    IndustryPlan,
    RawMaterial,
    SdeBlueprint,
    IndustrySettings
};

fn get_settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    Ok(dir.join("industry_settings.json"))
}

#[tauri::command]
pub fn save_industry_settings(
    app: AppHandle,
    settings: IndustrySettings,
) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize industry settings: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("Failed to write industry_settings.json: {}", e))
}

#[tauri::command]
pub fn load_industry_settings(app: AppHandle) -> Result<IndustrySettings, String> {
    let path = get_settings_path(&app)?;

    if !path.exists() {
        return Ok(IndustrySettings::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read industry_settings.json: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse industry_settings.json: {}", e))
}

fn get_plans_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    Ok(dir.join("plans.json"))
}

#[tauri::command]
pub fn save_plans(app: AppHandle, plans: Vec<IndustryPlan>) -> Result<(), String> {
    let path = get_plans_path(&app)?;
    let json = serde_json::to_string_pretty(&plans)
        .map_err(|e| format!("Failed to serialize plans: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write plans.json: {}", e))
}

#[tauri::command]
pub fn load_plans(app: AppHandle) -> Result<Vec<IndustryPlan>, String> {
    let path = get_plans_path(&app)?;

    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read plans.json: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse plans.json: {}", e))
}

pub(crate) fn get_cache_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    Ok(dir.join("blueprints_cache.json"))
}

#[tauri::command]
pub async fn get_blueprint_materials(
    app: AppHandle,
    blueprint_ids: Vec<u32>,
) -> Result<HashMap<u32, Vec<RawMaterial>>, String> {
    if blueprint_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let cache_path = get_cache_path(&app)?;
    let mut cache: HashMap<u32, Vec<RawMaterial>> = if cache_path.exists() {
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    let client = Client::new();
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

        let res = client
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
            // Leave empty if a blueprint has no SDE entry rather than failing the whole plan
            result.insert(bp_id, vec![]);
        }
    }

    if cache_updated {
        if let Ok(json) = serde_json::to_string_pretty(&cache) {
            let _ = fs::write(&cache_path, json);
        }
    }

    Ok(result)
}