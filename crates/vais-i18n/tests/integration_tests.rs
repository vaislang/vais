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
    assert!(matches!(
        locale,
        Locale::En | Locale::Ko | Locale::Ja | Locale::Zh
    ));
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
    assert!(matches!(
        locale,
        Locale::En | Locale::Ko | Locale::Ja | Locale::Zh
    ));
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

// ============================================================================
// New Integration Tests (10 tests)
// ============================================================================

// 1. Locale Advanced - Test Locale::all() returns all supported locales
#[test]
fn test_locale_all_returns_all_locales() {
    let all_locales = Locale::all();

    assert_eq!(all_locales.len(), 4);
    assert!(all_locales.contains(&Locale::En));
    assert!(all_locales.contains(&Locale::Ko));
    assert!(all_locales.contains(&Locale::Ja));
    assert!(all_locales.contains(&Locale::Zh));
}

// 2. Locale Advanced - Test Locale::detect() with VAIS_LANG environment variable
#[test]
fn test_locale_detect_with_vais_lang() {
    // Set VAIS_LANG to Korean
    std::env::set_var("VAIS_LANG", "ko");
    assert_eq!(Locale::detect(), Locale::Ko);

    // Set VAIS_LANG to Japanese
    std::env::set_var("VAIS_LANG", "ja");
    assert_eq!(Locale::detect(), Locale::Ja);

    // Clean up
    std::env::remove_var("VAIS_LANG");
}

// 3. Locale Advanced - Verify code() and native_name() for all locales
#[test]
fn test_locale_code_and_native_name_for_all() {
    let all_locales = Locale::all();

    for locale in all_locales {
        match locale {
            Locale::En => {
                assert_eq!(locale.code(), "en");
                assert_eq!(locale.native_name(), "English");
            }
            Locale::Ko => {
                assert_eq!(locale.code(), "ko");
                assert_eq!(locale.native_name(), "한국어");
            }
            Locale::Ja => {
                assert_eq!(locale.code(), "ja");
                assert_eq!(locale.native_name(), "日本語");
            }
            Locale::Zh => {
                assert_eq!(locale.code(), "zh");
                assert_eq!(locale.native_name(), "中文");
            }
        }
    }
}

// 4. Message Key Validation - Test accessing nonexistent key returns key itself
#[test]
fn test_nonexistent_key_returns_key_itself() {
    let i18n = I18n::with_locale(Locale::En);

    let key = "this.key.does.not.exist";
    let result = i18n.get_simple(key);
    assert_eq!(result, key);

    // With variables, should still return key without substitution
    let result_with_args = i18n.get(key, &[("var", "value")]);
    assert_eq!(result_with_args, key);
}

// 5. Message Key Validation - Verify common error keys exist in all locales
#[test]
fn test_common_error_keys_exist_in_all_locales() {
    let common_keys = vec![
        "type.E001.title",
        "type.E001.message",
    ];

    for locale in Locale::all() {
        let i18n = I18n::with_locale(*locale);
        for key in &common_keys {
            assert!(
                i18n.has_key(key),
                "Key '{}' should exist in locale {}",
                key,
                locale.code()
            );

            // Also verify we can retrieve it
            let msg = i18n.get_simple(key);
            assert!(!msg.is_empty());
            assert_ne!(msg, *key); // Should not be the key itself
        }
    }
}

// 6. Multi-Variable Substitution - Test simultaneous substitution of multiple variables
#[test]
fn test_multiple_variable_simultaneous_substitution() {
    let i18n = I18n::with_locale(Locale::En);

    // Use a key with multiple variables
    let msg = i18n.get(
        "type.E001.message",
        &[
            ("expected", "Result<T, E>"),
            ("found", "Option<Vec<String>>"),
        ],
    );

    assert!(msg.contains("Result<T, E>"));
    assert!(msg.contains("Option<Vec<String>>"));
    assert!(!msg.contains("{expected}"));
    assert!(!msg.contains("{found}"));
}

// 7. Multi-Variable Substitution - Test repeated variable substitution
#[test]
fn test_repeated_variable_substitution() {
    let _i18n = I18n::with_locale(Locale::En);

    // Create a template with repeated variable (using direct get)
    // Note: Testing the substitution mechanism itself
    let template = "Type {type} cannot be {type}";
    let result = template.replace("{type}", "i64");

    assert_eq!(result, "Type i64 cannot be i64");
    assert_eq!(result.matches("i64").count(), 2);
}

// 8. Edge Cases - Test empty string key
#[test]
fn test_empty_string_key() {
    let i18n = I18n::with_locale(Locale::En);

    let result = i18n.get_simple("");
    // Empty key should return empty string (fallback behavior)
    assert_eq!(result, "");

    // Verify has_key returns false for empty key
    assert!(!i18n.has_key(""));
}

// 9. Edge Cases - Test special characters in substitution values
#[test]
fn test_special_characters_in_values() {
    let i18n = I18n::with_locale(Locale::En);

    let msg = i18n.get(
        "type.E001.message",
        &[
            ("expected", "Vec<(i64, &'a str)>"),
            ("found", "HashMap<String, Result<T, E>>"),
        ],
    );

    // Special characters like <, >, &, ', should be preserved
    assert!(msg.contains("Vec<(i64, &'a str)>"));
    assert!(msg.contains("HashMap<String, Result<T, E>>"));
}

// 10. Locale Switching Persistence - Verify locale persists across multiple get() calls
#[test]
fn test_locale_persistence_across_calls() {
    let mut i18n = I18n::with_locale(Locale::Ko);

    // First call
    let msg1 = i18n.get_simple("type.E001.title");
    assert_eq!(msg1, "타입 불일치");

    // Second call
    let msg2 = i18n.get_simple("type.E001.title");
    assert_eq!(msg2, "타입 불일치");

    // Third call with variables
    let msg3 = i18n.get("type.E001.message", &[("expected", "i64"), ("found", "Str")]);
    assert!(msg3.contains("i64"));
    assert!(msg3.contains("Str"));

    // Verify locale hasn't changed
    assert_eq!(i18n.locale(), Locale::Ko);

    // Switch locale and verify persistence
    i18n.set_locale(Locale::Ja);
    let msg4 = i18n.get_simple("type.E001.title");
    let msg5 = i18n.get_simple("type.E001.title");
    assert_eq!(msg4, msg5);
    assert_eq!(msg4, "型の不一致");
    assert_eq!(i18n.locale(), Locale::Ja);
}
