//! Web UI handlers

use crate::db;
use crate::error::{ServerError, ServerResult};
use crate::handlers::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::Row;

/// Query parameters for search
#[derive(Debug, Deserialize)]
pub struct WebSearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default)]
    pub category: Option<String>,
}

fn default_sort() -> String {
    "downloads".to_string()
}

/// Home page with search
pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<WebSearchQuery>,
) -> ServerResult<Html<String>> {
    let template = include_str!("../../static/index.html");

    // Get category counts for filter chips
    let categories = db::get_category_counts(&state.pool)
        .await
        .unwrap_or_default();
    let category_chips = if !categories.is_empty() {
        let chips: Vec<String> = categories
            .iter()
            .take(10) // Show top 10 categories
            .map(|cat| {
                let active = query.category.as_ref() == Some(&cat.name);
                let active_class = if active { " active" } else { "" };
                format!(
                    r#"<a href="/?q={}&category={}&sort={}" class="category-chip{}">
                        {}
                        <span class="category-count">({})</span>
                    </a>"#,
                    html_escape(&query.q),
                    html_escape(&cat.name),
                    html_escape(&query.sort),
                    active_class,
                    html_escape(&cat.name),
                    cat.count
                )
            })
            .collect();
        chips.join("\n")
    } else {
        String::new()
    };

    let search_results = if !query.q.is_empty() {
        // Perform advanced search
        match db::search_packages_advanced(
            &state.pool,
            &query.q,
            20,
            0,
            &query.sort,
            query.category.as_deref(),
            None,
        )
        .await
        {
            Ok((packages, _total)) => {
                if packages.is_empty() {
                    r#"<div class="no-results">
                        <p>No packages found matching your query.</p>
                    </div>"#
                        .to_string()
                } else {
                    let mut html = String::new();
                    for pkg in packages {
                        let description = pkg
                            .description
                            .as_deref()
                            .unwrap_or("No description available.");
                        let keywords_html = if !pkg.keywords.is_empty() {
                            let keywords: Vec<String> = pkg
                                .keywords
                                .iter()
                                .map(|k| {
                                    format!(r#"<span class="keyword">{}</span>"#, html_escape(k))
                                })
                                .collect();
                            format!(r#"<div class="keywords">{}</div>"#, keywords.join(""))
                        } else {
                            String::new()
                        };

                        html.push_str(&format!(
                            r#"<div class="package-card">
                                <h3><a href="/packages/{}">{}</a></h3>
                                <p class="package-description">{}</p>
                                <div class="package-info">
                                    <span>Version: {}</span>
                                    <span>Downloads: {}</span>
                                    <span>Updated: {}</span>
                                </div>
                                {}
                            </div>"#,
                            html_escape(&pkg.name),
                            html_escape(&pkg.name),
                            html_escape(description),
                            html_escape(&pkg.latest_version),
                            pkg.downloads,
                            pkg.updated_at.format("%Y-%m-%d"),
                            keywords_html
                        ));
                    }
                    html
                }
            }
            Err(_) => r#"<div class="no-results">
                <p>Error performing search. Please try again.</p>
            </div>"#
                .to_string(),
        }
    } else {
        r#"<div class="no-results">
            <p>Enter a search query to find packages.</p>
        </div>"#
            .to_string()
    };

    // Sort selection attributes
    let sort_downloads = if query.sort == "downloads" {
        "selected"
    } else {
        ""
    };
    let sort_newest = if query.sort == "newest" {
        "selected"
    } else {
        ""
    };
    let sort_name = if query.sort == "name" { "selected" } else { "" };
    let sort_relevance = if query.sort == "relevance" {
        "selected"
    } else {
        ""
    };

    let html = template
        .replace("{{SEARCH_QUERY}}", &html_escape(&query.q))
        .replace("{{SEARCH_RESULTS}}", &search_results)
        .replace("{{CATEGORY_CHIPS}}", &category_chips)
        .replace("{{SORT_DOWNLOADS}}", sort_downloads)
        .replace("{{SORT_NEWEST}}", sort_newest)
        .replace("{{SORT_NAME}}", sort_name)
        .replace("{{SORT_RELEVANCE}}", sort_relevance);

    Ok(Html(html))
}

/// Package detail page
pub async fn package_detail(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ServerResult<Response> {
    let template = include_str!("../../static/package.html");

    // Get package info
    let package = db::get_package_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| ServerError::PackageNotFound(name.clone()))?;

    let versions = db::get_all_versions(&state.pool, package.id).await?;
    let owners = db::get_package_owners(&state.pool, package.id).await?;

    // Find latest version
    let latest_version = versions
        .iter()
        .filter(|v| !v.yanked)
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .or_else(|| versions.first())
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "N/A".to_string());

    // Get dependencies for latest version
    let latest_deps = if let Some(latest) = versions.iter().find(|v| v.version == latest_version) {
        db::get_dependencies(&state.pool, latest.id).await?
    } else {
        vec![]
    };

    // Build HTML replacements
    let description = package
        .description
        .as_deref()
        .unwrap_or("No description available.");

    let license = package.license.as_deref().unwrap_or("Not specified");

    let homepage_link = if let Some(homepage) = &package.homepage {
        format!(
            r#"<a href="{}" target="_blank">Homepage</a>"#,
            html_escape(homepage)
        )
    } else {
        String::new()
    };

    let repository_link = if let Some(repo) = &package.repository {
        format!(
            r#"<a href="{}" target="_blank">Repository</a>"#,
            html_escape(repo)
        )
    } else {
        String::new()
    };

    let documentation_link = if let Some(docs) = &package.documentation {
        format!(
            r#"<a href="{}" target="_blank">Documentation</a>"#,
            html_escape(docs)
        )
    } else {
        String::new()
    };

    let keywords = if !package.keywords.is_empty() {
        let keywords_html: Vec<String> = package
            .keywords
            .iter()
            .map(|k| format!(r#"<span class="keyword">{}</span>"#, html_escape(k)))
            .collect();
        format!(r#"<div class="keywords">{}</div>"#, keywords_html.join(""))
    } else {
        String::new()
    };

    // Build versions list
    let mut versions_html = String::new();
    for version in &versions {
        let yanked_class = if version.yanked {
            " version-yanked"
        } else {
            ""
        };
        let yanked_text = if version.yanked { " (yanked)" } else { "" };

        versions_html.push_str(&format!(
            r#"<div class="version-item">
                <div>
                    <span class="version-number{}">{}{}</span>
                    <div class="version-info">
                        <span>Size: {} bytes</span>
                        <span>Downloads: {}</span>
                        <span>Published: {}</span>
                    </div>
                </div>
                <a href="/api/v1/packages/{}/{}.tar.gz" class="download-link">Download</a>
            </div>"#,
            yanked_class,
            html_escape(&version.version),
            yanked_text,
            version.size,
            version.downloads,
            version.created_at.format("%Y-%m-%d"),
            html_escape(&name),
            html_escape(&version.version)
        ));
    }

    // Build dependencies list
    let dependencies_html = if latest_deps.is_empty() {
        "<p>No dependencies</p>".to_string()
    } else {
        let mut html = String::new();
        for dep in &latest_deps {
            let badges = if dep.optional {
                r#"<div class="dependency-badges">
                    <span class="badge badge-optional">OPTIONAL</span>
                </div>"#
                    .to_string()
            } else {
                String::new()
            };

            html.push_str(&format!(
                r#"<div class="dependency-item">
                    <div>
                        <span class="dependency-name">{}</span>
                        <span class="dependency-version">{}</span>
                    </div>
                    {}
                </div>"#,
                html_escape(&dep.name),
                html_escape(&dep.version_req),
                badges
            ));
        }
        html
    };

    // Build README section
    let readme_section = if let Some(latest) = versions.iter().find(|v| v.version == latest_version)
    {
        if let Some(readme) = &latest.readme {
            format!(
                r#"<section class="readme">
                    <h3>README</h3>
                    <div class="readme-content">
                        <pre>{}</pre>
                    </div>
                </section>"#,
                html_escape(readme)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Build owners list
    let owners_html: Vec<String> = owners
        .iter()
        .map(|owner| format!(r#"<span class="owner-item">{}</span>"#, html_escape(owner)))
        .collect();

    let html = template
        .replace("{{PACKAGE_NAME}}", &html_escape(&package.name))
        .replace("{{DESCRIPTION}}", &html_escape(description))
        .replace("{{LATEST_VERSION}}", &html_escape(&latest_version))
        .replace("{{DOWNLOADS}}", &package.downloads.to_string())
        .replace("{{LICENSE}}", &html_escape(license))
        .replace("{{HOMEPAGE_LINK}}", &homepage_link)
        .replace("{{REPOSITORY_LINK}}", &repository_link)
        .replace("{{DOCUMENTATION_LINK}}", &documentation_link)
        .replace("{{KEYWORDS}}", &keywords)
        .replace("{{VERSIONS_LIST}}", &versions_html)
        .replace("{{DEPENDENCIES_LIST}}", &dependencies_html)
        .replace("{{README_SECTION}}", &readme_section)
        .replace("{{OWNERS_LIST}}", &owners_html.join(""));

    Ok(Html(html).into_response())
}

/// Serve static CSS file
pub async fn serve_css() -> impl IntoResponse {
    let css = include_str!("../../static/styles.css");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        css,
    )
}

/// Dashboard page with statistics
pub async fn dashboard(State(state): State<AppState>) -> ServerResult<Html<String>> {
    let template = include_str!("../../static/dashboard.html");

    // Get total packages count
    let total_packages_result = sqlx::query("SELECT COUNT(*) as count FROM packages")
        .fetch_one(&state.pool)
        .await;
    let total_packages: i64 = total_packages_result
        .map(|row| row.get("count"))
        .unwrap_or(0);

    // Get total downloads
    let total_downloads_result =
        sqlx::query("SELECT COALESCE(SUM(downloads), 0) as total FROM packages")
            .fetch_one(&state.pool)
            .await;
    let total_downloads: i64 = total_downloads_result
        .map(|row| row.get("total"))
        .unwrap_or(0);

    // Get recent packages (10 most recently updated)
    let recent_packages =
        db::search_packages_advanced(&state.pool, "", 10, 0, "newest", None, None)
            .await
            .unwrap_or((vec![], 0))
            .0;

    let recent_html = if recent_packages.is_empty() {
        "<p>No packages yet.</p>".to_string()
    } else {
        let mut html = String::new();
        for pkg in recent_packages {
            let description = pkg
                .description
                .as_deref()
                .unwrap_or("No description available.");
            html.push_str(&format!(
                r#"<div class="package-card">
                    <h3><a href="/packages/{}">{}</a></h3>
                    <p class="package-description">{}</p>
                    <div class="package-info">
                        <span>Version: {}</span>
                        <span>Downloads: {}</span>
                        <span>Updated: {}</span>
                    </div>
                </div>"#,
                html_escape(&pkg.name),
                html_escape(&pkg.name),
                html_escape(description),
                html_escape(&pkg.latest_version),
                pkg.downloads,
                pkg.updated_at.format("%Y-%m-%d")
            ));
        }
        html
    };

    // Get popular packages (10 most downloaded)
    let popular_packages =
        db::search_packages_advanced(&state.pool, "", 10, 0, "downloads", None, None)
            .await
            .unwrap_or((vec![], 0))
            .0;

    let popular_html = if popular_packages.is_empty() {
        "<p>No packages yet.</p>".to_string()
    } else {
        let mut html = String::new();
        for pkg in popular_packages {
            let description = pkg
                .description
                .as_deref()
                .unwrap_or("No description available.");
            html.push_str(&format!(
                r#"<div class="package-card">
                    <h3><a href="/packages/{}">{}</a></h3>
                    <p class="package-description">{}</p>
                    <div class="package-info">
                        <span>Version: {}</span>
                        <span>Downloads: {}</span>
                        <span>Updated: {}</span>
                    </div>
                </div>"#,
                html_escape(&pkg.name),
                html_escape(&pkg.name),
                html_escape(description),
                html_escape(&pkg.latest_version),
                pkg.downloads,
                pkg.updated_at.format("%Y-%m-%d")
            ));
        }
        html
    };

    let html = template
        .replace("{{TOTAL_PACKAGES}}", &total_packages.to_string())
        .replace("{{TOTAL_DOWNLOADS}}", &total_downloads.to_string())
        .replace("{{RECENT_PACKAGES}}", &recent_html)
        .replace("{{POPULAR_PACKAGES}}", &popular_html);

    Ok(Html(html))
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
