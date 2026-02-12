//! Package management handlers

use crate::db;
use crate::error::{ServerError, ServerResult};
use crate::handlers::{AppState, AuthUser};
use crate::models::*;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

/// Publish a new package version
pub async fn publish(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> ServerResult<Json<serde_json::Value>> {
    auth.require_scope(scopes::PUBLISH)?;

    let mut metadata: Option<PublishRequest> = None;
    let mut archive_data: Option<Vec<u8>> = None;

    // Parse multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServerError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "metadata" => {
                let data = field.bytes().await.map_err(|e| {
                    ServerError::BadRequest(format!("Failed to read metadata: {}", e))
                })?;
                metadata = Some(serde_json::from_slice(&data)?);
            }
            "archive" => {
                let data = field.bytes().await.map_err(|e| {
                    ServerError::BadRequest(format!("Failed to read archive: {}", e))
                })?;

                // Check size limit
                if data.len() > state.config.max_upload_size {
                    return Err(ServerError::BadRequest(format!(
                        "Archive exceeds maximum size of {} bytes",
                        state.config.max_upload_size
                    )));
                }

                archive_data = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let metadata =
        metadata.ok_or_else(|| ServerError::BadRequest("Missing metadata".to_string()))?;
    let archive_data =
        archive_data.ok_or_else(|| ServerError::BadRequest("Missing archive".to_string()))?;

    // Validate package name
    validate_package_name(&metadata.name)?;

    // Validate version
    validate_version(&metadata.version)?;

    // Validate archive
    let manifest = state.storage.validate_archive(&archive_data)?;

    // Check manifest matches metadata
    if manifest.package.name != metadata.name {
        return Err(ServerError::BadRequest(format!(
            "Manifest name '{}' doesn't match metadata name '{}'",
            manifest.package.name, metadata.name
        )));
    }

    if manifest.package.version != metadata.version {
        return Err(ServerError::BadRequest(format!(
            "Manifest version '{}' doesn't match metadata version '{}'",
            manifest.package.version, metadata.version
        )));
    }

    let now = Utc::now();

    // Get or create package
    let package = match db::get_package_by_name(&state.pool, &metadata.name).await? {
        Some(mut pkg) => {
            // Check ownership
            if !db::is_package_owner(&state.pool, pkg.id, auth.user_id).await? {
                return Err(ServerError::Forbidden(
                    "You don't have permission to publish to this package".to_string(),
                ));
            }

            // Update package metadata
            pkg.description = metadata.description.clone().or(pkg.description);
            pkg.homepage = metadata.homepage.clone().or(pkg.homepage);
            pkg.repository = metadata.repository.clone().or(pkg.repository);
            pkg.documentation = metadata.documentation.clone().or(pkg.documentation);
            pkg.license = metadata.license.clone().or(pkg.license);
            if !metadata.keywords.is_empty() {
                pkg.keywords = metadata.keywords.clone();
            }
            if !metadata.categories.is_empty() {
                pkg.categories = metadata.categories.clone();
            }
            pkg.updated_at = now;

            db::update_package(&state.pool, &pkg).await?;
            pkg
        }
        None => {
            // Create new package
            let pkg = Package {
                id: Uuid::new_v4(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                homepage: metadata.homepage.clone(),
                repository: metadata.repository.clone(),
                documentation: metadata.documentation.clone(),
                license: metadata.license.clone(),
                keywords: metadata.keywords.clone(),
                categories: metadata.categories.clone(),
                owner_id: auth.user_id,
                created_at: now,
                updated_at: now,
                downloads: 0,
            };

            db::create_package(&state.pool, &pkg).await?;
            pkg
        }
    };

    // Check if version already exists
    if db::get_version(&state.pool, package.id, &metadata.version)
        .await?
        .is_some()
    {
        return Err(ServerError::VersionExists(
            metadata.name.clone(),
            metadata.version.clone(),
        ));
    }

    // Store archive and get checksum
    let checksum = state
        .storage
        .store_archive(&metadata.name, &metadata.version, &archive_data)?;

    // Create version
    let version = PackageVersion {
        id: Uuid::new_v4(),
        package_id: package.id,
        version: metadata.version.clone(),
        checksum,
        size: archive_data.len() as i64,
        yanked: false,
        readme: metadata.readme.clone(),
        published_by: auth.user_id,
        created_at: now,
        downloads: 0,
    };

    db::create_version(&state.pool, &version).await?;

    // Create dependencies
    let mut deps = Vec::new();

    for (name, spec) in &metadata.dependencies {
        deps.push(Dependency {
            id: Uuid::new_v4(),
            version_id: version.id,
            name: name.clone(),
            version_req: spec.version_req().to_string(),
            features: spec.features(),
            optional: spec.is_optional(),
            target: spec.target().map(String::from),
            kind: DependencyKind::Normal,
        });
    }

    for (name, spec) in &metadata.dev_dependencies {
        deps.push(Dependency {
            id: Uuid::new_v4(),
            version_id: version.id,
            name: name.clone(),
            version_req: spec.version_req().to_string(),
            features: spec.features(),
            optional: spec.is_optional(),
            target: spec.target().map(String::from),
            kind: DependencyKind::Dev,
        });
    }

    if !deps.is_empty() {
        db::create_dependencies(&state.pool, &deps).await?;
    }

    Ok(Json(serde_json::json!({
        "name": metadata.name,
        "version": metadata.version,
        "checksum": version.checksum
    })))
}

/// Get package information
pub async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ServerResult<Json<PackageInfo>> {
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    let versions = db::get_all_versions(&state.pool, package.id).await?;
    let owners = db::get_package_owners(&state.pool, package.id).await?;

    let mut version_infos = Vec::new();
    for v in versions {
        let deps = db::get_dependencies(&state.pool, v.id).await?;
        version_infos.push(VersionInfo {
            version: v.version,
            checksum: v.checksum,
            size: v.size,
            yanked: v.yanked,
            downloads: v.downloads,
            created_at: v.created_at,
            dependencies: deps
                .into_iter()
                .map(|d| DependencyInfo {
                    name: d.name,
                    version_req: d.version_req,
                    features: d.features,
                    optional: d.optional,
                    target: d.target,
                    kind: d.kind,
                })
                .collect(),
        });
    }

    Ok(Json(PackageInfo {
        name: package.name,
        description: package.description,
        homepage: package.homepage,
        repository: package.repository,
        documentation: package.documentation,
        license: package.license,
        keywords: package.keywords,
        categories: package.categories,
        downloads: package.downloads,
        created_at: package.created_at,
        updated_at: package.updated_at,
        versions: version_infos,
        owners,
    }))
}

/// Download a package archive
pub async fn download(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> ServerResult<impl IntoResponse> {
    // Remove .tar.gz suffix if present
    let version = version
        .strip_suffix(".tar.gz")
        .unwrap_or(&version)
        .to_string();

    // Get package
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    // Get version
    let pkg_version = db::get_version(&state.pool, package.id, &version)
        .await?
        .ok_or_else(|| ServerError::VersionNotFound(name.clone(), version.clone()))?;

    // Read archive
    let data = state.storage.read_archive(&name, &version)?;

    // Increment download count
    db::increment_download(&state.pool, pkg_version.id).await?;

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/gzip"),
            ),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!(
                    "attachment; filename=\"{}-{}.tar.gz\"",
                    name, version
                ))
                .unwrap(),
            ),
            (
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&data.len().to_string()).unwrap(),
            ),
        ],
        data,
    ))
}

