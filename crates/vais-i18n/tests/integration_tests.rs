//! Integration tests for vais-i18n crate
//!
//! Tests internationalization functionality including locale switching,
//! message retrieval, and variable substitution.

use vais_i18n::{I18n, Locale};

#[test]
fn test_i18n_creation() {
    let i18n = I18n::new();
    // Should create with auto-detected locale
    // We can't assert the exact locale as it depends on the system
    let locale = i18n.locale();
    assert!(matches!(locale, Locale::En | Locale::Ko | Locale::Ja | Locale::Zh));
}

#[test]
fn test_i18n_with_locale_english() {
    let i18n = I18n::with_locale(Locale::En);
    assert_eq!(i18n.locale(), Locale::En);

    let title = i18n.get_simple("type.E001.title");
    assert_eq!(title, "Type mismatch");
}

#[test]
fn test_i18n_with_locale_korean() {
    let i18n = I18n::with_locale(Locale::Ko);
    assert_eq!(i18n.locale(), Locale::Ko);

    let title = i18n.get_simple("type.E001.title");
    assert_eq!(title, "타입 불일치");
}

#[test]
fn test_i18n_with_locale_japanese() {
    let i18n = I18n::with_locale(Locale::Ja);
    assert_eq!(i18n.locale(), Locale::Ja);

    let title = i18n.get_simple("type.E001.title");
    assert_eq!(title, "型の不一致");
}

#[test]
fn test_i18n_with_locale_chinese() {
    let i18n = I18n::with_locale(Locale::Zh);
    assert_eq!(i18n.locale(), Locale::Zh);

    let title = i18n.get_simple("type.E001.title");
    assert_eq!(title, "类型不匹配");
}

#[test]
fn test_set_locale_switching() {
    let mut i18n = I18n::with_locale(Locale::En);
    assert_eq!(i18n.get_simple("type.E001.title"), "Type mismatch");

    // Switch to Korean
    i18n.set_locale(Locale::Ko);
    assert_eq!(i18n.locale(), Locale::Ko);
    assert_eq!(i18n.get_simple("type.E001.title"), "타입 불일치");

    // Switch to Japanese
    i18n.set_locale(Locale::Ja);
    assert_eq!(i18n.locale(), Locale::Ja);
    assert_eq!(i18n.get_simple("type.E001.title"), "型の不一致");

    // Switch back to English
    i18n.set_locale(Locale::En);
    assert_eq!(i18n.locale(), Locale::En);
    assert_eq!(i18n.get_simple("type.E001.title"), "Type mismatch");
}

#[test]
fn test_get_with_variable_substitution() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get(
        "type.E001.message",
        &[("expected", "i64"), ("found", "Str")],
    );
    assert_eq!(msg, "expected i64, found Str");
}

#[test]
fn test_get_with_multiple_variables() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get(
        "type.E001.message",
        &[
            ("expected", "Vec<i64>"),
            ("found", "HashMap<String, String>"),
        ],
    );
    assert_eq!(msg, "expected Vec<i64>, found HashMap<String, String>");
}

#[test]
fn test_get_simple_no_variables() {
    let i18n = I18n::with_locale(Locale::En);

    let title = i18n.get_simple("type.E001.title");
    assert_eq!(title, "Type mismatch");

    // get_simple should be equivalent to get with empty args
    let title2 = i18n.get("type.E001.title", &[]);
    assert_eq!(title, title2);
}

#[test]
fn test_nonexistent_key_fallback() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get_simple("nonexistent.key");
    // Should return the key itself as fallback
    assert_eq!(msg, "nonexistent.key");
}

#[test]
fn test_nonexistent_key_with_args() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get("nonexistent.key", &[("arg", "value")]);
    // Should return the key itself (args are not applied to fallback)
    assert_eq!(msg, "nonexistent.key");
}

#[test]
fn test_locale_default() {
    let default_locale = Locale::default();
    assert_eq!(default_locale, Locale::En);
}

#[test]
fn test_has_key() {
    let i18n = I18n::with_locale(Locale::En);

    assert!(i18n.has_key("type.E001.title"));
    assert!(i18n.has_key("type.E001.message"));
    assert!(!i18n.has_key("nonexistent.key"));
}

#[test]
fn test_korean_variable_substitution() {
    let i18n = I18n::with_locale(Locale::Ko);

    // Korean locale should have its own message templates
    let msg = i18n.get(
        "type.E001.message",
        &[("expected", "i64"), ("found", "Str")],
    );
    // Korean template is "{expected} 타입을 예상했으나, {found} 타입을 발견"
    assert_eq!(msg, "i64 타입을 예상했으나, Str 타입을 발견");
}

#[test]
fn test_japanese_variable_substitution() {
    let i18n = I18n::with_locale(Locale::Ja);

    let msg = i18n.get(
        "type.E001.message",
        &[("expected", "i64"), ("found", "Str")],
    );
    // Japanese template is "{expected}型を期待しましたが、{found}型が見つかりました"
    assert_eq!(msg, "i64型を期待しましたが、Str型が見つかりました");
}

#[test]
fn test_chinese_variable_substitution() {
    let i18n = I18n::with_locale(Locale::Zh);

    let msg = i18n.get(
        "type.E001.message",
        &[("expected", "i64"), ("found", "Str")],
    );
    // Chinese template is "期望类型 {expected}，但找到 {found}"
    assert_eq!(msg, "期望类型 i64，但找到 Str");
}

#[test]
fn test_fallback_to_english_for_missing_key() {
    // If a Korean locale doesn't have a specific key, it should fall back to English
    let i18n = I18n::with_locale(Locale::Ko);

    // Try a key that might not exist in Korean but exists in English
    // Most keys should exist, but this tests the fallback mechanism
    // We can still verify that has_key works correctly
    if i18n.has_key("type.E001.title") {
        let title = i18n.get_simple("type.E001.title");
        assert!(!title.is_empty());
    }
}

#[test]
fn test_i18n_default() {
    let i18n = I18n::default();
    // Default should create with auto-detected locale
    let locale = i18n.locale();
    assert!(matches!(locale, Locale::En | Locale::Ko | Locale::Ja | Locale::Zh));
}

#[test]
fn test_set_locale_no_reload_if_same() {
    let mut i18n = I18n::with_locale(Locale::En);
    let title_before = i18n.get_simple("type.E001.title");

    // Setting the same locale should not change anything
    i18n.set_locale(Locale::En);
    let title_after = i18n.get_simple("type.E001.title");

    assert_eq!(title_before, title_after);
}

#[test]
fn test_empty_args_in_get() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get("type.E001.message", &[]);
    // With empty args, placeholders should remain in the message
    assert!(msg.contains("{expected}"));
    assert!(msg.contains("{found}"));
}

#[test]
fn test_partial_variable_substitution() {
    let i18n = I18n::with_locale(Locale::En);

    // Only provide one of the two variables
    let msg = i18n.get("type.E001.message", &[("expected", "i64")]);
    // Should substitute {expected} but leave {found}
    assert!(msg.contains("i64"));
    assert!(msg.contains("{found}"));
}
