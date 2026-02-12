//! Database operations for the registry server

use crate::error::ServerResult;
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::path::Path;
use uuid::Uuid;

/// Database connection pool
pub type DbPool = Pool<Sqlite>;

/// Initialize the database with schema
pub async fn init_db(path: &Path) -> ServerResult<DbPool> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    // Run migrations
    create_schema(&pool).await?;

    Ok(pool)
}

async fn create_schema(pool: &DbPool) -> ServerResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            email TEXT,
            is_admin INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            token_hash TEXT NOT NULL,
            scopes TEXT NOT NULL,
            expires_at TEXT,
            last_used_at TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tokens_user ON api_tokens(user_id);
        CREATE INDEX IF NOT EXISTS idx_tokens_hash ON api_tokens(token_hash);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packages (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            homepage TEXT,
            repository TEXT,
            documentation TEXT,
            license TEXT,
            keywords TEXT NOT NULL DEFAULT '[]',
            categories TEXT NOT NULL DEFAULT '[]',
            owner_id TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            downloads INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
        CREATE INDEX IF NOT EXISTS idx_packages_owner ON packages(owner_id);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS package_versions (
            id TEXT PRIMARY KEY,
            package_id TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            version TEXT NOT NULL,
            checksum TEXT NOT NULL,
            size INTEGER NOT NULL,
            yanked INTEGER NOT NULL DEFAULT 0,
            readme TEXT,
            published_by TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL,
            downloads INTEGER NOT NULL DEFAULT 0,
            UNIQUE(package_id, version)
        );

        CREATE INDEX IF NOT EXISTS idx_versions_package ON package_versions(package_id);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dependencies (
            id TEXT PRIMARY KEY,
            version_id TEXT NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            version_req TEXT NOT NULL,
            features TEXT NOT NULL DEFAULT '[]',
            optional INTEGER NOT NULL DEFAULT 0,
            target TEXT,
            kind TEXT NOT NULL DEFAULT 'normal'
        );

        CREATE INDEX IF NOT EXISTS idx_deps_version ON dependencies(version_id);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS package_owners (
            package_id TEXT NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            added_by TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL,
            PRIMARY KEY (package_id, user_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Full-text search for packages
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS packages_fts USING fts5(
            name, description, keywords,
            content=packages,
            content_rowid=rowid
        );
        "#,
    )
    .execute(pool)
    .await
    .ok(); // Ignore if already exists with different schema

    // FTS5 synchronization triggers
    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS packages_fts_insert AFTER INSERT ON packages BEGIN
            INSERT INTO packages_fts(rowid, name, description, keywords)
            VALUES (new.rowid, new.name, new.description, new.keywords);
        END;
        "#,
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS packages_fts_update AFTER UPDATE ON packages BEGIN
            UPDATE packages_fts SET name = new.name, description = new.description, keywords = new.keywords
            WHERE rowid = new.rowid;
        END;
        "#,
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS packages_fts_delete AFTER DELETE ON packages BEGIN
            DELETE FROM packages_fts WHERE rowid = old.rowid;
        END;
        "#,
    )
    .execute(pool)
    .await
    .ok();

    Ok(())
}

// ==================== User Operations ====================

pub async fn create_user(pool: &DbPool, user: &User) -> ServerResult<()> {
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user.id.to_string())
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.email)
    .bind(user.is_admin)
    .bind(user.created_at.to_rfc3339())
    .bind(user.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_by_username(pool: &DbPool, username: &str) -> ServerResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, username, password_hash, email, is_admin, created_at, updated_at
        FROM users WHERE username = ?
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        email: r.get("email"),
        is_admin: r.get::<i32, _>("is_admin") != 0,
        created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
            .unwrap()
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
            .unwrap()
            .with_timezone(&Utc),
    }))
}

