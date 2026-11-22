use std::collections::HashMap;
use reqwest::Client;
use tauri::async_runtime::Mutex;

use crate::core::models::eve::Asset;
use crate::core::models::industry::PriceProvider;
use crate::core::models::market::{EsiMarketPrice, MarketPrice};
use crate::core::models::sde::SdeIndex;

pub type MarketCacheKey = (PriceProvider, u32, u32);

#[derive(Default)]
pub struct EsiPriceCache {
    pub updated_at: u64,
    pub prices: HashMap<u32, EsiMarketPrice>,
}

#[derive(Clone)]
pub struct CachedAssets {
    pub updated_at: u64,
    pub assets: Vec<Asset>,
}

pub struct AppState {
    pub http: Client,
    pub auth_lock: Mutex<()>,
    pub market_cache: Mutex<HashMap<MarketCacheKey, MarketPrice>>,
    pub esi_prices_cache: Mutex<EsiPriceCache>,
    pub sde: Mutex<Option<SdeIndex>>,
    pub assets_cache: Mutex<HashMap<u32, CachedAssets>>,
    pub structure_names_cache: Mutex<HashMap<u64, String>>,
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
            market_cache: Mutex::new(HashMap::new()),
            esi_prices_cache: Mutex::new(EsiPriceCache::default()),
            sde: Mutex::new(None),
            assets_cache: Mutex::new(HashMap::new()),
            structure_names_cache: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}