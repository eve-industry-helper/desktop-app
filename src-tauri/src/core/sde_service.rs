use std::collections::HashMap;
use reqwest::{header, Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::core::models::industry::RawMaterial;
use crate::core::models::sde::{
    BlueprintActivity,
    SdeBlueprintEntry,
    SdeGroupInfo,
    SdeIndex,
    SdeTypeInfo,
};
use crate::core::state::AppState;
use crate::core::storage::{read_json_or_default, write_json_compact_atomic};

pub const SDE_INDEX_FILE: &str = "sde_index.json";
const SDE_LATEST_URL: &str = "https://sde.evetech.net/tranquility/latest/";

// ============================================================================
// RAW JSONL DESERIALIZERS
// ============================================================================

#[derive(Deserialize)]
#[serde(untagged)]
enum LocalizedText {
    Plain(String),
    Localized {
        #[serde(default)]
        en: String,
    },
}

impl LocalizedText {
    fn into_en(self) -> String {
        match self {
            LocalizedText::Plain(s) => s,
            LocalizedText::Localized { en } => en,
        }
    }
}

#[derive(Deserialize)]
struct RawBpQuantity {
    #[serde(rename = "typeID")]
    type_id: u32,
    quantity: u32,
}

#[derive(Deserialize)]
struct RawBpActivity {
    #[serde(default)]
    time: u32,
    #[serde(default)]
    materials: Vec<RawBpQuantity>,
    #[serde(default)]
    products: Vec<RawBpQuantity>,
}

#[derive(Deserialize)]
struct RawBpActivities {
    manufacturing: Option<RawBpActivity>,
    reaction: Option<RawBpActivity>,
}

#[derive(Deserialize)]
struct RawBlueprintLine {
    #[serde(rename = "blueprintTypeID", alias = "_key")]
    blueprint_type_id: u32,
    activities: RawBpActivities,
}

#[derive(Deserialize)]
struct RawReprocessMat {
    #[serde(rename = "materialTypeID")]
    material_type_id: u32,
    quantity: u32,
}

#[derive(Deserialize)]
struct RawTypeMaterialsLine {
    #[serde(rename = "typeID", alias = "_key")]
    type_id: u32,
    #[serde(default)]
    materials: Vec<RawReprocessMat>,
}

#[derive(Deserialize)]
struct RawTypeLine {
    #[serde(rename = "typeID", alias = "_key")]
    type_id: u32,
    #[serde(rename = "groupID", default)]
    group_id: u32,
    #[serde(rename = "portionSize", default = "default_portion_size")]
    portion_size: u32,
    #[serde(default)]
    volume: f64,
    #[serde(rename = "metaGroupID", default)]
    meta_group_id: Option<u32>,
    #[serde(default)]
    published: bool,
    name: Option<LocalizedText>,
}

fn default_portion_size() -> u32 {
    1
}

#[derive(Deserialize)]
struct RawGroupLine {
    #[serde(rename = "groupID", alias = "_key")]
    group_id: u32,
    #[serde(rename = "categoryID", default)]
    category_id: u32,
    name: Option<LocalizedText>,
}

// ============================================================================
// SYNC & INDEX BUILDER
// ============================================================================

struct ReleaseManifest {
    release: String,
    etag: Option<String>,
    base_url: String,
    table_urls: HashMap<String, String>,
}

fn extract_table_urls(body: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(tables) = body.get("tables") else {
        return map;
    };

    if let Some(obj) = tables.as_object() {
        for (name, entry) in obj {
            if let Some(url) = entry
                .get("jsonl")
                .and_then(|j| j.get("url"))
                .and_then(|u| u.as_str())
            {
                map.insert(name.clone(), url.to_string());
            }
        }
    } else if let Some(arr) = tables.as_array() {
        for entry in arr {
            if let (Some(name), Some(url)) = (
                entry.get("name").and_then(|n| n.as_str()),
                entry
                    .get("jsonl")
                    .and_then(|j| j.get("url"))
                    .and_then(|u| u.as_str()),
            ) {
                map.insert(name.to_string(), url.to_string());
            }
        }
    }

    map
}

async fn check_latest_release(
    http: &Client,
    current_etag: Option<&str>,
) -> Result<Option<ReleaseManifest>, String> {
    let mut req = http.get(SDE_LATEST_URL);
    if let Some(etag) = current_etag {
        req = req.header(header::IF_NONE_MATCH, etag);
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("Failed to reach SDE API: {}", e))?;

    if res.status() == StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    if !res.status().is_success() {
        let status = res.status();
        let err = res.text().await.unwrap_or_default();
        return Err(format!("SDE manifest error ({}): {}", status, err));
    }

    let etag = res
        .headers()
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut base_url = res.url().as_str().to_string();
    if !base_url.ends_with('/') {
        base_url.push('/');
    }

    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse SDE manifest JSON: {}", e))?;

    let release = body
        .get("release")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            body.get("buildNumber")
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
        })
        .unwrap_or_else(|| {
            base_url
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("unknown")
                .to_string()
        });

    let table_urls = extract_table_urls(&body);

    Ok(Some(ReleaseManifest {
        release,
        etag,
        base_url,
        table_urls,
    }))
}

async fn download_table(http: &Client, manifest: &ReleaseManifest, table: &str) -> Result<String, String> {
    let url = manifest
        .table_urls
        .get(table)
        .cloned()
        .unwrap_or_else(|| format!("{}{}.jsonl", manifest.base_url, table));

    let res = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download SDE table '{}': {}", table, e))?;

    if !res.status().is_success() {
        let status = res.status();
        return Err(format!("SDE table '{}' returned HTTP {}", table, status));
    }

    res.text()
        .await
        .map_err(|e| format!("Failed to read SDE table '{}': {}", table, e))
}

