//! Target architecture definitions for code generation
//!
//! Defines the TargetTriple enum and related methods for cross-compilation support.

/// Target architecture for code generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetTriple {
    /// Native target (auto-detect)
    Native,
    /// x86-64 Linux (GNU libc)
    X86_64Linux,
    /// x86-64 Linux (musl libc - static linking)
    X86_64LinuxMusl,
    /// x86-64 Windows (MSVC)
    X86_64WindowsMsvc,
    /// x86-64 Windows (GNU/MinGW)
    X86_64WindowsGnu,
    /// x86-64 macOS
    X86_64Darwin,
    /// ARM64 (Apple Silicon / macOS)
    Aarch64Darwin,
    /// ARM64 Linux (GNU libc)
    Aarch64Linux,
    /// ARM64 Linux (musl libc)
    Aarch64LinuxMusl,
    /// ARM64 Android
    Aarch64Android,
    /// ARM64 iOS
    Aarch64Ios,
    /// ARM64 iOS Simulator
    Aarch64IosSimulator,
    /// ARM32 Android (ARMv7)
    Armv7Android,
    /// WebAssembly 32-bit (no WASI)
    Wasm32Unknown,
    /// WebAssembly 32-bit with WASI (preview1)
    WasiPreview1,
    /// WebAssembly 32-bit with WASI (preview2)
    WasiPreview2,
    /// RISC-V 64-bit Linux (GNU libc)
    Riscv64LinuxGnu,
    /// ARM64 Windows (MSVC)
    Aarch64WindowsMsvc,
    /// x86-64 FreeBSD
    X86_64FreeBsd,
    /// ARM64 FreeBSD
    Aarch64FreeBsd,
}

