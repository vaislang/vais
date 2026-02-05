//! High-level hot reload orchestration

use crate::dylib_loader::DylibLoader;
use crate::error::{HotReloadError, Result};
use crate::file_watcher::{FileWatcher, WatchEvent};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Callback function type for reload notifications
pub type ReloadCallback = Arc<dyn Fn(&Path, usize) + Send + Sync>;

/// Configuration for hot reloading
#[derive(Clone)]
pub struct HotReloadConfig {
    /// Source file to watch
    pub source_path: PathBuf,

    /// Output directory for compiled dylib (default: same as source)
    pub output_dir: Option<PathBuf>,

    /// Compiler command (default: "vaisc")
    pub compiler_command: String,

    /// Additional compiler arguments
    pub compiler_args: Vec<String>,

    /// Debounce duration in milliseconds (default: 100ms)
    pub debounce_ms: u64,

    /// Timeout for compilation in seconds (default: 30s)
    pub compile_timeout_secs: u64,

    /// Enable verbose output
    pub verbose: bool,
}

impl HotReloadConfig {
    /// Create a new configuration for the given source file
    pub fn new<P: AsRef<Path>>(source_path: P) -> Self {
        HotReloadConfig {
            source_path: source_path.as_ref().to_path_buf(),
            output_dir: None,
            compiler_command: "vaisc".to_string(),
            compiler_args: vec![],
            debounce_ms: 100,
            compile_timeout_secs: 30,
            verbose: false,
        }
    }

    /// Set the output directory
    pub fn with_output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set the compiler command
    pub fn with_compiler(mut self, command: String) -> Self {
        self.compiler_command = command;
        self
    }

    /// Add compiler arguments
    pub fn with_compiler_args(mut self, args: Vec<String>) -> Self {
        self.compiler_args = args;
        self
    }

    /// Set debounce duration
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Main hot reload coordinator
pub struct HotReloader {
    config: HotReloadConfig,
    file_watcher: FileWatcher,
    dylib_loader: Option<DylibLoader>,
    dylib_path: PathBuf,
    is_reloading: Arc<Mutex<bool>>,
    reload_callback: Option<ReloadCallback>,
}

impl HotReloader {
    /// Create a new hot reloader with the given configuration
    pub fn new(config: HotReloadConfig) -> Result<Self> {
        let file_watcher = FileWatcher::with_debounce(config.debounce_ms)?;

        // Determine dylib path
        let dylib_path = Self::determine_dylib_path(&config)?;

        Ok(HotReloader {
            config,
            file_watcher,
            dylib_loader: None,
            dylib_path,
            is_reloading: Arc::new(Mutex::new(false)),
            reload_callback: None,
        })
    }

    /// Start watching for file changes
    pub fn start(&mut self) -> Result<()> {
        // Watch the source file
        self.file_watcher.watch(&self.config.source_path)?;

        // Perform initial compilation if dylib doesn't exist
        if !self.dylib_path.exists() {
            if self.config.verbose {
                println!("[HotReload] Performing initial compilation...");
            }
            self.compile_source()?;
        }

        // Load the dylib
        let mut loader = DylibLoader::new(&self.dylib_path)?;
        loader.load()?;
        self.dylib_loader = Some(loader);

        if self.config.verbose {
            println!(
                "[HotReload] Started watching {}",
                self.config.source_path.display()
            );
        }

        Ok(())
    }

    /// Check for changes and reload if necessary (non-blocking)
    /// Returns true if code was reloaded
    pub fn check(&mut self) -> Result<bool> {
        if let Some(event) = self.file_watcher.check()? {
            match event {
                WatchEvent::Modified(path) | WatchEvent::Created(path) => {
                    if path == self.config.source_path {
                        return self.reload();
                    }
                }
                WatchEvent::Removed(_) => {
                    // Source file was removed, ignore
                }
            }
        }
        Ok(false)
    }

    /// Wait for a change and reload (blocking)
    pub fn wait_and_reload(&mut self) -> Result<()> {
        let event = self.file_watcher.wait()?;
        match event {
            WatchEvent::Modified(path) | WatchEvent::Created(path) => {
                if path == self.config.source_path {
                    self.reload()?;
                }
            }
            WatchEvent::Removed(_) => {
                // Source file was removed, ignore
            }
        }
        Ok(())
    }

    /// Get a function pointer from the loaded dylib
    pub fn get_function<T>(&mut self, name: &str) -> Result<libloading::Symbol<'_, T>> {
        self.dylib_loader
            .as_mut()
            .ok_or(HotReloadError::NotInitialized)?
            .get_function(name)
    }