/// Yank a version
pub async fn yank(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((name, version)): Path<(String, String)>,
) -> ServerResult<StatusCode> {
    auth.require_scope(scopes::YANK)?;

    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    // Check ownership
    if !db::is_package_owner(&state.pool, package.id, auth.user_id).await? && !auth.is_admin {
        return Err(ServerError::Forbidden(
            "You don't have permission to yank this version".to_string(),
        ));
    }

    let yanked = db::set_version_yanked(&state.pool, package.id, &version, true).await?;

    if yanked {
        Ok(StatusCode::OK)
    } else {
        Err(ServerError::VersionNotFound(name, version))
    }
}

/// Unyank a version
pub async fn unyank(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((name, version)): Path<(String, String)>,
) -> ServerResult<StatusCode> {
    auth.require_scope(scopes::YANK)?;

    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    // Check ownership
    if !db::is_package_owner(&state.pool, package.id, auth.user_id).await? && !auth.is_admin {
        return Err(ServerError::Forbidden(
            "You don't have permission to unyank this version".to_string(),
        ));
    }

    let unyanked = db::set_version_yanked(&state.pool, package.id, &version, false).await?;

    if unyanked {
        Ok(StatusCode::OK)
    } else {
        Err(ServerError::VersionNotFound(name, version))
    }
}

