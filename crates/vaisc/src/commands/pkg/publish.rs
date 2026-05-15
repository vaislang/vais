//! Publishing and registry authentication commands.

use crate::package;
use crate::registry;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Publish a package to the registry
pub(super) fn cmd_pkg_publish(
    cwd: &Path,
    registry: Option<String>,
    token: Option<String>,
    dry_run: bool,
    verbose: bool,
) -> Result<(), String> {
    use package::{find_manifest, load_manifest};

    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    // Find and load manifest
    let pkg_dir = find_manifest(cwd).ok_or_else(|| "could not find vais.toml".to_string())?;
    let manifest = load_manifest(&pkg_dir).map_err(|e| e.to_string())?;

    let pkg_name = &manifest.package.name;
    let pkg_version = &manifest.package.version;

    println!(
        "{} Packaging {} v{}...",
        "Info".cyan(),
        pkg_name,
        pkg_version
    );

    // Pack the package into a temporary archive
    let tmp_dir = std::env::temp_dir().join(format!("vais-publish-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("failed to create temp directory: {}", e))?;
    let archive_path = tmp_dir.join(format!("{}-{}.tar.gz", pkg_name, pkg_version));

    registry::pack_package(&pkg_dir, &archive_path)
        .map_err(|e| format!("failed to pack package: {}", e))?;

    // Read archive and compute checksum
    let archive_bytes =
        fs::read(&archive_path).map_err(|e| format!("failed to read archive: {}", e))?;
    let checksum = registry::sha256_hex(&archive_bytes);

    if verbose {
        println!(
            "  Archive size: {} bytes, checksum: {}",
            archive_bytes.len(),
            &checksum[..16]
        );
    }

    // Build metadata JSON
    let deps: serde_json::Map<String, serde_json::Value> = manifest
        .dependencies
        .iter()
        .map(|(name, dep)| {
            let version_str = match dep {
                package::Dependency::Version(v) => v.clone(),
                package::Dependency::Detailed(d) => {
                    d.version.clone().unwrap_or_else(|| "*".to_string())
                }
            };
            (name.clone(), serde_json::Value::String(version_str))
        })
        .collect();

    let metadata = serde_json::json!({
        "name": pkg_name,
        "version": pkg_version,
        "description": manifest.package.description.as_deref().unwrap_or(""),
        "authors": manifest.package.authors,
        "license": manifest.package.license.as_deref().unwrap_or(""),
        "checksum": checksum,
        "dependencies": deps,
    });

    if dry_run {
        println!("{} Dry run - would publish:", "Info".cyan());
        println!("  Name: {}", pkg_name);
        println!("  Version: {}", pkg_version);
        println!("  Checksum: {}", checksum);
        println!("  Archive size: {} bytes", archive_bytes.len());
        // Clean up
        let _ = fs::remove_dir_all(&tmp_dir);
        println!("{} Dry run complete, package is valid", "✓".green());
        return Ok(());
    }

    // Resolve token: argument > credentials file > error
    let auth_token = token
        .or_else(|| load_credentials_token(&registry_url))
        .ok_or_else(|| {
            "authentication token required. Use --token or run `vaisc pkg login` first".to_string()
        })?;

    // Build multipart body
    let metadata_str = serde_json::to_string(&metadata)
        .map_err(|e| format!("failed to serialize metadata: {}", e))?;

    let boundary = format!("----vais-publish-{}", std::process::id());
    let mut body = Vec::new();

    // metadata part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json\r\n\r\n",
    );
    body.extend_from_slice(metadata_str.as_bytes());
    body.extend_from_slice(b"\r\n");

    // archive part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"archive\"; filename=\"{}-{}.tar.gz\"\r\nContent-Type: application/gzip\r\n\r\n",
            pkg_name, pkg_version
        )
        .as_bytes(),
    );
    body.extend_from_slice(&archive_bytes);
    body.extend_from_slice(b"\r\n");

    // closing boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    println!(
        "{} Publishing {} v{} to {}...",
        "Info".cyan(),
        pkg_name,
        pkg_version,
        registry_url
    );

    let publish_url = format!("{}/packages/publish", registry_url.trim_end_matches('/'));
    let response = ureq::post(&publish_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .send_bytes(&body)
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("publish failed (HTTP {}): {}", code, msg)
            }
            _ => format!("publish failed: {}", e),
        })?;

    let status = response.status();
    if verbose {
        println!("  Server responded with status {}", status);
    }

    // Verify checksum by fetching package metadata from registry
    if verbose {
        println!("{} Verifying checksum...", "Info".cyan());
    }

    let verify_url = format!(
        "{}/packages/{}/{}",
        registry_url.trim_end_matches('/'),
        pkg_name,
        pkg_version
    );

    let verify_response = ureq::get(&verify_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .call();

    match verify_response {
        Ok(resp) => {
            if let Ok(body) = resp.into_string() {
                if let Ok(pkg_info) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(server_checksum) = pkg_info.get("checksum").and_then(|c| c.as_str())
                    {
                        if server_checksum == checksum {
                            if verbose {
                                println!("  Checksum verified: {}", &checksum[..16]);
                            }
                        } else {
                            eprintln!(
                                "{} Warning: checksum mismatch (local: {}, server: {})",
                                "⚠".yellow(),
                                &checksum[..16],
                                &server_checksum[..16]
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("{} Could not verify checksum: {}", "⚠".yellow(), e);
            }
        }
    }

    // Clean up temp files
    let _ = fs::remove_dir_all(&tmp_dir);

    println!(
        "{} Published {} v{} to {}",
        "✓".green(),
        pkg_name,
        pkg_version,
        registry_url
    );
    Ok(())
}

/// Yank a published package version from the registry
pub(super) fn cmd_pkg_yank(
    name: &str,
    version: &str,
    token: Option<String>,
    registry: Option<String>,
    verbose: bool,
) -> Result<(), String> {
    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    let auth_token = token
        .or_else(|| load_credentials_token(&registry_url))
        .ok_or_else(|| {
            "authentication token required. Use --token or run `vaisc pkg login` first".to_string()
        })?;

    let yank_url = format!(
        "{}/packages/{}/{}/yank",
        registry_url.trim_end_matches('/'),
        name,
        version
    );

    if verbose {
        println!(
            "{} Yanking {}@{} from {}",
            "Info".cyan(),
            name,
            version,
            registry_url
        );
    }

    ureq::post(&yank_url)
        .set("Authorization", &format!("Bearer {}", auth_token))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("yank failed (HTTP {}): {}", code, msg)
            }
            _ => format!("yank failed: {}", e),
        })?;

    println!(
        "{} Yanked {}@{} from {}",
        "✓".green(),
        name,
        version,
        registry_url
    );
    Ok(())
}

