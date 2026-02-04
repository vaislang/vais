//! Integration tests for the Vais package registry server
//!
//! These tests verify the full API functionality using axum's test utilities
//! without starting a real TCP server.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::json;
use tempfile::TempDir;
use tower::ServiceExt; // for `oneshot`
use vais_registry_server::{create_router, ServerConfig};

/// Test helper to create a fresh test app with isolated storage
async fn setup_test_app() -> (axum::Router, TempDir, TempDir) {
    let db_dir = TempDir::new().unwrap();
    let storage_dir = TempDir::new().unwrap();

    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_path: db_dir.path().join("test.db"),
        storage_path: storage_dir.path().to_path_buf(),
        max_upload_size: 50 * 1024 * 1024,
        token_expiration_days: 365,
        cors_allow_all: true,
        cors_origins: vec![],
        enable_logging: false,
        admin_username: None,
        admin_password: None,
    };

    let pool = vais_registry_server::db::init_db(&config.database_path)
        .await
        .unwrap();
    let storage = vais_registry_server::storage::PackageStorage::new(config.storage_path.clone())
        .unwrap();

    let router = create_router(pool, storage, config);

    (router, db_dir, storage_dir)
}

/// Helper to create a test package archive with vais.toml
fn create_test_archive(name: &str, version: &str) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let mut buffer = Vec::new();

    {
        let encoder = GzEncoder::new(&mut buffer, Compression::default());
        let mut archive = Builder::new(encoder);

        // Create vais.toml manifest
        let manifest = format!(
            r#"[package]
name = "{}"
version = "{}"
description = "Test package"
license = "MIT"
"#,
            name, version
        );

        let manifest_bytes = manifest.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("vais.toml").unwrap();
        header.set_size(manifest_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, manifest_bytes).unwrap();

        // Create a dummy source file
        let source = format!("# Test source for {} v{}\nF main() {{ R 0 }}\n", name, version);
        let source_bytes = source.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("src/main.vais").unwrap();
        header.set_size(source_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, source_bytes).unwrap();

        let encoder = archive.into_inner().unwrap();
        encoder.finish().unwrap();
    }

    buffer
}

/// Helper to create a multipart body for package publish
fn create_publish_multipart(
    metadata: serde_json::Value,
    archive: Vec<u8>,
) -> (String, Vec<u8>) {
    let boundary = "----WebKitFormBoundaryTest123456789";
    let mut body = Vec::new();

    // Metadata part
    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"metadata\"\r\n");
    body.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    body.extend_from_slice(serde_json::to_string(&metadata).unwrap().as_bytes());
    body.extend_from_slice(b"\r\n");

    // Archive part
    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"archive\"; filename=\"package.tar.gz\"\r\n");
    body.extend_from_slice(b"Content-Type: application/gzip\r\n\r\n");
    body.extend_from_slice(&archive);
    body.extend_from_slice(b"\r\n");

    // End boundary
    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789--\r\n");

    (boundary.to_string(), body)
}

#[tokio::test]
async fn test_health_check() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_user_registration() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let payload = json!({
        "username": "testuser",
        "password": "testpass123",
        "email": "test@example.com"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["username"], "testuser");
    assert_eq!(json["email"], "test@example.com");
    assert_eq!(json["is_admin"], false);
}

