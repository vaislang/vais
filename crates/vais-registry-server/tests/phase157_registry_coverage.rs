//! Phase 157: Additional unit tests for vais-registry-server
//!
//! Covers: semver_resolve, signing, error, config, models, storage

use vais_registry_server::{
    config::ServerConfig,
    error::ServerError,
    models::{
        DependencyKind, DependencySpec, Package, PackageOwner, PackageVersion, PublishRequest,
        RegistryStats, SearchQuery, User,
    },
    semver_resolve::{are_compatible, compare_versions, parse_version_req, resolve_best_version, satisfies},
    signing::{validate_public_key, verify_signature, SignatureError},
    storage::{create_archive, sha256_hex, PackageStorage},
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================
// semver_resolve — extra coverage
// ============================================================

#[test]
fn test_resolve_multiple_prerelease_chooses_highest() {
    let versions = vec!["3.0.0-alpha", "3.0.0-beta", "3.0.0-rc.1"];
    let result = resolve_best_version(">=3.0.0-alpha", &versions);
    assert_eq!(result, Some("3.0.0-rc.1".to_string()));
}

#[test]
fn test_resolve_single_matching_version() {
    let versions = vec!["2.5.0"];
    let result = resolve_best_version("^2.0.0", &versions);
    assert_eq!(result, Some("2.5.0".to_string()));
}

#[test]
fn test_resolve_ignores_invalid_version_strings() {
    let versions = vec!["not-a-version", "1.0.0", "also-bad"];
    let result = resolve_best_version("^1.0.0", &versions);
    assert_eq!(result, Some("1.0.0".to_string()));
}

#[test]
fn test_resolve_exact_not_present_returns_none() {
    let versions = vec!["1.0.0", "1.1.0", "2.0.0"];
    let result = resolve_best_version("=1.5.0", &versions);
    assert!(result.is_none());
}

#[test]
fn test_resolve_greater_than_or_equal() {
    let versions = vec!["1.0.0", "2.0.0", "3.0.0"];
    let result = resolve_best_version(">=2.0.0", &versions);
    assert_eq!(result, Some("3.0.0".to_string()));
}

#[test]
fn test_parse_version_req_whitespace_trimmed() {
    let req = parse_version_req("  ^1.2.3  ");
    assert!(req.is_some());
}

#[test]
fn test_parse_version_req_invalid_returns_none() {
    let req = parse_version_req("totally_invalid!!!");
    assert!(req.is_none());
}

#[test]
fn test_parse_version_req_partial_bare_version() {
    let req = parse_version_req("1.2");
    assert!(req.is_some());
}

#[test]
fn test_are_compatible_both_exact_same() {
    assert!(are_compatible("=1.0.0", "=1.0.0"));
}

#[test]
fn test_are_compatible_tilde_and_caret_overlap() {
    assert!(are_compatible("~1.2.0", "^1.0.0"));
}

#[test]
fn test_are_compatible_invalid_b() {
    assert!(!are_compatible("^1.0.0", "????"));
}

#[test]
fn test_compare_versions_prerelease_ordering() {
    // alpha < beta by semver
    let ord = compare_versions("1.0.0-alpha", "1.0.0-beta");
    assert_eq!(ord, Some(std::cmp::Ordering::Less));
}

#[test]
fn test_compare_versions_both_invalid() {
    assert!(compare_versions("bad", "also_bad").is_none());
}

#[test]
fn test_satisfies_tilde() {
    assert!(satisfies("1.2.5", "~1.2.0"));
    assert!(!satisfies("1.3.0", "~1.2.0"));
}

#[test]
fn test_satisfies_range() {
    assert!(satisfies("1.5.0", ">=1.0.0, <2.0.0"));
    assert!(!satisfies("2.0.0", ">=1.0.0, <2.0.0"));
}

// ============================================================
// signing — extra coverage
// ============================================================

#[test]
fn test_signature_error_display_verification_failed() {
    let e = SignatureError::VerificationFailed;
    assert_eq!(e.to_string(), "signature verification failed");
}

#[test]
fn test_signature_error_display_required() {
    let e = SignatureError::SignatureRequired;
    assert_eq!(e.to_string(), "signature required but not provided");
}

#[test]
fn test_signature_error_display_invalid_pk() {
    let e = SignatureError::InvalidPublicKey("too short".to_string());
    assert!(e.to_string().contains("too short"));
}

#[test]
fn test_signature_error_display_invalid_sig() {
    let e = SignatureError::InvalidSignature("bad bytes".to_string());
    assert!(e.to_string().contains("bad bytes"));
}

#[test]
fn test_signature_error_display_hex_decode() {
    let e = SignatureError::HexDecode("odd length".to_string());
    assert!(e.to_string().contains("odd length"));
}

#[test]
fn test_validate_public_key_empty_hex() {
    // Empty string → hex::decode("") = Ok([]) → length 0 → InvalidPublicKey
    let result = validate_public_key("");
    assert!(result.is_err());
}

#[test]
fn test_validate_public_key_too_long() {
    // 33 bytes encoded as hex = 66 chars
    let long_key = hex::encode(vec![0u8; 33]);
    let result = validate_public_key(&long_key);
    assert!(matches!(result, Err(SignatureError::InvalidPublicKey(_))));
}

#[test]
fn test_verify_signature_wrong_pk_length() {
    // 16-byte (too short) PK
    let short_pk = hex::encode([0u8; 16]);
    let result = verify_signature(&short_pk, b"data", &hex::encode([0u8; 64]));
    assert!(matches!(result, Err(SignatureError::InvalidPublicKey(_))));
}

#[test]
fn test_verify_signature_wrong_sig_length() {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let sk = SigningKey::generate(&mut OsRng);
    let pk_hex = hex::encode(sk.verifying_key().as_bytes());
    let short_sig = hex::encode([0u8; 16]);
    let result = verify_signature(&pk_hex, b"data", &short_sig);
    assert!(matches!(result, Err(SignatureError::InvalidSignature(_))));
}

// ============================================================
// error — extra coverage
// ============================================================

#[test]
fn test_error_version_not_found_display() {
    let e = ServerError::VersionNotFound("my-pkg".to_string(), "0.9.0".to_string());
    assert_eq!(e.to_string(), "Version not found: my-pkg@0.9.0");
}

#[test]
fn test_error_internal_response_status() {
    let e = ServerError::Internal("something broke".to_string());
    let resp = e.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_error_database_response_status() {
    // We can't easily construct sqlx::Error without a runtime, so just check display
    let e = ServerError::Archive("corrupt tar".to_string());
    let resp = e.into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_bad_request_display() {
    let e = ServerError::BadRequest("missing field x".to_string());
    assert!(e.to_string().contains("missing field x"));
}

#[test]
fn test_error_invalid_checksum_display() {
    assert_eq!(ServerError::InvalidChecksum.to_string(), "Invalid checksum");
}

#[test]
fn test_error_user_not_found_display() {
    let e = ServerError::UserNotFound("alice".to_string());
    assert!(e.to_string().contains("alice"));
}

// ============================================================
// config — extra coverage
// ============================================================

#[test]
fn test_config_default_values() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.host, "0.0.0.0");
    assert_eq!(cfg.port, 3000);
    assert_eq!(cfg.max_upload_size, 50 * 1024 * 1024);
    assert_eq!(cfg.token_expiration_days, 365);
    assert!(!cfg.cors_allow_all);
    assert!(cfg.cors_origins.is_empty());
    assert!(cfg.enable_logging);
    assert!(cfg.admin_username.is_none());
    assert!(cfg.admin_password.is_none());
}

#[test]
fn test_config_bind_addr_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.bind_addr(), "0.0.0.0:3000");
}