impl TargetTriple {
    /// Parse a target triple string
    pub fn parse(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            // Native
            "native" | "auto" => Some(Self::Native),

            // x86-64 Linux
            "x86_64-linux" | "x86_64-unknown-linux-gnu" | "x86_64-pc-linux-gnu" => {
                Some(Self::X86_64Linux)
            }
            "x86_64-linux-musl" | "x86_64-unknown-linux-musl" => Some(Self::X86_64LinuxMusl),

            // x86-64 Windows
            "x86_64-windows-msvc" | "x86_64-pc-windows-msvc" => Some(Self::X86_64WindowsMsvc),
            "x86_64-windows-gnu" | "x86_64-pc-windows-gnu" | "x86_64-w64-mingw32" => {
                Some(Self::X86_64WindowsGnu)
            }

            // x86-64 macOS
            "x86_64-darwin" | "x86_64-apple-darwin" => Some(Self::X86_64Darwin),

            // ARM64 macOS
            "aarch64" | "aarch64-darwin" | "aarch64-apple-darwin" | "arm64" | "arm64-darwin" => {
                Some(Self::Aarch64Darwin)
            }

            // ARM64 Linux
            "aarch64-linux" | "aarch64-unknown-linux-gnu" => Some(Self::Aarch64Linux),
            "aarch64-linux-musl" | "aarch64-unknown-linux-musl" => Some(Self::Aarch64LinuxMusl),

            // ARM64 Android
            "aarch64-android" | "aarch64-linux-android" => Some(Self::Aarch64Android),

            // ARM64 iOS
            "aarch64-ios" | "aarch64-apple-ios" => Some(Self::Aarch64Ios),
            "aarch64-ios-sim" | "aarch64-apple-ios-sim" => Some(Self::Aarch64IosSimulator),

            // ARM32 Android
            "armv7-android" | "armv7-linux-androideabi" | "arm-android" => Some(Self::Armv7Android),

            // WebAssembly
            "wasm32" | "wasm32-unknown-unknown" => Some(Self::Wasm32Unknown),
            "wasi" | "wasm32-wasi" | "wasi-preview1" => Some(Self::WasiPreview1),
            "wasi-preview2" | "wasm32-wasip2" => Some(Self::WasiPreview2),

            // RISC-V
            "riscv64" | "riscv64-linux" | "riscv64gc-unknown-linux-gnu" => {
                Some(Self::Riscv64LinuxGnu)
            }

            // Windows ARM64
            "aarch64-windows-msvc" | "aarch64-pc-windows-msvc" => Some(Self::Aarch64WindowsMsvc),

            // FreeBSD
            "x86_64-freebsd" | "x86_64-unknown-freebsd" => Some(Self::X86_64FreeBsd),
            "aarch64-freebsd" | "aarch64-unknown-freebsd" => Some(Self::Aarch64FreeBsd),

            _ => None,
        }
    }

    /// Get the LLVM target triple string
    pub fn triple_str(&self) -> &'static str {
        match self {
            Self::Native => "", // Let clang auto-detect
            Self::X86_64Linux => "x86_64-unknown-linux-gnu",
            Self::X86_64LinuxMusl => "x86_64-unknown-linux-musl",
            Self::X86_64WindowsMsvc => "x86_64-pc-windows-msvc",
            Self::X86_64WindowsGnu => "x86_64-pc-windows-gnu",
            Self::X86_64Darwin => "x86_64-apple-darwin",
            Self::Aarch64Darwin => "aarch64-apple-darwin",
            Self::Aarch64Linux => "aarch64-unknown-linux-gnu",
            Self::Aarch64LinuxMusl => "aarch64-unknown-linux-musl",
            Self::Aarch64Android => "aarch64-linux-android",
            Self::Aarch64Ios => "aarch64-apple-ios",
            Self::Aarch64IosSimulator => "aarch64-apple-ios-simulator",
            Self::Armv7Android => "armv7-linux-androideabi",
            Self::Wasm32Unknown => "wasm32-unknown-unknown",
            Self::WasiPreview1 => "wasm32-wasi",
            Self::WasiPreview2 => "wasm32-wasip2",
            Self::Riscv64LinuxGnu => "riscv64gc-unknown-linux-gnu",
            Self::Aarch64WindowsMsvc => "aarch64-pc-windows-msvc",
            Self::X86_64FreeBsd => "x86_64-unknown-freebsd",
            Self::Aarch64FreeBsd => "aarch64-unknown-freebsd",
        }
    }

    /// Get the LLVM data layout for this target
    pub fn data_layout(&self) -> &'static str {
        match self {
            Self::Native => "", // Let clang auto-detect

            // x86-64 (all platforms have same layout)
            Self::X86_64Linux
            | Self::X86_64LinuxMusl
            | Self::X86_64WindowsMsvc
            | Self::X86_64WindowsGnu
            | Self::X86_64Darwin => {
                "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            }

            // ARM64 (macOS uses different mangling)
            Self::Aarch64Darwin | Self::Aarch64Ios | Self::Aarch64IosSimulator => {
                "e-m:o-i64:64-i128:128-n32:64-S128"
            }
            Self::Aarch64Linux | Self::Aarch64LinuxMusl | Self::Aarch64Android => {
                "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
            }

            // ARM32 Android
            Self::Armv7Android => "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64",

            // WebAssembly (32-bit pointers)
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2 => {
                "e-m:e-p:32:32-i64:64-n32:64-S128"
            }

            // RISC-V 64
            Self::Riscv64LinuxGnu => "e-m:e-p:64:64-i64:64-i128:128-n64-S128",

            // Windows ARM64 (uses MSVC-specific mangling)
            Self::Aarch64WindowsMsvc => "e-m:w-p:64:64-i32:32-i64:64-i128:128-n32:64-S128",

            // FreeBSD x86-64 (same as Linux x86-64)
            Self::X86_64FreeBsd => {
                "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            }

            // FreeBSD ARM64 (same as Linux ARM64)
            Self::Aarch64FreeBsd => "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
        }
    }

    /// Check if this is a WebAssembly target
    pub fn is_wasm(&self) -> bool {
        matches!(
            self,
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2
        )
    }

    /// Check if this is a Windows target
    pub fn is_windows(&self) -> bool {
        matches!(
            self,
            Self::X86_64WindowsMsvc | Self::X86_64WindowsGnu | Self::Aarch64WindowsMsvc
        )
    }

    /// Check if this is an Apple platform (macOS, iOS)
    pub fn is_apple(&self) -> bool {
        matches!(
            self,
            Self::X86_64Darwin | Self::Aarch64Darwin | Self::Aarch64Ios | Self::Aarch64IosSimulator
        )
    }

    /// Check if this is an Android target
    pub fn is_android(&self) -> bool {
        matches!(self, Self::Aarch64Android | Self::Armv7Android)
    }

    /// Check if this is an iOS target
    pub fn is_ios(&self) -> bool {
        matches!(self, Self::Aarch64Ios | Self::Aarch64IosSimulator)
    }

    /// Check if this uses musl libc
    pub fn is_musl(&self) -> bool {
        matches!(self, Self::X86_64LinuxMusl | Self::Aarch64LinuxMusl)
    }

    /// Check if this is a FreeBSD target
    pub fn is_freebsd(&self) -> bool {
        matches!(self, Self::X86_64FreeBsd | Self::Aarch64FreeBsd)
    }

    /// Check if this is a Linux target
    pub fn is_linux(&self) -> bool {
        matches!(
            self,
            Self::X86_64Linux
                | Self::X86_64LinuxMusl
                | Self::Aarch64Linux
                | Self::Aarch64LinuxMusl
                | Self::Riscv64LinuxGnu
        )
    }

    /// Get the target OS name for cfg matching
    pub fn target_os(&self) -> &'static str {
        match self {
            Self::Native => {
                if cfg!(target_os = "macos") {
                    "macos"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else if cfg!(target_os = "windows") {
                    "windows"
                } else if cfg!(target_os = "freebsd") {
                    "freebsd"
                } else {
                    "unknown"
                }
            }
            Self::X86_64Darwin | Self::Aarch64Darwin => "macos",
            Self::Aarch64Ios | Self::Aarch64IosSimulator => "ios",
            Self::X86_64Linux
            | Self::X86_64LinuxMusl
            | Self::Aarch64Linux
            | Self::Aarch64LinuxMusl
            | Self::Riscv64LinuxGnu => "linux",
            Self::X86_64WindowsMsvc | Self::X86_64WindowsGnu | Self::Aarch64WindowsMsvc => {
                "windows"
            }
            Self::Aarch64Android | Self::Armv7Android => "android",
            Self::X86_64FreeBsd | Self::Aarch64FreeBsd => "freebsd",
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2 => "wasm",
        }
    }

    /// Get the target architecture name for cfg matching
    pub fn target_arch(&self) -> &'static str {
        match self {
            Self::Native => {
                if cfg!(target_arch = "x86_64") {
                    "x86_64"
                } else if cfg!(target_arch = "aarch64") {
                    "aarch64"
                } else {
                    "unknown"
                }
            }
            Self::X86_64Linux
            | Self::X86_64LinuxMusl
            | Self::X86_64WindowsMsvc
            | Self::X86_64WindowsGnu
            | Self::X86_64Darwin
            | Self::X86_64FreeBsd => "x86_64",
            Self::Aarch64Darwin
            | Self::Aarch64Linux
            | Self::Aarch64LinuxMusl
            | Self::Aarch64Android
            | Self::Aarch64Ios
            | Self::Aarch64IosSimulator
            | Self::Aarch64WindowsMsvc
            | Self::Aarch64FreeBsd => "aarch64",
            Self::Armv7Android => "arm",
            Self::Riscv64LinuxGnu => "riscv64",
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2 => "wasm32",
        }
    }

    /// Build a HashMap of cfg key-value pairs for this target
    pub fn cfg_values(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("target_os".to_string(), self.target_os().to_string());
        map.insert("target_arch".to_string(), self.target_arch().to_string());
        if self.is_wasm() {
            map.insert("target_family".to_string(), "wasm".to_string());
        } else if self.is_windows() {
            map.insert("target_family".to_string(), "windows".to_string());
        } else {
            map.insert("target_family".to_string(), "unix".to_string());
        }
        map
    }

    /// Get pointer size in bits for this target
    pub fn pointer_bits(&self) -> usize {
        match self {
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2 | Self::Armv7Android => {
                32
            }
            _ => 64,
        }
    }

    /// Get clang flags for this target
    pub fn clang_flags(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();

        // Target triple
        if !matches!(self, Self::Native) {
            flags.push("--target");
        }

        // Platform-specific flags
        match self {
            Self::Native => {}

            // musl requires static linking
            Self::X86_64LinuxMusl | Self::Aarch64LinuxMusl => {
                flags.push("-static");
            }

            // Windows MSVC (x86-64 and ARM64)
            Self::X86_64WindowsMsvc | Self::Aarch64WindowsMsvc => {
                flags.push("-fms-extensions");
                flags.push("-fms-compatibility");
            }

            // Windows GNU/MinGW
            Self::X86_64WindowsGnu => {
                flags.push("-mconsole");
            }

            // Android NDK
            Self::Aarch64Android | Self::Armv7Android => {
                // Requires ANDROID_NDK_HOME to be set
                flags.push("-fPIC");
            }

            // iOS
            Self::Aarch64Ios => {
                flags.push("-mios-version-min=12.0");
            }
            Self::Aarch64IosSimulator => {
                flags.push("-mios-simulator-version-min=12.0");
            }

            // WebAssembly
            Self::Wasm32Unknown => {
                flags.push("--no-standard-libraries");
                flags.push("-Wl,--no-entry");
                flags.push("-Wl,--export-all");
            }
            Self::WasiPreview1 | Self::WasiPreview2 => {
                // WASI SDK should be installed
            }

            _ => {}
        }

        flags
    }

    /// Get the default output file extension for this target
    pub fn output_extension(&self) -> &'static str {
        match self {
            Self::X86_64WindowsMsvc | Self::X86_64WindowsGnu | Self::Aarch64WindowsMsvc => "exe",
            Self::Wasm32Unknown | Self::WasiPreview1 | Self::WasiPreview2 => "wasm",
            Self::Aarch64Ios | Self::Aarch64IosSimulator => "", // no extension for iOS
            _ => "", // no extension for Unix-like systems
        }
    }

    /// Get all supported targets as a list
    pub fn all_targets() -> &'static [&'static str] {
        &[
            "native",
            "x86_64-linux",
            "x86_64-linux-musl",
            "x86_64-windows-msvc",
            "x86_64-windows-gnu",
            "x86_64-darwin",
            "x86_64-freebsd",
            "aarch64-darwin",
            "aarch64-linux",
            "aarch64-linux-musl",
            "aarch64-android",
            "aarch64-ios",
            "aarch64-ios-sim",
            "aarch64-windows-msvc",
            "aarch64-freebsd",
            "armv7-android",
            "wasm32",
            "wasi",
            "wasi-preview2",
            "riscv64-linux",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== parse ==========

    #[test]
    fn test_parse_native() {
        assert_eq!(TargetTriple::parse("native"), Some(TargetTriple::Native));
        assert_eq!(TargetTriple::parse("auto"), Some(TargetTriple::Native));
    }

    #[test]
    fn test_parse_x86_64_linux() {
        assert_eq!(
            TargetTriple::parse("x86_64-linux"),
            Some(TargetTriple::X86_64Linux)
        );
        assert_eq!(
            TargetTriple::parse("x86_64-unknown-linux-gnu"),
            Some(TargetTriple::X86_64Linux)
        );
    }

    #[test]
    fn test_parse_x86_64_linux_musl() {
        assert_eq!(
            TargetTriple::parse("x86_64-linux-musl"),
            Some(TargetTriple::X86_64LinuxMusl)
        );
    }

    #[test]
    fn test_parse_x86_64_windows() {
        assert_eq!(
            TargetTriple::parse("x86_64-windows-msvc"),
            Some(TargetTriple::X86_64WindowsMsvc)
        );
        assert_eq!(
            TargetTriple::parse("x86_64-windows-gnu"),
            Some(TargetTriple::X86_64WindowsGnu)
        );
    }

    #[test]
    fn test_parse_aarch64_darwin() {
        assert_eq!(
            TargetTriple::parse("aarch64-darwin"),
            Some(TargetTriple::Aarch64Darwin)
        );
        assert_eq!(
            TargetTriple::parse("aarch64-apple-darwin"),
            Some(TargetTriple::Aarch64Darwin)
        );
        assert_eq!(
            TargetTriple::parse("arm64"),
            Some(TargetTriple::Aarch64Darwin)
        );
    }

    #[test]
    fn test_parse_wasm() {
        assert_eq!(
            TargetTriple::parse("wasm32"),
            Some(TargetTriple::Wasm32Unknown)
        );
        assert_eq!(
            TargetTriple::parse("wasm32-unknown-unknown"),
            Some(TargetTriple::Wasm32Unknown)
        );
        assert_eq!(
            TargetTriple::parse("wasi"),
            Some(TargetTriple::WasiPreview1)
        );
        assert_eq!(
            TargetTriple::parse("wasi-preview2"),
            Some(TargetTriple::WasiPreview2)
        );
    }

    #[test]
    fn test_parse_riscv64() {
        assert_eq!(
            TargetTriple::parse("riscv64"),
            Some(TargetTriple::Riscv64LinuxGnu)
        );
    }

    #[test]
    fn test_parse_freebsd() {
        assert_eq!(
            TargetTriple::parse("x86_64-freebsd"),
            Some(TargetTriple::X86_64FreeBsd)
        );
        assert_eq!(
            TargetTriple::parse("aarch64-freebsd"),
            Some(TargetTriple::Aarch64FreeBsd)
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(
            TargetTriple::parse("NATIVE"),
            Some(TargetTriple::Native)
        );
        assert_eq!(
            TargetTriple::parse("X86_64-Linux"),
            Some(TargetTriple::X86_64Linux)
        );
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(TargetTriple::parse("unknown-target"), None);
        assert_eq!(TargetTriple::parse(""), None);
    }

    #[test]
    fn test_parse_android() {
        assert_eq!(
            TargetTriple::parse("aarch64-android"),
            Some(TargetTriple::Aarch64Android)
        );
        assert_eq!(
            TargetTriple::parse("armv7-android"),
            Some(TargetTriple::Armv7Android)
        );
    }

    #[test]
    fn test_parse_ios() {
        assert_eq!(
            TargetTriple::parse("aarch64-ios"),
            Some(TargetTriple::Aarch64Ios)
        );
        assert_eq!(
            TargetTriple::parse("aarch64-ios-sim"),
            Some(TargetTriple::Aarch64IosSimulator)
        );
    }

    // ========== triple_str ==========

    #[test]
    fn test_triple_str() {
        assert_eq!(TargetTriple::X86_64Linux.triple_str(), "x86_64-unknown-linux-gnu");
        assert_eq!(TargetTriple::Aarch64Darwin.triple_str(), "aarch64-apple-darwin");
        assert_eq!(TargetTriple::Wasm32Unknown.triple_str(), "wasm32-unknown-unknown");
        assert_eq!(TargetTriple::Native.triple_str(), "");
    }

    // ========== predicates ==========

    #[test]
    fn test_is_wasm() {
        assert!(TargetTriple::Wasm32Unknown.is_wasm());
        assert!(TargetTriple::WasiPreview1.is_wasm());
        assert!(TargetTriple::WasiPreview2.is_wasm());
        assert!(!TargetTriple::X86_64Linux.is_wasm());
    }

    #[test]
    fn test_is_windows() {
        assert!(TargetTriple::X86_64WindowsMsvc.is_windows());
        assert!(TargetTriple::X86_64WindowsGnu.is_windows());
        assert!(TargetTriple::Aarch64WindowsMsvc.is_windows());
        assert!(!TargetTriple::X86_64Linux.is_windows());
    }

    #[test]
    fn test_is_apple() {
        assert!(TargetTriple::X86_64Darwin.is_apple());
        assert!(TargetTriple::Aarch64Darwin.is_apple());
        assert!(TargetTriple::Aarch64Ios.is_apple());
        assert!(TargetTriple::Aarch64IosSimulator.is_apple());
        assert!(!TargetTriple::X86_64Linux.is_apple());
    }

    #[test]
    fn test_is_android() {
        assert!(TargetTriple::Aarch64Android.is_android());
        assert!(TargetTriple::Armv7Android.is_android());
        assert!(!TargetTriple::Aarch64Linux.is_android());
    }

    #[test]
    fn test_is_ios() {
        assert!(TargetTriple::Aarch64Ios.is_ios());
        assert!(TargetTriple::Aarch64IosSimulator.is_ios());
        assert!(!TargetTriple::Aarch64Darwin.is_ios());
    }

    #[test]
    fn test_is_musl() {
        assert!(TargetTriple::X86_64LinuxMusl.is_musl());
        assert!(TargetTriple::Aarch64LinuxMusl.is_musl());
        assert!(!TargetTriple::X86_64Linux.is_musl());
    }

    #[test]
    fn test_is_freebsd() {
        assert!(TargetTriple::X86_64FreeBsd.is_freebsd());
        assert!(TargetTriple::Aarch64FreeBsd.is_freebsd());
        assert!(!TargetTriple::X86_64Linux.is_freebsd());
    }

    #[test]
    fn test_is_linux() {
        assert!(TargetTriple::X86_64Linux.is_linux());
        assert!(TargetTriple::Aarch64Linux.is_linux());
        assert!(TargetTriple::Riscv64LinuxGnu.is_linux());
        assert!(!TargetTriple::Aarch64Darwin.is_linux());
        assert!(!TargetTriple::Wasm32Unknown.is_linux());
    }

    // ========== target_os & target_arch ==========

    #[test]
    fn test_target_os() {
        assert_eq!(TargetTriple::X86_64Linux.target_os(), "linux");
        assert_eq!(TargetTriple::Aarch64Darwin.target_os(), "macos");
        assert_eq!(TargetTriple::X86_64WindowsMsvc.target_os(), "windows");
        assert_eq!(TargetTriple::Wasm32Unknown.target_os(), "wasm");
        assert_eq!(TargetTriple::Aarch64Android.target_os(), "android");
        assert_eq!(TargetTriple::X86_64FreeBsd.target_os(), "freebsd");
        assert_eq!(TargetTriple::Aarch64Ios.target_os(), "ios");
    }

    #[test]
    fn test_target_arch() {
        assert_eq!(TargetTriple::X86_64Linux.target_arch(), "x86_64");
        assert_eq!(TargetTriple::Aarch64Darwin.target_arch(), "aarch64");
        assert_eq!(TargetTriple::Armv7Android.target_arch(), "arm");
        assert_eq!(TargetTriple::Riscv64LinuxGnu.target_arch(), "riscv64");
        assert_eq!(TargetTriple::Wasm32Unknown.target_arch(), "wasm32");
    }

    // ========== pointer_bits ==========

    #[test]
    fn test_pointer_bits_64() {
        assert_eq!(TargetTriple::X86_64Linux.pointer_bits(), 64);
        assert_eq!(TargetTriple::Aarch64Darwin.pointer_bits(), 64);
    }

    #[test]
    fn test_pointer_bits_32() {
        assert_eq!(TargetTriple::Wasm32Unknown.pointer_bits(), 32);
        assert_eq!(TargetTriple::WasiPreview1.pointer_bits(), 32);
        assert_eq!(TargetTriple::Armv7Android.pointer_bits(), 32);
    }

    // ========== output_extension ==========

    #[test]
    fn test_output_extension_windows() {
        assert_eq!(TargetTriple::X86_64WindowsMsvc.output_extension(), "exe");
        assert_eq!(TargetTriple::X86_64WindowsGnu.output_extension(), "exe");
    }

    #[test]
    fn test_output_extension_wasm() {
        assert_eq!(TargetTriple::Wasm32Unknown.output_extension(), "wasm");
        assert_eq!(TargetTriple::WasiPreview1.output_extension(), "wasm");
    }

    #[test]
    fn test_output_extension_unix() {
        assert_eq!(TargetTriple::X86_64Linux.output_extension(), "");
        assert_eq!(TargetTriple::Aarch64Darwin.output_extension(), "");
    }

    // ========== cfg_values ==========

    #[test]
    fn test_cfg_values_linux() {
        let cfg = TargetTriple::X86_64Linux.cfg_values();
        assert_eq!(cfg.get("target_os").unwrap(), "linux");
        assert_eq!(cfg.get("target_arch").unwrap(), "x86_64");
        assert_eq!(cfg.get("target_family").unwrap(), "unix");
    }

    #[test]
    fn test_cfg_values_windows() {
        let cfg = TargetTriple::X86_64WindowsMsvc.cfg_values();
        assert_eq!(cfg.get("target_os").unwrap(), "windows");
        assert_eq!(cfg.get("target_family").unwrap(), "windows");
    }

    #[test]
    fn test_cfg_values_wasm() {
        let cfg = TargetTriple::Wasm32Unknown.cfg_values();
        assert_eq!(cfg.get("target_os").unwrap(), "wasm");
        assert_eq!(cfg.get("target_family").unwrap(), "wasm");
    }

    // ========== all_targets ==========

    #[test]
    fn test_all_targets_parseable() {
        for target_str in TargetTriple::all_targets() {
            assert!(
                TargetTriple::parse(target_str).is_some(),
                "Target '{}' should be parseable",
                target_str
            );
        }
    }

    #[test]
    fn test_all_targets_not_empty() {
        assert!(!TargetTriple::all_targets().is_empty());
        assert!(TargetTriple::all_targets().len() >= 15);
    }

    // ========== data_layout ==========

    #[test]
    fn test_data_layout_not_empty_for_non_native() {
        let targets = [
            TargetTriple::X86_64Linux,
            TargetTriple::Aarch64Darwin,
            TargetTriple::Wasm32Unknown,
            TargetTriple::Riscv64LinuxGnu,
        ];
        for target in targets {
            assert!(
                !target.data_layout().is_empty(),
                "Data layout for {:?} should not be empty",
                target
            );
        }
    }

    #[test]
    fn test_data_layout_native_is_empty() {
        assert_eq!(TargetTriple::Native.data_layout(), "");
    }

    // ========== equality ==========

    #[test]
    fn test_target_equality() {
        assert_eq!(TargetTriple::Native, TargetTriple::Native);
        assert_ne!(TargetTriple::Native, TargetTriple::X86_64Linux);
    }

    #[test]
    fn test_target_clone() {
        let t = TargetTriple::Aarch64Darwin;
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }
}