/// Login to a package registry and store credentials
pub(super) fn cmd_pkg_login(registry: Option<String>, verbose: bool) -> Result<(), String> {
    let registry_url = registry.unwrap_or_else(|| "https://registry.vais.dev".to_string());

    println!("{} Logging in to {}", "Info".cyan(), registry_url);

    // Prompt for username
    eprint!("Username: ");
    let mut username = String::new();
    std::io::stdin()
        .read_line(&mut username)
        .map_err(|e| format!("failed to read username: {}", e))?;
    let username = username.trim().to_string();

    if username.is_empty() {
        return Err("username cannot be empty".to_string());
    }

    // Prompt for password
    eprint!("Password: ");
    let mut password = String::new();
    std::io::stdin()
        .read_line(&mut password)
        .map_err(|e| format!("failed to read password: {}", e))?;
    let password = password.trim().to_string();

    if password.is_empty() {
        return Err("password cannot be empty".to_string());
    }

    let login_url = format!("{}/auth/login", registry_url.trim_end_matches('/'));

    if verbose {
        println!("  Authenticating as {}...", username);
    }

    let response = ureq::post(&login_url)
        .send_json(serde_json::json!({
            "username": username,
            "password": password,
        }))
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                format!("login failed (HTTP {}): {}", code, msg)
            }
            _ => format!("login failed: {}", e),
        })?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("failed to parse login response: {}", e))?;

    let token = body
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "login response did not contain a token".to_string())?
        .to_string();

    // Save token to ~/.vais/credentials.toml
    let home = dirs::home_dir().ok_or_else(|| "could not determine home directory".to_string())?;
    let vais_dir = home.join(".vais");
    fs::create_dir_all(&vais_dir).map_err(|e| format!("failed to create ~/.vais: {}", e))?;

    let creds_path = vais_dir.join("credentials.toml");

    // Load existing credentials or start fresh
    let mut creds: toml::Value = if creds_path.exists() {
        let content = fs::read_to_string(&creds_path)
            .map_err(|e| format!("failed to read credentials: {}", e))?;
        content
            .parse()
            .unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Store token under the registry URL key
    if let Some(table) = creds.as_table_mut() {
        let mut registry_table = toml::map::Map::new();
        registry_table.insert("token".to_string(), toml::Value::String(token));
        table.insert(registry_url.clone(), toml::Value::Table(registry_table));
    }

    let creds_content = toml::to_string_pretty(&creds)
        .map_err(|e| format!("failed to serialize credentials: {}", e))?;
    fs::write(&creds_path, creds_content)
        .map_err(|e| format!("failed to write credentials: {}", e))?;

    println!(
        "{} Logged in to {} as {}",
        "✓".green(),
        registry_url,
        username
    );
    println!("  Token saved to {}", creds_path.display());
    Ok(())
}

/// Load authentication token from ~/.vais/credentials.toml for a given registry
pub(super) fn load_credentials_token(registry_url: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let creds_path = home.join(".vais").join("credentials.toml");
    let content = fs::read_to_string(&creds_path).ok()?;
    let creds: toml::Value = content.parse().ok()?;
    creds
        .get(registry_url)?
        .get("token")?
        .as_str()
        .map(|s| s.to_string())
}
