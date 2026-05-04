// Re-export FileCategory classify helper
pub use crate::index::FileCategory;

pub fn classify_path(path: &std::path::Path) -> FileCategory {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    FileCategory::from_ext(ext)
}