    /// Get a raw function pointer (for FFI)
    pub fn get_function_ptr(&mut self, name: &str) -> Result<*mut std::ffi::c_void> {
        self.dylib_loader
            .as_mut()
            .ok_or(HotReloadError::NotInitialized)?
            .get_function_ptr(name)
    }

    /// Set a callback to be called when code is reloaded
    pub fn set_reload_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Path, usize) + Send + Sync + 'static,
    {
        self.reload_callback = Some(Arc::new(callback));
    }

    /// Get the current version number
    pub fn version(&self) -> usize {
        self.dylib_loader.as_ref().map(|l| l.version()).unwrap_or(0)
    }

    /// Manually trigger a reload
    pub fn reload(&mut self) -> Result<bool> {
        // Check if already reloading
        {
            let mut is_reloading = self
                .is_reloading
                .lock()
                .map_err(|_| HotReloadError::ReloadInProgress)?;
            if *is_reloading {
                return Err(HotReloadError::ReloadInProgress);
            }
            *is_reloading = true;
        }

        let result = self.perform_reload();

        // Clear reloading flag
        {
            let mut is_reloading = self
                .is_reloading
                .lock()
                .map_err(|_| HotReloadError::ReloadInProgress)?;
            *is_reloading = false;
        }

        result
    }

    fn perform_reload(&mut self) -> Result<bool> {
        if self.config.verbose {
            println!(
                "[HotReload] Detected change in {}",
                self.config.source_path.display()
            );
        }

        // Compile the source
        self.compile_source()?;

        // Reload the dylib
        if let Some(loader) = &mut self.dylib_loader {
            loader.load()?;

            if self.config.verbose {
                println!("[HotReload] Reloaded dylib (version {})", loader.version());
            }

            // Call reload callback if set
            if let Some(callback) = &self.reload_callback {
                callback(&self.dylib_path, loader.version());
            }

            // Clean up old versions
            loader.cleanup_old_versions()?;

            Ok(true)
        } else {
            Err(HotReloadError::NotInitialized)
        }
    }

    fn compile_source(&self) -> Result<()> {
        let mut cmd = Command::new(&self.config.compiler_command);

        // Add standard arguments for dylib compilation
        cmd.arg("build")
            .arg("--hot")
            .arg(&self.config.source_path)
            .arg("-o")
            .arg(&self.dylib_path);

        // Add custom compiler arguments
        for arg in &self.config.compiler_args {
            cmd.arg(arg);
        }

        if self.config.verbose {
            println!("[HotReload] Compiling: {:?}", cmd);
        }

        // Execute compilation
        let output = cmd.output().map_err(|e| {
            HotReloadError::CompilationError(format!("Failed to run compiler: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HotReloadError::CompilationError(format!(
                "Compilation failed:\n{}",
                stderr
            )));
        }

        Ok(())
    }

    fn determine_dylib_path(config: &HotReloadConfig) -> Result<PathBuf> {
        let output_dir = if let Some(ref dir) = config.output_dir {
            dir.clone()
        } else {
            config
                .source_path
                .parent()
                .ok_or_else(|| HotReloadError::InvalidPath("No parent directory".to_string()))?
                .to_path_buf()
        };

        let stem = config
            .source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| HotReloadError::InvalidPath("Invalid source file name".to_string()))?;

        // Platform-specific extension
        #[cfg(target_os = "macos")]
        let ext = "dylib";
        #[cfg(target_os = "linux")]
        let ext = "so";
        #[cfg(target_os = "windows")]
        let ext = "dll";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let ext = "so";

        let dylib_name = format!("lib{}.{}", stem, ext);
        Ok(output_dir.join(dylib_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = HotReloadConfig::new("test.vais");
        assert_eq!(config.source_path.to_str().unwrap(), "test.vais");
        assert_eq!(config.compiler_command, "vaisc");
        assert_eq!(config.debounce_ms, 100);
    }

    #[test]
    fn test_config_builder() {
        let config = HotReloadConfig::new("test.vais")
            .with_output_dir("/tmp")
            .with_compiler("custom-vaisc".to_string())
            .with_debounce(200)
            .with_verbose(true);

        assert_eq!(config.output_dir.unwrap().to_str().unwrap(), "/tmp");
        assert_eq!(config.compiler_command, "custom-vaisc");
        assert_eq!(config.debounce_ms, 200);
        assert!(config.verbose);
    }

    #[test]
    fn test_dylib_path_determination() {
        let config = HotReloadConfig::new("/tmp/test.vais");
        let path = HotReloader::determine_dylib_path(&config);
        assert!(path.is_ok());

        let path = path.unwrap();
        #[cfg(target_os = "macos")]
        assert!(path.to_str().unwrap().ends_with("libtest.dylib"));
        #[cfg(target_os = "linux")]
        assert!(path.to_str().unwrap().ends_with("libtest.so"));
    }
}
