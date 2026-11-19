use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum AppMode {
    Standalone,
    Alliance,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EveToken {
    pub character_id: u32,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub is_setup: bool,
    pub mode: AppMode,
    pub api_url: Option<String>,
    pub tokens: Vec<EveToken>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            is_setup: false,
            mode: AppMode::Standalone,
            api_url: None,
            tokens: Vec::new(),
        }
    }
}