use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const CONFIG_FILE: &str = "settings.json";

fn resolve_data_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    Ok(dir.join(filename))
}

pub fn read_json_or_default<T>(app: &AppHandle, filename: &str) -> Result<T, String>
where
    T: DeserializeOwned + Default,
{
    let path = resolve_data_path(app, filename)?;
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", filename, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", filename, e))
}

pub fn write_json_atomic<T>(app: &AppHandle, filename: &str, data: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = resolve_data_path(app, filename)?;
    let tmp_path = path.with_extension("json.tmp");

    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize {}: {}", filename, e))?;

    fs::write(&tmp_path, json)
        .map_err(|e| format!("Failed to write temp file for {}: {}", filename, e))?;

    #[cfg(windows)]
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to finalize {}: {}", filename, e))
}

pub fn write_json_compact_atomic<T>(app: &AppHandle, filename: &str, data: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = resolve_data_path(app, filename)?;
    let tmp_path = path.with_extension("json.tmp");

    let json = serde_json::to_string(data)
        .map_err(|e| format!("Failed to serialize {}: {}", filename, e))?;

    fs::write(&tmp_path, json)
        .map_err(|e| format!("Failed to write temp file for {}: {}", filename, e))?;

    #[cfg(windows)]
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to finalize {}: {}", filename, e))
}