#[test]
fn test_config_bind_addr_localhost() {
    let cfg = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        ..ServerConfig::default()
    };
    assert_eq!(cfg.bind_addr(), "127.0.0.1:8080");
}

#[test]
fn test_config_clone_preserves_all_fields() {
    let cfg = ServerConfig {
        host: "192.168.1.1".to_string(),
        port: 9000,
        cors_allow_all: true,
        enable_logging: false,
        admin_username: Some("admin".to_string()),
        admin_password: Some("secret".to_string()),
        cors_origins: vec!["https://example.com".to_string()],
        ..ServerConfig::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.host, cfg.host);
    assert_eq!(cloned.port, cfg.port);
    assert_eq!(cloned.cors_allow_all, cfg.cors_allow_all);
    assert_eq!(cloned.enable_logging, cfg.enable_logging);
    assert_eq!(cloned.admin_username, cfg.admin_username);
    assert_eq!(cloned.cors_origins, cfg.cors_origins);
}

#[test]
fn test_config_toml_roundtrip_full() {
    let toml = r#"
host = "localhost"
port = 4000
max_upload_size = 10485760
token_expiration_days = 30
cors_allow_all = true
cors_origins = ["https://a.com", "https://b.com"]
enable_logging = false
admin_username = "admin"
admin_password = "pw"
"#;
    let cfg: ServerConfig = toml::from_str(toml).unwrap();
    assert_eq!(cfg.host, "localhost");
    assert_eq!(cfg.port, 4000);
    assert_eq!(cfg.max_upload_size, 10_485_760);
    assert_eq!(cfg.token_expiration_days, 30);
    assert!(cfg.cors_allow_all);
    assert_eq!(cfg.cors_origins.len(), 2);
    assert!(!cfg.enable_logging);
    assert_eq!(cfg.admin_username, Some("admin".to_string()));
    assert_eq!(cfg.admin_password, Some("pw".to_string()));
}

