use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::models::{VersionMeta, VersionEntry, FileVersionHistory};

pub const VERSIONS_DIR_NAME: &str = ".versions";
pub const MAX_VERSIONS: u32 = 10;

/// Returns the .versions/ directory for a given file's parent directory.
fn versions_dir_for(file_path: &Path) -> PathBuf {
    file_path
        .parent()
        .unwrap_or(file_path)
        .join(VERSIONS_DIR_NAME)
}

/// Returns the per-file version storage directory.
fn file_version_dir(file_path: &Path) -> PathBuf {
    let file_name = file_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    versions_dir_for(file_path).join(&file_name)
}

/// Read meta.json for a file, or return empty VersionMeta if none exists.
pub fn read_version_meta(file_path: &Path) -> Result<VersionMeta> {
    let meta_path = file_version_dir(file_path).join("meta.json");
    if meta_path.exists() {
        let data = std::fs::read_to_string(&meta_path)?;
        let meta: VersionMeta = serde_json::from_str(&data)?;
        Ok(meta)
    } else {
        Ok(VersionMeta {
            max_versions: MAX_VERSIONS,
            versions: Vec::new(),
        })
    }
}

/// Write meta.json for a file.
fn write_version_meta(file_path: &Path, meta: &VersionMeta) -> Result<()> {
    let ver_dir = file_version_dir(file_path);
    std::fs::create_dir_all(&ver_dir)?;
    let meta_path = ver_dir.join("meta.json");
    let data = serde_json::to_string_pretty(meta)?;
    std::fs::write(&meta_path, data)?;
    Ok(())
}

/// Find the version file on disk matching a given version number.
fn find_version_file(ver_dir: &Path, version: u32) -> Option<PathBuf> {
    let prefix = format!("v{}_", version);
    if let Ok(entries) = std::fs::read_dir(ver_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name != "meta.json" {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Save the current content of `file_path` as a new version before overwrite.
/// Returns the version number assigned.
pub fn save_version(file_path: &Path, comment: &str) -> Result<u32> {
    if !file_path.exists() || !file_path.is_file() {
        anyhow::bail!("File does not exist: {}", file_path.display());
    }

    let ver_dir = file_version_dir(file_path);
    std::fs::create_dir_all(&ver_dir)?;

    let mut meta = read_version_meta(file_path)?;

    // Determine next version number
    let next_version = meta.versions.last().map(|v| v.version + 1).unwrap_or(1);

    // Enforce MAX_VERSIONS: remove oldest if at cap
    while meta.versions.len() >= MAX_VERSIONS as usize {
        let oldest = meta.versions.remove(0);
        if let Some(f) = find_version_file(&ver_dir, oldest.version) {
            let _ = std::fs::remove_file(f);
        }
    }

    // Copy current file to version storage
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("dat");
    let timestamp = Utc::now().timestamp();
    let ver_filename = format!("v{}_{}.{}", next_version, timestamp, ext);
    let ver_path = ver_dir.join(&ver_filename);

    std::fs::copy(file_path, &ver_path)?;

    let file_size = std::fs::metadata(&ver_path)?.len();

    meta.versions.push(VersionEntry {
        version: next_version,
        created_at: Utc::now(),
        size: file_size,
        comment: comment.to_string(),
    });

    write_version_meta(file_path, &meta)?;

    Ok(next_version)
}

/// Retrieve version history for a file.
pub fn get_version_history(file_path: &Path) -> Result<FileVersionHistory> {
    let meta = read_version_meta(file_path)?;
    let file_meta = std::fs::metadata(file_path)?;
    let modified: DateTime<Utc> = file_meta.modified()?.into();

    Ok(FileVersionHistory {
        file_path: file_path.to_string_lossy().to_string(),
        current_size: file_meta.len(),
        current_modified_at: modified,
        versions: meta.versions,
    })
}

/// Rollback: copy version N back to the active file location.
/// The current file is saved as a new version first (non-destructive).
pub fn rollback_to_version(file_path: &Path, version: u32) -> Result<()> {
    let ver_dir = file_version_dir(file_path);
    let meta = read_version_meta(file_path)?;

    // Find the requested version
    meta.versions
        .iter()
        .find(|v| v.version == version)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", version))?;

    let ver_file = find_version_file(&ver_dir, version)
        .ok_or_else(|| anyhow::anyhow!("Version file for v{} not found on disk", version))?;

    // Save current state as a new version before rollback
    if file_path.exists() {
        save_version(
            file_path,
            &format!("Auto-saved before rollback to v{}", version),
        )?;
    }

    // Copy version file back to active location
    std::fs::copy(&ver_file, file_path)?;

    Ok(())
}

/// Get the version count for a file (0 if no versions exist).
pub fn version_count(file_path: &Path) -> u32 {
    read_version_meta(file_path)
        .map(|m| m.versions.len() as u32)
        .unwrap_or(0)
}

/// Delete all versions for a file (called when the file itself is deleted).
pub fn delete_versions(file_path: &Path) -> Result<()> {
    let ver_dir = file_version_dir(file_path);
    if ver_dir.exists() {
        std::fs::remove_dir_all(&ver_dir)?;
    }
    // If .versions/ parent dir is now empty, remove it too
    let parent_versions = versions_dir_for(file_path);
    if parent_versions.exists() {
        if let Ok(mut entries) = std::fs::read_dir(&parent_versions) {
            if entries.next().is_none() {
                let _ = std::fs::remove_dir(&parent_versions);
            }
        }
    }
    Ok(())
}

/// Check if a directory name is the versions directory.
pub fn is_versions_dir(name: &str) -> bool {
    name == VERSIONS_DIR_NAME
}
