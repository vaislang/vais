//! Dynamic Vais module loader
//!
//! Provides runtime loading, unloading, and hot-reloading of Vais modules.

use notify::{Event, EventKind, RecommendedWatcher, Watcher};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::SystemTime;

use crate::error::{DynloadError, Result};

/// Type alias for reload callback functions
type ReloadCallback = Box<dyn Fn(ReloadEvent) + Send + Sync>;

/// Event types for module reload
#[derive(Debug, Clone)]
pub enum ReloadEvent {
    /// Module was reloaded successfully
    Reloaded { module_id: String, version: u64 },
    /// Module reload failed
    ReloadFailed { module_id: String, error: String },
    /// Module was unloaded
    Unloaded { module_id: String },
}

/// Handle to a loaded module
#[derive(Debug, Clone)]
pub struct ModuleHandle {
    /// Module identifier
    pub id: String,
    /// Module version (incremented on reload)
    pub version: u64,
    /// Source file path
    pub source_path: PathBuf,
    /// Compiled library path
    pub lib_path: PathBuf,
    /// Last modification time
    pub last_modified: SystemTime,
}

/// A dynamically loaded Vais module
pub struct LoadedModule {
    /// Module handle
    pub handle: ModuleHandle,
    /// The loaded library
    library: libloading::Library,
    /// Function cache
    function_cache: HashMap<String, *mut std::ffi::c_void>,
}

// SAFETY: LoadedModule is safe to Send between threads because:
// - The library field is Send
// - The function pointers are from the loaded library and remain valid
unsafe impl Send for LoadedModule {}

// SAFETY: LoadedModule is safe to share between threads because:
// - Access to mutable state (function_cache) is controlled by the Mutex wrapper
// - The library field is Sync
unsafe impl Sync for LoadedModule {}

impl LoadedModule {
    /// Get a function pointer from the module
    pub fn get_function<T>(&mut self, name: &str) -> Result<libloading::Symbol<'_, T>> {
        unsafe {
            self.library
                .get(name.as_bytes())
                .map_err(|_| DynloadError::SymbolNotFound(name.to_string()))
        }
    }

    /// Get a raw function pointer (cached)
    pub fn get_function_ptr(&mut self, name: &str) -> Result<*mut std::ffi::c_void> {
        if let Some(&ptr) = self.function_cache.get(name) {
            return Ok(ptr);
        }

        unsafe {
            let symbol: libloading::Symbol<*mut std::ffi::c_void> = self
                .library
                .get(name.as_bytes())
                .map_err(|_| DynloadError::SymbolNotFound(name.to_string()))?;
            let ptr = *symbol;
            self.function_cache.insert(name.to_string(), ptr);
            Ok(ptr)
        }
    }

    /// Call a function that takes no arguments and returns i64
    pub fn call_i64(&mut self, name: &str) -> Result<i64> {
        unsafe {
            let func: libloading::Symbol<extern "C" fn() -> i64> = self
                .library
                .get(name.as_bytes())
                .map_err(|_| DynloadError::SymbolNotFound(name.to_string()))?;
            Ok(func())
        }
    }

    /// Call a function that takes i64 and returns i64
    pub fn call_i64_i64(&mut self, name: &str, arg: i64) -> Result<i64> {
        unsafe {
            let func: libloading::Symbol<extern "C" fn(i64) -> i64> = self
                .library
                .get(name.as_bytes())
                .map_err(|_| DynloadError::SymbolNotFound(name.to_string()))?;
            Ok(func(arg))
        }
    }

    /// Call a function that takes two i64s and returns i64
    pub fn call_i64_i64_i64(&mut self, name: &str, arg1: i64, arg2: i64) -> Result<i64> {
        unsafe {
            let func: libloading::Symbol<extern "C" fn(i64, i64) -> i64> = self
                .library
                .get(name.as_bytes())
                .map_err(|_| DynloadError::SymbolNotFound(name.to_string()))?;
            Ok(func(arg1, arg2))
        }
    }
}

/// Configuration for the module loader
#[derive(Debug, Clone)]
pub struct ModuleLoaderConfig {
    /// Compiler command (default: "vaisc")
    pub compiler_command: String,
    /// Additional compiler arguments
    pub compiler_args: Vec<String>,
    /// Output directory for compiled libraries
    pub output_dir: Option<PathBuf>,
    /// Enable hot reload watching
    pub hot_reload: bool,
    /// Debounce duration for file changes (milliseconds)
    pub debounce_ms: u64,
    /// Maximum number of cached modules
    pub max_cache_size: usize,
}

impl Default for ModuleLoaderConfig {
    fn default() -> Self {
        Self {
            compiler_command: "vaisc".to_string(),
            compiler_args: vec![],
            output_dir: None,
            hot_reload: true,
            debounce_ms: 100,
            max_cache_size: 50,
        }
    }
}

