//! Vais Package Registry Server
//!
//! A REST API server for hosting Vais packages.
//!
//! ## Features
//! - Package publishing and retrieval
//! - User authentication with API tokens
//! - Package search and discovery
//! - SQLite-based metadata storage
//! - Version yanking support

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod router;
pub mod storage;

pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};
pub use router::create_router;