#[test]
fn test_config_from_file_nonexistent() {
    let result = ServerConfig::from_file(std::path::Path::new("/no/such/file.toml"));
    assert!(result.is_err());
}

#[test]
fn test_config_from_file_valid() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        r#"host = "0.0.0.0"
port = 3000
"#,
    )
    .unwrap();
    let cfg = ServerConfig::from_file(&path).unwrap();
    assert_eq!(cfg.port, 3000);
}

#[test]
fn test_config_database_path_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.database_path, PathBuf::from("./data/registry.db"));
}

#[test]
fn test_config_storage_path_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.storage_path, PathBuf::from("./data/packages"));
}

// ============================================================
// models — extra coverage
// ============================================================

#[test]
fn test_user_is_admin_field() {
    let user = User {
        id: Uuid::new_v4(),
        username: "root".to_string(),
        password_hash: "hash".to_string(),
        email: None,
        is_admin: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    assert!(user.is_admin);
    let json = serde_json::to_string(&user).unwrap();
    assert!(!json.contains("password_hash"));
    assert!(json.contains("\"is_admin\":true"));
}

#[test]
fn test_package_serde_roundtrip() {
    let pkg = Package {
        id: Uuid::new_v4(),
        name: "test-lib".to_string(),
        description: Some("A library".to_string()),
        homepage: None,
        repository: Some("https://github.com/x/y".to_string()),
        documentation: None,
        license: Some("MIT".to_string()),
        keywords: vec!["test".to_string()],
        categories: vec!["utilities".to_string()],
        owner_id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        downloads: 42,
    };
    let json = serde_json::to_string(&pkg).unwrap();
    let parsed: Package = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "test-lib");
    assert_eq!(parsed.downloads, 42);
}

#[test]
fn test_package_version_serde_roundtrip() {
    let pv = PackageVersion {
        id: Uuid::new_v4(),
        package_id: Uuid::new_v4(),
        version: "1.2.3".to_string(),
        checksum: "abc123".to_string(),
        size: 1024,
        yanked: false,
        readme: Some("# Hello".to_string()),
        published_by: Uuid::new_v4(),
        created_at: Utc::now(),
        downloads: 10,
    };
    let json = serde_json::to_string(&pv).unwrap();
    let parsed: PackageVersion = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.version, "1.2.3");
    assert!(!parsed.yanked);
}

