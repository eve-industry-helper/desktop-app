use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use reqwest::Client;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tiny_http::{Response, Server};
use url::Url;

use crate::core::models::auth::TokenResponse;

const CLIENT_ID: &str = "58cad1f6ea184f809a79d3d2d6311aa3";
const CALLBACK_URL: &str = "http://localhost:24443/callback";
const AUTH_URL: &str = "https://login.eveonline.com/v2/oauth/authorize";
const TOKEN_URL: &str = "https://login.eveonline.com/v2/oauth/token";

#[tauri::command]
pub async fn login_eve_standalone(app: AppHandle) -> Result<TokenResponse, String> {
    // 1. Generate the PKCE Verifier and Challenge
    let mut verifier_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // 2. Construct the EVE SSO URL with your requested scopes
    let scopes = "esi-skills.read_skills.v1 esi-assets.read_assets.v1 esi-industry.read_character_jobs.v1 esi-characters.read_blueprints.v1 esi-universe.read_structures.v1";

    let mut auth_url = Url::parse(AUTH_URL).unwrap();
    auth_url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", CALLBACK_URL)
        .append_pair("client_id", CLIENT_ID)
        .append_pair("scope", scopes)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", "auth_state_dev"); // Used for CSRF protection

    // 3. Open the user's default web browser
    app.opener().open_url(auth_url.as_str(), None::<&str>).map_err(|e| e.to_string())?;

    // 4. Start a tiny local web server on a background thread to catch the redirect
    let auth_code = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let server = Server::http("127.0.0.1:24443").map_err(|e| e.to_string())?;

        for request in server.incoming_requests() {
            let url = format!("http://localhost{}", request.url());
            let parsed_url = Url::parse(&url).map_err(|e| e.to_string())?;

            if parsed_url.path() == "/callback" {
                let code = parsed_url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned());

                let response_text = if code.is_some() {
                    "Authentication successful! You can safely close this tab and return to the EVE Industry Helper."
                } else {
                    "Authentication failed! No code returned."
                };

                let response = Response::from_string(response_text);
                let _ = request.respond(response);

                if let Some(c) = code {
                    return Ok(c);
                }
            }
        }
        Err("Local server shut down unexpectedly".into())
    })
        .await
        .map_err(|e| e.to_string())??; // Double unwrap to handle both the thread Result and the HTTP Result

    // 5. Exchange the auth code and the original verifier for the actual tokens
    let client = Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("code", &auth_code),
        ("code_verifier", &code_verifier),
    ];

    let res = client.post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let token_res = res.json::<TokenResponse>().await.map_err(|e| e.to_string())?;
        Ok(token_res) // Return the tokens to Svelte
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Token exchange failed: {}", err_text))
    }
}

#[tauri::command]
pub async fn refresh_eve_token(refresh_token: String) -> Result<TokenResponse, String> {
    let client = Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", CLIENT_ID), // Uses the same CLIENT_ID defined at the top of auth.rs
        ("refresh_token", &refresh_token),
    ];

    let res = client.post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let token_res = res.json::<TokenResponse>().await.map_err(|e| e.to_string())?;
        Ok(token_res)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Token refresh failed: {}", err_text))
    }
}