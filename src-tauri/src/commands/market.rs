use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::Client;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use tauri::State;

use crate::core::models::industry::PriceProvider;
use crate::core::models::market::{EsiMarketPrice, MarketPrice};
use crate::core::state::AppState;

const MARKET_CACHE_TTL_SECS: u64 = 15 * 60; // 15 minutes
const ESI_PRICES_TTL_SECS: u64 = 60 * 60;   // 60 minutes

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// Fuzzwork returns aggregate numbers as JSON strings ("5.95"), sometimes numbers
fn de_f64_flexible<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct FlexF64Visitor;

    impl<'de> Visitor<'de> for FlexF64Visitor {
        type Value = f64;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a float, integer, or numeric string")
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<f64, E> {
            Ok(v.parse::<f64>().unwrap_or(0.0))
        }
    }

    deserializer.deserialize_any(FlexF64Visitor)
}

// ============================================================================
// PROVIDER 1: FUZZWORK
// ============================================================================

#[derive(Deserialize)]
struct FuzzworkSide {
    #[serde(default, deserialize_with = "de_f64_flexible")]
    max: f64,
    #[serde(default, deserialize_with = "de_f64_flexible")]
    min: f64,
}

#[derive(Deserialize)]
struct FuzzworkAggregate {
    buy: FuzzworkSide,
    sell: FuzzworkSide,
}

async fn fetch_from_fuzzwork(
    http: &Client,
    region_id: u32,
    type_ids: &[u32],
    now: u64,
) -> Result<HashMap<u32, MarketPrice>, String> {
    let mut results = HashMap::new();

    for chunk in type_ids.chunks(200) {
        let ids_param = chunk
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let url = format!(
            "https://market.fuzzwork.co.uk/aggregates/?region={}&types={}",
            region_id, ids_param
        );

        let res = http
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Fuzzwork network error: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err = res.text().await.unwrap_or_default();
            return Err(format!("Fuzzwork API error ({}): {}", status, err));
        }

        let payload: HashMap<String, FuzzworkAggregate> = res
            .json()
            .await
            .map_err(|e| format!("Fuzzwork JSON parse error: {}", e))?;

        for (id_str, agg) in payload {
            if let Ok(type_id) = id_str.parse::<u32>() {
                results.insert(
                    type_id,
                    MarketPrice::from_buy_sell(agg.buy.max, agg.sell.min, now),
                );
            }
        }
    }

    Ok(results)
}

// ============================================================================
// PROVIDER 2: EVETYCOON
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EveTycoonStats {
    #[serde(default)]
    buy_avg_five_percent: f64,
    #[serde(default)]
    sell_avg_five_percent: f64,
    #[serde(default)]
    max_buy: f64,
    #[serde(default)]
    min_sell: f64,
}

async fn fetch_from_evetycoon(
    http: &Client,
    region_id: u32,
    type_ids: &[u32],
    now: u64,
) -> Result<HashMap<u32, MarketPrice>, String> {
    let mut results = HashMap::new();

    // Bounded concurrency (8 requests in flight at a time)
    for chunk in type_ids.chunks(8) {
        let mut handles = Vec::with_capacity(chunk.len());

        for &type_id in chunk {
            let client = http.clone();
            let url = format!(
                "https://evetycoon.com/api/v1/market/stats/{}/{}",
                region_id, type_id
            );

            handles.push(tauri::async_runtime::spawn(async move {
                let res = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("EVETycoon network error for {}: {}", type_id, e))?;

                if !res.status().is_success() {
                    let status = res.status();
                    let err = res.text().await.unwrap_or_default();
                    return Err(format!("EVETycoon API error ({}): {}", status, err));
                }

                let stats: EveTycoonStats = res
                    .json()
                    .await
                    .map_err(|e| format!("EVETycoon JSON parse error for {}: {}", type_id, e))?;

                let buy = if stats.buy_avg_five_percent > 0.0 {
                    stats.buy_avg_five_percent
                } else {
                    stats.max_buy
                };
                let sell = if stats.sell_avg_five_percent > 0.0 {
                    stats.sell_avg_five_percent
                } else {
                    stats.min_sell
                };

                Ok::<(u32, MarketPrice), String>((
                    type_id,
                    MarketPrice::from_buy_sell(buy, sell, now),
                ))
            }));
        }

        for handle in handles {
            let (type_id, price) = handle
                .await
                .map_err(|e| format!("EVETycoon task failed: {}", e))??;
            results.insert(type_id, price);
        }
    }

    Ok(results)
}

