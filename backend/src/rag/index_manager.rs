use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tokio::sync::Mutex;

use crate::indexer::walker::{walk_directory, SupportedFormat};
use crate::indexer::extractor::extract_text;
use crate::indexer::chunker::chunk_text;
use crate::models::{FileInfo, DirEntry};
use super::embeddings::EmbeddingGenerator;
use super::vector_store::VectorStore;
use super::versioning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub is_indexing: bool,
    pub last_indexed_at: Option<DateTime<Utc>>,
    pub total_files: usize,
    pub total_chunks: usize,
    pub failed_files: Vec<String>,
    pub auto_index_interval_minutes: u64,
    pub last_error: Option<String>,
}

pub struct IndexManager {
    status: Mutex<IndexStatus>,
    upload_dir: PathBuf,
    embeddings: Arc<EmbeddingGenerator>,
    vector_store: Arc<VectorStore>,
}

fn file_id(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

impl IndexManager {
    pub fn new(
        upload_dir: PathBuf,
        embeddings: Arc<EmbeddingGenerator>,
        vector_store: Arc<VectorStore>,
        interval_minutes: u64,
    ) -> Self {
        Self {
            status: Mutex::new(IndexStatus {
                is_indexing: false,
                last_indexed_at: None,
                total_files: 0,
                total_chunks: 0,
                failed_files: Vec::new(),
                auto_index_interval_minutes: interval_minutes,
                last_error: None,
            }),
            upload_dir,
            embeddings,
            vector_store,
        }
    }

    pub fn upload_dir(&self) -> &Path {
        &self.upload_dir
    }

    pub async fn get_status(&self) -> IndexStatus {
        self.status.lock().await.clone()
    }

    pub async fn set_interval(&self, minutes: u64) {
        self.status.lock().await.auto_index_interval_minutes = minutes;
    }

    pub async fn is_indexing(&self) -> bool {
        self.status.lock().await.is_indexing
    }

    pub fn list_files(&self) -> Vec<FileInfo> {
        let files = walk_directory(&self.upload_dir);
        files.iter().filter_map(|(path, format)| {
            let metadata = std::fs::metadata(path).ok()?;
            let modified = metadata.modified().ok()?;
            let modified_dt: DateTime<Utc> = modified.into();
            Some(FileInfo {
                name: path.file_name()?.to_string_lossy().to_string(),
                size: metadata.len(),
                format: format!("{:?}", format),
                modified_at: modified_dt,
            })
        }).collect()
    }

    /// Resolve a relative path safely, ensuring it stays within upload_dir.
    /// For paths that don't exist yet (mkdir/create), use `safe_resolve_new`.
    pub fn safe_resolve(&self, relative: &str) -> Result<PathBuf, String> {
        if relative.is_empty() {
            return Ok(self.upload_dir.clone());
        }
        let joined = self.upload_dir.join(relative);
        let canonical = joined.canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;
        let base = self.upload_dir.canonicalize()
            .map_err(|e| format!("Upload dir error: {}", e))?;
        if !canonical.starts_with(&base) {
            return Err("Path traversal not allowed".to_string());
        }
        Ok(canonical)
    }

    /// Resolve a path that may not exist yet (for mkdir/create file).
    /// Validates the parent exists and is within upload_dir.
    pub fn safe_resolve_new(&self, relative: &str) -> Result<PathBuf, String> {
        if relative.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        // Reject obvious traversal attempts
        if relative.contains("..") {
            return Err("Path traversal not allowed".to_string());
        }
        let target = self.upload_dir.join(relative);
        // Verify the parent directory exists and is within upload_dir
        if let Some(parent) = target.parent() {
            if parent != self.upload_dir {
                let parent_canonical = parent.canonicalize()
                    .map_err(|e| format!("Parent directory does not exist: {}", e))?;
                let base = self.upload_dir.canonicalize()
                    .map_err(|e| format!("Upload dir error: {}", e))?;
                if !parent_canonical.starts_with(&base) {
                    return Err("Path traversal not allowed".to_string());
                }
            }
        }
        Ok(target)
    }

    /// List entries (files + directories) at a specific path level.
    pub fn list_dir_entries(&self, relative_path: &str) -> Result<Vec<DirEntry>, String> {
        let dir = self.safe_resolve(relative_path)?;
        if !dir.is_dir() {
            return Err("Not a directory".to_string());
        }

        let mut entries = Vec::new();
        let read_dir = std::fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = metadata.is_dir();

            // Skip .versions directory
            if versioning::is_versions_dir(&name) {
                continue;
            }

            if is_dir {
                entries.push(DirEntry {
                    name,
                    is_dir: true,
                    size: None,
                    format: None,
                    modified_at: metadata.modified().ok().map(|t| t.into()),
                    version_count: None,
                });
            } else {
                let path = entry.path();
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                let format = SupportedFormat::from_extension(ext)
                    .map(|f| format!("{:?}", f));
                let vc = versioning::version_count(&path);
                entries.push(DirEntry {
                    name,
                    is_dir: false,
                    size: Some(metadata.len()),
                    format,
                    modified_at: metadata.modified().ok().map(|t| t.into()),
                    version_count: if vc > 0 { Some(vc) } else { None },
                });
            }
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
        });

        Ok(entries)
    }

    pub async fn run_index(&self) -> Result<()> {
        {
            let mut status = self.status.lock().await;
            if status.is_indexing {
                anyhow::bail!("Indexing already in progress");
            }
            status.is_indexing = true;
            status.last_error = None;
            status.failed_files.clear();
        }

        // Use AssertUnwindSafe + catch_unwind to catch panics (e.g., from chunker)
        // so that is_indexing always resets to false
        let result = std::panic::AssertUnwindSafe(self.do_index())
            .catch_unwind()
            .await;

        match result {
            Ok(Ok(())) => {
                let mut status = self.status.lock().await;
                status.is_indexing = false;
                status.last_indexed_at = Some(Utc::now());
                status.last_error = None;
            }
            Ok(Err(e)) => {
                let error_msg = format!("Indexing error: {}", e);
                tracing::error!("{}", error_msg);
                let mut status = self.status.lock().await;
                status.is_indexing = false;
                status.last_error = Some(error_msg);
            }
            Err(panic_info) => {
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    format!("Indexing panicked: {}", s)
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    format!("Indexing panicked: {}", s)
                } else {
                    "Indexing panicked with unknown error".to_string()
                };
                tracing::error!("{}", panic_msg);
                let mut status = self.status.lock().await;
                status.is_indexing = false;
                status.last_error = Some(panic_msg);
            }
        }

        // Return the original error if any
        let status = self.status.lock().await;
        if let Some(ref err) = status.last_error {
            anyhow::bail!("{}", err);
        }
        Ok(())
    }

    async fn do_index(&self) -> Result<()> {
        let files = walk_directory(&self.upload_dir);
        tracing::info!("Indexing {} files from {}", files.len(), self.upload_dir.display());

        let mut success_count = 0usize;
        let mut total_chunks = 0usize;
        let mut failed_files = Vec::new();
        let mut current_ids: HashSet<String> = HashSet::new();

        // Collect all file hashes for files on disk (including ones that fail)
        let existing_file_hashes: HashSet<String> = files.iter()
            .map(|(path, _)| file_id(path))
            .collect();

        for (path, format) in &files {
            match self.process_file(path, *format).await {
                Ok(chunk_ids) => {
                    current_ids.extend(chunk_ids.iter().cloned());
                    total_chunks += chunk_ids.len();
                    success_count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to index {}: {}", path.display(), e);
                    failed_files.push(
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.display().to_string())
                    );
                }
            }
        }

        // Stale cleanup: delete points whose file no longer exists on disk
        match self.vector_store.scroll_all_point_ids().await {
            Ok(all_ids) => {
                let stale_ids: Vec<String> = all_ids.into_iter()
                    .filter(|id| {
                        let file_hash = id.split('_').next().unwrap_or("");
                        !existing_file_hashes.contains(file_hash)
                    })
                    .collect();

                if !stale_ids.is_empty() {
                    tracing::info!("Cleaning up {} stale points", stale_ids.len());
                    if let Err(e) = self.vector_store.delete_points(stale_ids).await {
                        tracing::error!("Failed to clean up stale points: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to scroll point IDs for cleanup: {}", e);
            }
        }

        // Update status
        {
            let mut status = self.status.lock().await;
            status.total_files = success_count;
            status.total_chunks = total_chunks;
            status.failed_files = failed_files;
        }

        tracing::info!("Indexing complete: {} files, {} chunks", success_count, total_chunks);
        Ok(())
    }

    async fn process_file(&self, path: &Path, format: SupportedFormat) -> Result<Vec<String>> {
        let text = extract_text(path, format)?;
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let chunks = chunk_text(&text, 1000, 200);
        let path_id = file_id(path);
        let mut chunk_ids = Vec::new();

        let batch_size = 32;
        for batch in chunks.chunks(batch_size) {
            let texts: Vec<String> = batch.iter().map(|c| c.text.clone()).collect();
            let embeddings_batch = self.embeddings.generate(texts)?;

            for (chunk, embedding) in batch.iter().zip(embeddings_batch.into_iter()) {
                let chunk_id = format!("{}_{}", path_id, chunk.chunk_index);
                let metadata = serde_json::json!({
                    "file_path": path.to_string_lossy(),
                    "chunk_index": chunk.chunk_index,
                    "format": format!("{:?}", format),
                });

                self.vector_store.add_document(&chunk_id, &chunk.text, embedding, metadata).await?;
                chunk_ids.push(chunk_id);
            }
        }

        Ok(chunk_ids)
    }

    pub fn start_scheduler(manager: Arc<Self>) {
        tokio::spawn(async move {
            // Wait before first run to let services start
            tokio::time::sleep(Duration::from_secs(60)).await;

            loop {
                tracing::info!("Scheduled indexing starting...");
                if let Err(e) = manager.run_index().await {
                    tracing::error!("Scheduled indexing failed: {}", e);
                }

                let interval_minutes = {
                    manager.status.lock().await.auto_index_interval_minutes
                };
                tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;
            }
        });
    }
}
