use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use serde::{Serialize, Deserialize};

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    pub name: String,
    pub path: String,   // full path to steamapps/common/<installdir>
    pub size: u64,      // from FileIndex if scanned, else from .acf SizeOnDisk
}

/// Find all Steam library root directories by reading libraryfolders.vdf.
fn steam_library_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    let home = std::env::var("HOME").unwrap_or_default();
    let default_root = PathBuf::from(&home).join(".local/share/Steam");
    if default_root.exists() {
        roots.push(default_root.clone());
    }

    let vdf_path = default_root.join("steamapps/libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        // Modern VDF: each library has a "path" key inside a numbered block.
        // Simple line-by-line scan: grab every "path" value.
        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("\"path\"") { continue; }
            if let Some(val) = extract_vdf_value(line, "path") {
                let p = PathBuf::from(&val);
                if p.exists() && !roots.contains(&p) {
                    roots.push(p);
                }
            }
        }
    }

    roots
}

/// Extract the value for a known key from a single VDF line like:
///   "key"   "value"
fn extract_vdf_value(line: &str, key: &str) -> Option<String> {
    let key_token = format!("\"{}\"", key);
    let line = line.trim();
    if !line.starts_with(&key_token) { return None; }
    let rest = line[key_token.len()..].trim();
    if rest.starts_with('"') && rest.len() >= 2 {
        let inner = &rest[1..];
        let end = inner.find('"')?;
        return Some(inner[..end].to_string());
    }
    None
}

/// Returns true if the name looks like a Steam runtime/tool rather than a real game.
/// Modern .acf files don't contain a "type" field, so we match by name patterns.
fn is_steam_tool(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("proton")
        || n.contains("linux runtime")
        || n.contains("redistributable")
        || n.contains("steamworks")
        || n.contains(" sdk")
        || n.ends_with(" sdk")
}

/// Parse a single appmanifest_*.acf file.
/// Returns None if the entry looks like a tool/runtime rather than a game.
fn parse_acf(content: &str) -> Option<(String, String, u64)> {
    let mut name: Option<String> = None;
    let mut install_dir: Option<String> = None;
    let mut size_on_disk: u64 = 0;

    for line in content.lines() {
        let line = line.trim();
        if name.is_none() {
            if let Some(v) = extract_vdf_value(line, "name") { name = Some(v); continue; }
        }
        if install_dir.is_none() {
            if let Some(v) = extract_vdf_value(line, "installdir") { install_dir = Some(v); continue; }
        }
        if size_on_disk == 0 {
            if let Some(v) = extract_vdf_value(line, "SizeOnDisk") {
                size_on_disk = v.parse().unwrap_or(0);
            }
        }
    }

    let name = name?;
    if is_steam_tool(&name) { return None; }

    Some((name, install_dir?, size_on_disk))
}

#[tauri::command]
pub async fn get_steam_games(
    state: State<'_, AppState>,
) -> Result<Vec<SteamGame>, String> {
    let index = &state.index;
    let roots = steam_library_roots();
    let mut games: Vec<SteamGame> = Vec::new();

    for root in roots {
        let steamapps = root.join("steamapps");
        let common    = steamapps.join("common");

        let entries = match std::fs::read_dir(&steamapps) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !fname.starts_with("appmanifest_") || !fname.ends_with(".acf") { continue; }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let (name, install_dir, size_on_disk) = match parse_acf(&content) {
                Some(t) => t,
                None    => continue,
            };

            let game_path = common.join(&install_dir);

            // Prefer size from index (post-scan, includes Proton prefix etc.)
            let size = {
                let arc = Arc::new(game_path.clone());
                let id = index.path_map.get(&arc).map(|v| *v);
                id.and_then(|id| index.get_node(id)).map(|n| n.size)
                    .unwrap_or(size_on_disk)
            };

            games.push(SteamGame {
                name,
                path: game_path.to_string_lossy().into_owned(),
                size,
            });
        }
    }

    games.sort_unstable_by(|a, b| b.size.cmp(&a.size));
    Ok(games)
}
