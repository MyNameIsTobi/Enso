use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub scanned: u64,
    pub bytes:   u64,
    pub path:    String,
    pub errors:  u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanComplete {
    pub root_id:     u32,
    pub files:       u64,
    pub dirs:        u64,
    pub bytes:       u64,
    pub errors:      u32,
    pub duration_ms: u64,
}
