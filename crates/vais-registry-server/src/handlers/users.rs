//! User management handlers

use crate::db;
use crate::error::{ServerError, ServerResult};
use crate::handlers::{AppState, AuthUser};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

/// Get user profile by username
pub async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ServerResult<Json<UserProfile>> {
    let user = db::get_user_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| ServerError::UserNotFound(username))?;

    // Get packages owned by user
    let packages = get_user_packages(&state.pool, user.id).await?;

    Ok(Json(UserProfile {
        username: user.username,
        email: None, // Don't expose email publicly
        created_at: user.created_at,
        packages,
    }))
}

/// User profile response
#[derive(serde::Serialize)]
pub struct UserProfile {
    pub username: String,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub packages: Vec<UserPackage>,
}

#[derive(serde::Serialize)]
pub struct UserPackage {
    pub name: String,
    pub description: Option<String>,
    pub downloads: i64,
}

async fn get_user_packages(pool: &crate::db::DbPool, user_id: uuid::Uuid) -> ServerResult<Vec<UserPackage>> {
    let rows = sqlx::query(
        r#"
        SELECT p.name, p.description, p.downloads
        FROM packages p
        JOIN package_owners po ON p.id = po.package_id
        WHERE po.user_id = ?
        ORDER BY p.downloads DESC
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            use sqlx::Row;
            UserPackage {
                name: r.get("name"),
                description: r.get("description"),
                downloads: r.get("downloads"),
            }
        })
        .collect())
}

/// Add a new owner to a package
pub async fn add_owner(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(name): Path<String>,
    Json(req): Json<AddOwnerRequest>,
) -> ServerResult<StatusCode> {
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    // Check if requester is an owner
    if !db::is_package_owner(&state.pool, package.id, auth.user_id).await? && !auth.is_admin {
        return Err(ServerError::Forbidden(
            "Only owners can add new owners".to_string(),
        ));
    }

    // Get user to add
    let new_owner = db::get_user_by_username(&state.pool, &req.username)
        .await?
        .ok_or_else(|| ServerError::UserNotFound(req.username.clone()))?;

    // Check if already an owner
    if db::is_package_owner(&state.pool, package.id, new_owner.id).await? {
        return Err(ServerError::BadRequest(format!(
            "'{}' is already an owner of '{}'",
            req.username, name
        )));
    }

    // Add owner
    sqlx::query(
        r#"
        INSERT INTO package_owners (package_id, user_id, added_by, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(package.id.to_string())
    .bind(new_owner.id.to_string())
    .bind(auth.user_id.to_string())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::CREATED)
}

/// Remove an owner from a package
pub async fn remove_owner(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((name, username)): Path<(String, String)>,
) -> ServerResult<StatusCode> {
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    // Check if requester is an owner
    if !db::is_package_owner(&state.pool, package.id, auth.user_id).await? && !auth.is_admin {
        return Err(ServerError::Forbidden(
            "Only owners can remove owners".to_string(),
        ));
    }

    // Get user to remove
    let user_to_remove = db::get_user_by_username(&state.pool, &username)
        .await?
        .ok_or_else(|| ServerError::UserNotFound(username.clone()))?;

    // Can't remove the last owner
    let owners = db::get_package_owners(&state.pool, package.id).await?;
    if owners.len() == 1 {
        return Err(ServerError::BadRequest(
            "Cannot remove the last owner of a package".to_string(),
        ));
    }

    // Remove owner
    let result = sqlx::query(
        "DELETE FROM package_owners WHERE package_id = ? AND user_id = ?",
    )
    .bind(package.id.to_string())
    .bind(user_to_remove.id.to_string())
    .execute(&state.pool)
    .await?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ServerError::BadRequest(format!(
            "'{}' is not an owner of '{}'",
            username, name
        )))
    }
}

#[derive(serde::Deserialize)]
pub struct AddOwnerRequest {
    pub username: String,
}
