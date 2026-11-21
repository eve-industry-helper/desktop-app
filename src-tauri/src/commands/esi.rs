use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use reqwest::Client;

use crate::core::models::eve::{
    Asset,
    Blueprint,
    CharacterProfile,
    IndustryJob,
    JwtPayload,
    UniverseName,
    StructureInfo,
    UniverseIdsResponse,
    UniverseIdEntry
};

#[tauri::command]
pub async fn resolve_eve_names(ids: Vec<u64>) -> Result<Vec<UniverseName>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let url = "https://esi.evetech.net/universe/names/";
    let client = Client::new();

    // The names endpoint takes a POST request with a JSON array of IDs
    let res = client.post(url)
        .json(&ids)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let names: Vec<UniverseName> = res.json().await.map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(names)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error: {}", err_text))
    }
}

#[tauri::command]
pub fn get_active_character(access_token: String) -> Result<CharacterProfile, String> {
    let parts: Vec<&str> = access_token.split('.').collect();
    if parts.len() != 3 {
        return Ok(CharacterProfile::default());
    }

    let mut payload = parts[1].to_string();
    while payload.len() % 4 != 0 {
        payload.push('=');
    }

    let decoded = URL_SAFE.decode(&payload)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    // If the JSON is totally mangled, just return the defaults
    let json: JwtPayload = match serde_json::from_slice(&decoded) {
        Ok(j) => j,
        Err(_) => return Ok(CharacterProfile::default()),
    };

    let character_name = json.name.unwrap_or_else(|| "Unknown Character".to_string());
    let sub = json.sub.unwrap_or_else(|| "0".to_string());

    let id_str = sub.split(':').last().unwrap_or("0");
    let character_id = id_str.parse::<u32>().unwrap_or(0);

    // Ensure we don't request an invalid portrait ID
    let portrait_id = if character_id > 0 { character_id } else { 1 };

    Ok(CharacterProfile {
        character_id,
        character_name,
        portrait_url: format!("https://images.evetech.net/characters/{}/portrait?size=128", portrait_id),
    })
}

#[tauri::command]
pub async fn get_industry_jobs(character_id: u32, access_token: String) -> Result<Vec<IndustryJob>, String> {
    let url = format!("https://esi.evetech.net/latest/characters/{}/industry/jobs/", character_id);

    let client = Client::new();
    let res = client.get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let jobs: Vec<IndustryJob> = res.json().await.map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(jobs)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error: {}", err_text))
    }
}

#[tauri::command]
pub async fn get_blueprints(character_id: u32, access_token: String) -> Result<Vec<Blueprint>, String> {
    let url = format!("https://esi.evetech.net/latest/characters/{}/blueprints/", character_id);
    let client = Client::new();
    let res = client.get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let blueprints: Vec<Blueprint> = res.json().await.map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(blueprints)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error: {}", err_text))
    }
}

#[tauri::command]
pub async fn get_character_assets(character_id: u32, access_token: String) -> Result<Vec<Asset>, String> {
    let url = format!("https://esi.evetech.net/latest/characters/{}/assets/", character_id);

    let client = Client::new();
    let res = client.get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let assets: Vec<Asset> = res.json().await.map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(assets)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error: {}", err_text))
    }
}

#[tauri::command]
pub async fn get_structure_name(structure_id: u64, access_token: String) -> Result<StructureInfo, String> { // Return the full struct
    let url = format!("https://esi.evetech.net/latest/universe/structures/{}/", structure_id);
    let client = Client::new();

    let res = client.get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let info: StructureInfo = res.json().await.map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(info)
    } else {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error (Status {}): {}", status, err_text))
    }
}

#[tauri::command]
pub async fn resolve_eve_ids(names: Vec<String>) -> Result<Vec<UniverseIdEntry>, String> {
    if names.is_empty() {
        return Ok(vec![]);
    }

    let url = "https://esi.evetech.net/latest/universe/ids/";
    let client = Client::new();

    let res = client
        .post(url)
        .json(&names)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if res.status().is_success() {
        let data: UniverseIdsResponse = res
            .json()
            .await
            .map_err(|e| format!("JSON parsing failed: {}", e))?;
        Ok(data.inventory_types.unwrap_or_default())
    } else {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("ESI API error (Status {}): {}", status, err_text))
    }
}