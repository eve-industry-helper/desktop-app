use tauri::{AppHandle, State};

use crate::core::models::sde::{SdeIndex, SdeStatus};
use crate::core::sde_service::{sync_sde_index, SDE_INDEX_FILE};
use crate::core::state::AppState;
use crate::core::storage::read_json_or_default;

#[tauri::command]
pub async fn sync_sde(
    app: AppHandle,
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<SdeStatus, String> {
    let index = sync_sde_index(&app, &state, force.unwrap_or(false)).await?;
    Ok(SdeStatus::from(&index))
}

#[tauri::command]
pub async fn get_sde_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SdeStatus, String> {
    let mut guard = state.sde.lock().await;
    if let Some(idx) = guard.as_ref() {
        return Ok(SdeStatus::from(idx));
    }

    let disk: SdeIndex = read_json_or_default(&app, SDE_INDEX_FILE).unwrap_or_default();
    if !disk.blueprints.is_empty() {
        let status = SdeStatus::from(&disk);
        *guard = Some(disk);
        return Ok(status);
    }

    Ok(SdeStatus::from(&disk))
}