/// Search packages with sorting, category, and keyword filtering
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ServerResult<Json<SearchResult>> {
    let (packages, total) = db::search_packages_advanced(
        &state.pool,
        &query.q,
        query.limit,
        query.offset,
        &query.sort,
        query.category.as_deref(),
        query.keyword.as_deref(),
    )
    .await?;

    // Get category counts for faceted search
    let categories = db::get_category_counts(&state.pool).await?;

    Ok(Json(SearchResult {
        packages,
        total,
        categories,
    }))
}

/// Validate package name
fn validate_package_name(name: &str) -> ServerResult<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(ServerError::InvalidPackageName(
            "Name must be 1-64 characters".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(ServerError::InvalidPackageName(
            "Name can only contain lowercase letters, digits, - and _".to_string(),
        ));
    }

    if name.starts_with('-') || name.starts_with('_') {
        return Err(ServerError::InvalidPackageName(
            "Name cannot start with - or _".to_string(),
        ));
    }

    // Reserved names
    let reserved = ["std", "core", "vais", "test", "main", "lib"];
    if reserved.contains(&name) {
        return Err(ServerError::InvalidPackageName(format!(
            "'{}' is a reserved name",
            name
        )));
    }

    Ok(())
}

/// List all categories with package counts
pub async fn list_categories(
    State(state): State<AppState>,
) -> ServerResult<Json<Vec<CategoryCount>>> {
    let categories = db::get_category_counts(&state.pool).await?;
    Ok(Json(categories))
}

/// Browse packages by category
pub async fn browse_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
    Query(params): Query<BrowseParams>,
) -> ServerResult<Json<SearchResult>> {
    let (packages, total) = db::search_packages_advanced(
        &state.pool,
        "",
        params.limit,
        params.offset,
        &params.sort,
        Some(&category),
        None,
    )
    .await?;

    let categories = db::get_category_counts(&state.pool).await?;

    Ok(Json(SearchResult {
        packages,
        total,
        categories,
    }))
}

/// Get popular packages (sorted by downloads)
pub async fn popular(
    State(state): State<AppState>,
    Query(params): Query<BrowseParams>,
) -> ServerResult<Json<SearchResult>> {
    let (packages, total) = db::search_packages_advanced(
        &state.pool,
        "",
        params.limit,
        params.offset,
        "downloads",
        None,
        None,
    )
    .await?;

    Ok(Json(SearchResult {
        packages,
        total,
        categories: vec![],
    }))
}

/// Get recently updated packages
pub async fn recent(
    State(state): State<AppState>,
    Query(params): Query<BrowseParams>,
) -> ServerResult<Json<SearchResult>> {
    let (packages, total) = db::search_packages_advanced(
        &state.pool,
        "",
        params.limit,
        params.offset,
        "newest",
        None,
        None,
    )
    .await?;

    Ok(Json(SearchResult {
        packages,
        total,
        categories: vec![],
    }))
}

/// Browse parameters
#[derive(Debug, Deserialize)]
pub struct BrowseParams {
    #[serde(default = "default_browse_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_browse_sort")]
    pub sort: String,
}

fn default_browse_limit() -> usize {
    20
}

fn default_browse_sort() -> String {
    "downloads".to_string()
}

/// Get registry statistics
pub async fn stats(State(state): State<AppState>) -> ServerResult<Json<RegistryStats>> {
    let stats = db::get_registry_stats(&state.pool).await?;
    Ok(Json(stats))
}

/// Validate version string
fn validate_version(version: &str) -> ServerResult<()> {
    // Simple semver validation
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() < 3 {
        return Err(ServerError::InvalidVersion(
            "Version must be in semver format (e.g., 1.0.0)".to_string(),
        ));
    }

    // Check major.minor.patch are numbers
    for (i, part) in parts.iter().take(3).enumerate() {
        // Handle prerelease/build metadata
        let num_part = if i == 2 {
            part.split(['-', '+']).next().unwrap_or(part)
        } else {
            part
        };

        if num_part.parse::<u64>().is_err() {
            return Err(ServerError::InvalidVersion(format!(
                "Invalid version component: {}",
                part
            )));
        }
    }

    Ok(())
}
