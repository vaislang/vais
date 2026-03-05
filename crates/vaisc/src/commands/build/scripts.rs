//! Build script support for vais.build files
//!
//! Allows packages to define pre-build, post-build, and custom build steps
//! in their vais.toml manifest via `[build]` section:
//!
//!
//! ```toml
//! [build]
//! pre = "echo pre-build step"
//! post = "echo post-build step"
//! script = "scripts/build.sh"
//! ```

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

/// Build script configuration (parsed from vais.toml [build] section)
#[derive(Debug, Clone, Default)]
pub struct BuildScriptConfig {
    /// Shell command to run before compilation
    pub pre_build: Option<String>,
    /// Shell command to run after compilation
    pub post_build: Option<String>,
    /// Path to a build script file (relative to project root)
    pub script: Option<String>,
    /// Environment variables to set during build
    pub env: Vec<(String, String)>,
}

impl BuildScriptConfig {
    /// Parse from the [build] section of a vais.toml manifest
    pub fn from_toml(table: &toml::Table) -> Self {
        let mut config = Self::default();

        if let Some(pre) = table.get("pre").and_then(|v| v.as_str()) {
            config.pre_build = Some(pre.to_string());
        }
        if let Some(post) = table.get("post").and_then(|v| v.as_str()) {
            config.post_build = Some(post.to_string());
        }
        if let Some(script) = table.get("script").and_then(|v| v.as_str()) {
            config.script = Some(script.to_string());
        }
        if let Some(env_table) = table.get("env").and_then(|v| v.as_table()) {
            for (k, v) in env_table {
                if let Some(val) = v.as_str() {
                    config.env.push((k.clone(), val.to_string()));
                }
            }
        }

        config
    }

    /// Check if there are any build scripts to run
    pub fn has_scripts(&self) -> bool {
        self.pre_build.is_some() || self.post_build.is_some() || self.script.is_some()
    }
}

/// Run pre-build steps
///
/// # Security
/// Build scripts from vais.toml execute shell commands. A warning is printed
/// before execution so users are aware of what will run. Environment variables
/// are passed per-process (not via set_var) to avoid multi-thread unsoundness.
pub fn run_pre_build(
    config: &BuildScriptConfig,
    project_dir: &Path,
    verbose: bool,
) -> Result<(), String> {
    if config.has_scripts() && !config.env.is_empty() {
        eprintln!(
            "  Warning: build scripts will set environment variables: {:?}",
            config
                .env
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>()
        );
    }

    // Run pre-build command
    if let Some(ref cmd) = config.pre_build {
        if verbose {
            println!("  Running pre-build: {}", cmd);
        }
        run_shell_command(cmd, project_dir, &config.env)?;
    }

    // Run build script
    if let Some(ref script_path) = config.script {
        let full_path = project_dir.join(script_path);
        if !full_path.exists() {
            return Err(format!(
                "build script '{}' not found at '{}'",
                script_path,
                full_path.display()
            ));
        }

        if verbose {
            println!("  Running build script: {}", script_path);
        }

        // Determine how to run the script based on extension.
        // Use Command::arg() instead of string interpolation to prevent injection.
        let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "sh" | "bash" => run_script_command("bash", &full_path, project_dir, &config.env)?,
            "py" => run_script_command("python3", &full_path, project_dir, &config.env)?,
            "vais" => run_script_command("vaisc", &full_path, project_dir, &config.env)?,
            _ => {
                // Try to make it executable and run directly
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = std::fs::metadata(&full_path) {
                        let mut perms = meta.permissions();
                        perms.set_mode(perms.mode() | 0o111);
                        let _ = std::fs::set_permissions(&full_path, perms);
                    }
                }
                run_script_command(
                    full_path.to_str().unwrap_or(""),
                    &full_path,
                    project_dir,
                    &config.env,
                )?;
            }
        };
    }

    Ok(())
}

