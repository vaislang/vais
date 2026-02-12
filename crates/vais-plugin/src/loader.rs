//! Dynamic plugin loader
//!
//! Loads plugins from shared libraries (.so, .dylib, .dll)

use crate::traits::{
    AnalysisPlugin, CodegenPlugin, FormatterPlugin, LintPlugin, OptimizePlugin, Plugin, PluginType,
    TransformPlugin,
};
use libloading::{Library, Symbol};
use std::path::Path;

/// Type of the plugin creation function exported by plugins
#[allow(improper_ctypes_definitions)]
pub type CreatePluginFn = unsafe fn() -> *mut dyn Plugin;

/// Type of the plugin type query function exported by plugins
pub type GetPluginTypeFn = unsafe fn() -> PluginType;

/// Type-specific plugin creation functions
#[allow(improper_ctypes_definitions)]
pub type CreateLintPluginFn = unsafe fn() -> *mut dyn LintPlugin;
#[allow(improper_ctypes_definitions)]
pub type CreateTransformPluginFn = unsafe fn() -> *mut dyn TransformPlugin;
#[allow(improper_ctypes_definitions)]
pub type CreateOptimizePluginFn = unsafe fn() -> *mut dyn OptimizePlugin;
#[allow(improper_ctypes_definitions)]
pub type CreateCodegenPluginFn = unsafe fn() -> *mut dyn CodegenPlugin;
#[allow(improper_ctypes_definitions)]
pub type CreateFormatterPluginFn = unsafe fn() -> *mut dyn FormatterPlugin;
#[allow(improper_ctypes_definitions)]
pub type CreateAnalysisPluginFn = unsafe fn() -> *mut dyn AnalysisPlugin;

/// A loaded plugin with its library handle
pub struct LoadedPlugin {
    /// The base plugin instance (for info/init)
    pub plugin: Box<dyn Plugin>,
    /// Plugin type
    pub plugin_type: PluginType,
    /// Type-specific plugin interface
    plugin_impl: PluginImpl,
    /// Library handle (kept alive to prevent unloading).
    /// Must be retained for RAII - dropping this would unload the plugin's symbols.
    #[allow(dead_code)]
    library: Library,
}

/// Type-specific plugin implementations
enum PluginImpl {
    Lint(Box<dyn LintPlugin>),
    Transform(Box<dyn TransformPlugin>),
    Optimize(Box<dyn OptimizePlugin>),
    Codegen(Box<dyn CodegenPlugin>),
    Formatter(Box<dyn FormatterPlugin>),
    Analysis(Box<dyn AnalysisPlugin>),
}

