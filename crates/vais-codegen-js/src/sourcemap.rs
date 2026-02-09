//! Source Map v3 Generator
//!
//! Implements the Source Map v3 specification for mapping JavaScript output
//! back to original Vais source files.
//!
//! # References
//!
//! - [Source Map Revision 3 Proposal](https://sourcemaps.info/spec.html)
//! - [Google Source Map v3 Format](https://docs.google.com/document/d/1U1RGAehQwRypUTovF1KRlpiOFze0b-_2gc6fAH0KY0k)

/// A source map for mapping generated JavaScript to original Vais source
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Original .vais source file path
    source_file: String,
    /// Generated .js output file path
    generated_file: String,
    /// List of source→generated position mappings
    mappings: Vec<Mapping>,
}

/// A single position mapping from source to generated location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mapping {
    /// 0-indexed line number in generated .js file
    generated_line: u32,
    /// 0-indexed column number in generated .js file
    generated_col: u32,
    /// 0-indexed line number in source .vais file
    source_line: u32,
    /// 0-indexed column number in source .vais file
    source_col: u32,
}

impl SourceMap {
    /// Create a new source map
    ///
    /// # Arguments
    ///
    /// * `source` - Path to the original .vais source file
    /// * `generated` - Path to the generated .js output file
    pub fn new(source: &str, generated: &str) -> Self {
        Self {
            source_file: source.to_string(),
            generated_file: generated.to_string(),
            mappings: Vec::new(),
        }
    }

    /// Add a mapping from generated position to source position
    ///
    /// # Arguments
    ///
    /// * `gen_line` - 0-indexed line number in generated .js file
    /// * `gen_col` - 0-indexed column number in generated .js file
    /// * `src_line` - 0-indexed line number in source .vais file
    /// * `src_col` - 0-indexed column number in source .vais file
    pub fn add_mapping(&mut self, gen_line: u32, gen_col: u32, src_line: u32, src_col: u32) {
        self.mappings.push(Mapping {
            generated_line: gen_line,
            generated_col: gen_col,
            source_line: src_line,
            source_col: src_col,
        });
    }

    /// Generate the JSON source map string
    ///
    /// Returns a JSON string conforming to Source Map v3 format.
    pub fn to_json(&self) -> String {
        let mappings_str = self.encode_mappings();

        format!(
            r#"{{"version":3,"file":"{}","sourceRoot":"","sources":["{}"],"names":[],"mappings":"{}"}}"#,
            escape_json(&self.generated_file),
            escape_json(&self.source_file),
            mappings_str
        )
    }

    /// Generate an inline source map comment with data URI
    ///
    /// Returns a comment like:
    /// `//# sourceMappingURL=data:application/json;charset=utf-8;base64,...`
    pub fn to_inline_comment(&self) -> String {
        let json = self.to_json();
        let base64 = base64_encode(json.as_bytes());
        format!("//# sourceMappingURL=data:application/json;charset=utf-8;base64,{}", base64)
    }

    /// Generate a file reference source map comment
    ///
    /// # Arguments
    ///
    /// * `map_file` - Path to the .map file (typically `output.js.map`)
    ///
    /// Returns a comment like: `//# sourceMappingURL=output.js.map`
    pub fn to_file_comment(map_file: &str) -> String {
        format!("//# sourceMappingURL={}", map_file)
    }

    /// Encode mappings into VLQ Base64 format
    fn encode_mappings(&self) -> String {
        if self.mappings.is_empty() {
            return String::new();
        }

        // Sort mappings by generated position (line, then column)
        let mut sorted = self.mappings.clone();
        sorted.sort_by_key(|m| (m.generated_line, m.generated_col));

        let mut result = String::new();
        let mut current_gen_line = 0;
        let mut prev_gen_col = 0;
        let mut prev_src_line = 0;
        let mut prev_src_col = 0;
        let mut first_in_line = true;

        for mapping in &sorted {
            // Add line separators (;) for each new line
            while current_gen_line < mapping.generated_line {
                result.push(';');
                current_gen_line += 1;
                prev_gen_col = 0; // Reset column on new line
                first_in_line = true;
            }

            // Add comma separator between mappings on same line
            if !first_in_line {
                result.push(',');
            }
            first_in_line = false;

            // Encode 5-tuple: [gen_col, source_idx, src_line, src_col, name_idx]
            // We use 4-tuple since we don't track names: [gen_col, source_idx, src_line, src_col]
            let gen_col_delta = mapping.generated_col as i32 - prev_gen_col as i32;
            let src_idx_delta = 0; // Always 0 (first source)
            let src_line_delta = mapping.source_line as i32 - prev_src_line as i32;
            let src_col_delta = mapping.source_col as i32 - prev_src_col as i32;

            encode_vlq_value(gen_col_delta, &mut result);
            encode_vlq_value(src_idx_delta, &mut result);
            encode_vlq_value(src_line_delta, &mut result);
            encode_vlq_value(src_col_delta, &mut result);

            // Update state
            prev_gen_col = mapping.generated_col;
            prev_src_line = mapping.source_line;
            prev_src_col = mapping.source_col;
        }

        result
    }
}

