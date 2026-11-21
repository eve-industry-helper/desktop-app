use reqwest::Client;
use tauri::async_runtime::Mutex;

pub struct AppState {
    pub http: Client,
    pub auth_lock: Mutex<()>,
}

impl AppState {
    pub fn new() -> Self {
        let http = Client::builder()
            .user_agent("eve-industry-helper/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            auth_lock: Mutex::new(()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}