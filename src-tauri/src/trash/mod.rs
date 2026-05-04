/// Cross-platform "move to trash" via the `trash` crate.
///
/// - Linux: writes to ~/.local/share/Trash per freedesktop.org spec
/// - macOS: uses NSFileManager.trashItem(at:) → ~/.Trash
/// - Windows: uses IFileOperation
use std::path::Path;

pub fn move_to_trash(src: &Path) -> anyhow::Result<()> {
    trash::delete(src).map_err(|e| anyhow::anyhow!("{}", e))
}
