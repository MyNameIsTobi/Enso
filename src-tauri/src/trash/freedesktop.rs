/// freedesktop.org Trash spec implementation for Linux.
///
/// Protocol:
/// 1. Write .trashinfo FIRST (safe: orphaned info is ok; orphaned file is not)
/// 2. rename() to Trash/files/<name>
/// 3. Cross-filesystem: copy + remove_all, then trashinfo
use std::path::{Path, PathBuf};
use std::io::Write;

pub fn move_to_trash(src: &Path) -> anyhow::Result<()> {
    let home = std::env::var("HOME")
        .or_else(|_| {
            // Fallback: read passwd
            let uid = unsafe { libc::getuid() };
            Ok::<String, std::env::VarError>(format!("/home/{uid}"))
        })
        .unwrap_or_else(|_| "/tmp".to_string());

    let trash_dir   = PathBuf::from(&home).join(".local/share/Trash");
    let files_dir   = trash_dir.join("files");
    let info_dir    = trash_dir.join("info");

    std::fs::create_dir_all(&files_dir)?;
    std::fs::create_dir_all(&info_dir)?;

    let file_name = src.file_name()
        .ok_or_else(|| anyhow::anyhow!("Path has no filename: {}", src.display()))?
        .to_string_lossy()
        .into_owned();

    // Find a non-colliding destination name
    let (dest_files, dest_info, final_name) = find_dest(&files_dir, &info_dir, &file_name);

    // 1. Write .trashinfo FIRST
    write_trashinfo(&dest_info, src, &final_name)?;

    // 2. Attempt rename (same filesystem)
    match std::fs::rename(src, &dest_files) {
        Ok(_) => return Ok(()),
        Err(e) if is_cross_device(&e) => {
            // 3. Cross-filesystem: copy then delete
            if src.is_dir() {
                copy_dir_all(src, &dest_files)?;
                std::fs::remove_dir_all(src)?;
            } else {
                std::fs::copy(src, &dest_files)?;
                std::fs::remove_file(src)?;
            }
            Ok(())
        }
        Err(e) => {
            // Clean up orphaned trashinfo
            let _ = std::fs::remove_file(&dest_info);
            Err(e.into())
        }
    }
}

fn find_dest(files_dir: &Path, info_dir: &Path, name: &str) -> (PathBuf, PathBuf, String) {
    let mut candidate = name.to_string();
    let mut i = 0u32;
    loop {
        let files = files_dir.join(&candidate);
        let info  = info_dir.join(format!("{candidate}.trashinfo"));
        if !files.exists() && !info.exists() {
            return (files, info, candidate);
        }
        i += 1;
        candidate = format!("{name}_{i}");
    }
}

fn write_trashinfo(info_path: &Path, src: &Path, _name: &str) -> anyhow::Result<()> {
    use chrono::Local;
    let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let abs_path = src.canonicalize()
        .unwrap_or_else(|_| src.to_path_buf());

    let content = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        abs_path.to_string_lossy(),
        now,
    );

    let mut file = std::fs::OpenOptions::new()
        .write(true).create_new(true).open(info_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn is_cross_device(e: &std::io::Error) -> bool {
    e.raw_os_error() == Some(libc::EXDEV)
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest  = dst.join(entry.file_name());
        let ft    = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}
