use std::fs;
use tauri::{AppHandle, Manager};
use crate::core::models::app_config::AppConfig;

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let config_path = app_dir.join("settings.json");
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    fs::write(config_path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = app_dir.join("settings.json");

    if config_path.exists() {
        let json = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
        let config: AppConfig = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}