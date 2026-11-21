use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use tauri::AppHandle;

use crate::core::models::app_config::AppConfig;
use crate::core::models::auth::TokenResponse;
use crate::core::state::AppState;
use crate::core::storage::{read_json_or_default, write_json_atomic, CONFIG_FILE};

pub const CLIENT_ID: &str = "58cad1f6ea184f809a79d3d2d6311aa3";
pub const TOKEN_URL: &str = "https://login.eveonline.com/v2/oauth/token";

pub async fn refresh_token_request(
    http: &Client,
    refresh_token: &str,
) -> Result<TokenResponse, String> {
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", CLIENT_ID),
        ("refresh_token", refresh_token),
    ];

    let res = http
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Network error during token refresh: {}", e))?;

    if res.status().is_success() {
        res.json::<TokenResponse>()
            .await
            .map_err(|e| format!("Failed to parse token refresh response: {}", e))
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Token refresh failed: {}", err_text))
    }
}

pub async fn get_valid_access_token(
    app: &AppHandle,
    state: &AppState,
    character_id: u32,
) -> Result<String, String> {
    // Serialize checks/refreshes so concurrent ESI calls don't double-refresh
    let _guard = state.auth_lock.lock().await;

    let mut config: AppConfig = read_json_or_default(app, CONFIG_FILE)?;
    let token = config
        .tokens
        .iter_mut()
        .find(|t| t.character_id == character_id)
        .ok_or_else(|| format!("Authentication token missing for character {}.", character_id))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .as_secs();

    if token.expires_at > now.saturating_add(60) {
        return Ok(token.access_token.clone());
    }

    let refresh_token = token
        .refresh_token
        .clone()
        .ok_or_else(|| "Session expired and no refresh token is available.".to_string())?;

    let refreshed = refresh_token_request(&state.http, &refresh_token).await?;
    let new_access_token = refreshed.access_token.clone();

    token.access_token = refreshed.access_token;
    if let Some(new_refresh) = refreshed.refresh_token {
        token.refresh_token = Some(new_refresh);
    }
    token.expires_at = now + refreshed.expires_in;

    write_json_atomic(app, CONFIG_FILE, &config)?;
    Ok(new_access_token)
}