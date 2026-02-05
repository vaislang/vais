//! Dynamic library loading and unloading

use crate::error::{HotReloadError, Result};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Type alias for function pointers
pub type FunctionSymbol = *mut std::ffi::c_void;

/// Manages loading and unloading of dynamic libraries
pub struct DylibLoader {
    current_lib: Option<Library>,
    dylib_path: PathBuf,
    version: usize,
    function_cache: HashMap<String, FunctionSymbol>,
}

impl DylibLoader {
    /// Create a new dylib loader for the specified path
    pub fn new<P: AsRef<Path>>(dylib_path: P) -> Result<Self> {
        let path = dylib_path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(HotReloadError::InvalidPath(format!(
                "Dylib not found: {}",
                path.display()
            )));
        }

        Ok(DylibLoader {
            current_lib: None,
            dylib_path: path,
            version: 0,
            function_cache: HashMap::new(),
        })
    }

    /// Load or reload the dynamic library
    pub fn load(&mut self) -> Result<()> {
        // Unload current library if loaded
        self.unload();

        // Create a versioned copy to allow reloading
        // This is necessary because some OSes lock loaded libraries
        let versioned_path = self.create_versioned_copy()?;

        // Load the library
        let lib = unsafe { Library::new(&versioned_path)? };

        self.current_lib = Some(lib);
        self.version += 1;
        self.function_cache.clear();

        Ok(())
    }

    /// Unload the current library
    pub fn unload(&mut self) {
        if let Some(lib) = self.current_lib.take() {
            // Library is automatically closed when dropped
            drop(lib);
        }
        self.function_cache.clear();
    }

    /// Get a function symbol from the loaded library
    pub fn get_function<T>(&mut self, name: &str) -> Result<Symbol<'_, T>> {
        let lib = self
            .current_lib
            .as_ref()
            .ok_or(HotReloadError::NotInitialized)?;

        unsafe {
            lib.get(name.as_bytes())
                .map_err(|_| HotReloadError::SymbolNotFound(name.to_string()))
        }
    }

    /// Get a raw function pointer (for FFI)
    pub fn get_function_ptr(&mut self, name: &str) -> Result<FunctionSymbol> {
        // Check cache first
        if let Some(&ptr) = self.function_cache.get(name) {
            return Ok(ptr);
        }

        let lib = self
            .current_lib
            .as_ref()
            .ok_or(HotReloadError::NotInitialized)?;

        unsafe {
            let symbol: Symbol<FunctionSymbol> = lib
                .get(name.as_bytes())
                .map_err(|_| HotReloadError::SymbolNotFound(name.to_string()))?;
            let ptr = *symbol;
            self.function_cache.insert(name.to_string(), ptr);
            Ok(ptr)
        }
    }

    /// Check if the library is currently loaded
    pub fn is_loaded(&self) -> bool {
        self.current_lib.is_some()
    }

    /// Get the current version number
    pub fn version(&self) -> usize {
        self.version
    }

    /// Get the dylib path
    pub fn path(&self) -> &Path {
        &self.dylib_path
    }

    /// Create a versioned copy of the dylib to allow reloading
    fn create_versioned_copy(&self) -> Result<PathBuf> {
        let original = &self.dylib_path;
        let parent = original
            .parent()
            .ok_or_else(|| HotReloadError::InvalidPath("No parent directory".to_string()))?;

        let stem = original
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| HotReloadError::InvalidPath("Invalid file name".to_string()))?;

        let ext = original.extension().and_then(OsStr::to_str).unwrap_or("");

        let versioned_name = if ext.is_empty() {
            format!("{}.v{}", stem, self.version)
        } else {
            format!("{}.v{}.{}", stem, self.version, ext)
        };

        let versioned_path = parent.join(versioned_name);

        // Copy the file
        std::fs::copy(original, &versioned_path)?;

        Ok(versioned_path)
    }

    /// Clean up old versioned copies
    pub fn cleanup_old_versions(&self) -> Result<()> {
        let parent = self
            .dylib_path
            .parent()
            .ok_or_else(|| HotReloadError::InvalidPath("No parent directory".to_string()))?;

        let stem = self
            .dylib_path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| HotReloadError::InvalidPath("Invalid file name".to_string()))?;

        // Remove versioned files older than current version
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(OsStr::to_str) {
                    // Match pattern: stem.v{version}.*
                    if name.starts_with(&format!("{}.v", stem)) {
                        // Extract version number
                        if let Some(version_str) = name.strip_prefix(&format!("{}.v", stem)) {
                            if let Some(version_end) =
                                version_str.find('.').or(Some(version_str.len()))
                            {
                                if let Ok(version) = version_str[..version_end].parse::<usize>() {
                                    // Remove if older than current version
                                    if version < self.version {
                                        let _ = std::fs::remove_file(&path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for DylibLoader {
    fn drop(&mut self) {
        self.unload();
        // Try to clean up versioned copies
        let _ = self.cleanup_old_versions();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dylib_loader_creation() {
        // Test with non-existent path
        let result = DylibLoader::new("/nonexistent/lib.dylib");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_increment() {
        // This test requires a real dylib, so we just test the version logic
        let mut loader = DylibLoader {
            current_lib: None,
            dylib_path: PathBuf::from("test.dylib"),
            version: 0,
            function_cache: HashMap::new(),
        };

        assert_eq!(loader.version(), 0);
        loader.version += 1;
        assert_eq!(loader.version(), 1);
    }

    #[test]
    fn test_is_loaded() {
        let loader = DylibLoader {
            current_lib: None,
            dylib_path: PathBuf::from("test.dylib"),
            version: 0,
            function_cache: HashMap::new(),
        };

        assert!(!loader.is_loaded());
    }
}
