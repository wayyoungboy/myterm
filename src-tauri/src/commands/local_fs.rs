use crate::db::models::SftpEntry;
use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};

fn normalize_path(path: &str) -> PathBuf {
    if path.trim().is_empty() {
        PathBuf::from("/")
    } else {
        PathBuf::from(path)
    }
}

#[cfg(unix)]
fn permissions_string(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    format!("{:o}", metadata.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn permissions_string(metadata: &fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "444".to_string()
    } else {
        "666".to_string()
    }
}

fn modified_string(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .map(|time| {
            let dt: DateTime<Local> = DateTime::from(time);
            dt.to_rfc3339()
        })
        .unwrap_or_default()
}

fn entry_from_path(path: &Path) -> Result<SftpEntry, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to read metadata: {}", e))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| path.to_str().unwrap_or("/"))
        .to_string();

    Ok(SftpEntry {
        name,
        path: path.to_string_lossy().to_string(),
        is_dir: metadata.is_dir(),
        size: if metadata.is_dir() { 0 } else { metadata.len() },
        permissions: permissions_string(&metadata),
        modified: modified_string(&metadata),
    })
}

#[tauri::command]
pub fn list_local_dir(path: String) -> Result<Vec<SftpEntry>, String> {
    let path = normalize_path(&path);
    let mut entries = Vec::new();

    let read_dir =
        fs::read_dir(&path).map_err(|e| format!("Failed to list {}: {}", path.display(), e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let entry_path = entry.path();
        if let Ok(item) = entry_from_path(&entry_path) {
            entries.push(item);
        }
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

#[tauri::command]
pub fn write_local_file(path: String, data: Vec<u8>) -> Result<(), String> {
    let path = normalize_path(&path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }
    fs::write(&path, data).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

#[tauri::command]
pub fn remove_local_file(path: String) -> Result<(), String> {
    let path = normalize_path(&path);
    let metadata =
        fs::metadata(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    if metadata.is_dir() {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_file(&path)
    }
    .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))
}

#[tauri::command]
pub fn rename_local_file(src: String, dst: String) -> Result<(), String> {
    let src = normalize_path(&src);
    let dst = normalize_path(&dst);
    fs::rename(&src, &dst).map_err(|e| {
        format!(
            "Failed to rename {} to {}: {}",
            src.display(),
            dst.display(),
            e
        )
    })
}

#[tauri::command]
pub fn create_local_dir(path: String) -> Result<(), String> {
    let path = normalize_path(&path);
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create {}: {}", path.display(), e))
}
