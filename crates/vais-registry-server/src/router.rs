//! HTTP router configuration

use crate::config::ServerConfig;
use crate::db::DbPool;
use crate::handlers::{self, AppState};
use crate::storage::PackageStorage;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Create the application router
pub fn create_router(pool: DbPool, storage: PackageStorage, config: ServerConfig) -> Router {
    let state = AppState {
        pool,
        storage: Arc::new(storage),
        config: Arc::new(config.clone()),
    };

    let api_routes = Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Index routes (compatible with existing client)
        .route("/index.json", get(handlers::index::get_full_index))
        .route(
            "/packages/:name/index.json",
            get(handlers::index::get_package_index),
        )
        // Package routes
        .route("/packages/publish", post(handlers::packages::publish))
        .route("/packages/:name", get(handlers::packages::get_package))
        .route(
            "/packages/:name/:version",
            get(handlers::packages::download),
        )
        .route(
            "/packages/:name/:version/yank",
            post(handlers::packages::yank),
        )
        .route(
            "/packages/:name/:version/unyank",
            post(handlers::packages::unyank),
        )
        // Search & Discovery
        .route("/search", get(handlers::packages::search))
        .route("/categories", get(handlers::packages::list_categories))
        .route(
            "/categories/:category",
            get(handlers::packages::browse_category),
        )
        .route("/popular", get(handlers::packages::popular))
        .route("/recent", get(handlers::packages::recent))
        // Auth routes
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/tokens", get(handlers::auth::list_tokens))
        .route("/auth/tokens", post(handlers::auth::create_token))
        .route("/auth/tokens/:id", delete(handlers::auth::delete_token))
        // User routes
        .route("/users/:username", get(handlers::users::get_user))
        // Owner management
        .route("/packages/:name/owners", post(handlers::users::add_owner))
        .route(
            "/packages/:name/owners/:username",
            delete(handlers::users::remove_owner),
        );

    let cors = if config.cors_allow_all {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<_> = config
            .cors_origins
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Web UI routes
    let web_routes = Router::new()
        .route("/", get(handlers::web::index))
        .route("/dashboard", get(handlers::web::dashboard))
        .route("/health", get(handlers::health)) // Root-level health check
        .route("/packages/:name", get(handlers::web::package_detail))
        .route("/static/styles.css", get(handlers::web::serve_css));

    let app = Router::new()
        .merge(web_routes)
        .nest("/api/v1", api_routes)
        .layer(cors);

    let app = if config.enable_logging {
        app.layer(TraceLayer::new_for_http())
    } else {
        app
    };

    app.with_state(state)
}
