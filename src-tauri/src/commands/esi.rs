use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use reqwest::Response;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, State};

use crate::core::auth_service::get_valid_access_token;
use crate::core::models::eve::{
    Asset,
    Blueprint,
    CharacterProfile,
    CharacterSkillsResponse,
    IndustryJob,
    JwtPayload,
    StructureInfo,
    UniverseIdEntry,
    UniverseIdsResponse,
    UniverseName,
};
use crate::core::state::AppState;

async fn parse_esi_response<T: DeserializeOwned>(res: Response) -> Result<T, String> {
    if res.status().is_success() {
        res.json::<T>()
            .await
            .map_err(|e| format!("JSON parsing failed: {}", e))
    } else {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error (Status {}): {}", status, err_text))
    }
}

async fn esi_get_authed<T: DeserializeOwned>(
    app: &AppHandle,
    state: &AppState,
    character_id: u32,
    url: &str,
) -> Result<T, String> {
    let access_token = get_valid_access_token(app, state, character_id).await?;
    let res = state
        .http
        .get(url)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    parse_esi_response(res).await
}

#[tauri::command]
pub async fn resolve_eve_names(
    state: State<'_, AppState>,
    ids: Vec<u64>,
) -> Result<Vec<UniverseName>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let res = state
        .http
        .post("https://esi.evetech.net/universe/names/")
        .json(&ids)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    parse_esi_response(res).await
}

#[tauri::command]
pub fn get_active_character(access_token: String) -> Result<CharacterProfile, String> {
    let parts: Vec<&str> = access_token.split('.').collect();
    if parts.len() != 3 {
        return Ok(CharacterProfile::default());
    }

    let mut payload = parts[1].to_string();
    while !payload.len().is_multiple_of(4) {
        payload.push('=');
    }

    let decoded = URL_SAFE
        .decode(&payload)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let json: JwtPayload = match serde_json::from_slice(&decoded) {
        Ok(j) => j,
        Err(_) => return Ok(CharacterProfile::default()),
    };

    let character_name = json.name.unwrap_or_else(|| "Unknown Character".to_string());
    let sub = json.sub.unwrap_or_else(|| "0".to_string());
    let character_id = sub
        .split(':')
        .next_back()
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);

    let portrait_id = if character_id > 0 { character_id } else { 1 };

    Ok(CharacterProfile {
        character_id,
        character_name,
        portrait_url: format!(
            "https://images.evetech.net/characters/{}/portrait?size=128",
            portrait_id
        ),
    })
}

#[tauri::command]
pub async fn get_industry_jobs(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
) -> Result<Vec<IndustryJob>, String> {
    let url = format!(
        "https://esi.evetech.net/latest/characters/{}/industry/jobs/",
        character_id
    );
    esi_get_authed(&app, &state, character_id, &url).await
}

#[tauri::command]
pub async fn get_blueprints(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
) -> Result<Vec<Blueprint>, String> {
    let url = format!(
        "https://esi.evetech.net/latest/characters/{}/blueprints/",
        character_id
    );
    esi_get_authed(&app, &state, character_id, &url).await
}

#[tauri::command]
pub async fn get_character_assets(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
) -> Result<Vec<Asset>, String> {
    let url = format!(
        "https://esi.evetech.net/latest/characters/{}/assets/",
        character_id
    );
    esi_get_authed(&app, &state, character_id, &url).await
}

#[tauri::command]
pub async fn get_structure_name(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
    structure_id: u64,
) -> Result<StructureInfo, String> {
    let url = format!(
        "https://esi.evetech.net/latest/universe/structures/{}/",
        structure_id
    );
    esi_get_authed(&app, &state, character_id, &url).await
}

#[tauri::command]
pub async fn resolve_eve_ids(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> Result<Vec<UniverseIdEntry>, String> {
    if names.is_empty() {
        return Ok(vec![]);
    }

    let res = state
        .http
        .post("https://esi.evetech.net/latest/universe/ids/")
        .json(&names)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: UniverseIdsResponse = parse_esi_response(res).await?;
    Ok(data.inventory_types.unwrap_or_default())
}

#[tauri::command]
pub async fn get_character_skills(
    app: AppHandle,
    state: State<'_, AppState>,
    character_id: u32,
) -> Result<CharacterSkillsResponse, String> {
    let url = format!(
        "https://esi.evetech.net/latest/characters/{}/skills/",
        character_id
    );
    esi_get_authed(&app, &state, character_id, &url).await
}