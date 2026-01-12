//! Document management

use ropey::Rope;
use tower_lsp::lsp_types::*;

/// 문서 상태
#[derive(Debug, Clone)]
pub struct Document {
    /// 문서 내용
    pub content: Rope,
    /// 버전
    pub version: i32,
}

impl Document {
    pub fn new(content: String, version: i32) -> Self {
        Self {
            content: Rope::from_str(&content),
            version,
        }
    }

    /// 전체 텍스트 반환
    pub fn text(&self) -> String {
        self.content.to_string()
    }

    /// 오프셋을 Position으로 변환
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self.content.char_to_line(offset.min(self.content.len_chars()));
        let line_start = self.content.line_to_char(line);
        let character = offset.saturating_sub(line_start);

        Position {
            line: line as u32,
            character: character as u32,
        }
    }

    /// Position을 오프셋으로 변환
    pub fn position_to_offset(&self, position: Position) -> usize {
        let line = position.line as usize;
        if line >= self.content.len_lines() {
            return self.content.len_chars();
        }
        let line_start = self.content.line_to_char(line);
        let line_len = self.content.line(line).len_chars();
        let character = (position.character as usize).min(line_len);
        line_start + character
    }

    /// 증분 업데이트
    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        if let Some(range) = change.range {
            let start = self.position_to_offset(range.start);
            let end = self.position_to_offset(range.end);
            self.content.remove(start..end);
            self.content.insert(start, &change.text);
        } else {
            // 전체 교체
            self.content = Rope::from_str(&change.text);
        }
    }
}
