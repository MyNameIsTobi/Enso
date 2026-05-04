use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total:     u64,
    pub used:      u64,
    pub available: u64,
    pub mount:     String,
}

pub fn get_storage_info(path: &Path) -> anyhow::Result<StorageInfo> {
    use nix::sys::statvfs::statvfs;

    let stat = statvfs(path)?;
    let bsize = stat.block_size() as u64;
    let total     = stat.blocks()           * bsize;
    let available = stat.blocks_available() * bsize;
    let free      = stat.blocks_free()      * bsize;
    let used      = total.saturating_sub(free);

    Ok(StorageInfo {
        total,
        used,
        available,
        mount: path.to_string_lossy().into_owned(),
    })
}
