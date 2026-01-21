//! Locale detection and management

use std::fmt;

/// Supported locales for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    /// English (default)
    #[default]
    En,
    /// Korean (한국어)
    Ko,
    /// Japanese (日本語)
    Ja,
    /// Chinese (中文)
    Zh,
}

impl Locale {
    /// Detect the system locale from environment variables
    ///
    /// Checks in order:
    /// 1. `VAIS_LANG` - Vais-specific language setting
    /// 2. `LANG` - System locale
    /// 3. Falls back to English
    pub fn detect() -> Self {
        // 1. Check VAIS_LANG environment variable
        if let Ok(lang) = std::env::var("VAIS_LANG") {
            if let Some(locale) = Self::from_str(&lang) {
                return locale;
            }
        }

        // 2. Check LANG environment variable
        if let Ok(lang) = std::env::var("LANG") {
            let lang_lower = lang.to_lowercase();
            if lang_lower.starts_with("ko") {
                return Self::Ko;
            }
            if lang_lower.starts_with("ja") {
                return Self::Ja;
            }
            if lang_lower.starts_with("zh") {
                return Self::Zh;
            }
        }

        // 3. Default to English
        Self::En
    }

    /// Parse a locale from a string
    ///
    /// Accepts various formats:
    /// - Language codes: "en", "ko", "ja", "zh"
    /// - Full names: "english", "korean", "japanese", "chinese"
    /// - Native names: "한국어", "日本語", "中文"
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Self::En),
            "ko" | "korean" | "한국어" => Some(Self::Ko),
            "ja" | "japanese" | "日本語" => Some(Self::Ja),
            "zh" | "chinese" | "中文" => Some(Self::Zh),
            _ => None,
        }
    }

    /// Get the locale code (e.g., "en", "ko", "ja", "zh")
    pub fn code(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Ja => "ja",
            Self::Zh => "zh",
        }
    }

    /// Get the native name of the locale
    pub fn native_name(&self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ko => "한국어",
            Self::Ja => "日本語",
            Self::Zh => "中文",
        }
    }

    /// Get all supported locales
    pub fn all() -> &'static [Locale] {
        &[Self::En, Self::Ko, Self::Ja, Self::Zh]
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Locale::from_str("en"), Some(Locale::En));
        assert_eq!(Locale::from_str("ko"), Some(Locale::Ko));
        assert_eq!(Locale::from_str("ja"), Some(Locale::Ja));
        assert_eq!(Locale::from_str("zh"), Some(Locale::Zh));
        assert_eq!(Locale::from_str("english"), Some(Locale::En));
        assert_eq!(Locale::from_str("한국어"), Some(Locale::Ko));
        assert_eq!(Locale::from_str("日本語"), Some(Locale::Ja));
        assert_eq!(Locale::from_str("中文"), Some(Locale::Zh));
        assert_eq!(Locale::from_str("invalid"), None);
    }

    #[test]
    fn test_code() {
        assert_eq!(Locale::En.code(), "en");
        assert_eq!(Locale::Ko.code(), "ko");
        assert_eq!(Locale::Ja.code(), "ja");
        assert_eq!(Locale::Zh.code(), "zh");
    }

    #[test]
    fn test_native_name() {
        assert_eq!(Locale::En.native_name(), "English");
        assert_eq!(Locale::Ko.native_name(), "한국어");
        assert_eq!(Locale::Ja.native_name(), "日本語");
        assert_eq!(Locale::Zh.native_name(), "中文");
    }

    #[test]
    fn test_default() {
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn test_detect_with_env() {
        // Test with VAIS_LANG
        std::env::set_var("VAIS_LANG", "ko");
        assert_eq!(Locale::detect(), Locale::Ko);

        // Clean up
        std::env::remove_var("VAIS_LANG");
    }
}
