//! Message formatting and variable substitution

use crate::loader::{load_fallback, load_messages};
use crate::Locale;
use std::collections::HashMap;

/// Internationalization message system
///
/// Provides localized messages with variable substitution support.
/// Falls back to English if a message is not found in the current locale.
pub struct I18n {
    locale: Locale,
    messages: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

impl I18n {
    /// Create a new I18n instance with system locale detection
    pub fn new() -> Self {
        Self::with_locale(Locale::detect())
    }

    /// Create a new I18n instance with a specific locale
    pub fn with_locale(locale: Locale) -> Self {
        let messages = load_messages(locale);
        let fallback = if locale != Locale::En {
            load_fallback()
        } else {
            HashMap::new()
        };

        Self {
            locale,
            messages,
            fallback,
        }
    }

    /// Get the current locale
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Set the locale and reload messages
    pub fn set_locale(&mut self, locale: Locale) {
        if self.locale != locale {
            self.locale = locale;
            self.messages = load_messages(locale);
            self.fallback = if locale != Locale::En {
                load_fallback()
            } else {
                HashMap::new()
            };
        }
    }

    /// Get a message with variable substitution
    ///
    /// # Arguments
    /// * `key` - Message key (e.g., "type.E001.message")
    /// * `args` - Variable substitutions as (name, value) pairs
    ///
    /// # Returns
    /// The message with variables substituted, or the key if not found
    ///
    /// # Example
    /// ```ignore
    /// let msg = i18n.get("type.E001.message", &[("expected", "i64"), ("found", "Str")]);
    /// // Returns: "expected i64, found Str"
    /// ```
    pub fn get(&self, key: &str, args: &[(&str, &str)]) -> String {
        // Try current locale first, then fallback to English
        let template = self
            .messages
            .get(key)
            .or_else(|| self.fallback.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key);

        substitute_variables(template, args)
    }

    /// Get a simple message without variable substitution
    pub fn get_simple(&self, key: &str) -> String {
        self.get(key, &[])
    }

    /// Check if a message key exists
    pub fn has_key(&self, key: &str) -> bool {
        self.messages.contains_key(key) || self.fallback.contains_key(key)
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

/// Substitute variables in a message template
///
/// Variables are in the format `{name}` and will be replaced with their values.
fn substitute_variables(template: &str, args: &[(&str, &str)]) -> String {
    let mut result = template.to_string();

    for (name, value) in args {
        let placeholder = format!("{{{}}}", name);
        result = result.replace(&placeholder, value);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple() {
        let i18n = I18n::with_locale(Locale::En);
        let title = i18n.get_simple("type.E001.title");
        assert_eq!(title, "Type mismatch");
    }

    #[test]
    fn test_get_with_args() {
        let i18n = I18n::with_locale(Locale::En);
        let msg = i18n.get("type.E001.message", &[("expected", "i64"), ("found", "Str")]);
        assert_eq!(msg, "expected i64, found Str");
    }

    #[test]
    fn test_fallback() {
        let i18n = I18n::with_locale(Locale::Ko);

        // This should return the Korean message
        let title = i18n.get_simple("type.E001.title");
        assert_eq!(title, "타입 불일치");
    }

    #[test]
    fn test_substitute_variables() {
        let result = substitute_variables(
            "expected {expected}, found {found}",
            &[("expected", "i64"), ("found", "Str")],
        );
        assert_eq!(result, "expected i64, found Str");
    }

    #[test]
    fn test_substitute_no_variables() {
        let result = substitute_variables("Type mismatch", &[]);
        assert_eq!(result, "Type mismatch");
    }

    #[test]
    fn test_substitute_unused_args() {
        let result = substitute_variables("Type mismatch", &[("unused", "value")]);
        assert_eq!(result, "Type mismatch");
    }

    #[test]
    fn test_missing_key_returns_key() {
        let i18n = I18n::with_locale(Locale::En);
        let msg = i18n.get_simple("nonexistent.key");
        assert_eq!(msg, "nonexistent.key");
    }

    #[test]
    fn test_set_locale() {
        let mut i18n = I18n::with_locale(Locale::En);
        assert_eq!(i18n.get_simple("type.E001.title"), "Type mismatch");

        i18n.set_locale(Locale::Ko);
        assert_eq!(i18n.get_simple("type.E001.title"), "타입 불일치");

        i18n.set_locale(Locale::Ja);
        assert_eq!(i18n.get_simple("type.E001.title"), "型の不一致");
    }
}
