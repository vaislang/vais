//! Runtime library discovery and resolution.

use std::collections::HashSet;
use std::path::PathBuf;
use vais_ast::{Item, Module};

pub(crate) struct RuntimeInfo {
    /// The C runtime file name (e.g., "http_runtime.c")
    pub(crate) file: &'static str,
    /// Whether this runtime requires pthread linking
    pub(crate) needs_pthread: bool,
    /// Additional system libraries required (e.g., "-lssl", "-lcrypto")
    pub(crate) libs: &'static [&'static str],
}

/// Get the C runtime file info for a given module path.
/// Returns None if the module doesn't have a C runtime dependency.
pub(crate) fn get_runtime_for_module(module_path: &str) -> Option<RuntimeInfo> {
    match module_path {
        // Network modules
        "std::http" => Some(RuntimeInfo {
            file: "http_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),
        "std::http_server" => Some(RuntimeInfo {
            file: "http_server_runtime.c",
            needs_pthread: true,
            libs: &[],
        }),
        "std::http_client" => Some(RuntimeInfo {
            file: "http_client_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),
        "std::websocket" => Some(RuntimeInfo {
            file: "websocket_runtime.c",
            needs_pthread: true,
            libs: &[],
        }),
        "std::tls" => Some(RuntimeInfo {
            file: "tls_runtime.c",
            needs_pthread: false,
            libs: &["-lssl", "-lcrypto"],
        }),

        // Concurrency modules
        "std::thread" => Some(RuntimeInfo {
            file: "thread_runtime.c",
            needs_pthread: true,
            libs: &[],
        }),
        "std::sync" => Some(RuntimeInfo {
            file: "sync_runtime.c",
            needs_pthread: true,
            libs: &[],
        }),

        // Database modules
        "std::sqlite" => Some(RuntimeInfo {
            file: "sqlite_runtime.c",
            needs_pthread: false,
            libs: &["-lsqlite3"],
        }),
        "std::postgres" => Some(RuntimeInfo {
            file: "postgres_runtime.c",
            needs_pthread: false,
            libs: &["-lpq"],
        }),
        "std::orm" => Some(RuntimeInfo {
            file: "orm_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),

        // GPU modules
        "std::gpu" => Some(RuntimeInfo {
            file: "gpu_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),
        "std::opencl" => Some(RuntimeInfo {
            file: "opencl_runtime.c",
            needs_pthread: false,
            libs: &["-framework", "OpenCL"],
        }),

        // Utility modules
        "std::compress" => Some(RuntimeInfo {
            file: "compress_runtime.c",
            needs_pthread: false,
            libs: &["-lz"],
        }),
        "std::template" => Some(RuntimeInfo {
            file: "template_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),
        "std::log" => Some(RuntimeInfo {
            file: "log_runtime.c",
            needs_pthread: true,
            libs: &[],
        }),
        "std::contract" => Some(RuntimeInfo {
            file: "contract_runtime.c",
            needs_pthread: false,
            libs: &[],
        }),

        // Async I/O modules (platform-specific, selected at compile time)
        "std::async" => {
            #[cfg(target_os = "macos")]
            return Some(RuntimeInfo {
                file: "async_kqueue.c",
                needs_pthread: true,
                libs: &[],
            });
            #[cfg(target_os = "linux")]
            return Some(RuntimeInfo {
                file: "async_epoll.c",
                needs_pthread: true,
                libs: &[],
            });
            #[cfg(target_os = "windows")]
            return Some(RuntimeInfo {
                file: "async_iocp.c",
                needs_pthread: false,
                libs: &["-lws2_32"],
            });
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            None
        }

        _ => None,
    }
}

/// Extract all module paths used via `use` statements from the AST.
/// Returns a set of module paths like "std::http", "std::thread", etc.
/// Handles both "std/thread" (file path style) and "std::thread" (module path style).
pub(crate) fn extract_used_modules(ast: &Module) -> HashSet<String> {
    let mut modules = HashSet::new();

    for item in &ast.items {
        if let Item::Use(use_stmt) = &item.node {
            // Build module path from the use statement path
            let path_parts: Vec<&str> = use_stmt.path.iter().map(|s| s.node.as_str()).collect();

            if path_parts.is_empty() {
                continue;
            }

            // Check if first part contains "/" (file path style like "std/thread")
            let first = path_parts[0];
            if first.contains('/') {
                // File path style: "std/thread" or "std/http_server"
                let parts: Vec<&str> = first.split('/').collect();
                if parts.len() >= 2 && parts[0] == "std" {
                    // Normalize to std::module format
                    let module_path = format!("std::{}", parts[1]);
                    modules.insert(module_path);
                } else {
                    // Keep as-is but with :: separator
                    modules.insert(parts.join("::"));
                }
            } else if path_parts.len() >= 2 && first == "std" {
                // Module path style: std::http or std::thread::spawn
                // Use first two parts as the module identifier
                let module_path = format!("{}::{}", first, path_parts[1]);
                modules.insert(module_path);
            } else {
                // Non-std imports, use the full path
                modules.insert(path_parts.join("::"));
            }
        }
    }

    modules
}

/// Find a runtime C file in the std directory.
/// Searches: std/ relative to cwd, then next to compiler executable.
pub(crate) fn find_runtime_file(filename: &str) -> Option<PathBuf> {
    // Try std/ relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let rt_path = cwd.join("std").join(filename);
        if rt_path.exists() {
            return Some(rt_path);
        }
    }

    // Try next to the compiler executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check ../std/ relative to the binary
            if let Some(parent) = exe_dir.parent() {
                let rt_path = parent.join("std").join(filename);
                if rt_path.exists() {
                    return Some(rt_path);
                }
            }
        }
    }

    // Try VAIS_STD_DIR environment variable
    if let Ok(std_dir) = std::env::var("VAIS_STD_DIR") {
        let rt_path = PathBuf::from(&std_dir).join(filename);
        if rt_path.exists() {
            return Some(rt_path);
        }
    }

    None
}

/// Find the HTTP runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
pub(crate) fn find_http_runtime() -> Option<PathBuf> {
    // Try std/ relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let http_rt = cwd.join("std").join("http_runtime.c");
        if http_rt.exists() {
            return Some(http_rt);
        }
    }

    // Try next to the compiler executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check ../std/ relative to the binary
            if let Some(parent) = exe_dir.parent() {
                let http_rt = parent.join("std").join("http_runtime.c");
                if http_rt.exists() {
                    return Some(http_rt);
                }
            }
        }
    }

    // Try VAIS_HTTP_RUNTIME environment variable
    if let Ok(rt_path) = std::env::var("VAIS_HTTP_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the thread runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
pub(crate) fn find_thread_runtime() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let thread_rt = cwd.join("std").join("thread_runtime.c");
        if thread_rt.exists() {
            return Some(thread_rt);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Some(parent) = exe_dir.parent() {
                let thread_rt = parent.join("std").join("thread_runtime.c");
                if thread_rt.exists() {
                    return Some(thread_rt);
                }
            }
        }
    }

    if let Ok(rt_path) = std::env::var("VAIS_THREAD_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the sync runtime C source file for linking.
/// Searches: std/ relative to cwd, then next to compiler executable.
pub(crate) fn find_sync_runtime() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let sync_rt = cwd.join("std").join("sync_runtime.c");
        if sync_rt.exists() {
            return Some(sync_rt);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Some(parent) = exe_dir.parent() {
                let sync_rt = parent.join("std").join("sync_runtime.c");
                if sync_rt.exists() {
                    return Some(sync_rt);
                }
            }
        }
    }

    if let Ok(rt_path) = std::env::var("VAIS_SYNC_RUNTIME") {
        let path = PathBuf::from(&rt_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find the directory containing libvais_gc.a for GC runtime linking.
/// Searches: next to the compiler executable, then target/release/ in cwd.
pub(crate) fn find_gc_library() -> Option<PathBuf> {
    // Try next to the compiler executable (e.g. target/release/)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let gc_lib = exe_dir.join("libvais_gc.a");
            if gc_lib.exists() {
                return Some(exe_dir.to_path_buf());
            }
        }
    }

    // Try target/release/ relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let release_dir = cwd.join("target").join("release");
        if release_dir.join("libvais_gc.a").exists() {
            return Some(release_dir);
        }
        // Also try target/debug/
        let debug_dir = cwd.join("target").join("debug");
        if debug_dir.join("libvais_gc.a").exists() {
            return Some(debug_dir);
        }
    }

    // Try VAIS_GC_LIB_DIR environment variable
    if let Ok(gc_dir) = std::env::var("VAIS_GC_LIB_DIR") {
        let path = PathBuf::from(&gc_dir);
        if path.join("libvais_gc.a").exists() {
            return Some(path);
        }
    }

    None
}
