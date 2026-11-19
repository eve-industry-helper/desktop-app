mod commands;
mod core;

use crate::commands::config::{save_config, load_config};
use crate::commands::auth::{login_eve_standalone, refresh_eve_token};
use crate::commands::esi::{
    get_active_character, get_industry_jobs, resolve_eve_names,
    get_blueprints, get_character_assets
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            login_eve_standalone,
            get_active_character,
            get_industry_jobs,
            resolve_eve_names,
            refresh_eve_token,
            get_blueprints,
            get_character_assets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}