/// Encode a signed integer as VLQ (Variable Length Quantity) Base64
///
/// VLQ encoding:
/// 1. Convert to unsigned: sign bit becomes LSB (value << 1 if positive, else (-value << 1) | 1)
/// 2. Split into 5-bit chunks (6 bits - 1 continuation bit)
/// 3. Set continuation bit (bit 5) for all chunks except the last
/// 4. Encode each 6-bit value as base64
fn encode_vlq_value(value: i32, output: &mut String) {
    // Step 1: Convert signed to unsigned with sign bit as LSB
    let mut vlq = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    // Step 2-4: Encode as base64 chunks
    loop {
        let mut digit = vlq & 0b11111; // Take 5 bits
        vlq >>= 5;

        if vlq > 0 {
            digit |= 0b100000; // Set continuation bit
        }

        output.push(base64_digit(digit as u8));

        if vlq == 0 {
            break;
        }
    }
}

/// Convert a 6-bit value (0-63) to a Base64 character
///
/// Base64 alphabet: A-Z (0-25), a-z (26-51), 0-9 (52-61), + (62), / (63)
fn base64_digit(value: u8) -> char {
    match value {
        0..=25 => (b'A' + value) as char,
        26..=51 => (b'a' + (value - 26)) as char,
        52..=61 => (b'0' + (value - 52)) as char,
        62 => '+',
        63 => '/',
        _ => panic!("Invalid base64 digit: {}", value),
    }
}

