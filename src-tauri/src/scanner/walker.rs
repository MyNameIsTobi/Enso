use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use jwalk::{DirEntry, WalkDirGeneric, Parallelism};
use crossbeam_channel::{bounded, Sender};
use tauri::{AppHandle, Emitter};

use crate::index::{FileIndex, FileNode, FileCategory, NodeId};
use crate::scanner::progress::{ScanProgress, ScanComplete};
use crate::scanner::categories::classify_path;

const BATCH_SIZE:         usize    = 2_000;
const PROGRESS_INTERVAL:  Duration = Duration::from_millis(150);
const CANCEL_CHECK_EVERY: usize    = 100;

pub struct ScanEntry {
    pub node: FileNode,
}

pub async fn scan(
    app: AppHandle,
    root_path: PathBuf,
    index: Arc<FileIndex>,
    cancelled: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let start = Instant::now();

    // Reset index
    {
        *index.root.write()    = None;
        *index.errors.write()  = Vec::new();
        *index.storage.write() = None;
        index.nodes.write().clear();
        index.children.clear();
        index.path_map.clear();
    }

    let scanned = Arc::new(AtomicU64::new(0));
    let bytes   = Arc::new(AtomicU64::new(0));
    let errors  = Arc::new(AtomicU32::new(0));
    let dirs    = Arc::new(AtomicU64::new(0));

    let (tx, rx) = bounded::<Vec<ScanEntry>>(64);

    let index_c    = index.clone();
    let cancel_c   = cancelled.clone();
    let root_c     = root_path.clone();
    let sc = scanned.clone();
    let bc = bytes.clone();
    let ec = errors.clone();
    let dc = dirs.clone();
    let tx_c = tx.clone();

    let virtual_mounts = Arc::new(read_virtual_mounts());

    tokio::task::spawn_blocking(move || {
        walk_dir(root_c, index_c, cancel_c, sc, bc, ec, dc, tx_c, virtual_mounts);
    });
    drop(tx);

    // Receive batches, emit progress
    let mut last_emit  = Instant::now();
    let mut last_path  = root_path.to_string_lossy().into_owned();

    while let Ok(batch) = tokio::task::block_in_place(|| rx.recv()) {
        if cancelled.load(Ordering::Relaxed) { break; }
        for entry in batch {
            last_path = entry.node.path.to_string_lossy().into_owned();
            index.insert_node(entry.node);
        }
        if last_emit.elapsed() >= PROGRESS_INTERVAL {
            last_emit = Instant::now();
            let _ = app.emit("scan://progress", ScanProgress {
                scanned: scanned.load(Ordering::Relaxed),
                bytes:   bytes.load(Ordering::Relaxed),
                path:    last_path.clone(),
                errors:  errors.load(Ordering::Relaxed),
            });
        }
    }

    if cancelled.load(Ordering::Relaxed) {
        let _ = app.emit("scan://cancelled", ());
        return Ok(());
    }

    // Fix parent/child links: the walker looked up parents while building
    // batches, but many were not yet in path_map at that point.  Rebuild from
    // the complete path_map now that all nodes are inserted.
    index.rebuild_tree();

    index.compute_dir_sizes();

    if let Ok(si) = crate::index::stats::get_storage_info(&root_path) {
        *index.storage.write() = Some(si);
    }

    let root_id     = index.root.read().unwrap_or(0);
    let total_bytes = index.get_node(root_id).map(|n| n.size).unwrap_or(0);

    let _ = app.emit("scan://complete", ScanComplete {
        root_id,
        files:   scanned.load(Ordering::Relaxed),
        dirs:    dirs.load(Ordering::Relaxed),
        bytes:   total_bytes,
        errors:  errors.load(Ordering::Relaxed),
        duration_ms: start.elapsed().as_millis() as u64,
    });

    Ok(())
}

// ── The blocking walker (runs in spawn_blocking) ──────────────────────────────

type ClientState = ((), ());

