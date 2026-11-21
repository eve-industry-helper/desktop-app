use tauri::AppHandle;

use crate::core::models::app_config::AppConfig;
use crate::core::storage::{read_json_or_default, write_json_atomic, CONFIG_FILE};

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    write_json_atomic(&app, CONFIG_FILE, &config)
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    read_json_or_default(&app, CONFIG_FILE)
}