/// Escape a string for JSON
fn escape_json(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '"' => result.push_str(r#"\""#),
            '\\' => result.push_str(r#"\\"#),
            '\n' => result.push_str(r#"\n"#),
            '\r' => result.push_str(r#"\r"#),
            '\t' => result.push_str(r#"\t"#),
            _ => result.push(ch),
        }
    }
    result
}

/// Encode bytes as Base64
fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(base64_digit(((n >> 18) & 63) as u8));
        result.push(base64_digit(((n >> 12) & 63) as u8));
        result.push(if i + 1 < data.len() {
            base64_digit(((n >> 6) & 63) as u8)
        } else {
            '='
        });
        result.push(if i + 2 < data.len() {
            base64_digit((n & 63) as u8)
        } else {
            '='
        });

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_encoding_zero() {
        let mut output = String::new();
        encode_vlq_value(0, &mut output);
        assert_eq!(output, "A"); // 0 -> 0b000000 -> 'A'
    }

    #[test]
    fn test_vlq_encoding_one() {
        let mut output = String::new();
        encode_vlq_value(1, &mut output);
        assert_eq!(output, "C"); // 1 -> 0b000010 -> 'C'
    }

    #[test]
    fn test_vlq_encoding_negative_one() {
        let mut output = String::new();
        encode_vlq_value(-1, &mut output);
        assert_eq!(output, "D"); // -1 -> 0b000011 -> 'D'
    }

    #[test]
    fn test_vlq_encoding_sixteen() {
        let mut output = String::new();
        encode_vlq_value(16, &mut output);
        assert_eq!(output, "gB"); // 16 -> 0b100000 -> continuation + 'B'
    }

    #[test]
    fn test_vlq_encoding_large_number() {
        let mut output = String::new();
        encode_vlq_value(123, &mut output);
        // 123 -> 246 (0b11110110)
        // First 5 bits: 0b10110 (22) + continuation (54) -> '2'
        // Next 5 bits: 0b00111 (7) -> 'H'
        assert_eq!(output, "2H");
    }

    #[test]
    fn test_vlq_encoding_negative_large() {
        let mut output = String::new();
        encode_vlq_value(-123, &mut output);
        // -123 -> 247 (0b11110111)
        assert_eq!(output, "3H");
    }

    #[test]
    fn test_simple_mapping_serialization() {
        let mut map = SourceMap::new("src/main.vais", "dist/main.js");
        map.add_mapping(0, 0, 0, 0); // First line maps to first line

        let encoded = map.encode_mappings();
        // Expected: gen_col=0, src_idx=0, src_line=0, src_col=0
        // All zeros: AAAA
        assert_eq!(encoded, "AAAA");
    }

    #[test]
    fn test_multiple_mappings_same_line() {
        let mut map = SourceMap::new("test.vais", "test.js");
        map.add_mapping(0, 0, 0, 0);  // Column 0
        map.add_mapping(0, 10, 0, 5); // Column 10, delta +10, src col delta +5

        let encoded = map.encode_mappings();
        // First: AAAA
        // Second (comma separated): U (10), A (0), A (0), K (5)
        // U = 20 (delta 10 << 1), K = 10 (delta 5 << 1)
        assert_eq!(encoded, "AAAA,UAAK");
    }

    #[test]
    fn test_multiple_lines() {
        let mut map = SourceMap::new("test.vais", "test.js");
        map.add_mapping(0, 0, 0, 0);
        map.add_mapping(1, 0, 1, 0);

        let encoded = map.encode_mappings();
        // Line 0: AAAA
        // Semicolon for line 1
        // Line 1: AAAA (all deltas are 0 from previous line's final state)
        // BUT: gen_col resets to 0 on new line, so first delta is 0
        // src_line delta: 1-0=1 -> E
        assert_eq!(encoded, "AAAA;AACA");
    }

    #[test]
    fn test_json_output_format() {
        let mut map = SourceMap::new("example.vais", "example.js");
        map.add_mapping(0, 0, 0, 0);

        let json = map.to_json();
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""file":"example.js""#));
        assert!(json.contains(r#""sources":["example.vais"]"#));
        assert!(json.contains(r#""mappings":"AAAA""#));
    }

    #[test]
    fn test_inline_comment_generation() {
        let mut map = SourceMap::new("test.vais", "test.js");
        map.add_mapping(0, 0, 0, 0);

        let comment = map.to_inline_comment();
        assert!(comment.starts_with("//# sourceMappingURL=data:application/json;charset=utf-8;base64,"));

        // Verify it contains valid base64
        let base64_part = comment.strip_prefix("//# sourceMappingURL=data:application/json;charset=utf-8;base64,").unwrap();
        assert!(!base64_part.is_empty());
    }

    #[test]
    fn test_file_comment_generation() {
        let comment = SourceMap::to_file_comment("output.js.map");
        assert_eq!(comment, "//# sourceMappingURL=output.js.map");
    }

    #[test]
    fn test_base64_encoding() {
        // Test basic base64 encoding
        assert_eq!(base64_encode(b"A"), "QQ==");
        assert_eq!(base64_encode(b"AB"), "QUI=");
        assert_eq!(base64_encode(b"ABC"), "QUJD");
    }

    #[test]
    fn test_json_escaping() {
        assert_eq!(escape_json(r#"test"quote"#), r#"test\"quote"#);
        assert_eq!(escape_json("test\nline"), r#"test\nline"#);
        assert_eq!(escape_json(r#"path\file"#), r#"path\\file"#);
    }

    #[test]
    fn test_base64_digit_ranges() {
        assert_eq!(base64_digit(0), 'A');
        assert_eq!(base64_digit(25), 'Z');
        assert_eq!(base64_digit(26), 'a');
        assert_eq!(base64_digit(51), 'z');
        assert_eq!(base64_digit(52), '0');
        assert_eq!(base64_digit(61), '9');
        assert_eq!(base64_digit(62), '+');
        assert_eq!(base64_digit(63), '/');
    }

    #[test]
    fn test_realistic_source_map() {
        let mut map = SourceMap::new("src/example.vais", "dist/example.js");

        // Simulate a simple function:
        // Vais (line 0): F add(a: i64, b: i64) -> i64 { a + b }
        // JS (line 0): function add(a, b) {
        // JS (line 1):   return a + b;
        // JS (line 2): }

        map.add_mapping(0, 0, 0, 0);   // function -> F
        map.add_mapping(0, 9, 0, 2);   // add -> add
        map.add_mapping(1, 2, 0, 37);  // return -> R or body start
        map.add_mapping(1, 9, 0, 38);  // a -> a
        map.add_mapping(1, 13, 0, 42); // b -> b

        let json = map.to_json();
        assert!(json.contains(r#""version":3"#));
        assert!(json.contains(r#""sources":["src/example.vais"]"#));

        // Verify the encoded mappings are non-empty
        let encoded = map.encode_mappings();
        assert!(!encoded.is_empty());
        assert!(encoded.contains(';')); // Should have line separator
        assert!(encoded.contains(',')); // Should have comma separator
    }

    #[test]
    fn test_empty_source_map() {
        let map = SourceMap::new("empty.vais", "empty.js");
        let encoded = map.encode_mappings();
        assert_eq!(encoded, "");

        let json = map.to_json();
        assert!(json.contains(r#""mappings":""#));
        assert!(json.contains(r#""file":"empty.js""#));
    }
}