pub async fn get_user_by_id(pool: &DbPool, id: Uuid) -> ServerResult<Option<User>> {
    let row = sqlx::query(
        r#"
        SELECT id, username, password_hash, email, is_admin, created_at, updated_at
        FROM users WHERE id = ?
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
        username: r.get("username"),
        password_hash: r.get("password_hash"),
        email: r.get("email"),
        is_admin: r.get::<i32, _>("is_admin") != 0,
        created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
            .unwrap()
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
            .unwrap()
            .with_timezone(&Utc),
    }))
}

// ==================== Token Operations ====================

pub async fn create_token(pool: &DbPool, token: &ApiToken) -> ServerResult<()> {
    let scopes_json = serde_json::to_string(&token.scopes)?;

    sqlx::query(
        r#"
        INSERT INTO api_tokens (id, user_id, name, token_hash, scopes, expires_at, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(token.id.to_string())
    .bind(token.user_id.to_string())
    .bind(&token.name)
    .bind(&token.token_hash)
    .bind(&scopes_json)
    .bind(token.expires_at.map(|t| t.to_rfc3339()))
    .bind(token.created_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_token_by_hash(pool: &DbPool, hash: &str) -> ServerResult<Option<ApiToken>> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, name, token_hash, scopes, expires_at, last_used_at, created_at
        FROM api_tokens WHERE token_hash = ?
        "#,
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        let scopes: Vec<String> =
            serde_json::from_str(r.get::<&str, _>("scopes")).unwrap_or_default();

        ApiToken {
            id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
            user_id: Uuid::parse_str(r.get::<&str, _>("user_id")).unwrap(),
            name: r.get("name"),
            token_hash: r.get("token_hash"),
            scopes,
            expires_at: r
                .get::<Option<&str>, _>("expires_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|t| t.with_timezone(&Utc)),
            last_used_at: r
                .get::<Option<&str>, _>("last_used_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|t| t.with_timezone(&Utc)),
            created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
                .unwrap()
                .with_timezone(&Utc),
        }
    }))
}

pub async fn update_token_last_used(pool: &DbPool, token_id: Uuid) -> ServerResult<()> {
    sqlx::query("UPDATE api_tokens SET last_used_at = ? WHERE id = ?")
        .bind(Utc::now().to_rfc3339())
        .bind(token_id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_user_tokens(pool: &DbPool, user_id: Uuid) -> ServerResult<Vec<ApiToken>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, name, token_hash, scopes, expires_at, last_used_at, created_at
        FROM api_tokens WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let scopes: Vec<String> =
                serde_json::from_str(r.get::<&str, _>("scopes")).unwrap_or_default();

            ApiToken {
                id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
                user_id: Uuid::parse_str(r.get::<&str, _>("user_id")).unwrap(),
                name: r.get("name"),
                token_hash: "".to_string(), // Don't expose hash
                scopes,
                expires_at: r
                    .get::<Option<&str>, _>("expires_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|t| t.with_timezone(&Utc)),
                last_used_at: r
                    .get::<Option<&str>, _>("last_used_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|t| t.with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            }
        })
        .collect())
}

pub async fn delete_token(pool: &DbPool, token_id: Uuid, user_id: Uuid) -> ServerResult<bool> {
    let result = sqlx::query("DELETE FROM api_tokens WHERE id = ? AND user_id = ?")
        .bind(token_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ==================== Package Operations ====================

pub async fn create_package(pool: &DbPool, pkg: &Package) -> ServerResult<()> {
    let keywords_json = serde_json::to_string(&pkg.keywords)?;
    let categories_json = serde_json::to_string(&pkg.categories)?;

    sqlx::query(
        r#"
        INSERT INTO packages (id, name, description, homepage, repository, documentation,
                              license, keywords, categories, owner_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(pkg.id.to_string())
    .bind(&pkg.name)
    .bind(&pkg.description)
    .bind(&pkg.homepage)
    .bind(&pkg.repository)
    .bind(&pkg.documentation)
    .bind(&pkg.license)
    .bind(&keywords_json)
    .bind(&categories_json)
    .bind(pkg.owner_id.to_string())
    .bind(pkg.created_at.to_rfc3339())
    .bind(pkg.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    // Add owner
    sqlx::query(
        r#"
        INSERT INTO package_owners (package_id, user_id, added_by, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(pkg.id.to_string())
    .bind(pkg.owner_id.to_string())
    .bind(pkg.owner_id.to_string())
    .bind(pkg.created_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_package_by_name(pool: &DbPool, name: &str) -> ServerResult<Option<Package>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, description, homepage, repository, documentation, license,
               keywords, categories, owner_id, created_at, updated_at, downloads
        FROM packages WHERE name = ?
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Package {
        id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
        name: r.get("name"),
        description: r.get("description"),
        homepage: r.get("homepage"),
        repository: r.get("repository"),
        documentation: r.get("documentation"),
        license: r.get("license"),
        keywords: serde_json::from_str(r.get::<&str, _>("keywords")).unwrap_or_default(),
        categories: serde_json::from_str(r.get::<&str, _>("categories")).unwrap_or_default(),
        owner_id: Uuid::parse_str(r.get::<&str, _>("owner_id")).unwrap(),
        created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
            .unwrap()
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
            .unwrap()
            .with_timezone(&Utc),
        downloads: r.get("downloads"),
    }))
}

pub async fn update_package(pool: &DbPool, pkg: &Package) -> ServerResult<()> {
    let keywords_json = serde_json::to_string(&pkg.keywords)?;
    let categories_json = serde_json::to_string(&pkg.categories)?;

    sqlx::query(
        r#"
        UPDATE packages SET
            description = ?, homepage = ?, repository = ?, documentation = ?,
            license = ?, keywords = ?, categories = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&pkg.description)
    .bind(&pkg.homepage)
    .bind(&pkg.repository)
    .bind(&pkg.documentation)
    .bind(&pkg.license)
    .bind(&keywords_json)
    .bind(&categories_json)
    .bind(pkg.updated_at.to_rfc3339())
    .bind(pkg.id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_package_owner(
    pool: &DbPool,
    package_id: Uuid,
    user_id: Uuid,
) -> ServerResult<bool> {
    let row = sqlx::query("SELECT 1 FROM package_owners WHERE package_id = ? AND user_id = ?")
        .bind(package_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

pub async fn get_package_owners(pool: &DbPool, package_id: Uuid) -> ServerResult<Vec<String>> {
    let rows = sqlx::query(
        r#"
        SELECT u.username FROM package_owners po
        JOIN users u ON po.user_id = u.id
        WHERE po.package_id = ?
        "#,
    )
    .bind(package_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.get("username")).collect())
}

// ==================== Version Operations ====================

pub async fn create_version(pool: &DbPool, version: &PackageVersion) -> ServerResult<()> {
    sqlx::query(
        r#"
        INSERT INTO package_versions (id, package_id, version, checksum, size, yanked, readme, published_by, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(version.id.to_string())
    .bind(version.package_id.to_string())
    .bind(&version.version)
    .bind(&version.checksum)
    .bind(version.size)
    .bind(version.yanked)
    .bind(&version.readme)
    .bind(version.published_by.to_string())
    .bind(version.created_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_version(
    pool: &DbPool,
    package_id: Uuid,
    version: &str,
) -> ServerResult<Option<PackageVersion>> {
    let row = sqlx::query(
        r#"
        SELECT id, package_id, version, checksum, size, yanked, readme, published_by, created_at, downloads
        FROM package_versions WHERE package_id = ? AND version = ?
        "#,
    )
    .bind(package_id.to_string())
    .bind(version)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| PackageVersion {
        id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
        package_id: Uuid::parse_str(r.get::<&str, _>("package_id")).unwrap(),
        version: r.get("version"),
        checksum: r.get("checksum"),
        size: r.get("size"),
        yanked: r.get::<i32, _>("yanked") != 0,
        readme: r.get("readme"),
        published_by: Uuid::parse_str(r.get::<&str, _>("published_by")).unwrap(),
        created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
            .unwrap()
            .with_timezone(&Utc),
        downloads: r.get("downloads"),
    }))
}

pub async fn get_all_versions(
    pool: &DbPool,
    package_id: Uuid,
) -> ServerResult<Vec<PackageVersion>> {
    let rows = sqlx::query(
        r#"
        SELECT id, package_id, version, checksum, size, yanked, readme, published_by, created_at, downloads
        FROM package_versions WHERE package_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(package_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PackageVersion {
            id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
            package_id: Uuid::parse_str(r.get::<&str, _>("package_id")).unwrap(),
            version: r.get("version"),
            checksum: r.get("checksum"),
            size: r.get("size"),
            yanked: r.get::<i32, _>("yanked") != 0,
            readme: r.get("readme"),
            published_by: Uuid::parse_str(r.get::<&str, _>("published_by")).unwrap(),
            created_at: DateTime::parse_from_rfc3339(r.get("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            downloads: r.get("downloads"),
        })
        .collect())
}

pub async fn set_version_yanked(
    pool: &DbPool,
    package_id: Uuid,
    version: &str,
    yanked: bool,
) -> ServerResult<bool> {
    let result =
        sqlx::query("UPDATE package_versions SET yanked = ? WHERE package_id = ? AND version = ?")
            .bind(yanked)
            .bind(package_id.to_string())
            .bind(version)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn increment_download(pool: &DbPool, version_id: Uuid) -> ServerResult<()> {
    // Increment version downloads
    sqlx::query("UPDATE package_versions SET downloads = downloads + 1 WHERE id = ?")
        .bind(version_id.to_string())
        .execute(pool)
        .await?;

    // Also increment package downloads
    sqlx::query(
        r#"
        UPDATE packages SET downloads = downloads + 1
        WHERE id = (SELECT package_id FROM package_versions WHERE id = ?)
        "#,
    )
    .bind(version_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

// ==================== Dependency Operations ====================

pub async fn create_dependencies(pool: &DbPool, deps: &[Dependency]) -> ServerResult<()> {
    for dep in deps {
        let features_json = serde_json::to_string(&dep.features)?;
        let kind_str = match dep.kind {
            DependencyKind::Normal => "normal",
            DependencyKind::Dev => "dev",
            DependencyKind::Build => "build",
        };

        sqlx::query(
            r#"
            INSERT INTO dependencies (id, version_id, name, version_req, features, optional, target, kind)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(dep.id.to_string())
        .bind(dep.version_id.to_string())
        .bind(&dep.name)
        .bind(&dep.version_req)
        .bind(&features_json)
        .bind(dep.optional)
        .bind(&dep.target)
        .bind(kind_str)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_dependencies(pool: &DbPool, version_id: Uuid) -> ServerResult<Vec<Dependency>> {
    let rows = sqlx::query(
        r#"
        SELECT id, version_id, name, version_req, features, optional, target, kind
        FROM dependencies WHERE version_id = ?
        "#,
    )
    .bind(version_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let kind = match r.get::<&str, _>("kind") {
                "dev" => DependencyKind::Dev,
                "build" => DependencyKind::Build,
                _ => DependencyKind::Normal,
            };

            Dependency {
                id: Uuid::parse_str(r.get::<&str, _>("id")).unwrap(),
                version_id: Uuid::parse_str(r.get::<&str, _>("version_id")).unwrap(),
                name: r.get("name"),
                version_req: r.get("version_req"),
                features: serde_json::from_str(r.get::<&str, _>("features")).unwrap_or_default(),
                optional: r.get::<i32, _>("optional") != 0,
                target: r.get("target"),
                kind,
            }
        })
        .collect())
}

// ==================== Search Operations ====================

pub async fn search_packages(
    pool: &DbPool,
    query: &str,
    limit: usize,
    offset: usize,
) -> ServerResult<(Vec<PackageSearchEntry>, usize)> {
    search_packages_advanced(pool, query, limit, offset, "downloads", None, None).await
}

/// Advanced search with sorting, category, and keyword filtering
pub async fn search_packages_advanced(
    pool: &DbPool,
    query: &str,
    limit: usize,
    offset: usize,
    sort: &str,
    category: Option<&str>,
    keyword: Option<&str>,
) -> ServerResult<(Vec<PackageSearchEntry>, usize)> {
    let search_pattern = format!("%{}%", query.to_lowercase());

    // Build WHERE clause dynamically
    let mut where_clauses = vec![
        "(LOWER(p.name) LIKE ?1 OR LOWER(p.description) LIKE ?1 OR LOWER(p.keywords) LIKE ?1)"
            .to_string(),
    ];

    if let Some(cat) = category {
        where_clauses.push(format!(
            "LOWER(p.categories) LIKE '%{}%'",
            cat.to_lowercase().replace('\'', "''")
        ));
    }

    if let Some(kw) = keyword {
        where_clauses.push(format!(
            "LOWER(p.keywords) LIKE '%{}%'",
            kw.to_lowercase().replace('\'', "''")
        ));
    }

    let where_sql = where_clauses.join(" AND ");

    // Sort order
    let order_sql = match sort {
        "newest" => "p.updated_at DESC, p.name ASC",
        "name" => "p.name ASC",
        "relevance" => "CASE WHEN LOWER(p.name) = LOWER(?1) THEN 0 WHEN LOWER(p.name) LIKE ?1 THEN 1 ELSE 2 END, p.downloads DESC",
        _ => "p.downloads DESC, p.name ASC", // "downloads" default
    };

    // Get total count
    let count_sql = format!(
        "SELECT COUNT(*) as count FROM packages p WHERE {}",
        where_sql
    );
    let count_row = sqlx::query(&count_sql)
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

    let total: i64 = count_row.get("count");

    // Get packages
    let query_sql = format!(
        r#"SELECT p.name, p.description, p.keywords, p.categories, p.downloads, p.updated_at,
               (SELECT version FROM package_versions WHERE package_id = p.id AND yanked = 0
                ORDER BY created_at DESC LIMIT 1) as latest_version
        FROM packages p
        WHERE {}
        ORDER BY {}
        LIMIT ?2 OFFSET ?3"#,
        where_sql, order_sql
    );

    let rows = sqlx::query(&query_sql)
        .bind(&search_pattern)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

    let packages = rows
        .into_iter()
        .filter_map(|r| {
            let latest: Option<String> = r.get("latest_version");
            latest.map(|v| PackageSearchEntry {
                name: r.get("name"),
                description: r.get("description"),
                latest_version: v,
                downloads: r.get("downloads"),
                keywords: serde_json::from_str(r.get::<&str, _>("keywords")).unwrap_or_default(),
                categories: serde_json::from_str(r.get::<&str, _>("categories"))
                    .unwrap_or_default(),
                updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })
        .collect();

    Ok((packages, total as usize))
}

/// Get category counts for faceted search
pub async fn get_category_counts(pool: &DbPool) -> ServerResult<Vec<CategoryCount>> {
    // Since categories are stored as JSON arrays, we need to aggregate
    let rows = sqlx::query(
        r#"
        SELECT categories FROM packages WHERE categories != '[]' AND categories IS NOT NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for row in rows {
        let cats_str: &str = row.get("categories");
        if let Ok(cats) = serde_json::from_str::<Vec<String>>(cats_str) {
            for cat in cats {
                *counts.entry(cat).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<CategoryCount> = counts
        .into_iter()
        .map(|(name, count)| CategoryCount { name, count })
        .collect();
    result.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));

    Ok(result)
}

/// Get registry statistics
pub async fn get_registry_stats(pool: &DbPool) -> ServerResult<crate::models::RegistryStats> {
    // Get total packages
    let total_packages: i64 = sqlx::query("SELECT COUNT(*) as count FROM packages")
        .fetch_one(pool)
        .await?
        .get("count");

    // Get total downloads from packages table
    let total_downloads: i64 =
        sqlx::query("SELECT COALESCE(SUM(downloads), 0) as total FROM packages")
            .fetch_one(pool)
            .await?
            .get("total");

    // Get total versions
    let total_versions: i64 = sqlx::query("SELECT COUNT(*) as count FROM package_versions")
        .fetch_one(pool)
        .await?
        .get("count");

    Ok(crate::models::RegistryStats {
        total_packages,
        total_downloads,
        total_versions,
    })
}

/// Get recently updated packages
pub async fn get_recent_packages(
    pool: &DbPool,
    limit: usize,
) -> ServerResult<Vec<PackageSearchEntry>> {
    let rows = sqlx::query(
        r#"
        SELECT p.name, p.description, p.keywords, p.categories, p.downloads, p.updated_at,
               (SELECT version FROM package_versions WHERE package_id = p.id AND yanked = 0
                ORDER BY created_at DESC LIMIT 1) as latest_version
        FROM packages p
        ORDER BY p.updated_at DESC
        LIMIT ?
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let packages = rows
        .into_iter()
        .filter_map(|r| {
            let latest: Option<String> = r.get("latest_version");
            latest.map(|v| PackageSearchEntry {
                name: r.get("name"),
                description: r.get("description"),
                latest_version: v,
                downloads: r.get("downloads"),
                keywords: serde_json::from_str(r.get::<&str, _>("keywords")).unwrap_or_default(),
                categories: serde_json::from_str(r.get::<&str, _>("categories"))
                    .unwrap_or_default(),
                updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })
        .collect();

    Ok(packages)
}

/// Get popular packages by downloads
pub async fn get_popular_packages(
    pool: &DbPool,
    limit: usize,
) -> ServerResult<Vec<PackageSearchEntry>> {
    let rows = sqlx::query(
        r#"
        SELECT p.name, p.description, p.keywords, p.categories, p.downloads, p.updated_at,
               (SELECT version FROM package_versions WHERE package_id = p.id AND yanked = 0
                ORDER BY created_at DESC LIMIT 1) as latest_version
        FROM packages p
        ORDER BY p.downloads DESC, p.name ASC
        LIMIT ?
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    let packages = rows
        .into_iter()
        .filter_map(|r| {
            let latest: Option<String> = r.get("latest_version");
            latest.map(|v| PackageSearchEntry {
                name: r.get("name"),
                description: r.get("description"),
                latest_version: v,
                downloads: r.get("downloads"),
                keywords: serde_json::from_str(r.get::<&str, _>("keywords")).unwrap_or_default(),
                categories: serde_json::from_str(r.get::<&str, _>("categories"))
                    .unwrap_or_default(),
                updated_at: DateTime::parse_from_rfc3339(r.get("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })
        .collect();

    Ok(packages)
}

// ==================== Index Generation ====================

pub async fn get_full_index(pool: &DbPool) -> ServerResult<Vec<IndexEntry>> {
    let packages = sqlx::query(
        r#"
        SELECT id, name, description, homepage, repository, license, keywords
        FROM packages ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::new();

    for pkg_row in packages {
        let pkg_id: String = pkg_row.get("id");
        let pkg_uuid = Uuid::parse_str(&pkg_id).unwrap();

        let versions = get_all_versions(pool, pkg_uuid).await?;
        let owners = get_package_owners(pool, pkg_uuid).await?;

        let mut version_entries = Vec::new();
        for v in versions {
            let deps = get_dependencies(pool, v.id).await?;
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

        entries.push(IndexEntry {
            name: pkg_row.get("name"),
            description: pkg_row.get("description"),
            versions: version_entries,
            authors: owners,
            homepage: pkg_row.get("homepage"),
            repository: pkg_row.get("repository"),
            license: pkg_row.get("license"),
            keywords: serde_json::from_str(pkg_row.get::<&str, _>("keywords")).unwrap_or_default(),
        });
    }

    Ok(entries)
}