#[test]
fn test_package_owner_serde() {
    let owner = PackageOwner {
        package_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        added_by: Uuid::new_v4(),
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&owner).unwrap();
    let parsed: PackageOwner = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.package_id, owner.package_id);
}

#[test]
fn test_dependency_kind_normal_is_default() {
    let kind = DependencyKind::default();
    assert_eq!(kind, DependencyKind::Normal);
}

#[test]
fn test_dependency_kind_all_variants_serialize() {
    let cases = [
        (DependencyKind::Normal, "\"normal\""),
        (DependencyKind::Dev, "\"dev\""),
        (DependencyKind::Build, "\"build\""),
    ];
    for (kind, expected) in &cases {
        let s = serde_json::to_string(kind).unwrap();
        assert_eq!(&s, expected);
    }
}

#[test]
fn test_dependency_spec_simple_no_target() {
    let spec = DependencySpec::Simple("^2.0".to_string());
    assert!(spec.target().is_none());
    assert!(!spec.is_optional());
    assert!(spec.features().is_empty());
}

#[test]
fn test_dependency_spec_detailed_with_target() {
    let spec = DependencySpec::Detailed {
        version: Some("^1.0".to_string()),
        features: vec![],
        optional: false,
        target: Some("wasm32-unknown-unknown".to_string()),
    };
    assert_eq!(spec.target(), Some("wasm32-unknown-unknown"));
}

#[test]
fn test_publish_request_with_dev_dependencies() {
    let json = r#"{
        "name": "my-pkg",
        "version": "1.0.0",
        "dev_dependencies": {
            "test-lib": "^0.1"
        }
    }"#;
    let req: PublishRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.dev_dependencies.len(), 1);
    assert_eq!(req.dev_dependencies["test-lib"].version_req(), "^0.1");
}

#[test]
fn test_search_query_all_fields() {
    let json = r#"{"q": "async", "limit": 50, "offset": 20, "sort": "relevance", "category": "concurrency", "keyword": "async"}"#;
    let q: SearchQuery = serde_json::from_str(json).unwrap();
    assert_eq!(q.q, "async");
    assert_eq!(q.limit, 50);
    assert_eq!(q.offset, 20);
    assert_eq!(q.sort, "relevance");
    assert_eq!(q.category, Some("concurrency".to_string()));
    assert_eq!(q.keyword, Some("async".to_string()));
}

#[test]
fn test_registry_stats_zero_values() {
    let stats = RegistryStats {
        total_packages: 0,
        total_downloads: 0,
        total_versions: 0,
    };
    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("\"total_packages\":0"));
}

// ============================================================
// storage — extra coverage
// ============================================================

#[test]
fn test_sha256_known_value() {
    // SHA256("") known value
    let hash = sha256_hex(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_sha256_is_64_hex_chars() {
    let hash = sha256_hex(b"vais is awesome");
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_storage_new_creates_nested_dirs() {
    let base = TempDir::new().unwrap();
    let nested = base.path().join("a").join("b").join("c");
    let _storage = PackageStorage::new(nested.clone()).unwrap();
    assert!(nested.exists());
}

#[test]
fn test_storage_package_dir_path() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let pkg_dir = storage.package_dir("my-crate");
    assert!(pkg_dir.ends_with("my-crate"));
}

#[test]
fn test_storage_archive_path_format() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let path = storage.archive_path("my-crate", "2.3.4");
    assert!(path.to_string_lossy().ends_with("my-crate/2.3.4.tar.gz"));
}

#[test]
fn test_storage_store_returns_correct_checksum() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let data = b"deterministic content";
    let checksum = storage.store_archive("pkg", "1.0.0", data).unwrap();
    assert_eq!(checksum, sha256_hex(data));
}

#[test]
fn test_storage_verify_correct_checksum() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let data = b"content";
    let checksum = storage.store_archive("pkg", "1.0.0", data).unwrap();
    assert!(storage.verify_checksum("pkg", "1.0.0", &checksum).unwrap());
}

