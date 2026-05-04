use std::sync::Arc;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use parking_lot::RwLock;
use dashmap::DashMap;
use smallvec::SmallVec;
use serde::{Serialize, Deserialize};

pub mod stats;
pub use stats::StorageInfo;

pub type NodeId = u32;

// ── FileCategory ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FileCategory {
    Images      = 0,
    Videos      = 1,
    Documents   = 2,
    Development = 3,
    Archives    = 4,
    Other       = 5,
}

impl FileCategory {
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "jpg"|"jpeg"|"png"|"gif"|"bmp"|"svg"|"webp"|"ico"|"tiff"|"heic"|"avif" => Self::Images,
            "mp4"|"mkv"|"avi"|"mov"|"wmv"|"flv"|"webm"|"m4v"|"mpg"|"mpeg"          => Self::Videos,
            "pdf"|"doc"|"docx"|"xls"|"xlsx"|"ppt"|"pptx"|"txt"|"md"|"odt"|"rtf"    => Self::Documents,
            "rs"|"py"|"js"|"ts"|"go"|"c"|"cpp"|"h"|"java"|"rb"|"php"|"sh"|"toml"|"yaml"|"json"|"lock" => Self::Development,
            "zip"|"tar"|"gz"|"bz2"|"xz"|"7z"|"rar"|"zst"|"lz4"|"cab"              => Self::Archives,
            _ => Self::Other,
        }
    }

    pub fn as_u8(self) -> u8 { self as u8 }
}

// ── FileNode ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileNode {
    pub id:        NodeId,
    pub parent:    Option<NodeId>,
    pub name:      Arc<str>,
    pub path:      Arc<PathBuf>,
    pub size:      u64,
    pub is_dir:    bool,
    pub is_symlink:bool,
    pub category:  FileCategory,
    pub mtime:     i64,   // unix ms
    pub depth:     u16,
    pub error:     bool,
}

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub id:          u32,
    pub name:        String,
    pub path:        String,
    pub size:        u64,
    pub is_dir:      bool,
    pub category:    u8,
    pub mtime:       i64,
    pub child_count: u32,
}

impl NodeSummary {
    pub fn from_node(node: &FileNode, child_count: u32) -> Self {
        Self {
            id:    node.id,
            name:  node.name.to_string(),
            path:  node.path.to_string_lossy().into_owned(),
            size:  node.size,
            is_dir: node.is_dir,
            category: node.category.as_u8(),
            mtime: node.mtime,
            child_count,
        }
    }
}

// ── FileIndex ─────────────────────────────────────────────────────────────────