fn build_sde_index(
    release: String,
    etag: Option<String>,
    blueprints_raw: &str,
    type_materials_raw: &str,
    types_raw: &str,
    groups_raw: &str,
) -> SdeIndex {
    let mut blueprints = HashMap::new();
    let mut product_to_blueprint = HashMap::new();

    for line in blueprints_raw.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(bp) = serde_json::from_str::<RawBlueprintLine>(line) else {
            continue;
        };

        let (activity, act) = if let Some(mfg) = bp.activities.manufacturing {
            (BlueprintActivity::Manufacturing, mfg)
        } else if let Some(rxn) = bp.activities.reaction {
            (BlueprintActivity::Reaction, rxn)
        } else {
            continue;
        };

        let Some(product) = act.products.first() else {
            continue;
        };

        let materials = act
            .materials
            .into_iter()
            .map(|m| RawMaterial {
                material_type_id: m.type_id,
                quantity: m.quantity as u64,
            })
            .collect();

        product_to_blueprint
            .entry(product.type_id)
            .or_insert(bp.blueprint_type_id);

        blueprints.insert(
            bp.blueprint_type_id,
            SdeBlueprintEntry {
                blueprint_type_id: bp.blueprint_type_id,
                activity,
                product_type_id: product.type_id,
                product_quantity: product.quantity.max(1),
                time: act.time,
                materials,
            },
        );
    }

    let mut reprocessing = HashMap::new();
    for line in type_materials_raw.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(entry) = serde_json::from_str::<RawTypeMaterialsLine>(line) else {
            continue;
        };
        if entry.materials.is_empty() {
            continue;
        }
        let mats = entry
            .materials
            .into_iter()
            .map(|m| RawMaterial {
                material_type_id: m.material_type_id,
                quantity: m.quantity as u64,
            })
            .collect();
        reprocessing.insert(entry.type_id, mats);
    }

    let mut types = HashMap::new();
    for line in types_raw.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(item) = serde_json::from_str::<RawTypeLine>(line) else {
            continue;
        };
        if !item.published {
            continue;
        }
        let name = item.name.map(LocalizedText::into_en).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        types.insert(
            item.type_id,
            SdeTypeInfo {
                name,
                group_id: item.group_id,
                portion_size: item.portion_size.max(1),
                volume: item.volume,
                meta_group_id: item.meta_group_id,
            },
        );
    }

    let mut groups = HashMap::new();
    for line in groups_raw.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(grp) = serde_json::from_str::<RawGroupLine>(line) else {
            continue;
        };
        groups.insert(
            grp.group_id,
            SdeGroupInfo {
                name: grp.name.map(LocalizedText::into_en).unwrap_or_default(),
                category_id: grp.category_id,
            },
        );
    }

    SdeIndex {
        release,
        etag,
        blueprints,
        product_to_blueprint,
        reprocessing,
        types,
        groups,
    }
}

/// Ensures the cached local SDE index is loaded into `AppState` (reading from disk if needed).
pub async fn ensure_sde_loaded(app: &AppHandle, state: &AppState) -> Result<SdeIndex, String> {
    {
        let guard = state.sde.lock().await;
        if let Some(idx) = guard.as_ref() {
            return Ok(idx.clone());
        }
    }

    let disk_index: SdeIndex = read_json_or_default(app, SDE_INDEX_FILE).unwrap_or_default();
    if !disk_index.blueprints.is_empty() {
        let mut guard = state.sde.lock().await;
        *guard = Some(disk_index.clone());
        return Ok(disk_index);
    }

    sync_sde_index(app, state, false).await
}

/// Checks `sde.evetech.net` via ETag and refreshes the local compact SDE index if outdated.
pub async fn sync_sde_index(
    app: &AppHandle,
    state: &AppState,
    force: bool,
) -> Result<SdeIndex, String> {
    let existing: SdeIndex = {
        let guard = state.sde.lock().await;
        match guard.as_ref() {
            Some(idx) => idx.clone(),
            None => read_json_or_default(app, SDE_INDEX_FILE).unwrap_or_default(),
        }
    };

    let current_etag = if !force && !existing.blueprints.is_empty() {
        existing.etag.as_deref()
    } else {
        None
    };

    let Some(manifest) = check_latest_release(&state.http, current_etag).await? else {
        let mut guard = state.sde.lock().await;
        *guard = Some(existing.clone());
        return Ok(existing);
    };

    if !force && !existing.blueprints.is_empty() && existing.release == manifest.release {
        let mut guard = state.sde.lock().await;
        *guard = Some(existing.clone());
        return Ok(existing);
    }

    let (bp_raw, mat_raw, types_raw, groups_raw) = tokio::try_join!(
        download_table(&state.http, &manifest, "blueprints"),
        download_table(&state.http, &manifest, "typeMaterials"),
        download_table(&state.http, &manifest, "types"),
        download_table(&state.http, &manifest, "groups"),
    )?;

    let index = build_sde_index(
        manifest.release,
        manifest.etag,
        &bp_raw,
        &mat_raw,
        &types_raw,
        &groups_raw,
    );

    write_json_compact_atomic(app, SDE_INDEX_FILE, &index)?;

    let mut guard = state.sde.lock().await;
    *guard = Some(index.clone());

    Ok(index)
}