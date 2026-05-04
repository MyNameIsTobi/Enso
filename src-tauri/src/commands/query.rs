use tauri::State;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

use crate::AppState;
use crate::index::{NodeSummary, NodeId, StorageInfo};

const MAX_CHILDREN: usize = 5_000;
const MAX_SEARCH:   usize = 500;

#[tauri::command]
pub async fn get_children(
    state: State<'_, AppState>,
    node_id: u32,
    offset: usize,
) -> Result<Vec<NodeSummary>, String> {
    let index = &state.index;
    let mut child_ids = index.get_children(node_id as NodeId);

    // Sort: dirs first, then by size descending
    {
        let nodes = index.nodes.read();
        child_ids.sort_unstable_by(|&a, &b| {
            let na = nodes.get(a as usize);
            let nb = nodes.get(b as usize);
            match (na, nb) {
                (Some(na), Some(nb)) => {
                    nb.is_dir.cmp(&na.is_dir)
                        .then_with(|| nb.size.cmp(&na.size))
                }
                _ => std::cmp::Ordering::Equal,
            }
        });
    }

    let slice = &child_ids[offset.min(child_ids.len())..];
    let slice = &slice[..slice.len().min(MAX_CHILDREN)];

    let nodes = index.nodes.read();
    let result: Vec<NodeSummary> = slice.iter()
        .filter_map(|&id| {
            let node = nodes.get(id as usize)?;
            if node.id == u32::MAX { return None; }
            let cc = index.child_count(id);
            Some(NodeSummary::from_node(node, cc))
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn get_node(
    state: State<'_, AppState>,
    node_id: u32,
) -> Result<Option<NodeSummary>, String> {
    Ok(state.index.get_summary(node_id as NodeId))
}

#[tauri::command]
pub async fn get_storage_info(
    state: State<'_, AppState>,
    path: String,
) -> Result<StorageInfo, String> {
    // Return cached if available
    if let Some(si) = state.index.storage.read().clone() {
        return Ok(si);
    }
    crate::index::stats::get_storage_info(std::path::Path::new(&path))
        .map_err(|e| e.to_string())
}

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub name_pattern:          Option<String>,
    pub categories:            Vec<u8>,
    pub extensions:            Vec<String>,
    pub size_min:              Option<u64>,
    pub size_max:              Option<u64>,
    pub mtime_older_than_days: Option<i64>,
    pub mtime_newer_than_days: Option<i64>,
    pub include_dirs:          bool,
    pub limit:                 usize,
    pub offset:                usize,
    pub root_node_id:          Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub nodes: Vec<NodeSummary>,
    pub total: usize,
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: SearchQuery,
) -> Result<SearchResult, String> {
    let index = &state.index;
    let now_ms = chrono::Utc::now().timestamp_millis();

    let name_lower = query.name_pattern.as_deref()
        .map(|s| s.to_ascii_lowercase());

    let cat_set: std::collections::HashSet<u8> = query.categories.iter().copied().collect();
    let ext_set: std::collections::HashSet<String> = query.extensions.iter()
        .map(|e| e.to_ascii_lowercase())
        .collect();

    // Parallel filter over all nodes
    let nodes_guard = index.nodes.read();
    let limit = query.limit.min(MAX_SEARCH);

    let matched: Vec<NodeSummary> = nodes_guard
        .par_iter()
        .filter(|n| {
            if n.id == u32::MAX || n.error { return false; }
            if !query.include_dirs && n.is_dir { return false; }

            // Category filter
            if !cat_set.is_empty() && !cat_set.contains(&n.category.as_u8()) {
                return false;
            }

            // Extension filter
            if !ext_set.is_empty() {
                let ext = n.path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_ascii_lowercase())
                    .unwrap_or_default();
                if !ext_set.contains(&ext) { return false; }
            }

            // Name pattern
            if let Some(ref pat) = name_lower {
                let name_lower = n.name.to_ascii_lowercase();
                if !name_lower.contains(pat.as_str()) { return false; }
            }

            // Size filters
            if let Some(min) = query.size_min {
                if n.size < min { return false; }
            }
            if let Some(max) = query.size_max {
                if n.size > max { return false; }
            }

            // Age filters
            if let Some(older_days) = query.mtime_older_than_days {
                let cutoff_ms = now_ms - older_days * 86_400_000;
                if n.mtime > cutoff_ms { return false; }
            }
            if let Some(newer_days) = query.mtime_newer_than_days {
                let cutoff_ms = now_ms - newer_days * 86_400_000;
                if n.mtime < cutoff_ms { return false; }
            }

            // Root scope filter: walk parent chain to check if node is under root_node_id
            if let Some(root_id) = query.root_node_id {
                if n.id != root_id {
                    let mut cur = n.parent;
                    loop {
                        match cur {
                            None => return false,
                            Some(pid) if pid == root_id => break,
                            Some(pid) => {
                                cur = nodes_guard.get(pid as usize).and_then(|p| p.parent);
                            }
                        }
                    }
                }
            }

            true
        })
        .collect::<Vec<_>>()
        .iter()
        .map(|n| {
            let cc = index.child_count(n.id);
            NodeSummary::from_node(n, cc)
        })
        .collect();

    let total = matched.len();

    // Sort by size desc
    let mut sorted = matched;
    sorted.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    let page = sorted
        .into_iter()
        .skip(query.offset)
        .take(limit)
        .collect();

    Ok(SearchResult { nodes: page, total })
}