#[tokio::test]
async fn test_user_registration_duplicate() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let payload = json!({
        "username": "testuser",
        "password": "testpass123"
    });

    // First registration should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second registration should fail
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_user_login() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Register a user first
    let register_payload = json!({
        "username": "logintest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Now login
    let login_payload = json!({
        "username": "logintest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["token"].is_string());
    assert!(json["token"].as_str().unwrap().starts_with("vais_"));
    assert!(json["expires_at"].is_string());
    assert_eq!(json["user"]["username"], "logintest");
}

#[tokio::test]
async fn test_user_login_invalid_credentials() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Register a user first
    let register_payload = json!({
        "username": "logintest2",
        "password": "correctpass"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Try login with wrong password
    let login_payload = json!({
        "username": "logintest2",
        "password": "wrongpass"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_me() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Register and login
    let register_payload = json!({
        "username": "metest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "metest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap();

    // Test /auth/me
    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["username"], "metest");
    assert_eq!(json["is_admin"], false);
}

#[tokio::test]
async fn test_auth_me_without_token() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_package_publish() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Register and login to get token
    let register_payload = json!({
        "username": "publisher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "publisher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Create a test package
    let archive = create_test_archive("test-package", "1.0.0");
    let metadata = json!({
        "name": "test-package",
        "version": "1.0.0",
        "description": "A test package",
        "license": "MIT",
        "keywords": ["test", "example"],
        "categories": ["testing"],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    // Publish the package
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "test-package");
    assert_eq!(json["version"], "1.0.0");
    assert!(json["checksum"].is_string());
}

#[tokio::test]
async fn test_package_publish_duplicate_version() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Register and login
    let register_payload = json!({
        "username": "publisher2",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "publisher2",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish first time
    let archive = create_test_archive("dup-test", "1.0.0");
    let metadata = json!({
        "name": "dup-test",
        "version": "1.0.0",
        "description": "Test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Publish same version again - should fail
    let archive = create_test_archive("dup-test", "1.0.0");
    let metadata = json!({
        "name": "dup-test",
        "version": "1.0.0",
        "description": "Test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_package_get_info() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Setup: Register, login, and publish a package
    let register_payload = json!({
        "username": "infopublisher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "infopublisher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish package
    let archive = create_test_archive("info-test", "1.2.3");
    let metadata = json!({
        "name": "info-test",
        "version": "1.2.3",
        "description": "Package info test",
        "license": "MIT",
        "keywords": ["info", "test"],
        "categories": ["testing"],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Get package info
    let request = Request::builder()
        .uri("/api/v1/packages/info-test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "info-test");
    assert_eq!(json["description"], "Package info test");
    assert_eq!(json["license"], "MIT");
    assert_eq!(json["keywords"], json!(["info", "test"]));
    assert_eq!(json["categories"], json!(["testing"]));
    assert!(json["versions"].is_array());
    assert_eq!(json["versions"].as_array().unwrap().len(), 1);
    assert_eq!(json["versions"][0]["version"], "1.2.3");
}

#[tokio::test]
async fn test_package_download() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Setup: Register, login, and publish a package
    let register_payload = json!({
        "username": "downloader",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "downloader",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish package
    let archive = create_test_archive("download-test", "2.0.0");
    let metadata = json!({
        "name": "download-test",
        "version": "2.0.0",
        "description": "Download test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Download the package
    let request = Request::builder()
        .uri("/api/v1/packages/download-test/2.0.0")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check headers
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type, "application/gzip");

    let content_disposition = response.headers().get(header::CONTENT_DISPOSITION).unwrap();
    assert!(content_disposition.to_str().unwrap().contains("download-test-2.0.0.tar.gz"));

    // Verify we got data
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_package_search() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Setup: Register, login, and publish multiple packages
    let register_payload = json!({
        "username": "searcher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "searcher",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish a few packages
    for (name, desc) in [
        ("search-foo", "A foo package"),
        ("search-bar", "A bar package"),
        ("other-package", "Something else"),
    ] {
        let archive = create_test_archive(name, "1.0.0");
        let metadata = json!({
            "name": name,
            "version": "1.0.0",
            "description": desc,
            "license": "MIT",
            "keywords": [],
            "categories": [],
            "dependencies": {},
            "dev_dependencies": {},
        });

        let (boundary, body) = create_publish_multipart(metadata, archive);

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/packages/publish")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        app.clone().oneshot(request).await.unwrap();
    }

    // Search for "search"
    let request = Request::builder()
        .uri("/api/v1/search?q=search")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["packages"].is_array());
    let packages = json["packages"].as_array().unwrap();

    // Should find at least the two "search-" packages
    assert!(packages.len() >= 2);

    // Verify the search results contain our packages
    let names: Vec<&str> = packages
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"search-foo"));
    assert!(names.contains(&"search-bar"));
}

#[tokio::test]
async fn test_package_yank() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Setup: Register, login, and publish a package
    let register_payload = json!({
        "username": "yanker",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "yanker",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish package
    let archive = create_test_archive("yank-test", "1.0.0");
    let metadata = json!({
        "name": "yank-test",
        "version": "1.0.0",
        "description": "Yank test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Yank the version
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/yank-test/1.0.0/yank")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify it's yanked by getting package info
    let request = Request::builder()
        .uri("/api/v1/packages/yank-test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["versions"][0]["yanked"], true);
}

#[tokio::test]
async fn test_full_publish_install_roundtrip() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // 1. Register user
    let register_payload = json!({
        "username": "roundtrip",
        "password": "testpass123",
        "email": "roundtrip@example.com"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Login
    let login_payload = json!({
        "username": "roundtrip",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // 3. Publish multiple versions
    for version in ["1.0.0", "1.1.0", "2.0.0"] {
        let archive = create_test_archive("roundtrip-pkg", version);
        let metadata = json!({
            "name": "roundtrip-pkg",
            "version": version,
            "description": "Full roundtrip test package",
            "homepage": "https://example.com",
            "repository": "https://github.com/example/roundtrip-pkg",
            "license": "MIT",
            "keywords": ["test", "roundtrip"],
            "categories": ["testing", "example"],
            "dependencies": {},
            "dev_dependencies": {},
            "readme": "# Roundtrip Package\n\nThis is a test package."
        });

        let (boundary, body) = create_publish_multipart(metadata, archive);

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/packages/publish")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // 4. Search for the package
    let request = Request::builder()
        .uri("/api/v1/search?q=roundtrip")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["packages"].as_array().unwrap().len() > 0);
    assert_eq!(json["packages"][0]["name"], "roundtrip-pkg");

    // 5. Get package info
    let request = Request::builder()
        .uri("/api/v1/packages/roundtrip-pkg")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "roundtrip-pkg");
    assert_eq!(json["description"], "Full roundtrip test package");
    assert_eq!(json["homepage"], "https://example.com");
    assert_eq!(json["repository"], "https://github.com/example/roundtrip-pkg");
    assert_eq!(json["license"], "MIT");
    assert_eq!(json["keywords"], json!(["test", "roundtrip"]));
    assert_eq!(json["categories"], json!(["testing", "example"]));
    assert_eq!(json["versions"].as_array().unwrap().len(), 3);

    // 6. Download each version
    for version in ["1.0.0", "1.1.0", "2.0.0"] {
        let request = Request::builder()
            .uri(format!("/api/v1/packages/roundtrip-pkg/{}", version))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(!body.is_empty());
    }

    // 7. Verify package appears in search results
    let request = Request::builder()
        .uri("/api/v1/search?q=test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let packages = json["packages"].as_array().unwrap();
    let found = packages.iter().any(|p| p["name"] == "roundtrip-pkg");
    assert!(found, "Package should appear in search results");
}

#[tokio::test]
async fn test_package_not_found() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/packages/nonexistent-package")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_version_not_found() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    // Setup: Register, login, and publish a package
    let register_payload = json!({
        "username": "versiontest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    let login_payload = json!({
        "username": "versiontest",
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = json["token"].as_str().unwrap().to_string();

    // Publish package version 1.0.0
    let archive = create_test_archive("version-test", "1.0.0");
    let metadata = json!({
        "name": "version-test",
        "version": "1.0.0",
        "description": "Test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    app.clone().oneshot(request).await.unwrap();

    // Try to download nonexistent version
    let request = Request::builder()
        .uri("/api/v1/packages/version-test/2.0.0")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unauthorized_publish() {
    let (app, _db_dir, _storage_dir) = setup_test_app().await;

    let archive = create_test_archive("unauthorized", "1.0.0");
    let metadata = json!({
        "name": "unauthorized",
        "version": "1.0.0",
        "description": "Test",
        "license": "MIT",
        "keywords": [],
        "categories": [],
        "dependencies": {},
        "dev_dependencies": {},
    });

    let (boundary, body) = create_publish_multipart(metadata, archive);

    // Try to publish without authentication
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/publish")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
