use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum SupportedFormat {
    PlainText,
    Pdf,
    Docx,
    Xlsx,
    Pptx,
}

impl SupportedFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "json" | "yaml" | "yml" | "toml" => {
                Some(Self::PlainText)
            }
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            _ => None,
        }
    }
}

pub fn walk_directory(dir: &Path) -> Vec<(PathBuf, SupportedFormat)> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|entry| {
            // Skip .versions directories entirely
            entry.file_name().to_string_lossy() != ".versions"
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            let ext = path.extension()?.to_str()?;
            let format = SupportedFormat::from_extension(ext)?;
            Some((path, format))
        })
        .collect()
}
