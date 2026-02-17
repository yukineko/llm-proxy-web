use std::path::Path;
use std::io::Read;
use anyhow::{Result, Context};
use super::walker::SupportedFormat;

pub fn extract_text(path: &Path, format: SupportedFormat) -> Result<String> {
    match format {
        SupportedFormat::PlainText => extract_plain_text(path),
        SupportedFormat::Pdf => extract_pdf(path),
        SupportedFormat::Docx => extract_docx(path),
        SupportedFormat::Xlsx => extract_xlsx(path),
        SupportedFormat::Pptx => extract_pptx(path),
    }
}

fn extract_plain_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read text file: {}", path.display()))
}

fn extract_pdf(path: &Path) -> Result<String> {
    let text = pdf_extract::extract_text(path)
        .with_context(|| format!("Failed to extract PDF text: {}", path.display()))?;
    Ok(text)
}

fn extract_docx(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open DOCX: {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read DOCX as ZIP: {}", path.display()))?;

    let mut xml_content = String::new();
    if let Ok(mut entry) = archive.by_name("word/document.xml") {
        entry.read_to_string(&mut xml_content)?;
    } else {
        anyhow::bail!("No word/document.xml found in DOCX");
    }

    Ok(extract_text_from_xml(&xml_content, "w:t"))
}

fn extract_xlsx(path: &Path) -> Result<String> {
    use calamine::{Reader, open_workbook, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(path)
        .with_context(|| format!("Failed to open XLSX: {}", path.display()))?;

    let mut all_text = Vec::new();
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                let row_text: Vec<String> = row.iter()
                    .map(|cell| cell.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !row_text.is_empty() {
                    all_text.push(row_text.join("\t"));
                }
            }
        }
    }

    Ok(all_text.join("\n"))
}

fn extract_pptx(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open PPTX: {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read PPTX as ZIP: {}", path.display()))?;

    let mut all_text = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;
            let text = extract_text_from_xml(&xml_content, "a:t");
            if !text.is_empty() {
                all_text.push(text);
            }
        }
    }

    Ok(all_text.join("\n\n"))
}

fn extract_text_from_xml(xml: &str, tag: &str) -> String {
    let open_tag = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    let mut texts = Vec::new();
    let mut search_from = 0;

    while let Some(open_pos) = xml[search_from..].find(&open_tag) {
        let abs_open = search_from + open_pos;
        // Find the end of the opening tag (handle attributes)
        if let Some(tag_end) = xml[abs_open..].find('>') {
            let content_start = abs_open + tag_end + 1;
            if let Some(close_pos) = xml[content_start..].find(&close_tag) {
                let content = &xml[content_start..content_start + close_pos];
                if !content.is_empty() {
                    texts.push(content.to_string());
                }
                search_from = content_start + close_pos + close_tag.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    texts.join(" ")
}
