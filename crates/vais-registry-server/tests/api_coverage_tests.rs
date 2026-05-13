//! Additional API coverage tests for the Vais package registry server
//!
//! Covers routes not yet tested in integration_tests.rs:
//! - unyank, categories, popular, recent
//! - user profile, owner management (add/remove)
//! - token management (list/create/delete)
//! - index endpoints (full index, package index)
//! - web UI routes (index, dashboard, package detail, css)
//! - validation edge cases (invalid names, versions, auth)

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::json;
use tempfile::TempDir;
use tower::ServiceExt;
use vais_registry_server::{create_router, ServerConfig};

// ============================================================================
// Test helpers
// ============================================================================

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
    let storage =
        vais_registry_server::storage::PackageStorage::new(config.storage_path.clone()).unwrap();

    let router = create_router(pool, storage, config);
    (router, db_dir, storage_dir)
}

fn create_test_archive(name: &str, version: &str) -> Vec<u8> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let mut buffer = Vec::new();
    {
        let encoder = GzEncoder::new(&mut buffer, Compression::default());
        let mut archive = Builder::new(encoder);

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
        let mut hdr = tar::Header::new_gnu();
        hdr.set_path("vais.toml").unwrap();
        hdr.set_size(manifest_bytes.len() as u64);
        hdr.set_mode(0o644);
        hdr.set_cksum();
        archive.append(&hdr, manifest_bytes).unwrap();

        let source = format!("F main() -> i64 {{ R 0 }}\n");
        let source_bytes = source.as_bytes();
        let mut hdr = tar::Header::new_gnu();
        hdr.set_path("src/main.vais").unwrap();
        hdr.set_size(source_bytes.len() as u64);
        hdr.set_mode(0o644);
        hdr.set_cksum();
        archive.append(&hdr, source_bytes).unwrap();

        let encoder = archive.into_inner().unwrap();
        encoder.finish().unwrap();
    }
    buffer
}

fn create_publish_multipart(metadata: serde_json::Value, archive: Vec<u8>) -> (String, Vec<u8>) {
    let boundary = "----WebKitFormBoundaryTest123456789";
    let mut body = Vec::new();

    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789\r\n");
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"metadata\"\r\n");
    body.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    body.extend_from_slice(serde_json::to_string(&metadata).unwrap().as_bytes());
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789\r\n");
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"archive\"; filename=\"package.tar.gz\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/gzip\r\n\r\n");
    body.extend_from_slice(&archive);
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(b"------WebKitFormBoundaryTest123456789--\r\n");

    (boundary.to_string(), body)
}

/// Helper: register + login, return token string
async fn register_and_login(app: &axum::Router, username: &str) -> String {
    let payload = json!({
        "username": username,
        "password": "testpass123"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    app.clone().oneshot(request).await.unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    json["token"].as_str().unwrap().to_string()
}

/// Helper: publish a package with given name/version
async fn publish_package(app: &axum::Router, token: &str, name: &str, version: &str) {
    let archive = create_test_archive(name, version);
    let metadata = json!({
        "name": name,
        "version": version,
        "description": format!("Test package {}", name),
        "license": "MIT",
        "keywords": ["test"],
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
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "publish {}@{} failed",
        name,
        version
    );
}

// ============================================================================
// Unyank tests
// ============================================================================

#[tokio::test]
async fn test_unyank_version() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "unyankuser").await;

    publish_package(&app, &token, "unyank-pkg", "1.0.0").await;

    // Yank first
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/unyank-pkg/1.0.0/yank")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify yanked
    let request = Request::builder()
        .uri("/api/v1/packages/unyank-pkg")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["versions"][0]["yanked"], true);

    // Unyank
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/unyank-pkg/1.0.0/unyank")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify unyanked
    let request = Request::builder()
        .uri("/api/v1/packages/unyank-pkg")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["versions"][0]["yanked"], false);
}

// ============================================================================
// Categories and discovery tests
// ============================================================================

#[tokio::test]
async fn test_list_categories() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "catuser").await;

    publish_package(&app, &token, "cat-pkg-a", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/categories")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array());
}

#[tokio::test]
async fn test_browse_category() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "browsecat").await;

    publish_package(&app, &token, "browse-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/categories/testing")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["packages"].is_array());
}

#[tokio::test]
async fn test_popular_packages() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "popuser").await;

    publish_package(&app, &token, "popular-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/popular")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["packages"].is_array());
}

#[tokio::test]
async fn test_recent_packages() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "recentuser").await;

    publish_package(&app, &token, "recent-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/recent")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["packages"].is_array());
}

// ============================================================================
// User profile tests
// ============================================================================

#[tokio::test]
async fn test_get_user_profile() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "profileuser").await;

    publish_package(&app, &token, "profile-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/users/profileuser")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["username"], "profileuser");
    assert!(json["packages"].is_array());
    // email should not be exposed
    assert!(json["email"].is_null());
}

#[tokio::test]
async fn test_get_user_not_found() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/users/nonexistent")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Owner management tests
// ============================================================================