impl ModuleLoaderConfig {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the compiler command
    pub fn with_compiler(mut self, cmd: &str) -> Self {
        self.compiler_command = cmd.to_string();
        self
    }

    /// Add compiler arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.compiler_args = args;
        self
    }

    /// Set output directory
    pub fn with_output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Enable/disable hot reload
    pub fn with_hot_reload(mut self, enable: bool) -> Self {
        self.hot_reload = enable;
        self
    }
}

/// Dynamic Vais module loader with hot reload support
pub struct ModuleLoader {
    /// Configuration
    config: ModuleLoaderConfig,
    /// Loaded modules by ID
    modules: RwLock<HashMap<String, Arc<Mutex<LoadedModule>>>>,
    /// File watcher for hot reload
    _watcher: Option<RecommendedWatcher>,
    /// Channel for file change events
    event_rx: Option<Receiver<notify::Result<Event>>>,
    /// Event sender for file changes.
    /// Kept alive to maintain the channel; dropping would close event_rx.
    #[allow(dead_code)]
    event_tx: Option<Sender<notify::Result<Event>>>,
    /// Callbacks for reload events
    reload_callbacks: Mutex<Vec<ReloadCallback>>,
    /// Module version counter
    version_counter: Mutex<u64>,
}

impl ModuleLoader {
    /// Create a new module loader with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ModuleLoaderConfig::default())
    }

    /// Create a new module loader with custom configuration
    pub fn with_config(config: ModuleLoaderConfig) -> Result<Self> {
        let (watcher, event_rx, event_tx) = if config.hot_reload {
            let (tx, rx) = channel();
            let tx_clone = tx.clone();
            let watcher = RecommendedWatcher::new(
                move |res| {
                    let _ = tx_clone.send(res);
                },
                notify::Config::default(),
            )
            .map_err(|e| DynloadError::InternalError(format!("Failed to create watcher: {}", e)))?;
            (Some(watcher), Some(rx), Some(tx))
        } else {
            (None, None, None)
        };

        Ok(Self {
            config,
            modules: RwLock::new(HashMap::new()),
            _watcher: watcher,
            event_rx,
            event_tx,
            reload_callbacks: Mutex::new(Vec::new()),
            version_counter: Mutex::new(0),
        })
    }

    /// Load a Vais module from source file
    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<ModuleHandle> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(DynloadError::ModuleNotFound(path.to_path_buf()));
        }

        let module_id = self.path_to_id(path);

        // Check if already loaded
        {
            let modules = self.modules.read();
            if modules.contains_key(&module_id) {
                return Err(DynloadError::ModuleAlreadyLoaded(module_id));
            }
        }

        // Compile the module
        let lib_path = self.compile_module(path)?;

        // Load the library
        let library = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| DynloadError::LibraryLoadError(e.to_string()))?
        };

        // Get file modification time
        let last_modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        // Create module handle
        let version = self.next_version();
        let handle = ModuleHandle {
            id: module_id.clone(),
            version,
            source_path: path.to_path_buf(),
            lib_path: lib_path.clone(),
            last_modified,
        };

        // Create loaded module
        let loaded = LoadedModule {
            handle: handle.clone(),
            library,
            function_cache: HashMap::new(),
        };

        // Store the module
        {
            let mut modules = self.modules.write();
            modules.insert(module_id.clone(), Arc::new(Mutex::new(loaded)));
        }

        // Note: File watching is set up when the loader is created
        // Individual file watching would require interior mutability
        // For now, we rely on the initial watcher setup

        Ok(handle)
    }

    /// Unload a module
    pub fn unload(&self, module_id: &str) -> Result<()> {
        let module = {
            let mut modules = self.modules.write();
            modules.remove(module_id)
        };

        if module.is_none() {
            return Err(DynloadError::ModuleNotLoaded(module_id.to_string()));
        }

        // Notify callbacks
        self.notify_callbacks(ReloadEvent::Unloaded {
            module_id: module_id.to_string(),
        });

        Ok(())
    }

    /// Reload a module
    pub fn reload(&self, module_id: &str) -> Result<ModuleHandle> {
        let source_path = {
            let modules = self.modules.read();
            let module = modules
                .get(module_id)
                .ok_or_else(|| DynloadError::ModuleNotLoaded(module_id.to_string()))?;
            let guard = module.lock();
            guard.handle.source_path.clone()
        };

        // Unload old module
        self.unload(module_id)?;

        // Load new module
        match self.load(&source_path) {
            Ok(handle) => {
                self.notify_callbacks(ReloadEvent::Reloaded {
                    module_id: module_id.to_string(),
                    version: handle.version,
                });
                Ok(handle)
            }
            Err(e) => {
                self.notify_callbacks(ReloadEvent::ReloadFailed {
                    module_id: module_id.to_string(),
                    error: e.to_string(),
                });
                Err(e)
            }
        }
    }

    /// Get a loaded module
    pub fn get(&self, module_id: &str) -> Option<Arc<Mutex<LoadedModule>>> {
        self.modules.read().get(module_id).cloned()
    }

    /// Check if a module is loaded
    pub fn is_loaded(&self, module_id: &str) -> bool {
        self.modules.read().contains_key(module_id)
    }

    /// List all loaded module IDs
    pub fn list_modules(&self) -> Vec<String> {
        self.modules.read().keys().cloned().collect()
    }

    /// Register a callback for reload events
    pub fn on_reload<F>(&self, callback: F)
    where
        F: Fn(ReloadEvent) + Send + Sync + 'static,
    {
        self.reload_callbacks.lock().push(Box::new(callback));
    }

    /// Check for file changes and reload if necessary (non-blocking)
    pub fn check_for_changes(&self) -> Result<Vec<ReloadEvent>> {
        let mut events = Vec::new();

        if let Some(ref rx) = self.event_rx {
            while let Ok(result) = rx.try_recv() {
                if let Ok(event) = result {
                    if let Some(reload_event) = self.handle_file_event(event)? {
                        events.push(reload_event);
                    }
                }
            }
        }

        Ok(events)
    }

    /// Poll for changes with timeout
    pub fn poll_changes(&self, timeout_ms: u64) -> Result<Option<ReloadEvent>> {
        if let Some(ref rx) = self.event_rx {
            use std::time::Duration;
            match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
                Ok(Ok(event)) => self.handle_file_event(event),
                Ok(Err(_)) => Ok(None),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn handle_file_event(&self, event: Event) -> Result<Option<ReloadEvent>> {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in event.paths {
                    let module_id = self.path_to_id(&path);
                    if self.is_loaded(&module_id) {
                        let handle = self.reload(&module_id)?;
                        return Ok(Some(ReloadEvent::Reloaded {
                            module_id,
                            version: handle.version,
                        }));
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    let module_id = self.path_to_id(&path);
                    if self.is_loaded(&module_id) {
                        self.unload(&module_id)?;
                        return Ok(Some(ReloadEvent::Unloaded { module_id }));
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn compile_module(&self, source_path: &Path) -> Result<PathBuf> {
        let output_dir = self
            .config
            .output_dir
            .clone()
            .unwrap_or_else(|| source_path.parent().unwrap_or(Path::new(".")).to_path_buf());

        let stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                DynloadError::CompilationError("Invalid source file name".to_string())
            })?;

        // Platform-specific extension
        #[cfg(target_os = "macos")]
        let ext = "dylib";
        #[cfg(target_os = "linux")]
        let ext = "so";
        #[cfg(target_os = "windows")]
        let ext = "dll";
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let ext = "so";

        let lib_name = format!("lib{}.{}", stem, ext);
        let lib_path = output_dir.join(lib_name);

        // Run compiler
        let mut cmd = Command::new(&self.config.compiler_command);
        cmd.arg("build")
            .arg("--dylib")
            .arg(source_path)
            .arg("-o")
            .arg(&lib_path);

        for arg in &self.config.compiler_args {
            cmd.arg(arg);
        }

        let output = cmd.output().map_err(|e| {
            DynloadError::CompilationError(format!("Failed to run compiler: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DynloadError::CompilationError(format!(
                "Compilation failed:\n{}",
                stderr
            )));
        }

        Ok(lib_path)
    }

    fn path_to_id(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn next_version(&self) -> u64 {
        let mut counter = self.version_counter.lock();
        *counter += 1;
        *counter
    }

    fn notify_callbacks(&self, event: ReloadEvent) {
        let callbacks = self.reload_callbacks.lock();
        for callback in callbacks.iter() {
            callback(event.clone());
        }
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create default module loader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_module_loader_config() {
        let config = ModuleLoaderConfig::new()
            .with_compiler("custom-vaisc")
            .with_hot_reload(false)
            .with_args(vec!["-O3".to_string()]);

        assert_eq!(config.compiler_command, "custom-vaisc");
        assert!(!config.hot_reload);
        assert_eq!(config.compiler_args, vec!["-O3".to_string()]);
    }

    #[test]
    fn test_path_to_id() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        assert_eq!(
            loader.path_to_id(Path::new("/path/to/module.vais")),
            "module"
        );
        assert_eq!(loader.path_to_id(Path::new("simple.vais")), "simple");
    }

    #[test]
    fn test_version_increment() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        assert_eq!(loader.next_version(), 1);
        assert_eq!(loader.next_version(), 2);
        assert_eq!(loader.next_version(), 3);
    }

    #[test]
    fn test_module_not_found() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        let result = loader.load("/nonexistent/module.vais");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::ModuleNotFound(_)
        ));
    }

    #[test]
    fn test_is_loaded() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        assert!(!loader.is_loaded("test_module"));
    }

    #[test]
    fn test_list_modules_empty() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        assert!(loader.list_modules().is_empty());
    }

    #[test]
    fn test_unload_not_loaded() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        let result = loader.unload("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::ModuleNotLoaded(_)
        ));
    }

    #[test]
    fn test_reload_callbacks() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        loader.on_reload(move |_event| {
            *called_clone.lock() = true;
        });

        // Manually trigger a callback
        loader.notify_callbacks(ReloadEvent::Unloaded {
            module_id: "test".to_string(),
        });

        assert!(*called.lock());
    }

    #[test]
    fn test_check_for_changes_no_events() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(true)).unwrap();

        let events = loader.check_for_changes().unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_module_loader_default_config() {
        let config = ModuleLoaderConfig::default();
        assert_eq!(config.compiler_command, "vaisc");
        assert!(config.compiler_args.is_empty());
        assert!(config.output_dir.is_none());
        assert!(config.hot_reload);
        assert_eq!(config.debounce_ms, 100);
        assert_eq!(config.max_cache_size, 50);
    }

    #[test]
    fn test_module_loader_config_with_output_dir() {
        let config = ModuleLoaderConfig::new().with_output_dir("/tmp/test_output");
        assert_eq!(
            config.output_dir,
            Some(PathBuf::from("/tmp/test_output"))
        );
    }

    #[test]
    fn test_module_loader_no_hot_reload() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();
        assert!(loader.list_modules().is_empty());
    }

    #[test]
    fn test_get_nonexistent_module() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();
        assert!(loader.get("nonexistent").is_none());
    }

    #[test]
    fn test_reload_not_loaded() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();
        let result = loader.reload("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DynloadError::ModuleNotLoaded(_)
        ));
    }

    #[test]
    fn test_poll_changes_no_hot_reload() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();
        let result = loader.poll_changes(10).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_poll_changes_with_timeout() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(true)).unwrap();
        // Should return None after timeout with no events
        let result = loader.poll_changes(1).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_path_to_id_various() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();
        assert_eq!(
            loader.path_to_id(Path::new("/a/b/c/test.vais")),
            "test"
        );
        assert_eq!(
            loader.path_to_id(Path::new("no_ext")),
            "no_ext"
        );
    }

    #[test]
    fn test_multiple_reload_callbacks() {
        let loader =
            ModuleLoader::with_config(ModuleLoaderConfig::new().with_hot_reload(false)).unwrap();

        let count = Arc::new(Mutex::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();

        loader.on_reload(move |_| {
            *c1.lock() += 1;
        });
        loader.on_reload(move |_| {
            *c2.lock() += 1;
        });

        loader.notify_callbacks(ReloadEvent::Unloaded {
            module_id: "test".to_string(),
        });

        assert_eq!(*count.lock(), 2);
    }

    #[test]
    fn test_reload_event_variants() {
        // Ensure all ReloadEvent variants can be constructed
        let _e1 = ReloadEvent::Reloaded {
            module_id: "m".to_string(),
            version: 1,
        };
        let _e2 = ReloadEvent::ReloadFailed {
            module_id: "m".to_string(),
            error: "err".to_string(),
        };
        let _e3 = ReloadEvent::Unloaded {
            module_id: "m".to_string(),
        };
    }

    #[test]
    fn test_module_handle_clone() {
        let handle = ModuleHandle {
            id: "test".to_string(),
            version: 1,
            source_path: PathBuf::from("/test.vais"),
            lib_path: PathBuf::from("/test.dylib"),
            last_modified: SystemTime::now(),
        };
        let cloned = handle.clone();
        assert_eq!(cloned.id, handle.id);
        assert_eq!(cloned.version, handle.version);
    }

    #[test]
    fn test_module_handle_debug() {
        let handle = ModuleHandle {
            id: "debug_test".to_string(),
            version: 3,
            source_path: PathBuf::from("/debug.vais"),
            lib_path: PathBuf::from("/debug.dylib"),
            last_modified: SystemTime::now(),
        };
        let debug = format!("{:?}", handle);
        assert!(debug.contains("debug_test"));
    }

    #[test]
    fn test_module_loader_config_default() {
        let config = ModuleLoaderConfig::default();
        assert_eq!(config.compiler_command, "vaisc");
        assert!(config.hot_reload);
        assert_eq!(config.max_cache_size, 50);
    }
}
