//! HTTP request handlers

pub mod auth;
pub mod index;
pub mod packages;
pub mod users;
pub mod web;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_token(scopes: Vec<String>) -> ApiToken {
        ApiToken {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            token_hash: "hash".to_string(),
            scopes,
            expires_at: None,
            last_used_at: None,
            created_at: Utc::now(),
        }
    }

    fn make_auth_user(is_admin: bool, scopes: Vec<String>) -> AuthUser {
        let token = make_test_token(scopes);
        AuthUser {
            user_id: token.user_id,
            username: "testuser".to_string(),
            is_admin,
            token,
        }
    }

    #[test]
    fn test_has_scope_with_scope() {
        let user = make_auth_user(false, vec!["publish".to_string()]);
        assert!(user.has_scope("publish"));
        assert!(!user.has_scope("yank"));
        assert!(!user.has_scope("admin"));
    }

    #[test]
    fn test_has_scope_admin_has_all() {
        let user = make_auth_user(true, vec![]);
        assert!(user.has_scope("publish"));
        assert!(user.has_scope("yank"));
        assert!(user.has_scope("admin"));
        assert!(user.has_scope("anything"));
    }

    #[test]
    fn test_has_scope_empty_scopes() {
        let user = make_auth_user(false, vec![]);
        assert!(!user.has_scope("publish"));
    }

    #[test]
    fn test_has_scope_multiple_scopes() {
        let user = make_auth_user(false, vec!["publish".to_string(), "yank".to_string()]);
        assert!(user.has_scope("publish"));
        assert!(user.has_scope("yank"));
        assert!(!user.has_scope("admin"));
    }

    #[test]
    fn test_require_scope_ok() {
        let user = make_auth_user(false, vec!["publish".to_string()]);
        assert!(user.require_scope("publish").is_ok());
    }

    #[test]
    fn test_require_scope_forbidden() {
        let user = make_auth_user(false, vec!["publish".to_string()]);
        let result = user.require_scope("admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_require_scope_admin_always_ok() {
        let user = make_auth_user(true, vec![]);
        assert!(user.require_scope("publish").is_ok());
        assert!(user.require_scope("yank").is_ok());
        assert!(user.require_scope("admin").is_ok());
    }

    #[test]
    fn test_health_response_serialize() {
        let resp = HealthResponse {
            status: "ok".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }
}
