use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::commands::esi::{esi_get_authed, parse_esi_response};
use crate::core::models::eve::{Asset, StructureInfo, UniverseName};
use crate::core::models::industry::IndustryStructure;
use crate::core::state::{AppState, CachedAssets};

const ASSETS_CACHE_TTL_SECS: u64 = 5 * 60;
const MAX_INT32: u64 = 2_147_483_647;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HaulableStockEntry {
    pub location_id: u64,
    pub location_name: String,
    pub quantity: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetLocationOption {
    pub location_id: u64,
    pub name: String,
    pub item_count: usize,
}

#[derive(Debug, Default)]
pub struct StockInventory {
    pub local: HashMap<u32, u64>,
    pub haulable: HashMap<u32, Vec<HaulableStockEntry>>,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn is_fitted_slot(flag: &str) -> bool {
    flag.starts_with("HiSlot")
        || flag.starts_with("MedSlot")
        || flag.starts_with("LoSlot")
        || flag.starts_with("RigSlot")
        || flag.starts_with("SubSystemSlot")
        || flag.starts_with("ServiceSlot")
        || flag == "DroneBay"
        || flag.starts_with("Fighter")
}

#[allow(clippy::unnecessary_cast)]
fn normalize_asset(a: &Asset) -> (u64, u64, u32, u64) {
    (
        a.item_id as u64,
        a.location_id as u64,
        a.type_id as u32,
        a.quantity.max(0) as u64,
    )
}

pub async fn fetch_cached_character_assets(
    app: &AppHandle,
    state: &AppState,
    character_id: u32,
    force_refresh: bool,
) -> Result<Vec<Asset>, String> {
    let now = unix_now();
    if !force_refresh {
        let cache = state.assets_cache.lock().await;
        if let Some(entry) = cache.get(&character_id)
            && now.saturating_sub(entry.updated_at) < ASSETS_CACHE_TTL_SECS
        {
            return Ok(entry.assets.clone());
        }
    }

    let url = format!(
        "https://esi.evetech.net/latest/characters/{}/assets/",
        character_id
    );
    let assets: Vec<Asset> = esi_get_authed(app, state, character_id, &url).await?;

    let mut cache = state.assets_cache.lock().await;
    cache.insert(
        character_id,
        CachedAssets {
            updated_at: now,
            assets: assets.clone(),
        },
    );

    Ok(assets)
}

pub async fn build_stock_inventory(
    app: &AppHandle,
    state: &AppState,
    character_id: u32,
    assets: &[Asset],
    selected_location_id: Option<u64>,
    configured_structures: &[IndustryStructure],
    relevant_type_ids: &HashSet<u32>,
) -> StockInventory {
    // Map item_id -> (parent_location_id, location_flag) to walk containers up to station/structure
    let mut parent_map: HashMap<u64, (u64, &str)> = HashMap::with_capacity(assets.len());
    for a in assets {
        let (item_id, loc_id, _, _) = normalize_asset(a);
        parent_map.insert(item_id, (loc_id, a.location_flag.as_str()));
    }

    let resolve_root = |start_loc: u64, flag: &str| -> Option<u64> {
        if is_fitted_slot(flag) {
            return None;
        }
        let mut curr = start_loc;
        let mut depth = 0;
        while let Some(&(parent_loc, parent_flag)) = parent_map.get(&curr) {
            if is_fitted_slot(parent_flag) || depth > 16 {
                return None;
            }
            curr = parent_loc;
            depth += 1;
        }
        Some(curr)
    };

    let mut local: HashMap<u32, u64> = HashMap::new();
    // type_id -> (location_id -> qty)
    let mut remote_by_type: HashMap<u32, HashMap<u64, u64>> = HashMap::new();
    let mut remote_locations: HashSet<u64> = HashSet::new();

    for a in assets {
        let (_, loc_id, type_id, qty) = normalize_asset(a);
        if qty == 0 || !relevant_type_ids.contains(&type_id) {
            continue;
        }

        let Some(root_loc) = resolve_root(loc_id, &a.location_flag) else {
            continue;
        };

        if selected_location_id == Some(root_loc) {
            *local.entry(type_id).or_insert(0) += qty;
        } else {
            *remote_by_type
                .entry(type_id)
                .or_default()
                .entry(root_loc)
                .or_insert(0) += qty;
            remote_locations.insert(root_loc);
        }
    }

    // Resolve names for remote haulable locations
    let mut loc_names: HashMap<u64, String> = HashMap::new();
    for s in configured_structures {
        if let Some(lid) = s.location_id {
            loc_names.insert(lid, s.name.clone());
        }
    }

    {
        let cached = state.structure_names_cache.lock().await;
        for &lid in &remote_locations {
            if !loc_names.contains_key(&lid)
                && let Some(name) = cached.get(&lid)
            {
                loc_names.insert(lid, name.clone());
            }
        }
    }

    let npc_ids: Vec<u64> = remote_locations
        .iter()
        .copied()
        .filter(|id| *id <= MAX_INT32 && !loc_names.contains_key(id))
        .collect();

    if !npc_ids.is_empty() {
        for chunk in npc_ids.chunks(500) {
            if let Ok(res) = state
                .http
                .post("https://esi.evetech.net/universe/names/")
                .json(&chunk)
                .send()
                .await
                && let Ok(names) = parse_esi_response::<Vec<UniverseName>>(res).await
            {
                for n in names {
                    loc_names.insert(n.id, n.name);
                }
            }
        }
    }

    let player_structure_ids: Vec<u64> = remote_locations
        .iter()
        .copied()
        .filter(|id| *id > MAX_INT32 && !loc_names.contains_key(id))
        .collect();

    for sid in player_structure_ids {
        let url = format!(
            "https://esi.evetech.net/latest/universe/structures/{}/",
            sid
        );
        let name = match esi_get_authed::<StructureInfo>(app, state, character_id, &url).await {
            Ok(info) => info.name,
            Err(_) => format!("Structure {}", sid),
        };
        loc_names.insert(sid, name.clone());
        let mut cached = state.structure_names_cache.lock().await;
        cached.insert(sid, name);
    }

    let mut haulable: HashMap<u32, Vec<HaulableStockEntry>> = HashMap::new();
    for (type_id, loc_map) in remote_by_type {
        let mut entries: Vec<HaulableStockEntry> = loc_map
            .into_iter()
            .map(|(location_id, quantity)| HaulableStockEntry {
                location_id,
                location_name: loc_names
                    .get(&location_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Location {}", location_id)),
                quantity,
            })
            .collect();
        entries.sort_by_key(|a| std::cmp::Reverse(a.quantity));
        haulable.insert(type_id, entries);
    }

    StockInventory { local, haulable }
}

pub async fn list_asset_locations(
    app: &AppHandle,
    state: &AppState,
    character_id: u32,
    force_refresh: bool,
) -> Result<Vec<AssetLocationOption>, String> {
    let assets = fetch_cached_character_assets(app, state, character_id, force_refresh).await?;

    let mut parent_map: HashMap<u64, (u64, &str)> = HashMap::with_capacity(assets.len());
    for a in &assets {
        let (item_id, loc_id, _, _) = normalize_asset(a);
        parent_map.insert(item_id, (loc_id, a.location_flag.as_str()));
    }

    let mut counts: HashMap<u64, usize> = HashMap::new();
    for a in &assets {
        if is_fitted_slot(&a.location_flag) {
            continue;
        }
        let (_, mut curr, _, _) = normalize_asset(a);
        let mut depth = 0;
        let mut valid = true;
        while let Some(&(parent_loc, parent_flag)) = parent_map.get(&curr) {
            if is_fitted_slot(parent_flag) || depth > 16 {
                valid = false;
                break;
            }
            curr = parent_loc;
            depth += 1;
        }
        if valid && curr > 0 {
            *counts.entry(curr).or_insert(0) += 1;
        }
    }

    let mut loc_names: HashMap<u64, String> = HashMap::new();
    {
        let cached = state.structure_names_cache.lock().await;
        for &lid in counts.keys() {
            if let Some(name) = cached.get(&lid) {
                loc_names.insert(lid, name.clone());
            }
        }
    }

    // Only send valid NPC station / solar system IDs to /universe/names/
    let npc_ids: Vec<u64> = counts
        .keys()
        .copied()
        .filter(|&id| {
            !loc_names.contains_key(&id)
                && ((30_000_000..=39_999_999).contains(&id)
                || (60_000_000..=64_000_000).contains(&id))
        })
        .collect();

    if !npc_ids.is_empty() {
        for chunk in npc_ids.chunks(500) {
            if let Ok(res) = state
                .http
                .post("https://esi.evetech.net/universe/names/")
                .json(&chunk)
                .send()
                .await
                && let Ok(names) = parse_esi_response::<Vec<UniverseName>>(res).await
            {
                let mut cached = state.structure_names_cache.lock().await;
                for n in names {
                    loc_names.insert(n.id, n.name.clone());
                    cached.insert(n.id, n.name);
                }
            }
        }
    }

    let player_structure_ids: Vec<u64> = counts
        .keys()
        .copied()
        .filter(|&id| id > MAX_INT32 && !loc_names.contains_key(&id))
        .collect();

    for sid in player_structure_ids {
        let url = format!(
            "https://esi.evetech.net/latest/universe/structures/{}/",
            sid
        );
        let name = match esi_get_authed::<StructureInfo>(app, state, character_id, &url).await {
            Ok(info) => info.name,
            Err(_) => format!("Restricted Structure ({})", sid),
        };
        loc_names.insert(sid, name.clone());
        let mut cached = state.structure_names_cache.lock().await;
        cached.insert(sid, name);
    }

    let mut locations: Vec<AssetLocationOption> = counts
        .into_iter()
        .map(|(location_id, item_count)| AssetLocationOption {
            location_id,
            name: loc_names
                .get(&location_id)
                .cloned()
                .unwrap_or_else(|| format!("Location {}", location_id)),
            item_count,
        })
        .collect();

    locations.sort_by_key(|a| a.name.to_lowercase());
    Ok(locations)
}