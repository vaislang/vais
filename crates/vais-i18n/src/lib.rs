//! Internationalization (i18n) support for Vais compiler
//!
//! Provides localized error messages in multiple languages.
//! Currently supports English (en), Korean (ko), Japanese (ja), and Chinese (zh).
//!
//! # Example
//!
//! ```
//! use vais_i18n::{init, Locale, t, get};
//!
//! // Initialize with system locale detection
//! init(None);
//!
//! // Or with explicit locale
//! init(Some(Locale::Ko));
//!
//! // Get simple message
//! let title = t!("type.E001.title");
//!
//! // Get message with variables
//! let msg = get("type.E001.message", &[("expected", "i64"), ("found", "Str")]);
//! ```

mod loader;
mod locale;
mod message;

pub use locale::Locale;
pub use message::I18n;

use std::sync::OnceLock;
use std::sync::RwLock;

static I18N: OnceLock<RwLock<I18n>> = OnceLock::new();

/// Initialize the global i18n instance
///
/// If `locale` is `None`, the system locale will be auto-detected.
/// This function should be called once at program startup.
pub fn init(locale: Option<Locale>) {
    let i18n = match locale {
        Some(l) => I18n::with_locale(l),
        None => I18n::new(),
    };
    let _ = I18N.set(RwLock::new(i18n));
}

/// Set the locale for the global i18n instance
pub fn set_locale(locale: Locale) {
    if let Some(i18n) = I18N.get() {
        if let Ok(mut guard) = i18n.write() {
            guard.set_locale(locale);
        }
    }
}

/// Get the current locale
pub fn current_locale() -> Locale {
    I18N.get()
        .and_then(|i18n| i18n.read().ok())
        .map(|guard| guard.locale())
        .unwrap_or_default()
}

/// Get a localized message with variable substitution
///
/// # Arguments
/// * `key` - Message key (e.g., "type.E001.message")
/// * `args` - Variable substitutions as (name, value) pairs
///
/// # Returns
/// The localized message with variables substituted, or the key itself if not found
pub fn get(key: &str, args: &[(&str, &str)]) -> String {
    I18N.get()
        .and_then(|i18n| i18n.read().ok())
        .map(|guard| guard.get(key, args))
        .unwrap_or_else(|| key.to_string())
}

/// Get a simple localized message without variable substitution
///
/// # Arguments
/// * `key` - Message key (e.g., "type.E001.title")
///
/// # Returns
/// The localized message, or the key itself if not found
pub fn get_simple(key: &str) -> String {
    get(key, &[])
}

/// Check if a message key exists
pub fn has_key(key: &str) -> bool {
    I18N.get()
        .and_then(|i18n| i18n.read().ok())
        .map(|guard| guard.has_key(key))
        .unwrap_or(false)
}

/// Macro for getting simple messages
///
/// # Example
/// ```
/// use vais_i18n::t;
/// let msg = t!("type.E001.title");
/// ```
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::get_simple($key)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_get() {
        init(Some(Locale::En));
        // Explicitly set locale to ensure English, as init may have been called already
        set_locale(Locale::En);

        let title = get_simple("type.E001.title");
        assert_eq!(title, "Type mismatch");

        let msg = get(
            "type.E001.message",
            &[("expected", "i64"), ("found", "Str")],
        );
        assert_eq!(msg, "expected i64, found Str");
    }

    #[test]
    fn test_korean_locale() {
        init(Some(Locale::Ko));
        set_locale(Locale::Ko);

        let title = get_simple("type.E001.title");
        assert_eq!(title, "타입 불일치");
    }

    #[test]
    fn test_japanese_locale() {
        init(Some(Locale::Ja));
        set_locale(Locale::Ja);

        let title = get_simple("type.E001.title");
        assert_eq!(title, "型の不一致");
    }

    #[test]
    fn test_chinese_locale() {
        // Use I18n directly to avoid OnceLock race conditions
        let i18n = I18n::with_locale(Locale::Zh);
        let title = i18n.get_simple("type.E001.title");
        assert_eq!(title, "类型不匹配");
    }

    #[test]
    fn test_fallback_to_key() {
        init(Some(Locale::En));

        let msg = get_simple("nonexistent.key");
        assert_eq!(msg, "nonexistent.key");
    }

    #[test]
    fn test_has_key() {
        init(Some(Locale::En));

        assert!(has_key("type.E001.title"));
        assert!(!has_key("nonexistent.key"));
    }
}
