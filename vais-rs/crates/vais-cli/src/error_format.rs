//! Error Formatting Module
//!
//! 사용자 친화적인 에러 메시지 포맷팅을 제공합니다.
//! - 소스 코드 위치 표시
//! - 컬러 출력 지원
//! - 에러 유형별 제안사항

#![allow(dead_code)]

use std::fmt::Write;
use vais_lexer::Span;
use vais_parser::ParseError;
use vais_typeck::TypeError;
use vais_vm::RuntimeError;

/// ANSI 색상 코드
mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// 에러 포맷터
pub struct ErrorFormatter<'a> {
    source: &'a str,
    file_name: &'a str,
    use_colors: bool,
}

impl<'a> ErrorFormatter<'a> {
    pub fn new(source: &'a str, file_name: &'a str, use_colors: bool) -> Self {
        Self {
            source,
            file_name,
            use_colors,
        }
    }

    /// 파서 에러 포맷
    pub fn format_parse_error(&self, error: &ParseError) -> String {
        let mut output = String::new();

        let header = self.format_header("Parse Error", error.to_string());
        output.push_str(&header);

        let span = error.span();
        output.push_str(&self.format_source_location(span));

        // 에러 유형별 제안사항
        let suggestion = self.get_parse_suggestion(error);
        if !suggestion.is_empty() {
            output.push_str(&self.format_suggestion(&suggestion));
        }

        output
    }

    /// 타입 에러 포맷
    pub fn format_type_error(&self, error: &TypeError) -> String {
        let mut output = String::new();

        let header = self.format_header("Type Error", error.to_string());
        output.push_str(&header);

        if let Some(span) = self.get_type_error_span(error) {
            output.push_str(&self.format_source_location(span));
        }

        let suggestion = self.get_type_suggestion(error);
        if !suggestion.is_empty() {
            output.push_str(&self.format_suggestion(&suggestion));
        }

        output
    }

    /// 런타임 에러 포맷
    pub fn format_runtime_error(&self, error: &RuntimeError) -> String {
        let mut output = String::new();

        let header = self.format_header("Runtime Error", error.to_string());
        output.push_str(&header);

        let suggestion = self.get_runtime_suggestion(error);
        if !suggestion.is_empty() {
            output.push_str(&self.format_suggestion(&suggestion));
        }

        output
    }

    /// 헤더 포맷
    fn format_header(&self, error_type: &str, message: String) -> String {
        if self.use_colors {
            format!(
                "{}{}error[{}]{}: {}\n",
                colors::BOLD,
                colors::RED,
                error_type,
                colors::RESET,
                message
            )
        } else {
            format!("error[{}]: {}\n", error_type, message)
        }
    }

    /// 소스 코드 위치 표시
    fn format_source_location(&self, span: Span) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = self.source.lines().collect();

        // span.start로부터 행/열 계산
        let (line_num, col) = self.position_to_line_col(span.start);

        // 파일 위치 출력
        if self.use_colors {
            writeln!(
                output,
                "  {}-->{} {}:{}:{}",
                colors::BLUE,
                colors::RESET,
                self.file_name,
                line_num + 1,
                col + 1
            )
            .ok();
        } else {
            writeln!(output, "  --> {}:{}:{}", self.file_name, line_num + 1, col + 1).ok();
        }

        // 소스 코드 라인 출력
        if line_num < lines.len() {
            let line = lines[line_num];
            let line_num_str = format!("{}", line_num + 1);
            let padding = " ".repeat(line_num_str.len());

            if self.use_colors {
                writeln!(output, "{}   |", colors::BLUE).ok();
                writeln!(output, "{} {} | {}{}", colors::BLUE, line_num_str, colors::RESET, line)
                    .ok();
                write!(output, "{}   |{} ", colors::BLUE, colors::RESET).ok();
            } else {
                writeln!(output, "{}   |", padding).ok();
                writeln!(output, "{} | {}", line_num_str, line).ok();
                write!(output, "{}   | ", padding).ok();
            }

            // 에러 위치 표시
            let pointer_padding = " ".repeat(col);
            let pointer_len = (span.end.saturating_sub(span.start)).max(1).min(line.len().saturating_sub(col));
            let pointer = "^".repeat(pointer_len);

            if self.use_colors {
                writeln!(output, "{}{}{}", pointer_padding, colors::RED, pointer).ok();
                writeln!(output, "{}   |{}", colors::BLUE, colors::RESET).ok();
            } else {
                writeln!(output, "{}{}", pointer_padding, pointer).ok();
                writeln!(output, "{}   |", padding).ok();
            }
        }