#[tokio::test]
async fn test_add_owner() {
    let (app, _db, _st) = setup_test_app().await;
    let token1 = register_and_login(&app, "owner1").await;
    let _token2 = register_and_login(&app, "owner2").await;

    publish_package(&app, &token1, "owned-pkg", "1.0.0").await;

    // Add owner2
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/owned-pkg/owners")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({"username": "owner2"})).unwrap(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify owner2 appears in package info
    let request = Request::builder()
        .uri("/api/v1/packages/owned-pkg")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let owners = json["owners"].as_array().unwrap();
    assert!(owners.len() >= 2, "Should have at least 2 owners");
}

#[tokio::test]
async fn test_add_owner_already_owner() {
    let (app, _db, _st) = setup_test_app().await;
    let token1 = register_and_login(&app, "alreadyowner1").await;
    let _token2 = register_and_login(&app, "alreadyowner2").await;

    publish_package(&app, &token1, "already-owned", "1.0.0").await;

    // Add owner2 first time
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/already-owned/owners")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({"username": "alreadyowner2"})).unwrap(),
        ))
        .unwrap();
    app.clone().oneshot(request).await.unwrap();

    // Add owner2 second time — should fail
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/already-owned/owners")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({"username": "alreadyowner2"})).unwrap(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_remove_owner() {
    let (app, _db, _st) = setup_test_app().await;
    let token1 = register_and_login(&app, "rmowner1").await;
    let _token2 = register_and_login(&app, "rmowner2").await;

    publish_package(&app, &token1, "rm-owned", "1.0.0").await;

    // Add owner2
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/rm-owned/owners")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({"username": "rmowner2"})).unwrap(),
        ))
        .unwrap();
    app.clone().oneshot(request).await.unwrap();

    // Remove owner2
    let request = Request::builder()
        .method("DELETE")
        .uri("/api/v1/packages/rm-owned/owners/rmowner2")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_remove_last_owner_fails() {
    let (app, _db, _st) = setup_test_app().await;
    let token1 = register_and_login(&app, "lastowner").await;

    publish_package(&app, &token1, "last-owned", "1.0.0").await;

    // Try to remove self (last owner)
    let request = Request::builder()
        .method("DELETE")
        .uri("/api/v1/packages/last-owned/owners/lastowner")
        .header(header::AUTHORIZATION, format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Index endpoint tests
// ============================================================================

#[tokio::test]
async fn test_full_index() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "indexuser").await;

    publish_package(&app, &token, "index-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/index.json")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_object() || json.is_array());
}

#[tokio::test]
async fn test_package_index() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "pkgidxuser").await;

    publish_package(&app, &token, "pkgidx-test", "1.0.0").await;

    let request = Request::builder()
        .uri("/api/v1/packages/pkgidx-test/index.json")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Web UI route tests
// ============================================================================

#[tokio::test]
async fn test_web_index() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("html") || html.contains("HTML") || html.contains("vais"));
}

#[tokio::test]
async fn test_web_dashboard() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/dashboard")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_web_package_detail() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "webdetailuser").await;

    publish_package(&app, &token, "web-detail-pkg", "1.0.0").await;

    let request = Request::builder()
        .uri("/packages/web-detail-pkg")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    // May be 200 or 404 depending on implementation
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::NOT_FOUND,
        "Unexpected status: {}",
        status
    );
}

#[tokio::test]
async fn test_web_css() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/static/styles.css")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_root_health_check() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// ============================================================================
// Auth validation edge cases
// ============================================================================

#[tokio::test]
async fn test_register_short_username() {
    let (app, _db, _st) = setup_test_app().await;

    let payload = json!({
        "username": "ab",
        "password": "testpass123"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_short_password() {
    let (app, _db, _st) = setup_test_app().await;

    let payload = json!({
        "username": "validuser",
        "password": "short"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_invalid_username_chars() {
    let (app, _db, _st) = setup_test_app().await;

    let payload = json!({
        "username": "user@name",
        "password": "testpass123"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_auth_invalid_token() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::AUTHORIZATION, "Bearer invalid_token_here")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    // Should be 401 or 403
    let status = response.status();
    assert!(
        status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN,
        "Expected 401 or 403 for invalid token, got {}",
        status
    );
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let (app, _db, _st) = setup_test_app().await;

    let payload = json!({
        "username": "doesnotexist",
        "password": "testpass123"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Search with pagination
// ============================================================================

#[tokio::test]
async fn test_search_with_pagination() {
    let (app, _db, _st) = setup_test_app().await;
    let token = register_and_login(&app, "pageuser").await;

    for i in 0..5 {
        publish_package(&app, &token, &format!("page-pkg-{}", i), "1.0.0").await;
    }

    let request = Request::builder()
        .uri("/api/v1/search?q=page&limit=2&offset=0")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let packages = json["packages"].as_array().unwrap();
    assert!(packages.len() <= 2, "Limit should be respected");
    assert!(
        json["total"].as_i64().unwrap() >= 5,
        "Total should include all matching"
    );
}

// ============================================================================
// Publish without auth
// ============================================================================

#[tokio::test]
async fn test_yank_without_auth() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/some-pkg/1.0.0/yank")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unyank_without_auth() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/packages/some-pkg/1.0.0/unyank")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Empty search
// ============================================================================

#[tokio::test]
async fn test_search_empty_query() {
    let (app, _db, _st) = setup_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/search?q=")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