fn walk_dir(
    root_path:      PathBuf,
    index:          Arc<FileIndex>,
    cancelled:      Arc<AtomicBool>,
    scanned:        Arc<AtomicU64>,
    bytes:          Arc<AtomicU64>,
    errors:         Arc<AtomicU32>,
    dirs:           Arc<AtomicU64>,
    tx:             Sender<Vec<ScanEntry>>,
    virtual_mounts: Arc<std::collections::HashSet<PathBuf>>,
) {
    let parallelism = num_cpus::get();

    // Insert root node
    let root_id = index.alloc_id();
    {
        let root_meta    = std::fs::symlink_metadata(&root_path);
        let root_mtime   = root_meta.as_ref().map(|m| mtime_ms(m)).unwrap_or(0);
        let root_name: Arc<str> = Arc::from(
            root_path.file_name().and_then(|n| n.to_str()).unwrap_or("/")
        );
        index.insert_node(FileNode {
            id:        root_id,
            parent:    None,
            name:      root_name,
            path:      Arc::new(root_path.clone()),
            size:      0,
            is_dir:    true,
            is_symlink:false,
            category:  FileCategory::Other,
            mtime:     root_mtime,
            depth:     0,
            error:     false,
        });
        *index.root.write() = Some(root_id);
    }

    let mut batch: Vec<ScanEntry> = Vec::with_capacity(BATCH_SIZE);
    let mut count: usize = 0;

    let walker = WalkDirGeneric::<ClientState>::new(&root_path)
        .follow_links(false)
        .skip_hidden(false)
        .parallelism(Parallelism::RayonNewPool(parallelism));

    for result in walker {
        count += 1;
        if count % CANCEL_CHECK_EVERY == 0 && cancelled.load(Ordering::Relaxed) {
            break;
        }

        let entry: DirEntry<ClientState> = match result {
            Ok(e) => e,
            Err(e) => {
                errors.fetch_add(1, Ordering::Relaxed);
                let mut errs = index.errors.write();
                if errs.len() < 1_000 {
                    errs.push(e.to_string());
                }
                continue;
            }
        };

        let path: PathBuf = entry.path();
        if path == root_path { continue; }

        // Skip virtual/pseudo filesystems (proc, sysfs, devtmpfs, etc.)
        // but DO cross real block-device mount points (e.g. separate /home partition).
        if virtual_mounts.contains(&path) { continue; }

        let meta: std::fs::Metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => {
                errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let ft         = meta.file_type();
        let is_symlink = ft.is_symlink();
        let is_dir     = ft.is_dir() || meta.is_dir();
        let size       = if is_dir { 0 } else { meta.len() };
        let mtime      = mtime_ms(&meta);

        if is_dir {
            dirs.fetch_add(1, Ordering::Relaxed);
        } else {
            scanned.fetch_add(1, Ordering::Relaxed);
            bytes.fetch_add(size, Ordering::Relaxed);
        }

        let path_arc = Arc::new(path.clone());
        let name: Arc<str> = Arc::from(
            path.file_name()
                .and_then(|n: &std::ffi::OsStr| n.to_str())
                .unwrap_or("")
        );
        let category = if is_dir {
            FileCategory::Other
        } else {
            classify_path(&path)
        };

        // Find parent id via path_map
        let parent_id: Option<NodeId> = path.parent().and_then(|pp| {
            let pp_arc = Arc::new(pp.to_path_buf());
            index.path_map.get(&pp_arc).map(|v| *v)
        });

        let id = index.alloc_id();

        batch.push(ScanEntry {
            node: FileNode {
                id,
                parent: parent_id,
                name,
                path: path_arc,
                size,
                is_dir,
                is_symlink,
                category,
                mtime,
                depth: entry.depth as u16,
                error: false,
            },
        });

        if batch.len() >= BATCH_SIZE {
            let _ = tx.send(std::mem::replace(&mut batch, Vec::with_capacity(BATCH_SIZE)));
        }
    }

    if !batch.is_empty() {
        let _ = tx.send(batch);
    }
}

fn mtime_ms(meta: &std::fs::Metadata) -> i64 {
    use std::time::UNIX_EPOCH;
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Returns mount points that host virtual/pseudo filesystems we don't want to scan.
/// Linux: parsed from /proc/mounts. macOS: hardcoded list of system volumes.
fn read_virtual_mounts() -> std::collections::HashSet<PathBuf> {
    let mut mounts = std::collections::HashSet::new();

    #[cfg(target_os = "linux")]
    {
        const VIRTUAL_FS: &[&str] = &[
            "proc", "sysfs", "devtmpfs", "devpts", "tmpfs", "cgroup", "cgroup2",
            "pstore", "efivarfs", "bpf", "tracefs", "debugfs", "securityfs",
            "hugetlbfs", "mqueue", "fusectl", "configfs", "ramfs", "nsfs",
            "autofs", "rpc_pipefs", "nfsd", "selinuxfs",
        ];

        for path in ["/proc", "/sys", "/dev", "/run"] {
            mounts.insert(PathBuf::from(path));
        }

        if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let mut parts = line.splitn(4, ' ');
                let _device    = parts.next();
                let mountpoint = parts.next().unwrap_or("");
                let fstype     = parts.next().unwrap_or("");
                if VIRTUAL_FS.contains(&fstype) {
                    mounts.insert(PathBuf::from(mountpoint));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // APFS system volumes (Big Sur+) and other pseudo/system mounts.
        // The user-facing root '/' is a synthesized read-only view; skipping
        // these avoids permission-denied storms and double-counted data.
        for path in [
            "/dev",
            "/private/var/vm",
            "/System/Volumes/Recovery",
            "/System/Volumes/Preboot",
            "/System/Volumes/VM",
            "/System/Volumes/Hardware",
            "/System/Volumes/iSCPreboot",
            "/System/Volumes/xarts",
            "/System/Volumes/Update",
            "/Volumes/Recovery",
            "/Volumes/Preboot",
        ] {
            mounts.insert(PathBuf::from(path));
        }
    }

    mounts
}
