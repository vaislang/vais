//! Authentication handlers

use crate::db;
use crate::error::{ServerError, ServerResult};
use crate::handlers::{AppState, AuthUser};
use crate::models::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> ServerResult<Json<UserInfo>> {
    // Validate username
    if req.username.len() < 3 || req.username.len() > 32 {
        return Err(ServerError::BadRequest(
            "Username must be 3-32 characters".to_string(),
        ));
    }

    if !req
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ServerError::BadRequest(
            "Username can only contain alphanumeric characters, _ and -".to_string(),
        ));
    }

    // Validate password
    if req.password.len() < 8 {
        return Err(ServerError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check if user exists
    if db::get_user_by_username(&state.pool, &req.username)
        .await?
        .is_some()
    {
        return Err(ServerError::UserExists(req.username));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ServerError::Internal(e.to_string()))?
        .to_string();

    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        username: req.username,
        password_hash,
        email: req.email,
        is_admin: false,
        created_at: now,
        updated_at: now,
    };

    db::create_user(&state.pool, &user).await?;

    Ok(Json(UserInfo {
        id: user.id,
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
    }))
}

/// Login and get an API token
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ServerResult<Json<LoginResponse>> {
    // Get user
    let user = db::get_user_by_username(&state.pool, &req.username)
        .await?
        .ok_or(ServerError::InvalidCredentials)?;

    // Verify password
    let parsed_hash =
        PasswordHash::new(&user.password_hash).map_err(|e| ServerError::Internal(e.to_string()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| ServerError::InvalidCredentials)?;

    // Generate token
    let token = generate_token();
    let token_hash = hash_token(&token);

    let expires_at = Some(Utc::now() + Duration::days(state.config.token_expiration_days as i64));

    let api_token = ApiToken {
        id: Uuid::new_v4(),
        user_id: user.id,
        name: "login".to_string(),
        token_hash,
        scopes: vec![scopes::PUBLISH.to_string(), scopes::YANK.to_string()],
        expires_at,
        last_used_at: None,
        created_at: Utc::now(),
    };

    db::create_token(&state.pool, &api_token).await?;

    Ok(Json(LoginResponse {
        token,
        expires_at,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
        },
    }))
}

/// Create a new API token
pub async fn create_token(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTokenRequest>,
) -> ServerResult<Json<CreateTokenResponse>> {
    // Validate scopes
    let valid_scopes = [scopes::PUBLISH, scopes::YANK, scopes::ADMIN];
    for scope in &req.scopes {
        if !valid_scopes.contains(&scope.as_str()) {
            return Err(ServerError::BadRequest(format!("Invalid scope: {}", scope)));
        }

        // Non-admins can't create admin tokens
        if scope == scopes::ADMIN && !auth.is_admin {
            return Err(ServerError::Forbidden(
                "Only admins can create admin tokens".to_string(),
            ));
        }
    }

    let token = generate_token();
    let token_hash = hash_token(&token);

    let expires_at = req
        .expires_in_days
        .map(|days| Utc::now() + Duration::days(days as i64));

    let scopes = if req.scopes.is_empty() {
        vec![scopes::PUBLISH.to_string(), scopes::YANK.to_string()]
    } else {
        req.scopes
    };

    let api_token = ApiToken {
        id: Uuid::new_v4(),
        user_id: auth.user_id,
        name: req.name.clone(),
        token_hash,
        scopes: scopes.clone(),
        expires_at,
        last_used_at: None,
        created_at: Utc::now(),
    };

    db::create_token(&state.pool, &api_token).await?;

    Ok(Json(CreateTokenResponse {
        id: api_token.id,
        token,
        name: req.name,
        scopes,
        expires_at,
    }))
}

/// List user's tokens
pub async fn list_tokens(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ServerResult<Json<Vec<ApiToken>>> {
    let tokens = db::get_user_tokens(&state.pool, auth.user_id).await?;
    Ok(Json(tokens))
}

/// Delete a token
pub async fn delete_token(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(token_id): axum::extract::Path<Uuid>,
) -> ServerResult<axum::http::StatusCode> {
    let deleted = db::delete_token(&state.pool, token_id, auth.user_id).await?;

    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(ServerError::PackageNotFound(token_id.to_string()))
    }
}

/// Get current user info
pub async fn me(auth: AuthUser) -> Json<UserInfo> {
    Json(UserInfo {
        id: auth.user_id,
        username: auth.username,
        email: None, // Would need to fetch from DB
        is_admin: auth.is_admin,
    })
}

/// Generate a random API token
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    format!("vais_{}", hex::encode(bytes))
}

/// Hash a token for storage
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
