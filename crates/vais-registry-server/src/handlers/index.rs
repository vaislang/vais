//! Index handlers for registry client compatibility

use crate::db;
use crate::error::{ServerError, ServerResult};
use crate::handlers::AppState;
use crate::models::*;
use axum::{
    extract::{Path, State},
    Json,
};

/// Get the full package index (compatible with existing client)
pub async fn get_full_index(State(state): State<AppState>) -> ServerResult<Json<Vec<IndexEntry>>> {
    let entries = db::get_full_index(&state.pool).await?;
    Ok(Json(entries))
}

/// Get a single package's index entry
pub async fn get_package_index(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ServerResult<Json<IndexEntry>> {
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    let versions = db::get_all_versions(&state.pool, package.id).await?;
    let owners = db::get_package_owners(&state.pool, package.id).await?;

    let mut version_entries = Vec::new();
    for v in versions {
        let deps = db::get_dependencies(&state.pool, v.id).await?;
        let dep_map: std::collections::HashMap<String, IndexDependency> = deps
            .into_iter()
            .filter(|d| d.kind == DependencyKind::Normal)
            .map(|d| {
                (
                    d.name,
                    IndexDependency {
                        req: d.version_req,
                        features: d.features,
                        optional: d.optional,
                        target: d.target,
                    },
                )
            })
            .collect();

        version_entries.push(IndexVersionEntry {
            version: v.version,
            checksum: v.checksum,
            dependencies: dep_map,
            yanked: v.yanked,
            download_url: None,
            size: Some(v.size as u64),
        });
    }

    Ok(Json(IndexEntry {
        name: package.name,
        description: package.description,
        versions: version_entries,
        authors: owners,
        homepage: package.homepage,
        repository: package.repository,
        license: package.license,
        keywords: package.keywords,
    }))
}
