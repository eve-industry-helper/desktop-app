use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::core::models::industry::{IndustryPlan, IndustrySettings, PlanEntryKind, RawMaterial};
use crate::core::models::sde::BlueprintActivity;
use crate::core::optimizer::stock::{list_asset_locations, AssetLocationOption};
use crate::core::optimizer::{run_optimizer, OptimizationResult, OptimizePlanRequest};
use crate::core::state::AppState;
use crate::core::storage::{read_json_or_default, write_json_atomic};

const SETTINGS_FILE: &str = "industry_settings.json";
const PLANS_FILE: &str = "plans.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintSearchMatch {
    pub blueprint_type_id: u32,
    pub blueprint_name: String,
    pub product_type_id: u32,
    pub product_name: String,
    pub activity: BlueprintActivity,
}

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
pub async fn search_sde_blueprints(
    app: AppHandle,
    state: State<'_, AppState>,
    query: String,
    kind: PlanEntryKind,
    limit: Option<usize>,
) -> Result<Vec<BlueprintSearchMatch>, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(Vec::new());
    }

    let target_activity = match kind {
        PlanEntryKind::Reaction => BlueprintActivity::Reaction,
        PlanEntryKind::Blueprint | PlanEntryKind::Fit => BlueprintActivity::Manufacturing,
    };

    let sde = crate::core::sde_service::ensure_sde_loaded(&app, &state).await?;
    let max_results = limit.unwrap_or(20).clamp(1, 100);

    let mut scored: Vec<(u8, usize, BlueprintSearchMatch)> = Vec::new();

    for bp in sde.blueprints.values() {
        if bp.activity != target_activity {
            continue;
        }

        let bp_type_name = sde.types.get(&bp.blueprint_type_id).map(|t| t.name.as_str());
        let prod_type_name = sde.types.get(&bp.product_type_id).map(|t| t.name.as_str());

        if bp_type_name.is_none() && prod_type_name.is_none() {
            continue;
        }

        let product_name = prod_type_name
            .or(bp_type_name)
            .unwrap_or_default()
            .to_string();

        let blueprint_name = bp_type_name
            .map(ToString::to_string)
            .unwrap_or_else(|| match bp.activity {
                BlueprintActivity::Reaction => format!("{} Reaction Formula", product_name),
                BlueprintActivity::Manufacturing => format!("{} Blueprint", product_name),
            });

        let bp_name_lower = blueprint_name.to_lowercase();
        let prod_name_lower = product_name.to_lowercase();
        if !bp_name_lower.contains(&q) && !prod_name_lower.contains(&q) {
            continue;
        }

        let rank: u8 = if bp_name_lower == q || prod_name_lower == q {
            0
        } else if bp_name_lower.starts_with(&q) || prod_name_lower.starts_with(&q) {
            1
        } else if bp_name_lower.split_whitespace().any(|w| w.starts_with(&q)) {
            2
        } else {
            3
        };

        let name_len = blueprint_name.len();
        scored.push((
            rank,
            name_len,
            BlueprintSearchMatch {
                blueprint_type_id: bp.blueprint_type_id,
                blueprint_name,
                product_type_id: bp.product_type_id,
                product_name,
                activity: bp.activity,
            },
        ));
    }

    scored.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.blueprint_name.cmp(&b.2.blueprint_name))
    });
    scored.truncate(max_results);

    Ok(scored.into_iter().map(|(_, _, m)| m).collect())
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

#[cfg(target_os = "linux")]
fn wsl_windows_downloads_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("WSL_DISTRO_NAME")?;

    let output = std::process::Command::new("cmd.exe")
        .current_dir("/mnt/c")
        .args(["/C", "echo", "%USERPROFILE%"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let win_profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if win_profile.is_empty() || win_profile.contains("%USERPROFILE%") {
        return None;
    }

    let wsl_out = std::process::Command::new("wslpath")
        .arg("-u")
        .arg(&win_profile)
        .output()
        .ok()?;

    if !wsl_out.status.success() {
        return None;
    }

    let linux_profile = String::from_utf8_lossy(&wsl_out.stdout).trim().to_string();
    let downloads = std::path::PathBuf::from(linux_profile).join("Downloads");
    downloads.exists().then_some(downloads)
}

#[tauri::command]
pub fn export_structure_json(
    app: AppHandle,
    filename: String,
    json_content: String,
) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    let wsl_dir = wsl_windows_downloads_dir();
    #[cfg(not(target_os = "linux"))]
    let wsl_dir: Option<std::path::PathBuf> = None;

    let dir = match wsl_dir {
        Some(d) => d,
        None => app
            .path()
            .download_dir()
            .or_else(|_| app.path().home_dir().map(|h| h.join("Downloads")))
            .or_else(|_| app.path().app_data_dir())
            .map_err(|e| format!("Could not resolve export directory: {}", e))?,
    };

    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to prepare export directory: {}", e))?;

    let safe_name: String = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let final_name = if safe_name.is_empty() || safe_name == ".json" {
        "structure.json".to_string()
    } else if safe_name.ends_with(".json") {
        safe_name
    } else {
        format!("{}.json", safe_name)
    };

    let path = dir.join(final_name);
    std::fs::write(&path, json_content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}