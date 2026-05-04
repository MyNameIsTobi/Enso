use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::AppState;
use crate::index::NodeSummary;

const MAX_GROUPS: usize = 200;

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub name: String,
    pub size: u64,
    pub nodes: Vec<NodeSummary>,
}

#[tauri::command]
pub async fn find_duplicates(
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateGroup>, String> {
    let index = &state.index;
    let nodes_guard = index.nodes.read();

    // Collect candidates grouped by (size, name)
    let mut map: HashMap<(u64, Arc<str>), Vec<usize>> = HashMap::new();

    for (idx, node) in nodes_guard.iter().enumerate() {
        if node.id == u32::MAX || node.error || node.is_dir || node.is_symlink || node.size == 0 {
            continue;
        }
        map.entry((node.size, node.name.clone()))
            .or_default()
            .push(idx);
    }

    // Build groups with >= 2 entries
    let mut groups: Vec<DuplicateGroup> = map
        .into_iter()
        .filter(|(_, indices)| indices.len() >= 2)
        .map(|((size, name), indices)| {
            let nodes: Vec<NodeSummary> = indices
                .iter()
                .map(|&idx| {
                    let node = &nodes_guard[idx];
                    let cc = index.child_count(node.id);
                    NodeSummary::from_node(node, cc)
                })
                .collect();
            DuplicateGroup {
                name: name.to_string(),
                size,
                nodes,
            }
        })
        .collect();

    // Sort by reclaimable space descending
    groups.sort_unstable_by(|a, b| {
        let savings_a = a.size * (a.nodes.len() as u64 - 1);
        let savings_b = b.size * (b.nodes.len() as u64 - 1);
        savings_b.cmp(&savings_a)
    });

    groups.truncate(MAX_GROUPS);

    Ok(groups)
}
