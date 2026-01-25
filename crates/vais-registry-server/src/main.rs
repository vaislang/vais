//! Vais Package Registry Server
//!
//! Run with: cargo run --bin vais-registry-server
//!
//! Configuration via environment variables:
//! - VAIS_REGISTRY_HOST: Server host (default: 0.0.0.0)
//! - VAIS_REGISTRY_PORT: Server port (default: 3000)
//! - VAIS_REGISTRY_DB: SQLite database path (default: ./data/registry.db)
//! - VAIS_REGISTRY_STORAGE: Package storage path (default: ./data/packages)
//! - VAIS_REGISTRY_ADMIN_USER: Initial admin username
//! - VAIS_REGISTRY_ADMIN_PASS: Initial admin password

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vais_registry_server::{config::ServerConfig, create_router, db, storage::PackageStorage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vais_registry_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = ServerConfig::from_env();

    tracing::info!("Starting Vais Registry Server");
    tracing::info!("Database: {}", config.database_path.display());
    tracing::info!("Storage: {}", config.storage_path.display());

    // Initialize database
    let pool = db::init_db(&config.database_path).await?;
    tracing::info!("Database initialized");

    // Create initial admin user if configured
    if let (Some(username), Some(password)) = (&config.admin_username, &config.admin_password) {
        if db::get_user_by_username(&pool, username).await?.is_none() {
            use argon2::{
                password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
                Argon2,
            };
            use chrono::Utc;
            use uuid::Uuid;

            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| format!("Failed to hash password: {}", e))?
                .to_string();

            let now = Utc::now();
            let admin = vais_registry_server::models::User {
                id: Uuid::new_v4(),
                username: username.clone(),
                password_hash,
                email: None,
                is_admin: true,
                created_at: now,
                updated_at: now,
            };

            db::create_user(&pool, &admin).await?;
            tracing::info!("Created admin user: {}", username);
        }
    }

    // Initialize storage
    let storage = PackageStorage::new(config.storage_path.clone())?;
    tracing::info!("Storage initialized");

    // Create router
    let app = create_router(pool, storage, config.clone());

    // Start server
    let addr = config.bind_addr();
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
