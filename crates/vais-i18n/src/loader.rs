//! Message file loading and caching

use crate::Locale;
use serde_json::Value;
use std::collections::HashMap;

/// Embedded message files
mod embedded {
    pub const EN_MESSAGES: &str = include_str!("../locales/en.json");
    pub const KO_MESSAGES: &str = include_str!("../locales/ko.json");
    pub const JA_MESSAGES: &str = include_str!("../locales/ja.json");
    pub const ZH_MESSAGES: &str = include_str!("../locales/zh.json");
}

/// Load messages for a given locale
///
/// Messages are embedded at compile time for zero runtime I/O.
pub fn load_messages(locale: Locale) -> HashMap<String, String> {
    let json_str = match locale {
        Locale::En => embedded::EN_MESSAGES,
        Locale::Ko => embedded::KO_MESSAGES,
        Locale::Ja => embedded::JA_MESSAGES,
        Locale::Zh => embedded::ZH_MESSAGES,
    };

    parse_messages(json_str).unwrap_or_default()
}

/// Load fallback messages (English)
pub fn load_fallback() -> HashMap<String, String> {
    load_messages(Locale::En)
}

/// Parse JSON messages into a flat key-value map
///
/// Converts nested JSON structure to flat keys:
/// ```text
/// { "type": { "E001": { "title": "Type mismatch" } } }
/// ```
/// becomes:
/// ```text
/// "type.E001.title" -> "Type mismatch"
/// ```
fn parse_messages(json_str: &str) -> Result<HashMap<String, String>, serde_json::Error> {
    let value: Value = serde_json::from_str(json_str)?;
    let mut messages = HashMap::new();
    flatten_json(&value, String::new(), &mut messages);
    Ok(messages)
}

/// Recursively flatten a JSON value into dot-separated keys
fn flatten_json(value: &Value, prefix: String, messages: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                // Skip metadata
                if key.starts_with('_') {
                    continue;
                }

                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                flatten_json(val, new_prefix, messages);
            }
        }
        Value::String(s) => {
            messages.insert(prefix, s.clone());
        }
        _ => {
            // Ignore non-string, non-object values
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_messages() {
        let json = r#"{
            "type": {
                "E001": {
                    "title": "Type mismatch",
                    "message": "expected {expected}, found {found}"
                }
            }
        }"#;

        let messages = parse_messages(json).unwrap();

        assert_eq!(
            messages.get("type.E001.title"),
            Some(&"Type mismatch".to_string())
        );
        assert_eq!(
            messages.get("type.E001.message"),
            Some(&"expected {expected}, found {found}".to_string())
        );
    }

    #[test]
    fn test_load_messages() {
        let en_messages = load_messages(Locale::En);
        assert!(en_messages.contains_key("type.E001.title"));

        let ko_messages = load_messages(Locale::Ko);
        assert!(ko_messages.contains_key("type.E001.title"));

        let ja_messages = load_messages(Locale::Ja);
        assert!(ja_messages.contains_key("type.E001.title"));

        let zh_messages = load_messages(Locale::Zh);
        assert!(zh_messages.contains_key("type.E001.title"));
    }

    #[test]
    fn test_skip_metadata() {
        let json = r#"{
            "_meta": { "version": "1.0" },
            "type": { "E001": { "title": "Error" } }
        }"#;

        let messages = parse_messages(json).unwrap();

        assert!(!messages.contains_key("_meta.version"));
        assert!(messages.contains_key("type.E001.title"));
    }
}