        output
    }

    /// 제안사항 포맷
    fn format_suggestion(&self, suggestion: &str) -> String {
        if self.use_colors {
            format!(
                "{}{}help{}: {}\n",
                colors::BOLD,
                colors::CYAN,
                colors::RESET,
                suggestion
            )
        } else {
            format!("help: {}\n", suggestion)
        }
    }

    /// 오프셋을 행/열로 변환
    fn position_to_line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;

        for (pos, ch) in self.source.chars().enumerate() {
            if pos >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// 파서 에러 제안사항
    fn get_parse_suggestion(&self, error: &ParseError) -> String {
        match error {
            ParseError::UnexpectedToken { expected, found, .. } => {
                if expected.contains("expression") {
                    "식(expression)이 필요합니다. 값, 변수, 또는 함수 호출을 입력하세요.".to_string()
                } else if expected.contains(")") && matches!(found, vais_lexer::TokenKind::Newline) {
                    "닫는 괄호 ')'가 필요합니다. 함수 호출이나 괄호가 제대로 닫혔는지 확인하세요.".to_string()
                } else {
                    format!("'{}'이(가) 예상되었습니다.", expected)
                }
            }
            ParseError::UnexpectedEof { .. } => {
                "파일이 예상치 않게 끝났습니다. 괄호, 중괄호가 제대로 닫혔는지 확인하세요.".to_string()
            }
            ParseError::InvalidNumber { .. } => {
                "유효한 숫자 형식이 아닙니다. 정수(예: 42) 또는 실수(예: 3.14)를 사용하세요.".to_string()
            }
            ParseError::InvalidSyntax { message, .. } => {
                if message.contains("match") {
                    "match 표현식 구문을 확인하세요. 예: match x { 0 => \"zero\", _ => \"other\" }".to_string()
                } else if message.contains("if") {
                    "if 표현식 구문을 확인하세요. 예: if x > 0 then \"positive\" else \"negative\"".to_string()
                } else {
                    String::new()
                }
            }
            ParseError::ModuleNotFound { path, .. } => {
                format!("모듈 '{}'을(를) 찾을 수 없습니다. 파일 경로와 이름을 확인하세요.", path)
            }
            _ => String::new(),
        }
    }

    /// 타입 에러 제안사항
    fn get_type_suggestion(&self, error: &TypeError) -> String {
        match error {
            TypeError::Mismatch { expected, found, .. } => {
                if expected == "Int" && found == "Float" {
                    "정수가 필요합니다. `.floor()` 또는 `.round()`로 변환하세요.".to_string()
                } else if expected == "Float" && found == "Int" {
                    "실수가 필요합니다. `.0`을 붙이거나 `to_float()`을 사용하세요.".to_string()
                } else if expected == "Bool" {
                    "불리언 값(true/false)이 필요합니다.".to_string()
                } else if expected.contains("Array") {
                    "배열이 필요합니다. 예: [1, 2, 3]".to_string()
                } else {
                    format!("{} 타입이 필요하지만 {} 타입이 제공되었습니다.", expected, found)
                }
            }
            TypeError::UndefinedVariable { name, .. } => {
                format!("변수 '{}'이(가) 정의되지 않았습니다. 오타가 있는지 확인하거나, let으로 먼저 정의하세요.", name)
            }
            TypeError::UndefinedFunction { name, .. } => {
                format!("함수 '{}'이(가) 정의되지 않았습니다. 함수 이름을 확인하거나, 먼저 정의하세요.", name)
            }
            TypeError::ArgumentCount { expected, found, .. } => {
                format!(
                    "인자 개수가 맞지 않습니다. {} 개가 필요하지만 {} 개가 제공되었습니다.",
                    expected, found
                )
            }
            TypeError::InvalidOperator { op, ty, .. } => {
                format!("'{}' 연산자를 '{}' 타입에 사용할 수 없습니다.", op, ty)
            }
            TypeError::InvalidIndex { base, index, .. } => {
                if base.contains("Array") {
                    "배열 인덱스는 정수여야 합니다.".to_string()
                } else {
                    format!("'{}' 타입은 '{}' 타입으로 인덱싱할 수 없습니다.", base, index)
                }
            }
            TypeError::NotAFunction { ty, .. } => {
                format!("'{}' 타입은 함수가 아닙니다. 함수처럼 호출할 수 없습니다.", ty)
            }
            _ => String::new(),
        }
    }

    /// 런타임 에러 제안사항
    fn get_runtime_suggestion(&self, error: &RuntimeError) -> String {
        match error {
            RuntimeError::StackUnderflow => {
                "스택이 비어있습니다. 내부 오류일 수 있습니다.".to_string()
            }
            RuntimeError::DivisionByZero => {
                "0으로 나눌 수 없습니다. 나누기 전에 0인지 확인하세요.".to_string()
            }
            RuntimeError::TypeError(msg) => {
                format!("타입이 맞지 않습니다: {}", msg)
            }
            RuntimeError::UndefinedVariable(name) => {
                format!("변수 '{}'이(가) 정의되지 않았습니다.", name)
            }
            RuntimeError::UndefinedFunction(name) => {
                format!("함수 '{}'이(가) 정의되지 않았습니다.", name)
            }
            RuntimeError::IndexOutOfBounds { index, length } => {
                format!(
                    "인덱스 {}이(가) 범위를 벗어났습니다. 유효한 범위: 0..{}",
                    index, length
                )
            }
            RuntimeError::MaxRecursionDepth => {
                "재귀 깊이 제한에 도달했습니다. 종료 조건을 확인하거나 tail recursion을 사용하세요.".to_string()
            }
            RuntimeError::IntegerOverflow => {
                "정수 오버플로우가 발생했습니다. 더 작은 값을 사용하거나 Float을 사용하세요.".to_string()
            }
            RuntimeError::FileNotFound(path) => {
                format!("파일 '{}'을(를) 찾을 수 없습니다. 경로를 확인하세요.", path)
            }
            RuntimeError::PermissionDenied(path) => {
                format!("파일 '{}'에 대한 권한이 없습니다.", path)
            }
            RuntimeError::FfiError(msg) => {
                format!("FFI 호출 오류: {}. 라이브러리가 로드되었는지 확인하세요.", msg)
            }
            _ => String::new(),
        }
    }

    /// 타입 에러의 Span 추출
    fn get_type_error_span(&self, error: &TypeError) -> Option<Span> {
        match error {
            TypeError::Mismatch { span, .. } => Some(*span),
            TypeError::UndefinedVariable { span, .. } => Some(*span),
            TypeError::UndefinedFunction { span, .. } => Some(*span),
            TypeError::ArgumentCount { span, .. } => Some(*span),
            TypeError::RecursiveInference { span } => Some(*span),
            TypeError::InvalidOperator { span, .. } => Some(*span),
            TypeError::InvalidIndex { span, .. } => Some(*span),
            TypeError::InvalidField { span, .. } => Some(*span),
            TypeError::NotAFunction { span, .. } => Some(*span),
            TypeError::PatternMismatch { span, .. } => Some(*span),
            _ => None,
        }
    }
}

