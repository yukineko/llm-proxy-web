#[derive(Debug, Clone)]
pub struct TextChunk {
    pub text: String,
    pub chunk_index: usize,
}

/// バイト位置をchar境界に切り上げる
fn ceil_char_boundary(text: &str, byte_pos: usize) -> usize {
    if byte_pos >= text.len() {
        return text.len();
    }
    let mut pos = byte_pos;
    while pos < text.len() && !text.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

/// バイト位置をchar境界に切り下げる
fn floor_char_boundary(text: &str, byte_pos: usize) -> usize {
    if byte_pos >= text.len() {
        return text.len();
    }
    let mut pos = byte_pos;
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

pub fn chunk_text(text: &str, max_chunk_size: usize, overlap: usize) -> Vec<TextChunk> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    if text.len() <= max_chunk_size {
        return vec![TextChunk {
            text: text.to_string(),
            chunk_index: 0,
        }];
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let mut chunk_index = 0;

    while start < text.len() {
        let end = ceil_char_boundary(text, (start + max_chunk_size).min(text.len()));

        let actual_end = if end < text.len() {
            find_break_point(text, start, end)
        } else {
            end
        };

        let chunk_text = text[start..actual_end].trim().to_string();
        if !chunk_text.is_empty() {
            chunks.push(TextChunk {
                text: chunk_text,
                chunk_index,
            });
            chunk_index += 1;
        }

        let next_start = if actual_end > overlap {
            floor_char_boundary(text, actual_end - overlap)
        } else {
            actual_end
        };

        if next_start <= start {
            start = actual_end;
        } else {
            start = next_start;
        }
    }

    chunks
}

fn find_break_point(text: &str, start: usize, max_end: usize) -> usize {
    let segment = &text[start..max_end];

    if let Some(pos) = segment.rfind("\n\n") {
        return start + pos + 2;
    }
    if let Some(pos) = segment.rfind('\n') {
        return start + pos + 1;
    }
    // 日本語の句読点も区切りとして扱う
    for sentinel in ["。", "？", "！", ". ", "? ", "! "] {
        if let Some(pos) = segment.rfind(sentinel) {
            return start + pos + sentinel.len();
        }
    }
    if let Some(pos) = segment.rfind(' ') {
        return start + pos + 1;
    }
    max_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_text_chunking() {
        let text = "これはテスト文章です。日本語のマルチバイト文字を含むテキストを正しくチャンクに分割できるかテストします。句読点で分割されることを確認します。";
        let chunks = chunk_text(text, 60, 10);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
        }
    }

    #[test]
    fn test_mixed_text_chunking() {
        let text = "AI Security Conference 2026 イベントレポート。最新のセキュリティ技術について検討しました。参加者は100名を超えました。";
        let chunks = chunk_text(text, 50, 10);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
        }
    }

    #[test]
    fn test_small_text_single_chunk() {
        let text = "short";
        let chunks = chunk_text(text, 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "short");
    }

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", 100, 10);
        assert!(chunks.is_empty());
    }
}