// ============================================================================
// PROVIDER 3: JANICE
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JaniceItemType {
    eid: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JaniceImmediatePrices {
    #[serde(default)]
    buy_price: f64,
    #[serde(default)]
    sell_price: f64,
    #[serde(default)]
    split_price: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JanicePricerItem {
    item_type: JaniceItemType,
    immediate_prices: JaniceImmediatePrices,
}

fn region_to_janice_market(region_id: u32) -> u32 {
    match region_id {
        10000002 => 2,   // Jita
        10000043 => 115, // Amarr
        10000030 => 116, // Rens
        10000032 => 117, // Dodixie
        10000042 => 118, // Hek
        _ => 2,
    }
}

async fn fetch_from_janice(
    http: &Client,
    region_id: u32,
    type_ids: &[u32],
    api_key: &str,
    now: u64,
) -> Result<HashMap<u32, MarketPrice>, String> {
    if api_key.trim().is_empty() {
        return Err("Janice API key is required in Industry Settings.".to_string());
    }

    let market_id = region_to_janice_market(region_id);
    let url = format!("https://janice.e-351.com/api/rest/v2/pricer?market={}", market_id);
    let mut results = HashMap::new();

    for chunk in type_ids.chunks(250) {
        let body = chunk
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let res = http
            .post(&url)
            .header("X-ApiKey", api_key.trim())
            .header("Content-Type", "text/plain")
            .header("Accept", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Janice network error: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let err = res.text().await.unwrap_or_default();
            return Err(format!("Janice API error ({}): {}", status, err));
        }

        let items: Vec<JanicePricerItem> = res
            .json()
            .await
            .map_err(|e| format!("Janice JSON parse error: {}", e))?;

        for item in items {
            let p = item.immediate_prices;
            let split = if p.split_price > 0.0 {
                p.split_price
            } else if p.buy_price > 0.0 && p.sell_price > 0.0 {
                (p.buy_price + p.sell_price) / 2.0
            } else {
                p.buy_price.max(p.sell_price)
            };

            results.insert(
                item.item_type.eid,
                MarketPrice {
                    buy: p.buy_price,
                    sell: p.sell_price,
                    split,
                    updated_at: now,
                },
            );
        }
    }

    Ok(results)
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

pub(crate) async fn get_market_prices_internal(
    state: &AppState,
    provider: PriceProvider,
    region_id: u32,
    type_ids: Vec<u32>,
    janice_api_key: Option<String>,
    force_refresh: bool,
) -> Result<HashMap<u32, MarketPrice>, String> {
    if type_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let now = unix_now();
    let unique_ids: Vec<u32> = type_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();

    let mut result = HashMap::with_capacity(unique_ids.len());
    let mut missing_ids = Vec::new();

    {
        let cache = state.market_cache.lock().await;
        for &type_id in &unique_ids {
            if !force_refresh
                && let Some(cached) = cache.get(&(provider, region_id, type_id))
                && now.saturating_sub(cached.updated_at) < MARKET_CACHE_TTL_SECS
            {
                result.insert(type_id, cached.clone());
                continue;
            }
            missing_ids.push(type_id);
        }
    }

    if !missing_ids.is_empty() {
        let fetched = match provider {
            PriceProvider::Fuzzwork => {
                fetch_from_fuzzwork(&state.http, region_id, &missing_ids, now).await?
            }
            PriceProvider::EveTycoon => {
                fetch_from_evetycoon(&state.http, region_id, &missing_ids, now).await?
            }
            PriceProvider::Janice => {
                let key = janice_api_key.unwrap_or_default();
                fetch_from_janice(&state.http, region_id, &missing_ids, &key, now).await?
            }
        };

        let mut cache = state.market_cache.lock().await;
        for &type_id in &missing_ids {
            let entry = fetched
                .get(&type_id)
                .cloned()
                .unwrap_or_else(|| MarketPrice::from_buy_sell(0.0, 0.0, now));
            cache.insert((provider, region_id, type_id), entry.clone());
            result.insert(type_id, entry);
        }
    }

    Ok(result)
}

pub(crate) async fn get_esi_adjusted_prices_internal(
    state: &AppState,
    force_refresh: bool,
) -> Result<HashMap<u32, EsiMarketPrice>, String> {
    let now = unix_now();

    {
        let cache = state.esi_prices_cache.lock().await;
        if !force_refresh
            && !cache.prices.is_empty()
            && now.saturating_sub(cache.updated_at) < ESI_PRICES_TTL_SECS
        {
            return Ok(cache.prices.clone());
        }
    }

    let res = state
        .http
        .get("https://esi.evetech.net/latest/markets/prices/")
        .send()
        .await
        .map_err(|e| format!("ESI market prices network error: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let err = res.text().await.unwrap_or_default();
        return Err(format!("ESI market prices error ({}): {}", status, err));
    }

    let list: Vec<EsiMarketPrice> = res
        .json()
        .await
        .map_err(|e| format!("ESI market prices JSON error: {}", e))?;

    let map: HashMap<u32, EsiMarketPrice> = list.into_iter().map(|p| (p.type_id, p)).collect();

    let mut cache = state.esi_prices_cache.lock().await;
    cache.updated_at = now;
    cache.prices = map.clone();

    Ok(map)
}

#[tauri::command]
pub async fn fetch_market_prices(
    state: State<'_, AppState>,
    provider: PriceProvider,
    region_id: u32,
    type_ids: Vec<u32>,
    janice_api_key: Option<String>,
    force_refresh: Option<bool>,
) -> Result<HashMap<u32, MarketPrice>, String> {
    get_market_prices_internal(
        &state,
        provider,
        region_id,
        type_ids,
        janice_api_key,
        force_refresh.unwrap_or(false),
    )
        .await
}

#[tauri::command]
pub async fn fetch_esi_adjusted_prices(
    state: State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<HashMap<u32, EsiMarketPrice>, String> {
    get_esi_adjusted_prices_internal(&state, force_refresh.unwrap_or(false)).await
}