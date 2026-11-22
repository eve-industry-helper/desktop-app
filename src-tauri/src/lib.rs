mod commands;
mod core;

use crate::commands::auth::{login_eve_standalone};
use crate::commands::config::{load_config, save_config};
use crate::commands::esi::{
    get_active_character,
    get_blueprints,
    get_character_assets,
    get_character_skills,
    get_industry_jobs,
    get_structure_name,
    resolve_eve_ids,
    resolve_eve_names,
};
use crate::commands::industry::{
    get_blueprint_materials,
    load_industry_settings,
    load_plans,
    save_industry_settings,
    save_plans,
    optimize_plan,
    list_character_asset_locations
};
use crate::commands::market::{
    fetch_market_prices,
    fetch_esi_adjusted_prices
};
use crate::commands::sde::{
    sync_sde,
    get_sde_status
};
use crate::core::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_config,
            load_config,
            login_eve_standalone,
            get_active_character,
            get_industry_jobs,
            resolve_eve_names,
            get_blueprints,
            get_character_assets,
            get_structure_name,
            resolve_eve_ids,
            load_plans,
            save_plans,
            get_blueprint_materials,
            get_character_skills,
            save_industry_settings,
            load_industry_settings,
            fetch_esi_adjusted_prices,
            fetch_market_prices,
            sync_sde,
            get_sde_status,
            optimize_plan,
            list_character_asset_locations
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}