impl LoadedPlugin {
    /// Try to cast to a LintPlugin
    pub fn as_lint(&self) -> Option<&dyn LintPlugin> {
        match &self.plugin_impl {
            PluginImpl::Lint(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    /// Try to cast to a TransformPlugin
    pub fn as_transform(&self) -> Option<&dyn TransformPlugin> {
        match &self.plugin_impl {
            PluginImpl::Transform(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    /// Try to cast to an OptimizePlugin
    pub fn as_optimize(&self) -> Option<&dyn OptimizePlugin> {
        match &self.plugin_impl {
            PluginImpl::Optimize(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    /// Try to cast to a CodegenPlugin
    pub fn as_codegen(&self) -> Option<&dyn CodegenPlugin> {
        match &self.plugin_impl {
            PluginImpl::Codegen(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    /// Try to cast to a FormatterPlugin
    pub fn as_formatter(&self) -> Option<&dyn FormatterPlugin> {
        match &self.plugin_impl {
            PluginImpl::Formatter(p) => Some(p.as_ref()),
            _ => None,
        }
    }

    /// Try to cast to an AnalysisPlugin
    pub fn as_analysis(&self) -> Option<&dyn AnalysisPlugin> {
        match &self.plugin_impl {
            PluginImpl::Analysis(p) => Some(p.as_ref()),
            _ => None,
        }
    }
}

/// Load a plugin from a shared library
///
/// The library must export:
/// - `create_plugin`: Creates and returns a base plugin instance
/// - `get_plugin_type`: Returns the type of plugin
/// - One of: `create_lint_plugin`, `create_transform_plugin`, `create_optimize_plugin`, `create_codegen_plugin`
///
/// # Example plugin export
///
/// ```text
/// #[no_mangle]
/// pub extern "C" fn create_plugin() -> *mut dyn Plugin {
///     Box::into_raw(Box::new(MyLintPlugin::new()))
/// }
///
/// #[no_mangle]
/// pub extern "C" fn get_plugin_type() -> PluginType {
///     PluginType::Lint
/// }
///
/// #[no_mangle]
/// pub extern "C" fn create_lint_plugin() -> *mut dyn LintPlugin {
///     Box::into_raw(Box::new(MyLintPlugin::new()))
/// }
/// ```
pub fn load_plugin(path: &Path, allow_plugins: bool) -> Result<LoadedPlugin, String> {
    // Check if plugin loading is allowed
    if !allow_plugins {
        return Err(format!(
            "Plugin loading is disabled. Use --allow-plugins to enable loading '{}'.\n\
             WARNING: Plugins execute arbitrary native code. Only load plugins you trust.",
            path.display()
        ));
    }

    // Validate file exists and has correct extension
    if !path.exists() {
        return Err(format!("Plugin file not found: '{}'", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Plugin path is not a file: '{}'", path.display()));
    }
    if !is_plugin_library(path) {
        let expected_ext = library_extension();
        return Err(format!(
            "Plugin '{}' does not have expected extension '.{}'",
            path.display(),
            expected_ext
        ));
    }

    eprintln!(
        "Warning: Loading plugin '{}'. Plugins run arbitrary native code.",
        path.display()
    );

    // Load the library
    // SAFETY: Loading a shared library is inherently unsafe as it executes
    // native code from the library's initialization routines. We validate
    // the file exists and has the correct extension above.
    let library = unsafe {
        Library::new(path)
            .map_err(|e| format!("Failed to load plugin '{}': {}", path.display(), e))?
    };

    // Get the plugin type
    // SAFETY: We call a known symbol from the loaded library. The function
    // signature is guaranteed by the plugin ABI contract.
    let plugin_type = unsafe {
        let get_type: Symbol<GetPluginTypeFn> = library
            .get(b"get_plugin_type")
            .map_err(|e| format!("Plugin missing get_plugin_type function: {}", e))?;
        get_type()
    };

    // Create the base plugin for info/init
    // SAFETY: We call create_plugin which returns a heap-allocated Plugin trait object.
    // We check for null before calling Box::from_raw. The caller is responsible for
    // ensuring the plugin library remains loaded while the Box is alive.
    let plugin = unsafe {
        let create: Symbol<CreatePluginFn> = library
            .get(b"create_plugin")
            .map_err(|e| format!("Plugin missing create_plugin function: {}", e))?;

        let raw = create();
        if raw.is_null() {
            return Err("Plugin create_plugin returned null".to_string());
        }
        Box::from_raw(raw)
    };

    // Create the type-specific plugin
    // SAFETY: Each branch below calls a type-specific creation function from the plugin.
    // The raw pointer is checked for null before being converted to a Box.
    // The library handle is kept alive in the LoadedPlugin struct to prevent unloading.
    let plugin_impl = match plugin_type {
        PluginType::Lint => unsafe {
            let create: Symbol<CreateLintPluginFn> = library
                .get(b"create_lint_plugin")
                .map_err(|e| format!("Lint plugin missing create_lint_plugin function: {}", e))?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_lint_plugin returned null".to_string());
            }
            PluginImpl::Lint(Box::from_raw(raw))
        },
        PluginType::Transform => unsafe {
            let create: Symbol<CreateTransformPluginFn> =
                library.get(b"create_transform_plugin").map_err(|e| {
                    format!(
                        "Transform plugin missing create_transform_plugin function: {}",
                        e
                    )
                })?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_transform_plugin returned null".to_string());
            }
            PluginImpl::Transform(Box::from_raw(raw))
        },
        PluginType::Optimize => unsafe {
            let create: Symbol<CreateOptimizePluginFn> =
                library.get(b"create_optimize_plugin").map_err(|e| {
                    format!(
                        "Optimize plugin missing create_optimize_plugin function: {}",
                        e
                    )
                })?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_optimize_plugin returned null".to_string());
            }
            PluginImpl::Optimize(Box::from_raw(raw))
        },
        PluginType::Codegen => unsafe {
            let create: Symbol<CreateCodegenPluginFn> =
                library.get(b"create_codegen_plugin").map_err(|e| {
                    format!(
                        "Codegen plugin missing create_codegen_plugin function: {}",
                        e
                    )
                })?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_codegen_plugin returned null".to_string());
            }
            PluginImpl::Codegen(Box::from_raw(raw))
        },
        PluginType::Formatter => unsafe {
            let create: Symbol<CreateFormatterPluginFn> =
                library.get(b"create_formatter_plugin").map_err(|e| {
                    format!(
                        "Formatter plugin missing create_formatter_plugin function: {}",
                        e
                    )
                })?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_formatter_plugin returned null".to_string());
            }
            PluginImpl::Formatter(Box::from_raw(raw))
        },
        PluginType::Analysis => unsafe {
            let create: Symbol<CreateAnalysisPluginFn> =
                library.get(b"create_analysis_plugin").map_err(|e| {
                    format!(
                        "Analysis plugin missing create_analysis_plugin function: {}",
                        e
                    )
                })?;
            let raw = create();
            if raw.is_null() {
                return Err("Plugin create_analysis_plugin returned null".to_string());
            }
            PluginImpl::Analysis(Box::from_raw(raw))
        },
    };

    Ok(LoadedPlugin {
        plugin,
        plugin_type,
        plugin_impl,
        library,
    })
}

/// Get the platform-specific library extension
pub fn library_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "dll"
    }
    #[cfg(target_os = "macos")]
    {
        "dylib"
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        "so"
    }
}

/// Check if a path looks like a plugin library
pub fn is_plugin_library(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext == library_extension())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_extension() {
        let ext = library_extension();
        #[cfg(target_os = "macos")]
        assert_eq!(ext, "dylib");
        #[cfg(target_os = "linux")]
        assert_eq!(ext, "so");
        #[cfg(target_os = "windows")]
        assert_eq!(ext, "dll");
    }

    #[test]
    fn test_is_plugin_library() {
        #[cfg(target_os = "macos")]
        {
            assert!(is_plugin_library(Path::new("plugin.dylib")));
            assert!(!is_plugin_library(Path::new("plugin.so")));
        }
    }
}