/// 에러 요약 출력
pub fn format_error_summary(errors: &[String], warnings: &[String]) -> String {
    let mut output = String::new();

    if !errors.is_empty() {
        output.push_str(&format!(
            "\nerror: {} error(s) found\n",
            errors.len()
        ));
    }

    if !warnings.is_empty() {
        output.push_str(&format!(
            "warning: {} warning(s) emitted\n",
            warnings.len()
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_line_col() {
        let source = "hello\nworld\ntest";
        let formatter = ErrorFormatter::new(source, "test.vais", false);

        assert_eq!(formatter.position_to_line_col(0), (0, 0));
        assert_eq!(formatter.position_to_line_col(5), (0, 5));
        assert_eq!(formatter.position_to_line_col(6), (1, 0));
        assert_eq!(formatter.position_to_line_col(12), (2, 0));
    }

    #[test]
    fn test_format_header_no_color() {
        let formatter = ErrorFormatter::new("", "test.vais", false);
        let header = formatter.format_header("Parse Error", "test message".to_string());
        assert!(header.contains("error[Parse Error]"));
        assert!(header.contains("test message"));
    }

    #[test]
    fn test_format_suggestion() {
        let formatter = ErrorFormatter::new("", "test.vais", false);
        let suggestion = formatter.format_suggestion("Try this instead");
        assert!(suggestion.contains("help: Try this instead"));
    }
}