pub struct FileIndex {
    pub nodes:    RwLock<Vec<FileNode>>,
    pub children: DashMap<NodeId, SmallVec<[NodeId; 8]>>,
    pub path_map: DashMap<Arc<PathBuf>, NodeId>,
    next_id:      AtomicU32,
    pub root:     RwLock<Option<NodeId>>,
    pub storage:  RwLock<Option<StorageInfo>>,
    pub errors:   RwLock<Vec<String>>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            nodes:    RwLock::new(Vec::with_capacity(64_000)),
            children: DashMap::new(),
            path_map: DashMap::new(),
            next_id:  AtomicU32::new(0),
            root:     RwLock::new(None),
            storage:  RwLock::new(None),
            errors:   RwLock::new(Vec::new()),
        }
    }

    pub fn alloc_id(&self) -> NodeId {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn insert_node(&self, node: FileNode) {
        let path = node.path.clone();
        let id   = node.id;
        let parent = node.parent;

        let mut nodes = self.nodes.write();
        // Ensure vec is large enough (nodes vec indexed by id)
        let id_usize = id as usize;
        if nodes.len() <= id_usize {
            nodes.resize_with(id_usize + 1, || FileNode {
                id: u32::MAX,
                parent: None,
                name: Arc::from(""),
                path: Arc::new(PathBuf::new()),
                size: 0,
                is_dir: false,
                is_symlink: false,
                category: FileCategory::Other,
                mtime: 0,
                depth: 0,
                error: true,
            });
        }
        nodes[id_usize] = node;
        drop(nodes);

        self.path_map.insert(path, id);
        if let Some(pid) = parent {
            self.children.entry(pid).or_default().push(id);
        }
    }

    pub fn get_node(&self, id: NodeId) -> Option<FileNode> {
        let nodes = self.nodes.read();
        nodes.get(id as usize).filter(|n| n.id != u32::MAX).cloned()
    }

    pub fn get_children(&self, parent_id: NodeId) -> Vec<NodeId> {
        self.children
            .get(&parent_id)
            .map(|v| v.to_vec())
            .unwrap_or_default()
    }

    pub fn child_count(&self, id: NodeId) -> u32 {
        self.children.get(&id).map(|v| v.len() as u32).unwrap_or(0)
    }

    pub fn node_count(&self) -> usize {
        self.next_id.load(Ordering::Relaxed) as usize
    }

    pub fn get_summary(&self, id: NodeId) -> Option<NodeSummary> {
        let node = self.get_node(id)?;
        let cc = self.child_count(id);
        Some(NodeSummary::from_node(&node, cc))
    }

    /// Update cumulative dir sizes bottom-up using rayon.
    pub fn compute_dir_sizes(&self) {
        use std::collections::VecDeque;

        let root_id = match *self.root.read() {
            Some(id) => id,
            None => return,
        };

        // BFS to get depth order, then process deepest first
        let mut order: Vec<NodeId> = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(root_id);
        while let Some(id) = queue.pop_front() {
            order.push(id);
            for child_id in self.get_children(id) {
                queue.push_back(child_id);
            }
        }

        // Process in reverse BFS order (leaves first)
        for &id in order.iter().rev() {
            let children = self.get_children(id);
            if children.is_empty() { continue; }

            let child_size: u64 = {
                let nodes = self.nodes.read();
                children.iter()
                    .filter_map(|&cid| nodes.get(cid as usize))
                    .filter(|n| n.id != u32::MAX)
                    .map(|n| n.size)
                    .sum()
            };

            let mut nodes = self.nodes.write();
            if let Some(n) = nodes.get_mut(id as usize) {
                if n.is_dir {
                    n.size += child_size;
                }
            }
        }
    }

    /// Rebuild parent→child relationships from stored paths.
    ///
    /// The parallel walker looks up parent IDs while building batches, but the
    /// parent may not be in path_map yet (it arrives in a later batch).  This
    /// pass runs AFTER all nodes are inserted and uses the complete path_map to
    /// fix every parent/child link correctly.
    pub fn rebuild_tree(&self) {
        // Clear stale children map
        self.children.clear();

        // Collect (node_id, parent_id) pairs — read-only pass
        let nodes_guard = self.nodes.read();
        let mut parent_updates: Vec<(NodeId, NodeId)> = Vec::with_capacity(nodes_guard.len());

        for node in nodes_guard.iter() {
            if node.id == u32::MAX { continue; }
            if let Some(parent_path) = node.path.parent() {
                let pp_arc = Arc::new(parent_path.to_path_buf());
                if let Some(pid) = self.path_map.get(&pp_arc).map(|v| *v) {
                    parent_updates.push((node.id, pid));
                    self.children.entry(pid).or_default().push(node.id);
                }
            }
        }
        drop(nodes_guard);

        // Write parent_id back into each node
        let mut nodes_guard = self.nodes.write();
        for (node_id, parent_id) in parent_updates {
            if let Some(node) = nodes_guard.get_mut(node_id as usize) {
                node.parent = Some(parent_id);
            }
        }
    }

    /// Remove nodes (and descendants) after deletion; returns all removed ids.
    pub fn remove_nodes(&self, root_ids: &[NodeId]) -> Vec<NodeId> {
        use std::collections::VecDeque;

        let mut all_removed = Vec::new();
        let mut queue: VecDeque<NodeId> = root_ids.iter().copied().collect();

        while let Some(id) = queue.pop_front() {
            all_removed.push(id);
            for child_id in self.get_children(id) {
                queue.push_back(child_id);
            }
            self.children.remove(&id);
        }

        // Zero out sizes and mark error
        {
            let mut nodes = self.nodes.write();
            for &id in &all_removed {
                if let Some(n) = nodes.get_mut(id as usize) {
                    n.size  = 0;
                    n.error = true;
                }
            }
        }

        // Remove from parent's children list
        for &id in root_ids {
            let parent_id = self.get_node(id).and_then(|n| n.parent);
            if let Some(pid) = parent_id {
                if let Some(mut v) = self.children.get_mut(&pid) {
                    v.retain(|cid| *cid != id);
                }
            }
        }

        all_removed
    }

    /// Collect ancestor chain for a node (for post-deletion update events).
    pub fn ancestors_summaries(&self, id: NodeId) -> Vec<NodeSummary> {
        let mut result = Vec::new();
        let mut current = self.get_node(id).and_then(|n| n.parent);
        while let Some(pid) = current {
            if let Some(summary) = self.get_summary(pid) {
                current = self.get_node(pid).and_then(|n| n.parent);
                result.push(summary);
            } else {
                break;
            }
        }
        result
    }
}

impl Default for FileIndex {
    fn default() -> Self { Self::new() }
}
