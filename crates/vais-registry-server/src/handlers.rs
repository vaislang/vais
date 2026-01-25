//! HTTP request handlers

pub mod auth;
pub mod index;
pub mod packages;
pub mod users;

use crate::config::ServerConfig;
use crate::db::DbPool;
use crate::error::{ServerError, ServerResult};
use crate::models::ApiToken;
use crate::storage::PackageStorage;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub storage: Arc<PackageStorage>,
    pub config: Arc<ServerConfig>,
}

/// Authenticated user extracted from request
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub is_admin: bool,
    pub token: ApiToken,
}

impl AuthUser {
    /// Check if user has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.is_admin || self.token.scopes.contains(&scope.to_string())
    }

    /// Require a specific scope
    pub fn require_scope(&self, scope: &str) -> ServerResult<()> {
        if self.has_scope(scope) {
            Ok(())
        } else {
            Err(ServerError::Forbidden(format!(
                "Missing required scope: {}",
                scope
            )))
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ServerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ServerError::Unauthorized)?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ServerError::InvalidToken)?;

        // Hash the token
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        // Look up token in database
        let api_token = crate::db::get_token_by_hash(&state.pool, &token_hash)
            .await?
            .ok_or(ServerError::InvalidToken)?;

        // Check expiration
        if let Some(expires_at) = api_token.expires_at {
            if expires_at < Utc::now() {
                return Err(ServerError::TokenExpired);
            }
        }

        // Get user
        let user = crate::db::get_user_by_id(&state.pool, api_token.user_id)
            .await?
            .ok_or(ServerError::InvalidToken)?;

        // Update last used
        crate::db::update_token_last_used(&state.pool, api_token.id).await?;

        Ok(AuthUser {
            user_id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            token: api_token,
        })
    }
}

/// Optional authentication (doesn't fail if not authenticated)
pub struct OptionalAuth(pub Option<AuthUser>);

#[axum::async_trait]
impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = ServerError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuth(Some(user))),
            Err(_) => Ok(OptionalAuth(None)),
        }
    }
}

/// Health check response
#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check handler
pub async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
