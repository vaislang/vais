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
