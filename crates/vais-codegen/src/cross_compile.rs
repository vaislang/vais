//! Cross-compilation support for Vais
//!
//! This module provides utilities for cross-compiling Vais code to various
//! target platforms including:
//!
//! - Linux (x86_64, aarch64, riscv64) with GNU or musl libc
//! - Windows (x86_64) with MSVC or MinGW
//! - macOS (x86_64, aarch64)
//! - iOS (aarch64) and iOS Simulator
//! - Android (aarch64, armv7)
//! - WebAssembly (wasm32, WASI preview1/2)

use crate::TargetTriple;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Cross-compilation configuration
#[derive(Debug, Clone)]
pub struct CrossCompileConfig {
    /// Target triple
    pub target: TargetTriple,
    /// Sysroot path (optional)
    pub sysroot: Option<PathBuf>,
    /// Additional include paths
    pub include_paths: Vec<PathBuf>,
    /// Additional library paths
    pub lib_paths: Vec<PathBuf>,
    /// Additional linker flags
    pub linker_flags: Vec<String>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

impl CrossCompileConfig {
    /// Create a new cross-compilation configuration for the given target
    pub fn new(target: TargetTriple) -> Self {
        Self {
            target,
            sysroot: None,
            include_paths: Vec::new(),
            lib_paths: Vec::new(),
            linker_flags: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    /// Try to auto-detect the SDK/toolchain for the target
    pub fn auto_detect(&mut self) -> Result<(), CrossCompileError> {
        match &self.target {
            TargetTriple::Aarch64Android | TargetTriple::Armv7Android => {
                self.detect_android_ndk()?;
            }
            TargetTriple::Aarch64Ios | TargetTriple::Aarch64IosSimulator => {
                self.detect_ios_sdk()?;
            }
            TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
                self.detect_wasi_sdk()?;
            }
            TargetTriple::X86_64WindowsMsvc => {
                self.detect_msvc()?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Detect Android NDK
    fn detect_android_ndk(&mut self) -> Result<(), CrossCompileError> {
        // Check ANDROID_NDK_HOME first
        if let Ok(ndk_home) = env::var("ANDROID_NDK_HOME") {
            let ndk_path = PathBuf::from(&ndk_home);
            if ndk_path.exists() {
                self.sysroot = Some(ndk_path.join("toolchains/llvm/prebuilt/darwin-x86_64/sysroot"));
                self.env_vars.insert("ANDROID_NDK_HOME".to_string(), ndk_home);
                return Ok(());
            }
        }

        // Check ANDROID_HOME/ndk
        if let Ok(android_home) = env::var("ANDROID_HOME") {
            let ndk_path = PathBuf::from(&android_home).join("ndk");
            if ndk_path.exists() {
                // Find latest NDK version
                if let Ok(entries) = std::fs::read_dir(&ndk_path) {
                    let latest = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .map(|e| e.path())
                        .max();

                    if let Some(ndk_ver) = latest {
                        self.sysroot = Some(ndk_ver.join("toolchains/llvm/prebuilt/darwin-x86_64/sysroot"));
                        self.env_vars.insert("ANDROID_NDK_HOME".to_string(), ndk_ver.to_string_lossy().to_string());
                        return Ok(());
                    }
                }
            }
        }

        Err(CrossCompileError::SdkNotFound {
            target: format!("{:?}", self.target),
            hint: "Set ANDROID_NDK_HOME or install Android NDK via Android Studio".to_string(),
        })
    }

    /// Detect iOS SDK
    fn detect_ios_sdk(&mut self) -> Result<(), CrossCompileError> {
        // Use xcrun to find the SDK
        let sdk_name = if matches!(self.target, TargetTriple::Aarch64IosSimulator) {
            "iphonesimulator"
        } else {
            "iphoneos"
        };

        // Try to run xcrun to get SDK path
        let output = std::process::Command::new("xcrun")
            .args(["--sdk", sdk_name, "--show-sdk-path"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                self.sysroot = Some(PathBuf::from(&path));
                Ok(())
            }
            _ => {
                Err(CrossCompileError::SdkNotFound {
                    target: format!("{:?}", self.target),
                    hint: "Install Xcode and iOS SDK".to_string(),
                })
            }
        }
    }

    /// Detect WASI SDK
    fn detect_wasi_sdk(&mut self) -> Result<(), CrossCompileError> {
        // Check WASI_SDK_PATH first
        if let Ok(wasi_sdk) = env::var("WASI_SDK_PATH") {
            let wasi_path = PathBuf::from(&wasi_sdk);
            if wasi_path.exists() {
                self.sysroot = Some(wasi_path.join("share/wasi-sysroot"));
                self.env_vars.insert("WASI_SDK_PATH".to_string(), wasi_sdk);
                return Ok(());
            }
        }

        // Check common installation locations
        let home_wasi = PathBuf::from(env::var("HOME").unwrap_or_default())
            .join("wasi-sdk")
            .to_string_lossy()
            .to_string();
        let common_paths: [&str; 3] = [
            "/opt/wasi-sdk",
            "/usr/local/wasi-sdk",
            &home_wasi,
        ];

        for path in common_paths {
            let wasi_path = PathBuf::from(path);
            if wasi_path.exists() {
                self.sysroot = Some(wasi_path.join("share/wasi-sysroot"));
                self.env_vars.insert("WASI_SDK_PATH".to_string(), path.to_string());
                return Ok(());
            }
        }

        Err(CrossCompileError::SdkNotFound {
            target: format!("{:?}", self.target),
            hint: "Set WASI_SDK_PATH or install WASI SDK from https://github.com/WebAssembly/wasi-sdk".to_string(),
        })
    }

    /// Detect MSVC toolchain
    fn detect_msvc(&mut self) -> Result<(), CrossCompileError> {
        // Check for Visual Studio installation via vswhere
        let vswhere_paths = [
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe",
            "C:\\Program Files\\Microsoft Visual Studio\\Installer\\vswhere.exe",
        ];

        for path in &vswhere_paths {
            if Path::new(path).exists() {
                let output = std::process::Command::new(path)
                    .args(["-latest", "-property", "installationPath"])
                    .output();

                if let Ok(out) = output {
                    if out.status.success() {
                        let vs_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        self.env_vars.insert("VS_PATH".to_string(), vs_path);
                        return Ok(());
                    }
                }
            }
        }

        // Check LIB and INCLUDE environment variables (set by vcvars)
        if env::var("LIB").is_ok() && env::var("INCLUDE").is_ok() {
            return Ok(());
        }

        Err(CrossCompileError::SdkNotFound {
            target: format!("{:?}", self.target),
            hint: "Install Visual Studio with C++ workload or run from Developer Command Prompt".to_string(),
        })
    }

    /// Get clang compiler command with all necessary flags
    pub fn clang_command(&self) -> Vec<String> {
        let mut cmd = vec!["clang".to_string()];

        // Target triple
        if !matches!(self.target, TargetTriple::Native) {
            cmd.push(format!("--target={}", self.target.triple_str()));
        }

        // Sysroot
        if let Some(sysroot) = &self.sysroot {
            cmd.push(format!("--sysroot={}", sysroot.display()));
        }

        // Include paths
        for path in &self.include_paths {
            cmd.push(format!("-I{}", path.display()));
        }

        // Library paths
        for path in &self.lib_paths {
            cmd.push(format!("-L{}", path.display()));
        }

        // Platform-specific flags
        cmd.extend(self.target.clang_flags().iter().map(|s| s.to_string()));

        // Additional linker flags
        for flag in &self.linker_flags {
            cmd.push(format!("-Wl,{}", flag));
        }

        cmd
    }

    /// Get linker command (platform-specific)
    pub fn linker_command(&self) -> Vec<String> {
        match &self.target {
            TargetTriple::X86_64WindowsMsvc => {
                vec!["lld-link".to_string()]
            }
            TargetTriple::Wasm32Unknown | TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
                vec!["wasm-ld".to_string()]
            }
            _ => {
                // Use clang as the linker driver for most targets
                self.clang_command()
            }
        }
    }
}

/// Error type for cross-compilation issues
#[derive(Debug, Clone)]
pub enum CrossCompileError {
    /// SDK or toolchain not found
    SdkNotFound { target: String, hint: String },
    /// Unsupported target
    UnsupportedTarget(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for CrossCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SdkNotFound { target, hint } => {
                write!(f, "SDK not found for target '{}'. {}", target, hint)
            }
            Self::UnsupportedTarget(t) => {
                write!(f, "Unsupported target: {}", t)
            }
            Self::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CrossCompileError {}

/// Platform-specific runtime library requirements
#[derive(Debug, Clone)]
pub struct RuntimeLibs {
    /// Libraries to link
    pub libs: Vec<String>,
    /// System libraries (e.g., libc)
    pub system_libs: Vec<String>,
}

impl RuntimeLibs {
    /// Get runtime library requirements for a target
    pub fn for_target(target: &TargetTriple) -> Self {
        match target {
            TargetTriple::X86_64Linux | TargetTriple::Aarch64Linux | TargetTriple::Riscv64Linux => {
                Self {
                    libs: vec![],
                    system_libs: vec!["c".to_string(), "m".to_string(), "pthread".to_string()],
                }
            }
            TargetTriple::X86_64LinuxMusl | TargetTriple::Aarch64LinuxMusl => {
                Self {
                    libs: vec![],
                    system_libs: vec!["c".to_string()],  // musl has most things in libc
                }
            }
            TargetTriple::X86_64WindowsMsvc => {
                Self {
                    libs: vec!["msvcrt".to_string()],
                    system_libs: vec!["kernel32".to_string(), "user32".to_string()],
                }
            }
            TargetTriple::X86_64WindowsGnu => {
                Self {
                    libs: vec!["mingw32".to_string()],
                    system_libs: vec!["kernel32".to_string()],
                }
            }
            TargetTriple::X86_64Darwin | TargetTriple::Aarch64Darwin => {
                Self {
                    libs: vec![],
                    system_libs: vec!["System".to_string()],
                }
            }
            TargetTriple::Aarch64Android | TargetTriple::Armv7Android => {
                Self {
                    libs: vec![],
                    system_libs: vec!["c".to_string(), "m".to_string(), "log".to_string()],
                }
            }
            TargetTriple::Aarch64Ios | TargetTriple::Aarch64IosSimulator => {
                Self {
                    libs: vec![],
                    system_libs: vec!["System".to_string()],
                }
            }
            TargetTriple::Wasm32Unknown => {
                Self {
                    libs: vec![],
                    system_libs: vec![],  // No system libs for bare wasm
                }
            }
            TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
                Self {
                    libs: vec!["wasi-emulated-mman".to_string()],
                    system_libs: vec!["c".to_string()],
                }
            }
            TargetTriple::Native => {
                // Detect at runtime
                Self {
                    libs: vec![],
                    system_libs: vec!["c".to_string(), "m".to_string()],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_compile_config_new() {
        let config = CrossCompileConfig::new(TargetTriple::Aarch64Linux);
        assert!(config.sysroot.is_none());
        assert!(config.include_paths.is_empty());
    }

    #[test]
    fn test_clang_command_native() {
        let config = CrossCompileConfig::new(TargetTriple::Native);
        let cmd = config.clang_command();
        assert_eq!(cmd[0], "clang");
        assert!(!cmd.iter().any(|s| s.starts_with("--target")));
    }

    #[test]
    fn test_clang_command_cross() {
        let config = CrossCompileConfig::new(TargetTriple::Aarch64Linux);
        let cmd = config.clang_command();
        assert!(cmd.iter().any(|s| s.contains("aarch64")));
    }

    #[test]
    fn test_runtime_libs_linux() {
        let libs = RuntimeLibs::for_target(&TargetTriple::X86_64Linux);
        assert!(libs.system_libs.contains(&"c".to_string()));
        assert!(libs.system_libs.contains(&"pthread".to_string()));
    }

    #[test]
    fn test_runtime_libs_wasm() {
        let libs = RuntimeLibs::for_target(&TargetTriple::Wasm32Unknown);
        assert!(libs.system_libs.is_empty());
    }

    #[test]
    fn test_runtime_libs_wasi() {
        let libs = RuntimeLibs::for_target(&TargetTriple::WasiPreview1);
        assert!(libs.system_libs.contains(&"c".to_string()));
    }

    #[test]
    fn test_all_targets_list() {
        let targets = TargetTriple::all_targets();
        assert!(targets.contains(&"native"));
        assert!(targets.contains(&"wasi"));
        assert!(targets.contains(&"aarch64-android"));
    }

    #[test]
    fn test_target_classification() {
        assert!(TargetTriple::Wasm32Unknown.is_wasm());
        assert!(TargetTriple::WasiPreview1.is_wasm());
        assert!(TargetTriple::X86_64WindowsMsvc.is_windows());
        assert!(TargetTriple::Aarch64Darwin.is_apple());
        assert!(TargetTriple::Aarch64Ios.is_ios());
        assert!(TargetTriple::Aarch64Android.is_android());
        assert!(TargetTriple::X86_64LinuxMusl.is_musl());
    }

    #[test]
    fn test_output_extension() {
        assert_eq!(TargetTriple::X86_64WindowsMsvc.output_extension(), "exe");
        assert_eq!(TargetTriple::Wasm32Unknown.output_extension(), "wasm");
        assert_eq!(TargetTriple::X86_64Linux.output_extension(), "");
    }

    #[test]
    fn test_pointer_bits() {
        assert_eq!(TargetTriple::X86_64Linux.pointer_bits(), 64);
        assert_eq!(TargetTriple::Aarch64Darwin.pointer_bits(), 64);
        assert_eq!(TargetTriple::Wasm32Unknown.pointer_bits(), 32);
        assert_eq!(TargetTriple::Armv7Android.pointer_bits(), 32);
    }
}
