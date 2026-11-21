use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tiny_http::{Response, Server};
use url::Url;

use crate::core::auth_service::{CLIENT_ID, TOKEN_URL};
use crate::core::models::auth::TokenResponse;
use crate::core::state::AppState;

const CALLBACK_URL: &str = "http://localhost:24443/callback";
const AUTH_URL: &str = "https://login.eveonline.com/v2/oauth/authorize";

#[tauri::command]
pub async fn login_eve_standalone(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TokenResponse, String> {
    let mut verifier_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    let scopes = "esi-skills.read_skills.v1 esi-assets.read_assets.v1 esi-industry.read_character_jobs.v1 esi-characters.read_blueprints.v1 esi-universe.read_structures.v1";

    let mut auth_url = Url::parse(AUTH_URL).unwrap();
    auth_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", CALLBACK_URL)
        .append_pair("client_id", CLIENT_ID)
        .append_pair("scope", scopes)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", "auth_state_dev");

    app.opener()
        .open_url(auth_url.as_str(), None::<&str>)
        .map_err(|e| e.to_string())?;

    let auth_code = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let server = Server::http("127.0.0.1:24443").map_err(|e| e.to_string())?;

        for request in server.incoming_requests() {
            let url = format!("http://localhost{}", request.url());
            let parsed_url = Url::parse(&url).map_err(|e| e.to_string())?;

            if parsed_url.path() == "/callback" {
                let code = parsed_url
                    .query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned());

                let response_text = if code.is_some() {
                    "Authentication successful! You can safely close this tab and return to the EVE Industry Helper."
                } else {
                    "Authentication failed! No code returned."
                };

                let _ = request.respond(Response::from_string(response_text));

                if let Some(c) = code {
                    return Ok(c);
                }
            }
        }
        Err("Local server shut down unexpectedly".into())
    })
        .await
        .map_err(|e| e.to_string())??;

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", CLIENT_ID),
        ("code", &auth_code),
        ("code_verifier", &code_verifier),
    ];

    let res = state
        .http
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        res.json::<TokenResponse>().await.map_err(|e| e.to_string())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Token exchange failed: {}", err_text))
    }
}