/// Run post-build steps
pub fn run_post_build(
    config: &BuildScriptConfig,
    project_dir: &Path,
    verbose: bool,
) -> Result<(), String> {
    if let Some(ref cmd) = config.post_build {
        if verbose {
            println!("  Running post-build: {}", cmd);
        }
        run_shell_command(cmd, project_dir, &config.env)?;
    }
    Ok(())
}

/// Execute a shell command in the given working directory.
///
/// Environment variables are passed per-process via `Command::env()` to avoid
/// the unsoundness of `std::env::set_var` in multi-threaded contexts.
fn run_shell_command(cmd: &str, cwd: &Path, env_vars: &[(String, String)]) -> Result<(), String> {
    // Warn about shell command execution from vais.toml
    eprintln!("  Note: executing build command from vais.toml: {}", cmd);

    let shell = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut command = Command::new(shell.0);
    command.arg(shell.1).arg(cmd).current_dir(cwd);
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let output = command
        .output()
        .map_err(|e| format!("failed to execute build command '{}': {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "build command '{}' failed with exit code {}: {}",
            cmd,
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    Ok(())
}

/// Execute a script file via an interpreter, passing the path as an argument.
///
/// Uses `Command::arg()` instead of string interpolation to prevent command injection.
fn run_script_command(
    interpreter: &str,
    script_path: &Path,
    cwd: &Path,
    env_vars: &[(String, String)],
) -> Result<(), String> {
    eprintln!(
        "  Note: executing build script from vais.toml: {} {}",
        interpreter,
        script_path.display()
    );

    let mut command = Command::new(interpreter);
    command.arg(script_path).current_dir(cwd);
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let output = command.output().map_err(|e| {
        format!(
            "failed to execute script '{}': {}",
            script_path.display(),
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "script '{}' failed with exit code {}: {}",
            script_path.display(),
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    Ok(())
}

/// Load build script config from a vais.toml file
pub fn load_build_config(project_dir: &Path) -> Option<BuildScriptConfig> {
    let manifest_path = project_dir.join("vais.toml");
    if !manifest_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    if let Some(build_section) = table.get("build").and_then(|v| v.as_table()) {
        let config = BuildScriptConfig::from_toml(build_section);
        if config.has_scripts() {
            return Some(config);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_build_config() {
        let config = BuildScriptConfig::default();
        assert!(config.pre_build.is_none());
        assert!(config.post_build.is_none());
        assert!(config.script.is_none());
        assert!(config.env.is_empty());
        assert!(!config.has_scripts());
    }

    #[test]
    fn test_parse_build_config() {
        let toml_str = r#"
pre = "echo building"
post = "echo done"
script = "scripts/build.sh"

[env]
CC = "clang"
CFLAGS = "-O2"
"#;
        let table: toml::Table = toml_str.parse().unwrap();
        let config = BuildScriptConfig::from_toml(&table);

        assert_eq!(config.pre_build.as_deref(), Some("echo building"));
        assert_eq!(config.post_build.as_deref(), Some("echo done"));
        assert_eq!(config.script.as_deref(), Some("scripts/build.sh"));
        assert_eq!(config.env.len(), 2);
        assert!(config.has_scripts());
    }

    #[test]
    fn test_parse_partial_config() {
        let toml_str = r#"
pre = "make deps"
"#;
        let table: toml::Table = toml_str.parse().unwrap();
        let config = BuildScriptConfig::from_toml(&table);

        assert_eq!(config.pre_build.as_deref(), Some("make deps"));
        assert!(config.post_build.is_none());
        assert!(config.script.is_none());
        assert!(config.has_scripts());
    }

    #[test]
    fn test_run_shell_command_success() {
        let cwd = std::env::temp_dir();
        let result = run_shell_command("echo hello", &cwd, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_shell_command_failure() {
        let cwd = std::env::temp_dir();
        let result = run_shell_command("exit 1", &cwd, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_build_config_missing_file() {
        let config = load_build_config(Path::new("/nonexistent/path"));
        assert!(config.is_none());
    }
}