#[test]
fn test_storage_verify_wrong_checksum() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    storage.store_archive("pkg", "1.0.0", b"data").unwrap();
    assert!(!storage.verify_checksum("pkg", "1.0.0", "deadbeef").unwrap());
}

#[test]
fn test_storage_archive_exists_false_for_traversal() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    assert!(!storage.archive_exists("../etc", "1.0.0"));
    assert!(!storage.archive_exists("pkg", "../../etc"));
}

#[test]
fn test_storage_list_versions_empty_if_no_package() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let versions = storage.list_versions("unknown").unwrap();
    assert!(versions.is_empty());
}

#[test]
fn test_storage_delete_returns_true_when_exists() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    storage.store_archive("pkg", "1.0.0", b"x").unwrap();
    assert!(storage.delete_archive("pkg", "1.0.0").unwrap());
    assert!(!storage.archive_exists("pkg", "1.0.0"));
}

#[test]
fn test_storage_delete_returns_false_when_absent() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    assert!(!storage.delete_archive("pkg", "9.9.9").unwrap());
}

#[test]
fn test_storage_size_nonexistent_returns_err() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    assert!(storage.archive_size("ghost", "0.0.1").is_err());
}

#[test]
fn test_storage_total_size_accumulates() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    storage.store_archive("a", "1.0.0", b"12345").unwrap(); // 5 bytes
    storage.store_archive("b", "1.0.0", b"1234567890").unwrap(); // 10 bytes
    let total = storage.total_size().unwrap();
    assert_eq!(total, 15);
}

#[test]
fn test_storage_overwrite_version() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    storage.store_archive("pkg", "1.0.0", b"old").unwrap();
    storage.store_archive("pkg", "1.0.0", b"new content").unwrap();
    let data = storage.read_archive("pkg", "1.0.0").unwrap();
    assert_eq!(data, b"new content");
}

#[test]
fn test_create_archive_produces_valid_gzip() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("vais.toml"), "[package]\nname=\"x\"\nversion=\"1.0.0\"\n").unwrap();
    let archive = create_archive(dir.path()).unwrap();
    // GZip magic bytes: 1f 8b
    assert!(archive.len() >= 2);
    assert_eq!(archive[0], 0x1f);
    assert_eq!(archive[1], 0x8b);
}

#[test]
fn test_storage_validate_archive_success() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().join("storage")).unwrap();
    let pkg_dir = dir.path().join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    std::fs::write(
        pkg_dir.join("vais.toml"),
        "[package]\nname = \"hello\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let archive = create_archive(&pkg_dir).unwrap();
    let manifest = storage.validate_archive(&archive).unwrap();
    assert_eq!(manifest.package.name, "hello");
    assert_eq!(manifest.package.version, "0.1.0");
}

#[test]
fn test_storage_validate_archive_missing_manifest_err() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().join("storage")).unwrap();
    let pkg_dir = dir.path().join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();
    std::fs::write(pkg_dir.join("lib.vais"), "F main() -> i64 { 0 }").unwrap();
    let archive = create_archive(&pkg_dir).unwrap();
    assert!(storage.validate_archive(&archive).is_err());
}

#[test]
fn test_storage_path_traversal_null_byte_version() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    let result = storage.store_archive("pkg", "1\x00.0", b"x");
    assert!(result.is_err());
}

#[test]
fn test_storage_multiple_packages_isolated() {
    let dir = TempDir::new().unwrap();
    let storage = PackageStorage::new(dir.path().to_path_buf()).unwrap();
    storage.store_archive("pkg-a", "1.0.0", b"data_a").unwrap();
    storage.store_archive("pkg-b", "1.0.0", b"data_b").unwrap();
    let a = storage.read_archive("pkg-a", "1.0.0").unwrap();
    let b_data = storage.read_archive("pkg-b", "1.0.0").unwrap();
    assert_eq!(a, b"data_a");
    assert_eq!(b_data, b"data_b");
}
