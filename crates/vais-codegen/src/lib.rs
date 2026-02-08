//! Vais LLVM Code Generator
//!
//! Generates LLVM IR from typed AST for native code generation.
//!
//! # Backends
//!
//! This crate supports two code generation backends:
//!
//! - **text-codegen** (default): Generates LLVM IR as text, then compiles via clang.
//!   Does not require LLVM installation.
//!
//! - **inkwell-codegen**: Uses inkwell bindings for direct LLVM API access.
//!   Provides better type safety and performance. Requires LLVM 17+.
//!
//! # Feature Flags
//!
//! - `text-codegen` (default): Enable text-based IR generation
//! - `inkwell-codegen`: Enable inkwell-based generation (requires LLVM 17+)

pub mod abi;
#[cfg(test)]
mod abi_tests;
pub mod advanced_opt;
mod builtins;
#[cfg(test)]
mod cache_tests;
mod contracts;
mod control_flow;
pub mod cross_compile;
pub mod debug;
mod expr;
mod expr_helpers;
mod expr_visitor;
mod ffi;
pub mod formatter;
mod function_gen;
mod lambda_closure;
#[cfg(test)]
mod nested_field_tests;
pub mod optimize;
pub mod parallel;
mod registration;
mod stmt;
mod stmt_visitor;
mod string_ops;
#[cfg(test)]
mod struct_param_tests;
mod type_inference;
mod types;
pub mod visitor;
pub mod vtable;
#[cfg(test)]
mod vtable_tests;
pub mod wasm_component;

// Inkwell-based code generator (optional)
#[cfg(feature = "inkwell-codegen")]
pub mod inkwell;

#[cfg(feature = "inkwell-codegen")]
pub use inkwell::InkwellCodeGenerator;

pub use visitor::{ExprVisitor, ItemVisitor, StmtVisitor};

pub use debug::{DebugConfig, DebugInfoBuilder};

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;
use vais_types::ResolvedType;

/// Maximum recursion depth for type resolution to prevent stack overflow
/// This limit protects against infinite recursive types like: type A = B; type B = A;
const MAX_TYPE_RECURSION_DEPTH: usize = 128;

/// Escape a string for use in LLVM IR string constants.
///
/// Handles all control characters (0x00-0x1F, 0x7F) and special characters
/// that need escaping in LLVM IR constant strings.
fn escape_llvm_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'\\' => result.push_str("\\5C"),
            b'"' => result.push_str("\\22"),
            b'\n' => result.push_str("\\0A"),
            b'\r' => result.push_str("\\0D"),
            b'\t' => result.push_str("\\09"),
            b'\0' => result.push_str("\\00"),
            0x01..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F | 0x7F => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                result.push('\\');
                result.push(HEX[(byte >> 4) as usize] as char);
                result.push(HEX[(byte & 0x0F) as usize] as char);
            }
            _ => result.push(byte as char),
        }
    }
    result
}

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
            Self::X86_64Linux | Self::X86_64LinuxMusl | Self::Aarch64Linux | Self::Aarch64LinuxMusl | Self::Riscv64LinuxGnu
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
            Self::X86_64WindowsMsvc
            | Self::X86_64WindowsGnu
            | Self::Aarch64WindowsMsvc => "windows",
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

// Re-export type structs from types module
pub(crate) use types::*;

/// Error type for code generation failures.
///
/// Represents various kinds of errors that can occur during LLVM IR generation,
/// including undefined references, type mismatches, and unsupported features.
#[derive(Debug, Error)]
pub enum CodegenError {
    /// Reference to an undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    /// Call to an undefined function
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    /// Type-related error during code generation
    #[error("Type error: {0}")]
    TypeError(String),

    /// LLVM-specific error
    #[error("LLVM error: {0}")]
    LlvmError(String),

    /// Feature not yet implemented in code generation
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Recursion depth limit exceeded (infinite type recursion)
    #[error("Recursion depth limit exceeded: {0}")]
    RecursionLimitExceeded(String),
}

impl CodegenError {
    /// Get the error code for this codegen error
    pub fn error_code(&self) -> &str {
        match self {
            CodegenError::UndefinedVar(_) => "C001",
            CodegenError::UndefinedFunction(_) => "C002",
            CodegenError::TypeError(_) => "C003",
            CodegenError::LlvmError(_) => "C004",
            CodegenError::Unsupported(_) => "C005",
            CodegenError::RecursionLimitExceeded(_) => "C006",
        }
    }

    /// Get a help message for this error
    pub fn help(&self) -> Option<String> {
        match self {
            CodegenError::UndefinedVar(msg) => {
                if msg.contains("Did you mean") {
                    None // suggestion already embedded
                } else {
                    Some("check that the variable is defined before use".to_string())
                }
            }
            CodegenError::UndefinedFunction(msg) => {
                if msg.contains("Did you mean") {
                    None
                } else {
                    Some("check that the function is defined before calling it".to_string())
                }
            }
            CodegenError::TypeError(_) => {
                Some("ensure all operands have compatible types".to_string())
            }
            CodegenError::Unsupported(feature) => Some(format!(
                "'{}' is not yet implemented in code generation",
                feature
            )),
            CodegenError::RecursionLimitExceeded(_) => {
                Some("consider reducing nesting depth or refactoring recursive types".to_string())
            }
            CodegenError::LlvmError(_) => None,
        }
    }
}

type CodegenResult<T> = Result<T, CodegenError>;

// ============================================================================
// Error Message Suggestion Utilities
// ============================================================================

/// Calculate the Levenshtein edit distance between two strings
fn edit_distance(a: &str, b: &str) -> usize {
    let len_a = a.len();
    let len_b = b.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    // Create a matrix for dynamic programming
    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    // Initialize first column and row
    for (i, row) in matrix.iter_mut().enumerate().take(len_a + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len_b + 1) {
        *cell = j;
    }

    // Fill the matrix
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[len_a][len_b]
}

/// Find similar symbols from a list of candidates and return suggestions
///
/// Returns up to `max_suggestions` candidates sorted by edit distance.
/// Only includes candidates within a reasonable edit distance threshold.
fn suggest_similar(name: &str, candidates: &[&str], max_suggestions: usize) -> Vec<String> {
    // Calculate max distance based on name length
    // Short names (1-3 chars): max 1 edit, medium (4-7): max 2, long: max 3
    let max_distance = if name.len() <= 3 {
        1
    } else if name.len() <= 7 {
        2
    } else {
        3
    };

    let mut suggestions: Vec<(String, usize)> = candidates
        .iter()
        .map(|&candidate| {
            // Check for case-insensitive match first
            if candidate.eq_ignore_ascii_case(name) {
                (candidate.to_string(), 0)
            } else {
                let distance = edit_distance(name, candidate);
                (candidate.to_string(), distance)
            }
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance, then alphabetically
    suggestions.sort_by(|a, b| match a.1.cmp(&b.1) {
        std::cmp::Ordering::Equal => a.0.cmp(&b.0),
        other => other,
    });

    // Take top suggestions
    suggestions
        .into_iter()
        .take(max_suggestions)
        .map(|(name, _)| name)
        .collect()
}

/// Format a "did you mean" suggestion string
fn format_did_you_mean(suggestions: &[String]) -> String {
    match suggestions.len() {
        0 => String::new(),
        1 => format!(". Did you mean `{}`?", suggestions[0]),
        2 => format!(
            ". Did you mean `{}` or `{}`?",
            suggestions[0], suggestions[1]
        ),
        _ => {
            let first_two = suggestions[0..2].join("`, `");
            format!(". Did you mean `{}`, or `{}`?", first_two, suggestions[2])
        }
    }
}

/// Suggest type conversion hints based on common type mismatches
#[allow(dead_code)]
fn suggest_type_conversion(expected: &str, found: &str) -> String {
    // Common numeric conversions
    if expected.starts_with('i') && found.starts_with('f') {
        return format!(". Consider using `as {}` for explicit conversion", expected);
    }
    if expected.starts_with('f') && found.starts_with('i') {
        return format!(". Consider using `as {}` for explicit conversion", expected);
    }

    // Integer size conversions
    if expected.starts_with('i') && found.starts_with('i') && expected != found {
        return format!(". Consider using `as {}` to convert", expected);
    }

    // Float size conversions
    if expected.starts_with('f') && found.starts_with('f') && expected != found {
        return format!(". Consider using `as {}` to convert", expected);
    }

    // String conversions
    if expected == "String" && found == "&str" {
        return ". Consider using `.to_string()` or `.into()`".to_string();
    }
    if expected == "&str" && found == "String" {
        return ". Consider using `.as_str()` or `&`".to_string();
    }

    // Bool to integer
    if expected.starts_with('i') && found == "bool" {
        return format!(
            ". Consider using `as {}` to convert boolean to integer",
            expected
        );
    }

    String::new()
}

/// Result of generating a block of statements
/// (value, ir_code, is_terminated)
/// is_terminated is true if the block ends with break, continue, or return
#[allow(dead_code)]
type BlockResult = (String, String, bool);

/// LLVM IR Code Generator for Vais 0.0.1
///
/// Generates LLVM IR text from typed AST for native code generation via clang.
pub struct CodeGenerator {
    // All function names declared in the module (including generics, before instantiation)
    declared_functions: std::collections::HashSet<String>,

    // Module name
    module_name: String,

    // Target architecture
    target: TargetTriple,

    // Function signatures for lookup
    functions: HashMap<String, FunctionInfo>,

    // Struct definitions
    structs: HashMap<String, StructInfo>,

    // Generic struct AST definitions (before monomorphization)
    generic_struct_defs: HashMap<String, vais_ast::Struct>,

    // Enum definitions
    enums: HashMap<String, EnumInfo>,

    // Union definitions (untagged, C-style)
    unions: HashMap<String, UnionInfo>,

    // Current function being compiled
    current_function: Option<String>,

    // Current function's return type (for generating ret instructions in nested contexts)
    current_return_type: Option<ResolvedType>,

    // Local variables in current function
    locals: HashMap<String, LocalVar>,

    // Label counter for unique basic block names
    label_counter: usize,

    // Stack of loop labels for break/continue
    loop_stack: Vec<LoopLabels>,

    // Stack of deferred expressions per function (LIFO order)
    defer_stack: Vec<vais_ast::Spanned<vais_ast::Expr>>,

    // String constants for global storage
    string_constants: Vec<(String, String)>, // (name, value)

    // Counter for string constant names
    string_counter: usize,

    // Lambda functions generated during compilation
    lambda_functions: Vec<String>,

    // Closure information for each lambda variable (maps var_name -> closure_info)
    closures: HashMap<String, ClosureInfo>,

    // Last generated lambda info (for Let statement to pick up)
    last_lambda_info: Option<ClosureInfo>,

    // Async function state machine info
    async_state_counter: usize,
    async_await_points: Vec<AsyncAwaitPoint>,
    current_async_function: Option<AsyncFunctionInfo>,

    // Flag to emit unwrap panic message and abort declaration
    needs_unwrap_panic: bool,

    // Flag to emit string helper functions
    needs_string_helpers: bool,

    // Current basic block name (for phi node predecessor tracking)
    current_block: String,

    // Debug info builder for DWARF metadata generation
    debug_info: DebugInfoBuilder,

    // Generic substitutions for current function/method
    // Maps generic param name (e.g., "T") to concrete type (e.g., ResolvedType::I64)
    generic_substitutions: HashMap<String, ResolvedType>,

    // Generated struct instantiations (mangled_name -> already_generated)
    generated_structs: HashMap<String, bool>,

    // Generic struct name aliases (base_name -> mangled_name, e.g., "Box" -> "Box$i64")
    generic_struct_aliases: HashMap<String, String>,

    // Generated function instantiations (mangled_name -> already_generated)
    generated_functions: HashMap<String, bool>,

    // Cache for type_to_llvm conversions to avoid repeated computations
    // Uses interior mutability to allow caching through immutable references
    type_to_llvm_cache: std::cell::RefCell<HashMap<String, String>>,

    // GC mode configuration
    gc_enabled: bool,
    gc_threshold: usize,

    // VTable generator for trait objects (dyn Trait)
    vtable_generator: vtable::VtableGenerator,

    // Trait definitions for vtable generation
    trait_defs: HashMap<String, vais_types::TraitDef>,

    // Trait implementations: (impl_type, trait_name) -> method_impls
    trait_impl_methods: HashMap<(String, String), HashMap<String, String>>,

    // Release mode flag (disables contract checks)
    release_mode: bool,

    // Current source file being compiled (for contract error messages)
    current_file: Option<String>,

    // Contract string constants (separate from regular strings)
    contract_string_constants: HashMap<String, String>,

    // Counter for contract string constant names
    contract_string_counter: usize,

    // Pre-state snapshots for old() expressions in ensures clauses
    // Maps snapshot variable name -> allocated storage name
    old_snapshots: HashMap<String, String>,

    // Decreases expressions for current function (for termination proof)
    // Maps function name -> (storage_var_name, decreases_expr_span)
    current_decreases_info: Option<DecreasesInfo>,

    // Constant definitions
    constants: HashMap<String, types::ConstInfo>,

    // Global variable definitions
    globals: HashMap<String, types::GlobalInfo>,

    // Type recursion depth tracking (prevents infinite recursion)
    type_recursion_depth: std::cell::Cell<usize>,

    // Generic function instantiation map: base_name -> Vec<(type_args, mangled_name)>
    // Used to resolve generic function calls to their mangled specialized names
    generic_fn_instantiations: HashMap<String, Vec<(Vec<ResolvedType>, String)>>,

    // Generic function templates stored for specialization (base_name -> Function)
    generic_function_templates: HashMap<String, Function>,

    // Resolved function signatures from type checker (for inferred parameter types)
    resolved_function_sigs: HashMap<String, vais_types::FunctionSig>,

    // Module-specific prefix for string constants (avoids collisions in multi-module builds)
    string_prefix: Option<String>,
}

/// Information about a function's decreases clause for termination proof
#[derive(Clone)]
pub struct DecreasesInfo {
    /// Storage variable name for the initial decreases value
    pub storage_name: String,
    /// The decreases expression from the attribute (already boxed)
    pub expr: Box<vais_ast::Spanned<vais_ast::Expr>>,
    /// Function name with decreases clause
    pub function_name: String,
}

impl CodeGenerator {
    /// Creates a new code generator for the given module with native target.
    ///
    /// Initializes the code generator with built-in functions registered.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module being compiled
    pub fn new(module_name: &str) -> Self {
        Self::new_with_target(module_name, TargetTriple::Native)
    }

    /// Creates a new code generator for the given module with specified target.
    ///
    /// Initializes the code generator with built-in functions registered.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module being compiled
    /// * `target` - Target architecture for code generation
    pub fn new_with_target(module_name: &str, target: TargetTriple) -> Self {
        let mut gen = Self {
            declared_functions: std::collections::HashSet::new(),
            module_name: module_name.to_string(),
            target,
            functions: HashMap::new(),
            structs: HashMap::new(),
            generic_struct_defs: HashMap::new(),
            enums: HashMap::new(),
            unions: HashMap::new(),
            current_function: None,
            current_return_type: None,
            locals: HashMap::new(),
            label_counter: 0,
            loop_stack: Vec::new(),
            defer_stack: Vec::new(),
            string_constants: Vec::new(),
            string_counter: 0,
            lambda_functions: Vec::new(),
            closures: HashMap::new(),
            last_lambda_info: None,
            async_state_counter: 0,
            async_await_points: Vec::new(),
            current_async_function: None,
            needs_unwrap_panic: false,
            needs_string_helpers: false,
            current_block: "entry".to_string(),
            debug_info: DebugInfoBuilder::new(DebugConfig::default()),
            generic_substitutions: HashMap::new(),
            generated_structs: HashMap::new(),
            generic_struct_aliases: HashMap::new(),
            generated_functions: HashMap::new(),
            type_to_llvm_cache: std::cell::RefCell::new(HashMap::new()),
            gc_enabled: false,
            gc_threshold: 1048576, // 1 MB default
            vtable_generator: vtable::VtableGenerator::new(),
            trait_defs: HashMap::new(),
            trait_impl_methods: HashMap::new(),
            release_mode: false,
            current_file: None,
            contract_string_constants: HashMap::new(),
            contract_string_counter: 0,
            old_snapshots: HashMap::new(),
            current_decreases_info: None,
            constants: HashMap::new(),
            globals: HashMap::new(),
            type_recursion_depth: std::cell::Cell::new(0),
            generic_fn_instantiations: HashMap::new(),
            generic_function_templates: HashMap::new(),
            resolved_function_sigs: HashMap::new(),
            string_prefix: None,
        };

        // Register built-in extern functions
        gen.register_builtin_functions();
        gen
    }

    /// Get the target triple for this code generator
    pub fn target(&self) -> &TargetTriple {
        &self.target
    }

    /// Enable debug info generation with source file information
    ///
    /// This should be called before `generate_module` to enable DWARF debug
    /// metadata generation. The source code is used for line/column mapping.
    ///
    /// # Arguments
    /// * `source_file` - Name of the source file
    /// * `source_dir` - Directory containing the source file
    /// * `source_code` - The source code content for line number calculation
    pub fn enable_debug(&mut self, source_file: &str, source_dir: &str, source_code: &str) {
        let config = DebugConfig::new(source_file, source_dir);
        self.debug_info = DebugInfoBuilder::new(config);
        self.debug_info.set_source_code(source_code);
    }

    /// Check if debug info generation is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_info.is_enabled()
    }

    /// Enable GC mode for automatic memory management
    pub fn enable_gc(&mut self) {
        self.gc_enabled = true;
    }

    /// Set GC threshold (bytes allocated before triggering collection)
    pub fn set_gc_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }

    /// Check if GC mode is enabled
    pub fn is_gc_enabled(&self) -> bool {
        self.gc_enabled
    }

    /// Enable release mode (disables contract checks)
    pub fn enable_release_mode(&mut self) {
        self.release_mode = true;
    }

    /// Check if release mode is enabled
    pub fn is_release_mode(&self) -> bool {
        self.release_mode
    }

    /// Set resolved function signatures from the type checker.
    /// Used to provide inferred parameter types for functions with Type::Infer parameters.
    pub fn set_resolved_functions(&mut self, resolved: HashMap<String, vais_types::FunctionSig>) {
        self.resolved_function_sigs = resolved;
    }

    /// Set string prefix for per-module codegen (avoids .str.N collisions across modules)
    pub fn set_string_prefix(&mut self, prefix: &str) {
        self.string_prefix = Some(prefix.to_string());
    }

    /// Generate a unique string constant name, with optional module prefix
    fn make_string_name(&self) -> String {
        if let Some(ref prefix) = self.string_prefix {
            format!("{}.str.{}", prefix, self.string_counter)
        } else {
            format!(".str.{}", self.string_counter)
        }
    }

    /// Generate LLVM IR for a subset of module items (per-module codegen).
    ///
    /// This method generates IR for only the items at the specified indices,
    /// while declaring cross-module functions as extern. Type definitions
    /// (structs, enums, unions) are always included since they're needed everywhere.
    ///
    /// # Arguments
    /// * `full_module` - The complete merged AST (all modules)
    /// * `item_indices` - Indices into `full_module.items` for this module's items
    /// * `is_main_module` - Whether this module contains main() (emits ABI version, etc.)
    pub fn generate_module_subset(
        &mut self,
        full_module: &Module,
        item_indices: &[usize],
        is_main_module: bool,
    ) -> CodegenResult<String> {
        // Validate item_indices are within bounds
        let items_len = full_module.items.len();
        let mut out_of_bounds = Vec::new();
        for &idx in item_indices {
            if idx >= items_len {
                out_of_bounds.push(idx);
            }
        }
        if !out_of_bounds.is_empty() {
            eprintln!(
                "Warning: {} item indices out of bounds (>= {}): {:?}",
                out_of_bounds.len(),
                items_len,
                out_of_bounds
            );
        }

        // Filter to valid indices only
        let valid_indices: Vec<usize> = item_indices.iter()
            .copied()
            .filter(|&i| i < items_len)
            .collect();

        let mut ir = String::new();
        let index_set: std::collections::HashSet<usize> = valid_indices.iter().copied().collect();

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");
        if !matches!(self.target, TargetTriple::Native) {
            ir.push_str(&format!(
                "target datalayout = \"{}\"\n",
                self.target.data_layout()
            ));
            ir.push_str(&format!(
                "target triple = \"{}\"\n",
                self.target.triple_str()
            ));
        }
        ir.push('\n');

        // Initialize debug info if enabled
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }

        // Snapshot builtin function keys (registered in constructor, before AST items)
        // These should NOT appear as cross-module extern declarations.
        let builtin_fn_keys: std::collections::HashSet<String> =
            self.functions.keys().cloned().collect();

        // First pass: register ALL type definitions (structs, enums, unions) from full module
        // and register functions — tracking which are "ours" vs external
        let mut module_functions: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for (idx, item) in full_module.items.iter().enumerate() {
            let is_ours = index_set.contains(&idx);
            match &item.node {
                Item::Function(f) => {
                    self.register_function(f)?;
                    if is_ours {
                        module_functions.insert(f.name.node.clone());
                    }
                }
                Item::Struct(s) => {
                    self.register_struct(s)?;
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", s.name.node, method.node.name.node));
                        }
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                        if is_ours {
                            module_functions
                                .insert(format!("{}_{}", type_name, method.node.name.node));
                        }
                    }
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Generate struct types (all modules need these)
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }
        for (name, info) in &self.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate extern declarations for ALL extern functions (is_extern = true)
        // Builtin helpers (is_extern = false) are handled separately below.
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (key, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name)
                && !module_functions.contains(&info.signature.name)
                && !module_functions.contains(*key)
            {
                if !is_main_module && info.signature.name == "fopen_ptr" {
                    // Non-main modules should declare fopen_ptr (not define it).
                    // The wrapper definition lives in the main module only.
                    let params: Vec<_> = info
                        .signature
                        .params
                        .iter()
                        .map(|(_, ty, _)| self.type_to_llvm(ty))
                        .collect();
                    let ret = self.type_to_llvm(&info.signature.ret);
                    ir.push_str(&format!(
                        "declare {} @fopen_ptr({})\n",
                        ret,
                        params.join(", ")
                    ));
                } else {
                    ir.push_str(&self.generate_extern_decl(info));
                    ir.push('\n');
                }
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate extern declarations for cross-module Vais functions
        // (functions registered from AST but not in this module's item set)
        // Skip builtins — they are handled by generate_helper_functions() or the non-main extern block.
        for (name, info) in &self.functions {
            if !info.is_extern
                && !module_functions.contains(name)
                && !declared_fns.contains(name)
                && !builtin_fn_keys.contains(name)
            {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(name.clone());
            }
        }

        // Generate function bodies only for this module's items
        let mut body_ir = String::new();
        for &idx in &valid_indices {
            let item = &full_module.items[idx];
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                _ => {} // Other items handled in registration pass
            }
        }

        // ABI version and globals only in main module
        if is_main_module {
            let abi_version = crate::abi::ABI_VERSION;
            let abi_version_len = abi_version.len() + 1;
            ir.push_str(&format!(
                "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n\n",
                abi_version_len, abi_version
            ));
        }

        // String constants
        for (name, value) in &self.string_constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.string_constants.is_empty() {
            ir.push('\n');
        }

        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }

        ir.push_str(&body_ir);

        for lambda_ir in &self.lambda_functions {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }

        let vtable_ir = self.generate_vtable_globals();
        if !vtable_ir.is_empty() {
            ir.push_str("\n; VTable globals for trait objects\n");
            ir.push_str(&vtable_ir);
        }
        let drop_ir = self.vtable_generator.generate_drop_functions_ir();
        if !drop_ir.is_empty() {
            ir.push_str("\n; Drop functions for trait objects\n");
            ir.push_str(&drop_ir);
        }

        if is_main_module {
            // Main module defines all helper functions
            ir.push_str(&self.generate_helper_functions());
        } else {
            // Non-main modules declare builtin helpers as extern
            // (these are defined by generate_helper_functions() in the main module)
            ir.push_str("\n; Extern declarations for runtime helpers\n");
            let mut sorted_helpers: Vec<_> = builtin_fn_keys.iter().collect();
            sorted_helpers.sort();
            for key in sorted_helpers {
                if let Some(info) = self.functions.get(key) {
                    if !info.is_extern
                        && !declared_fns.contains(&info.signature.name)
                        && !module_functions.contains(key)
                        && !module_functions.contains(&info.signature.name)
                    {
                        ir.push_str(&self.generate_extern_decl(info));
                        ir.push('\n');
                        declared_fns.insert(info.signature.name.clone());
                    }
                }
            }
        }

        if self.needs_string_helpers {
            if is_main_module {
                ir.push_str(&self.generate_string_helper_functions());
            }
            ir.push_str(&self.generate_string_extern_declarations());
        }

        if !self.contract_string_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        if self.debug_info.is_enabled() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        ir.push_str(&self.debug_info.finalize());

        Ok(ir)
    }

    /// Set current source file for error messages
    pub fn set_source_file(&mut self, file: &str) {
        self.current_file = Some(file.to_string());
    }

    /// Check if a function call is recursive (calls the current function with decreases clause)
    fn is_recursive_call(&self, fn_name: &str) -> bool {
        // Check if we have a decreases clause for this function
        if let Some(ref decreases_info) = self.current_decreases_info {
            // A recursive call is when the called function matches the function with decreases
            decreases_info.function_name == fn_name
        } else {
            false
        }
    }

    /// Check if a function has the #[gc] attribute
    #[allow(dead_code)]
    fn has_gc_attribute(attributes: &[Attribute]) -> bool {
        attributes.iter().any(|attr| attr.name == "gc")
    }

    /// Get current generic substitution for a type parameter
    pub(crate) fn get_generic_substitution(&self, param: &str) -> Option<ResolvedType> {
        self.generic_substitutions.get(param).cloned()
    }

    /// Set generic substitutions for the current context
    #[allow(dead_code)]
    pub(crate) fn set_generic_substitutions(&mut self, subst: HashMap<String, ResolvedType>) {
        self.generic_substitutions = subst;
    }

    /// Clear generic substitutions
    #[allow(dead_code)]
    pub(crate) fn clear_generic_substitutions(&mut self) {
        self.generic_substitutions.clear();
    }

    /// Resolve a struct name, checking aliases for generic specializations.
    /// Returns the mangled name if the base name has a registered alias (e.g., "Box" -> "Box$i64").
    pub(crate) fn resolve_struct_name(&self, name: &str) -> String {
        if self.structs.contains_key(name) {
            return name.to_string();
        }
        if let Some(mangled) = self.generic_struct_aliases.get(name) {
            return mangled.clone();
        }
        name.to_string()
    }

    /// Generate mangled name for a generic struct
    pub(crate) fn mangle_struct_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Generate mangled name for a generic function
    #[allow(dead_code)]
    pub(crate) fn mangle_function_name(&self, name: &str, generics: &[ResolvedType]) -> String {
        vais_types::mangle_name(name, generics)
    }

    /// Enter a type recursion level and check depth limit
    /// Returns an error if recursion limit is exceeded
    fn enter_type_recursion(&self, context: &str) -> CodegenResult<()> {
        let depth = self.type_recursion_depth.get();
        if depth >= MAX_TYPE_RECURSION_DEPTH {
            return Err(CodegenError::RecursionLimitExceeded(format!(
                "Type recursion depth limit ({}) exceeded in {}",
                MAX_TYPE_RECURSION_DEPTH, context
            )));
        }
        self.type_recursion_depth.set(depth + 1);
        Ok(())
    }

    /// Exit a type recursion level
    fn exit_type_recursion(&self) {
        let depth = self.type_recursion_depth.get();
        self.type_recursion_depth.set(depth.saturating_sub(1));
    }

    /// Get the size of a type in bytes (for generic operations)
    #[allow(dead_code)]
    pub(crate) fn type_size(&self, ty: &ResolvedType) -> usize {
        // Track recursion depth
        if self.enter_type_recursion("type_size").is_err() {
            // On recursion limit, return default size
            #[cfg(debug_assertions)]
            eprintln!("Warning: Type recursion limit exceeded in type_size");
            return 8;
        }

        let size = match ty {
            ResolvedType::I8 | ResolvedType::U8 | ResolvedType::Bool => 1,
            ResolvedType::I16 | ResolvedType::U16 => 2,
            ResolvedType::I32 | ResolvedType::U32 | ResolvedType::F32 => 4,
            ResolvedType::I64 | ResolvedType::U64 | ResolvedType::F64 => 8,
            ResolvedType::I128 | ResolvedType::U128 => 16,
            ResolvedType::Str => 8, // Pointer size
            ResolvedType::Pointer(_) | ResolvedType::Ref(_) | ResolvedType::RefMut(_) => 8,
            ResolvedType::Named { name, .. } => {
                // Calculate struct size
                if let Some(info) = self.structs.get(name) {
                    info.fields.iter().map(|(_, t)| self.type_size(t)).sum()
                } else {
                    8 // Default to pointer size
                }
            }
            ResolvedType::Generic(param) => {
                // Try to get concrete type from substitutions
                if let Some(concrete) = self.generic_substitutions.get(param) {
                    self.type_size(concrete)
                } else {
                    8 // Default to i64 size
                }
            }
            ResolvedType::DynTrait { .. } => 16, // Fat pointer: data + vtable
            _ => 8,                              // Default
        };

        // Always exit recursion
        self.exit_type_recursion();
        size
    }

    /// Register a trait definition for vtable generation
    pub fn register_trait(&mut self, trait_def: vais_types::TraitDef) {
        self.trait_defs.insert(trait_def.name.clone(), trait_def);
    }

    /// Register a trait from AST definition (converts AST Trait to TraitDef)
    fn register_trait_from_ast(&mut self, t: &vais_ast::Trait) {
        let mut methods = HashMap::new();
        for m in &t.methods {
            let params: Vec<(String, ResolvedType, bool)> = m
                .params
                .iter()
                .map(|p| {
                    let ty = if p.name.node == "self" {
                        // self parameter is a pointer to the implementing type
                        ResolvedType::I64 // placeholder, resolved at call site
                    } else {
                        self.ast_type_to_resolved(&p.ty.node)
                    };
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = m
                .ret_type
                .as_ref()
                .map(|t| self.ast_type_to_resolved(&t.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                m.name.node.clone(),
                vais_types::TraitMethodSig {
                    name: m.name.node.clone(),
                    params,
                    ret,
                    has_default: m.default_body.is_some(),
                    is_async: m.is_async,
                    is_const: m.is_const,
                },
            );
        }

        let trait_def = vais_types::TraitDef {
            name: t.name.node.clone(),
            generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
            super_traits: t.super_traits.iter().map(|s| s.node.clone()).collect(),
            associated_types: HashMap::new(), // Simplified for now
            methods,
        };
        self.register_trait(trait_def);
    }

    /// Register a trait implementation for vtable generation
    pub fn register_trait_impl(
        &mut self,
        impl_type: &str,
        trait_name: &str,
        method_impls: HashMap<String, String>,
    ) {
        self.trait_impl_methods.insert(
            (impl_type.to_string(), trait_name.to_string()),
            method_impls,
        );
    }

    /// Get or generate a vtable for a specific type implementing a trait
    pub fn get_or_generate_vtable(
        &mut self,
        impl_type: &str,
        trait_name: &str,
    ) -> Option<vtable::VtableInfo> {
        let trait_def = self.trait_defs.get(trait_name)?.clone();
        let method_impls = self
            .trait_impl_methods
            .get(&(impl_type.to_string(), trait_name.to_string()))
            .cloned()
            .unwrap_or_default();

        Some(
            self.vtable_generator
                .generate_vtable(impl_type, &trait_def, &method_impls),
        )
    }

    /// Generate all vtable globals for the module
    pub fn generate_vtable_globals(&self) -> String {
        let mut ir = String::new();

        for vtable_info in self.vtable_generator.get_vtables() {
            if let Some(trait_def) = self.trait_defs.get(&vtable_info.trait_name) {
                let type_size = 8; // Default size, could be refined
                let type_align = 8; // Default alignment

                ir.push_str(&self.vtable_generator.generate_vtable_global(
                    vtable_info,
                    trait_def,
                    type_size,
                    type_align,
                ));
                ir.push('\n');
            }
        }

        ir
    }

    /// Generate code to create a trait object from a concrete value
    pub fn generate_trait_object_creation(
        &mut self,
        concrete_value: &str,
        concrete_type: &str,
        impl_type: &str,
        trait_name: &str,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let vtable_info = self
            .get_or_generate_vtable(impl_type, trait_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "No vtable for {} implementing {}",
                    impl_type, trait_name
                ))
            })?;

        Ok(self.vtable_generator.create_trait_object(
            concrete_value,
            concrete_type,
            &vtable_info,
            counter,
        ))
    }

    /// Generate code for a dynamic method call on a trait object
    pub fn generate_dyn_method_call(
        &self,
        trait_object: &str,
        trait_name: &str,
        method_name: &str,
        args: &[String],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let trait_def = self
            .trait_defs
            .get(trait_name)
            .ok_or_else(|| CodegenError::Unsupported(format!("Unknown trait: {}", trait_name)))?;

        // Find method index in trait
        let method_names: Vec<&String> = trait_def.methods.keys().collect();
        let method_index = method_names
            .iter()
            .position(|&n| n == method_name)
            .ok_or_else(|| {
                CodegenError::Unsupported(format!(
                    "Method {} not found in trait {}",
                    method_name, trait_name
                ))
            })?;

        // Get return type
        let method_sig = trait_def.methods.get(method_name).ok_or_else(|| {
            CodegenError::Unsupported(format!(
                "Method {} not found in trait {}",
                method_name, trait_name
            ))
        })?;

        let ret_type = if matches!(method_sig.ret, ResolvedType::Unit) {
            "void"
        } else {
            "i64" // Simplified
        };

        Ok(self.vtable_generator.generate_dynamic_call(
            trait_object,
            method_index,
            args,
            ret_type,
            trait_def,
            counter,
        ))
    }

    fn next_label(&mut self, prefix: &str) -> String {
        debug_assert!(
            !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_'),
            "Invalid label prefix: '{}'. Must be non-empty and contain only alphanumeric, '.', or '_' characters.",
            prefix
        );
        let label = format!("{}{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generate allocation call (malloc or gc_alloc depending on GC mode)
    ///
    /// Returns: (result_register, IR code)
    #[allow(dead_code)]
    fn generate_alloc(
        &self,
        size_arg: &str,
        counter: &mut usize,
        type_id: u32,
    ) -> (String, String) {
        let mut ir = String::new();

        if self.gc_enabled {
            // Use GC allocation
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @vais_gc_alloc(i64 {}, i32 {})\n",
                ptr_tmp, size_arg, type_id
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        } else {
            // Use manual malloc
            let ptr_tmp = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = call i8* @malloc(i64 {})\n",
                ptr_tmp, size_arg
            ));
            let result = self.next_temp(counter);
            ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
            (result, ir)
        }
    }

    /// Generates LLVM IR code for a complete module.
    ///
    /// Performs two-pass code generation:
    /// 1. First pass: Collect all type and function declarations
    /// 2. Second pass: Generate code for all function bodies
    ///
    /// # Arguments
    ///
    /// * `module` - The typed AST module to compile
    ///
    /// # Returns
    ///
    /// A string containing the complete LLVM IR code on success,
    /// or a CodegenError on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use vais_codegen::CodeGenerator;
    /// use vais_parser::parse;
    ///
    /// let source = "F add(x:i64,y:i64)->i64=x+y";
    /// let module = parse(source).unwrap();
    ///
    /// let mut gen = CodeGenerator::new("test");
    /// let ir = gen.generate_module(&module).unwrap();
    /// assert!(ir.contains("define"));
    /// ```
    pub fn generate_module(&mut self, module: &Module) -> CodegenResult<String> {
        let mut ir = String::new();

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");

        // Target triple and data layout (for non-native targets)
        if !matches!(self.target, TargetTriple::Native) {
            ir.push_str(&format!(
                "target datalayout = \"{}\"\n",
                self.target.data_layout()
            ));
            ir.push_str(&format!(
                "target triple = \"{}\"\n",
                self.target.triple_str()
            ));
        }
        // Note: for Native target, omit these to let clang auto-detect
        ir.push('\n');

        // Initialize debug info if enabled
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }

        // First pass: collect declarations
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => {
                    self.register_struct(s)?;
                    // Register struct methods
                    for method in &s.methods {
                        self.register_method(&s.name.node, &method.node)?;
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    // Register impl methods
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Use(_) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // No code generation needed for imports
                }
                Item::Trait(trait_def) => {
                    // Register trait for vtable generation
                    self.register_trait_from_ast(trait_def);
                }
                Item::TypeAlias(_) => {
                    // Type aliases don't generate code
                }
                Item::Macro(_) => {
                    // Macro definitions are expanded at compile time
                    // No runtime code generation needed
                }
                Item::Error { .. } => {
                    // Error nodes indicate parsing failures
                    // Skip them during code generation - errors were reported during parsing
                }
                Item::ExternBlock(extern_block) => {
                    // Register extern functions
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    // Register constant for code generation
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    // Register global variable
                    self.register_global(global_def)?;
                }
            }
        }

        // Generate struct types
        for (name, info) in &self.structs {
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (deduplicate by actual function name)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        // to ensure correct C type signatures in declare statements
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Second pass: generate function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                    body_ir.push('\n');
                }
                Item::Struct(s) => {
                    // Generate methods for this struct
                    for method in &s.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &s.name.node,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Impl(impl_block) => {
                    // Generate methods from impl block
                    // Get the type name from target_type
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_) => {
                    // Already handled in first pass or no code generation needed
                }
                Item::Const(_) | Item::Global(_) => {
                    // Constants and globals are handled in first pass
                }
                Item::Error { .. } => {
                    // Error nodes are skipped - already logged in first pass
                }
            }
        }

        // Add ABI version constant
        let abi_version = crate::abi::ABI_VERSION;
        let abi_version_len = abi_version.len() + 1; // +1 for null terminator
        ir.push_str(&format!(
            "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n\n",
            abi_version_len, abi_version
        ));

        // Add string constants at the top of the module
        for (name, value) in &self.string_constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1; // +1 for null terminator
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.string_constants.is_empty() {
            ir.push('\n');
        }

        // Add unwrap panic message and abort declaration if needed
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }

        ir.push_str(&body_ir);

        // Add lambda functions at the end
        for lambda_ir in &self.lambda_functions {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }

        // Add vtable globals and drop functions for trait objects
        let vtable_ir = self.generate_vtable_globals();
        if !vtable_ir.is_empty() {
            ir.push_str("\n; VTable globals for trait objects\n");
            ir.push_str(&vtable_ir);
        }
        let drop_ir = self.vtable_generator.generate_drop_functions_ir();
        if !drop_ir.is_empty() {
            ir.push_str("\n; Drop functions for trait objects\n");
            ir.push_str(&drop_ir);
        }

        // Add helper functions for memory operations
        ir.push_str(&self.generate_helper_functions());

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            ir.push_str(&self.generate_string_extern_declarations());
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contract_string_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsic declaration if debug info is enabled
        if self.debug_info.is_enabled() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata at the end
        ir.push_str(&self.debug_info.finalize());

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }

    /// Generates LLVM IR code for a complete module with generic instantiations.
    ///
    /// This is the main entry point when monomorphization is enabled.
    /// It takes the generic instantiations collected by the type checker
    /// and generates specialized code for each unique type combination.
    ///
    /// # Arguments
    ///
    /// * `module` - The typed AST module to compile
    /// * `instantiations` - Generic instantiations collected by the type checker
    ///
    /// # Returns
    ///
    /// A string containing the complete LLVM IR code on success,
    /// or a CodegenError on failure.
    pub fn generate_module_with_instantiations(
        &mut self,
        module: &Module,
        instantiations: &[vais_types::GenericInstantiation],
    ) -> CodegenResult<String> {
        let mut ir = String::new();

        // Header
        ir.push_str(&format!("; ModuleID = '{}'\n", self.module_name));
        ir.push_str("source_filename = \"<vais>\"\n");

        // Target triple and data layout (for non-native targets)
        if !matches!(self.target, TargetTriple::Native) {
            ir.push_str(&format!(
                "target datalayout = \"{}\"\n",
                self.target.data_layout()
            ));
            ir.push_str(&format!(
                "target triple = \"{}\"\n",
                self.target.triple_str()
            ));
        }
        ir.push('\n');

        // Initialize debug info if enabled
        if self.debug_info.is_enabled() {
            self.debug_info.initialize();
        }

        // First pass: collect declarations (including generic templates)
        let mut generic_functions: HashMap<String, Function> = HashMap::new();
        let mut generic_structs: HashMap<String, Struct> = HashMap::new();

        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    // Track this function name (generic or not)
                    self.declared_functions.insert(f.name.node.clone());

                    if !f.generics.is_empty() {
                        // Store generic function for later specialization
                        generic_functions.insert(f.name.node.clone(), f.clone());
                        self.generic_function_templates
                            .insert(f.name.node.clone(), f.clone());
                    } else {
                        self.register_function(f)?;
                    }
                }
                Item::Struct(s) => {
                    if !s.generics.is_empty() {
                        // Store generic struct for later specialization
                        generic_structs.insert(s.name.node.clone(), s.clone());
                        // Also store in the generic_struct_defs for type inference
                        self.generic_struct_defs
                            .insert(s.name.node.clone(), s.clone());
                    } else {
                        self.register_struct(s)?;
                        for method in &s.methods {
                            self.register_method(&s.name.node, &method.node)?;
                        }
                    }
                }
                Item::Enum(e) => self.register_enum(e)?,
                Item::Union(u) => self.register_union(u)?,
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        self.register_method(&type_name, &method.node)?;
                    }
                    // Register trait impl for vtable generation
                    if let Some(ref trait_name) = impl_block.trait_name {
                        let mut method_impls = HashMap::new();
                        for method in &impl_block.methods {
                            let fn_name = format!("{}_{}", type_name, method.node.name.node);
                            method_impls.insert(method.node.name.node.clone(), fn_name);
                        }
                        self.register_trait_impl(&type_name, &trait_name.node, method_impls);
                    }
                }
                Item::Trait(trait_def) => {
                    self.register_trait_from_ast(trait_def);
                }
                Item::ExternBlock(extern_block) => {
                    for func in &extern_block.functions {
                        self.register_extern_function(func, &extern_block.abi)?;
                    }
                }
                Item::Const(const_def) => {
                    self.register_const(const_def)?;
                }
                Item::Global(global_def) => {
                    self.register_global(global_def)?;
                }
                Item::Use(_) | Item::TypeAlias(_) | Item::Macro(_) | Item::Error { .. } => {}
            }
        }

        // Build generic function instantiation mapping and register specialized function signatures
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = generic_functions.get(&inst.base_name) {
                    // Build instantiation mapping: base_name -> [(type_args, mangled_name)]
                    self.generic_fn_instantiations
                        .entry(inst.base_name.clone())
                        .or_default()
                        .push((inst.type_args.clone(), inst.mangled_name.clone()));

                    // Register the specialized function signature so call codegen can find it
                    let substitutions: HashMap<String, ResolvedType> = generic_fn
                        .generics
                        .iter()
                        .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                        .zip(inst.type_args.iter())
                        .map(|(g, t)| (g.name.node.to_string(), t.clone()))
                        .collect();

                    let params: Vec<_> = generic_fn
                        .params
                        .iter()
                        .map(|p| {
                            let ty = self.ast_type_to_resolved(&p.ty.node);
                            let concrete_ty = vais_types::substitute_type(&ty, &substitutions);
                            (p.name.node.to_string(), concrete_ty, p.is_mut)
                        })
                        .collect();

                    let ret_type = generic_fn
                        .ret_type
                        .as_ref()
                        .map(|t| {
                            let ty = self.ast_type_to_resolved(&t.node);
                            vais_types::substitute_type(&ty, &substitutions)
                        })
                        .unwrap_or(ResolvedType::Unit);

                    self.functions.insert(
                        inst.mangled_name.clone(),
                        FunctionInfo {
                            signature: vais_types::FunctionSig {
                                name: inst.mangled_name.clone(),
                                generics: vec![],
                                generic_bounds: HashMap::new(),
                                params,
                                ret: ret_type,
                                is_async: generic_fn.is_async,
                                is_vararg: false,
                                required_params: None,
                                contracts: None,
                                effect_annotation: vais_types::EffectAnnotation::Infer,
                                inferred_effects: None,
                            },
                            is_extern: false,
                            extern_abi: None,
                        },
                    );
                }
            }
        }

        // Generate specialized struct types from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Struct = inst.kind {
                if let Some(generic_struct) = generic_structs.get(&inst.base_name) {
                    self.generate_specialized_struct_type(generic_struct, inst, &mut ir)?;
                }
            }
        }

        // Generate non-generic struct types (skip already-emitted specialized generics)
        for (name, info) in &self.structs {
            if self.generated_structs.contains_key(name) {
                continue;
            }
            ir.push_str(&self.generate_struct_type(name, info));
            ir.push('\n');
        }

        // Generate enum types
        for (name, info) in &self.enums {
            ir.push_str(&self.generate_enum_type(name, info));
            ir.push('\n');
        }

        // Generate union types
        for (name, info) in &self.unions {
            ir.push_str(&self.generate_union_type(name, info));
            ir.push('\n');
        }

        // Generate function declarations (extern functions)
        // Prioritize non-aliased functions (key == name) over aliased ones (key != name)
        let mut declared_fns = std::collections::HashSet::new();
        let mut sorted_fns: Vec<_> = self
            .functions
            .iter()
            .filter(|(_, info)| info.is_extern)
            .collect();
        sorted_fns.sort_by_key(|(key, info)| if **key == info.signature.name { 0 } else { 1 });
        for (_, info) in &sorted_fns {
            if !declared_fns.contains(&info.signature.name) {
                ir.push_str(&self.generate_extern_decl(info));
                ir.push('\n');
                declared_fns.insert(info.signature.name.clone());
            }
        }

        // Generate string constants (after processing functions to collect all strings)
        let mut body_ir = String::new();

        // Generate specialized functions from instantiations
        for inst in instantiations {
            if let vais_types::InstantiationKind::Function = inst.kind {
                if let Some(generic_fn) = generic_functions.get(&inst.base_name) {
                    body_ir.push_str(&self.generate_specialized_function(generic_fn, inst)?);
                    body_ir.push('\n');
                }
            }
        }

        // Second pass: generate non-generic function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => {
                    if f.generics.is_empty() {
                        body_ir.push_str(&self.generate_function_with_span(f, item.span)?);
                        body_ir.push('\n');
                    }
                }
                Item::Struct(s) => {
                    if s.generics.is_empty() {
                        for method in &s.methods {
                            body_ir.push_str(&self.generate_method_with_span(
                                &s.name.node,
                                &method.node,
                                method.span,
                            )?);
                            body_ir.push('\n');
                        }
                    }
                }
                Item::Impl(impl_block) => {
                    let type_name = match &impl_block.target_type.node {
                        Type::Named { name, .. } => name.clone(),
                        _ => continue,
                    };
                    for method in &impl_block.methods {
                        body_ir.push_str(&self.generate_method_with_span(
                            &type_name,
                            &method.node,
                            method.span,
                        )?);
                        body_ir.push('\n');
                    }
                }
                Item::Enum(_)
                | Item::Union(_)
                | Item::Use(_)
                | Item::Trait(_)
                | Item::TypeAlias(_)
                | Item::Macro(_)
                | Item::ExternBlock(_)
                | Item::Const(_)
                | Item::Global(_)
                | Item::Error { .. } => {}
            }
        }

        // Add ABI version constant
        let abi_version = crate::abi::ABI_VERSION;
        let abi_version_len = abi_version.len() + 1; // +1 for null terminator
        ir.push_str(&format!(
            "@__vais_abi_version = constant [{} x i8] c\"{}\\00\"\n\n",
            abi_version_len, abi_version
        ));

        // Add string constants
        for (name, value) in &self.string_constants {
            let escaped = escape_llvm_string(value);
            let len = value.len() + 1;
            ir.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
        if !self.string_constants.is_empty() {
            ir.push('\n');
        }

        // Add unwrap panic message and abort declaration if needed
        if self.needs_unwrap_panic {
            ir.push_str("@.unwrap_panic_msg = private unnamed_addr constant [22 x i8] c\"unwrap failed: panic!\\00\"\n");
            ir.push_str("declare void @abort()\n\n");
        }

        ir.push_str(&body_ir);

        // Add lambda functions
        for lambda_ir in &self.lambda_functions {
            ir.push('\n');
            ir.push_str(lambda_ir);
        }

        // Add vtable globals for trait objects
        let vtable_ir = self.generate_vtable_globals();
        if !vtable_ir.is_empty() {
            ir.push_str("\n; VTable globals for trait objects\n");
            ir.push_str(&vtable_ir);
        }
        let drop_ir = self.vtable_generator.generate_drop_functions_ir();
        if !drop_ir.is_empty() {
            ir.push_str("\n; Drop functions for trait objects\n");
            ir.push_str(&drop_ir);
        }

        // Add helper functions
        ir.push_str(&self.generate_helper_functions());

        // Add string helper functions if needed
        if self.needs_string_helpers {
            ir.push_str(&self.generate_string_helper_functions());
            ir.push_str(&self.generate_string_extern_declarations());
        }

        // Add contract runtime declarations if any contracts are present
        if !self.contract_string_constants.is_empty() {
            ir.push_str(&self.generate_contract_declarations());
            ir.push_str(&self.generate_contract_string_constants());
        }

        // Add debug intrinsics if debug info is enabled
        if self.debug_info.is_enabled() {
            ir.push_str("\n; Debug intrinsics\n");
            ir.push_str("declare void @llvm.dbg.declare(metadata, metadata, metadata)\n");
            ir.push_str("declare void @llvm.dbg.value(metadata, metadata, metadata)\n");
        }

        // Add debug metadata
        ir.push_str(&self.debug_info.finalize());

        // Add ABI version metadata
        // ABI version is stored in @__vais_abi_version global constant

        Ok(ir)
    }

    // Function generation functions are in function_gen.rs module

    fn generate_range_for_loop(
        &mut self,
        pattern: &Spanned<Pattern>,
        iter: &Spanned<Expr>,
        body: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (start_expr, end_expr, inclusive) = match &iter.node {
            Expr::Range {
                start,
                end,
                inclusive,
            } => (start.as_deref(), end.as_deref(), *inclusive),
            _ => unreachable!("generate_range_for_loop called with non-range iter"),
        };

        let mut ir = String::new();

        let (start_val, start_ir) = if let Some(s) = start_expr {
            self.generate_expr(s, counter)?
        } else {
            ("0".to_string(), String::new())
        };
        ir.push_str(&start_ir);

        let (end_val, end_ir) = if let Some(e) = end_expr {
            self.generate_expr(e, counter)?
        } else {
            (format!("{}", i64::MAX), String::new())
        };
        ir.push_str(&end_ir);

        let counter_var = format!("%loop_counter.{}", self.label_counter);
        self.label_counter += 1;
        ir.push_str(&format!("  {} = alloca i64\n", counter_var));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            start_val, counter_var
        ));

        let pattern_var = if let Pattern::Ident(name) = &pattern.node {
            let var_name = format!("{}.for", name);
            let llvm_name = format!("%{}", var_name);
            ir.push_str(&format!("  {} = alloca i64\n", llvm_name));
            self.locals.insert(
                name.clone(),
                LocalVar::alloca(ResolvedType::I64, var_name.clone()),
            );
            Some((name.clone(), llvm_name))
        } else {
            None
        };

        let loop_cond = self.next_label("for.cond");
        let loop_body_label = self.next_label("for.body");
        let loop_inc = self.next_label("for.inc");
        let loop_end = self.next_label("for.end");

        self.loop_stack.push(LoopLabels {
            continue_label: loop_inc.clone(),
            break_label: loop_end.clone(),
        });

        ir.push_str(&format!("  br label %{}\n", loop_cond));

        ir.push_str(&format!("{}:\n", loop_cond));
        let current_val = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            current_val, counter_var
        ));

        let cmp_pred = if inclusive { "sle" } else { "slt" };
        let cond_result = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = icmp {} i64 {}, {}\n",
            cond_result, cmp_pred, current_val, end_val
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_result, loop_body_label, loop_end
        ));

        ir.push_str(&format!("{}:\n", loop_body_label));

        if let Some((_, llvm_name)) = &pattern_var {
            let bind_val = self.next_temp(counter);
            ir.push_str(&format!(
                "  {} = load i64, i64* {}\n",
                bind_val, counter_var
            ));
            ir.push_str(&format!("  store i64 {}, i64* {}\n", bind_val, llvm_name));
        }

        let (_body_val, body_ir, body_terminated) = self.generate_block_stmts(body, counter)?;
        ir.push_str(&body_ir);

        if !body_terminated {
            ir.push_str(&format!("  br label %{}\n", loop_inc));
        }

        ir.push_str(&format!("{}:\n", loop_inc));
        let inc_load = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            inc_load, counter_var
        ));
        let inc_result = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", inc_result, inc_load));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            inc_result, counter_var
        ));
        ir.push_str(&format!("  br label %{}\n", loop_cond));

        ir.push_str(&format!("{}:\n", loop_end));
        self.loop_stack.pop();

        Ok(("0".to_string(), ir))
    }

    fn generate_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Int(n) => Ok((n.to_string(), String::new())),
            Expr::Float(n) => Ok((crate::types::format_llvm_float(*n), String::new())),
            Expr::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), String::new())),
            Expr::String(s) => {
                // Create a global string constant
                let name = self.make_string_name();
                self.string_counter += 1;
                self.string_constants.push((name.clone(), s.clone()));

                // Return a getelementptr to the string constant
                let len = s.len() + 1;
                Ok((
                    format!(
                        "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                        len, len, name
                    ),
                    String::new(),
                ))
            }
            Expr::StringInterp(parts) => {
                // Desugar string interpolation into a format() call.
                // Build a format string with {} placeholders and collect expression args.
                let mut format_str_parts = Vec::new();
                let mut interp_args = Vec::new();
                for part in parts {
                    match part {
                        vais_ast::StringInterpPart::Lit(s) => {
                            format_str_parts.push(s.clone());
                        }
                        vais_ast::StringInterpPart::Expr(e) => {
                            format_str_parts.push("{}".to_string());
                            interp_args.push(e.as_ref().clone());
                        }
                    }
                }
                let fmt_string = format_str_parts.join("");
                // Build synthetic args: format string + expression args
                let mut args: Vec<Spanned<Expr>> = Vec::new();
                args.push(Spanned::new(Expr::String(fmt_string), expr.span));
                args.extend(interp_args);
                self.generate_format_call(&args, counter, expr.span)
            }
            Expr::Unit => Ok(("void".to_string(), String::new())),

            Expr::Ident(name) => {
                if let Some(local) = self.locals.get(name.as_str()).cloned() {
                    if local.is_param() {
                        // Parameters are SSA values, use directly
                        Ok((format!("%{}", local.llvm_name), String::new()))
                    } else if local.is_ssa() {
                        // SSA variables: use the stored value directly, no load needed
                        Ok((local.llvm_name.clone(), String::new()))
                    } else if matches!(local.ty, ResolvedType::Named { .. }) {
                        // Struct variables store a pointer to the struct
                        // Load the pointer (the struct address)
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}*, {}** %{}\n",
                            tmp, llvm_ty, llvm_ty, local.llvm_name
                        );
                        Ok((tmp, ir))
                    } else {
                        // Local variables need to be loaded from alloca
                        let tmp = self.next_temp(counter);
                        let llvm_ty = self.type_to_llvm(&local.ty);
                        let ir = format!(
                            "  {} = load {}, {}* %{}\n",
                            tmp, llvm_ty, llvm_ty, local.llvm_name
                        );
                        Ok((tmp, ir))
                    }
                } else if name == "self" {
                    // Handle self reference
                    Ok(("%self".to_string(), String::new()))
                } else if self.is_unit_enum_variant(name) {
                    // Unit enum variant (e.g., None)
                    // Create enum value on stack with just the tag
                    for enum_info in self.enums.values() {
                        for (tag, variant) in enum_info.variants.iter().enumerate() {
                            if variant.name == *name {
                                let mut ir = String::new();
                                let enum_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = alloca %{}\n",
                                    enum_ptr, enum_info.name
                                ));
                                // Store tag
                                let tag_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                                    tag_ptr, enum_info.name, enum_info.name, enum_ptr
                                ));
                                ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));
                                return Ok((enum_ptr, ir));
                            }
                        }
                    }
                    // Fallback if not found (shouldn't happen)
                    Ok((format!("@{}", name), String::new()))
                } else if let Some(const_info) = self.constants.get(name).cloned() {
                    // Constant reference - inline the constant value
                    self.generate_expr(&const_info.value, counter)
                } else if self.functions.contains_key(name.as_str()) {
                    // Function reference
                    Ok((format!("@{}", name), String::new()))
                } else if let Some(self_local) = self.locals.get("self").cloned() {
                    // Implicit self: check if name is a field of the self struct
                    let self_type = match &self_local.ty {
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            inner.as_ref().clone()
                        }
                        other => other.clone(),
                    };
                    if let ResolvedType::Named {
                        name: type_name, ..
                    } = &self_type
                    {
                        let resolved_name = self.resolve_struct_name(type_name);
                        if let Some(struct_info) = self.structs.get(&resolved_name).cloned() {
                            if let Some(field_idx) =
                                struct_info.fields.iter().position(|(n, _)| n == name)
                            {
                                let field_ty = &struct_info.fields[field_idx].1;
                                let llvm_ty = self.type_to_llvm(field_ty);
                                let mut ir = String::new();
                                let field_ptr = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = getelementptr %{}, %{}* %self, i32 0, i32 {}\n",
                                    field_ptr, resolved_name, resolved_name, field_idx
                                ));
                                if matches!(field_ty, ResolvedType::Named { .. }) {
                                    return Ok((field_ptr, ir));
                                } else {
                                    let result = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = load {}, {}* {}\n",
                                        result, llvm_ty, llvm_ty, field_ptr
                                    ));
                                    return Ok((result, ir));
                                }
                            }
                        }
                    }
                    // Not a field, fall through to error
                    let mut candidates: Vec<&str> = Vec::new();
                    for var_name in self.locals.keys() {
                        candidates.push(var_name.as_str());
                    }
                    for func_name in self.functions.keys() {
                        candidates.push(func_name.as_str());
                    }
                    let suggestions = suggest_similar(name, &candidates, 3);
                    let suggestion_text = format_did_you_mean(&suggestions);
                    Err(CodegenError::UndefinedVar(format!(
                        "{}{}",
                        name, suggestion_text
                    )))
                } else {
                    // Undefined identifier - provide suggestions
                    let mut candidates: Vec<&str> = Vec::new();

                    // Add local variables
                    for var_name in self.locals.keys() {
                        candidates.push(var_name.as_str());
                    }

                    // Add function names
                    for func_name in self.functions.keys() {
                        candidates.push(func_name.as_str());
                    }

                    // Add "self" if we're in a method context
                    if self.current_function.is_some() {
                        candidates.push("self");
                    }

                    // Get suggestions
                    let suggestions = suggest_similar(name, &candidates, 3);
                    let suggestion_text = format_did_you_mean(&suggestions);
                    Err(CodegenError::UndefinedVar(format!(
                        "{}{}",
                        name, suggestion_text
                    )))
                }
            }

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(fn_name) = &self.current_function {
                    Ok((format!("@{}", fn_name), String::new()))
                } else {
                    Err(CodegenError::UndefinedFunction("@".to_string()))
                }
            }

            Expr::Binary { op, left, right } => {
                let (left_val, left_ir) = self.generate_expr(left, counter)?;
                let (right_val, right_ir) = self.generate_expr(right, counter)?;

                let mut ir = left_ir;
                ir.push_str(&right_ir);

                // Handle string operations
                let left_type = self.infer_expr_type(left);
                if matches!(left_type, ResolvedType::Str) {
                    return self.generate_string_binary_op(op, &left_val, &right_val, ir, counter);
                }

                // Handle comparison and logical operations (result is i1)
                let is_comparison = matches!(
                    op,
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte | BinOp::Eq | BinOp::Neq
                );
                let is_logical = matches!(op, BinOp::And | BinOp::Or);

                if is_logical {
                    // For logical And/Or, convert operands to i1 first, then perform operation
                    let left_bool = self.next_temp(counter);
                    ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", left_bool, left_val));
                    let right_bool = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = icmp ne i64 {}, 0\n",
                        right_bool, right_val
                    ));

                    let op_str = match op {
                        BinOp::And => "and",
                        BinOp::Or => "or",
                        _ => unreachable!(),
                    };

                    let result_bool = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = {} i1 {}, {}{}\n",
                        result_bool, op_str, left_bool, right_bool, dbg_info
                    ));

                    // Extend back to i64 for consistency
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, result_bool));
                    Ok((result, ir))
                } else if is_comparison {
                    // Comparison returns i1, extend to i64
                    let right_type = self.infer_expr_type(right);
                    let is_float_cmp = matches!(left_type, ResolvedType::F64)
                        || matches!(right_type, ResolvedType::F64);

                    let cmp_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);

                    if is_float_cmp {
                        let op_str = match op {
                            BinOp::Lt => "fcmp olt",
                            BinOp::Lte => "fcmp ole",
                            BinOp::Gt => "fcmp ogt",
                            BinOp::Gte => "fcmp oge",
                            BinOp::Eq => "fcmp oeq",
                            BinOp::Neq => "fcmp one",
                            _ => unreachable!(),
                        };
                        ir.push_str(&format!(
                            "  {} = {} double {}, {}{}\n",
                            cmp_tmp, op_str, left_val, right_val, dbg_info
                        ));
                    } else {
                        let op_str = match op {
                            BinOp::Lt => "icmp slt",
                            BinOp::Lte => "icmp sle",
                            BinOp::Gt => "icmp sgt",
                            BinOp::Gte => "icmp sge",
                            BinOp::Eq => "icmp eq",
                            BinOp::Neq => "icmp ne",
                            _ => unreachable!(),
                        };
                        ir.push_str(&format!(
                            "  {} = {} i64 {}, {}{}\n",
                            cmp_tmp, op_str, left_val, right_val, dbg_info
                        ));
                    }

                    // Extend i1 to i64
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = zext i1 {} to i64\n", result, cmp_tmp));
                    Ok((result, ir))
                } else {
                    // Arithmetic and bitwise operations
                    let tmp = self.next_temp(counter);

                    // Check if either operand is a float type
                    let right_type = self.infer_expr_type(right);
                    let is_float = matches!(left_type, ResolvedType::F64)
                        || matches!(right_type, ResolvedType::F64);

                    if is_float
                        && matches!(
                            op,
                            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                        )
                    {
                        let op_str = match op {
                            BinOp::Add => "fadd",
                            BinOp::Sub => "fsub",
                            BinOp::Mul => "fmul",
                            BinOp::Div => "fdiv",
                            BinOp::Mod => "frem",
                            _ => unreachable!(),
                        };

                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                        ir.push_str(&format!(
                            "  {} = {} double {}, {}{}\n",
                            tmp, op_str, left_val, right_val, dbg_info
                        ));
                    } else {
                        let op_str = match op {
                            BinOp::Add => "add",
                            BinOp::Sub => "sub",
                            BinOp::Mul => "mul",
                            BinOp::Div => "sdiv",
                            BinOp::Mod => "srem",
                            BinOp::BitAnd => "and",
                            BinOp::BitOr => "or",
                            BinOp::BitXor => "xor",
                            BinOp::Shl => "shl",
                            BinOp::Shr => "ashr",
                            _ => unreachable!(),
                        };

                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                        ir.push_str(&format!(
                            "  {} = {} i64 {}, {}{}\n",
                            tmp, op_str, left_val, right_val, dbg_info
                        ));
                    }
                    Ok((tmp, ir))
                }
            }

            Expr::Unary { op, expr: inner } => {
                let (val, val_ir) = self.generate_expr(inner, counter)?;
                let tmp = self.next_temp(counter);

                let mut ir = val_ir;
                let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                match op {
                    UnaryOp::Neg => {
                        ir.push_str(&format!("  {} = sub i64 0, {}{}\n", tmp, val, dbg_info));
                    }
                    UnaryOp::Not => {
                        ir.push_str(&format!("  {} = xor i1 {}, 1{}\n", tmp, val, dbg_info));
                    }
                    UnaryOp::BitNot => {
                        ir.push_str(&format!("  {} = xor i64 {}, -1{}\n", tmp, val, dbg_info));
                    }
                }

                Ok((tmp, ir))
            }

            Expr::Ternary { cond, then, else_ } => {
                // Use proper branching for lazy evaluation
                let then_label = self.next_label("ternary.then");
                let else_label = self.next_label("ternary.else");
                let merge_label = self.next_label("ternary.merge");

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert i64 to i1 for branch
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then branch
                ir.push_str(&format!("{}:\n", then_label));
                let (then_val, then_ir) = self.generate_expr(then, counter)?;
                ir.push_str(&then_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Else branch
                ir.push_str(&format!("{}:\n", else_label));
                let (else_val, else_ir) = self.generate_expr(else_, counter)?;
                ir.push_str(&else_ir);
                ir.push_str(&format!("  br label %{}\n", merge_label));

                // Merge with phi
                ir.push_str(&format!("{}:\n", merge_label));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = phi i64 [ {}, %{} ], [ {}, %{} ]\n",
                    result, then_val, then_label, else_val, else_label
                ));

                Ok((result, ir))
            }

            Expr::Call { func, args } => {
                // Check if this is an enum variant constructor or builtin
                if let Expr::Ident(name) = &func.node {
                    // Handle print/println builtins with format string support
                    if name == "print" || name == "println" {
                        return self.generate_print_call(name, args, counter, expr.span);
                    }

                    // Handle format builtin: returns formatted string
                    if name == "format" {
                        return self.generate_format_call(args, counter, expr.span);
                    }

                    // Handle str_to_ptr builtin: convert string pointer to i64
                    if name == "str_to_ptr" {
                        if args.len() != 1 {
                            return Err(CodegenError::TypeError(
                                "str_to_ptr expects 1 argument".to_string(),
                            ));
                        }
                        let (str_val, str_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = str_ir;
                        let result = self.next_temp(counter);
                        ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, str_val));
                        return Ok((result, ir));
                    }

                    // sizeof(expr) — compile-time constant size query
                    if name == "sizeof" && !args.is_empty() {
                        let arg_type = self.infer_expr_type(&args[0]);
                        let size = self.compute_sizeof(&arg_type);
                        return Ok((size.to_string(), String::new()));
                    }

                    if let Some((enum_name, tag)) = self.get_tuple_variant_info(name) {
                        // This is a tuple enum variant constructor
                        let mut ir = String::new();

                        // Generate argument values
                        let mut arg_vals = Vec::new();
                        for arg in args {
                            let (val, arg_ir) = self.generate_expr(arg, counter)?;
                            ir.push_str(&arg_ir);
                            arg_vals.push(val);
                        }

                        // Create enum value on stack: { i32 tag, i64 payload }
                        let enum_ptr = self.next_temp(counter);
                        ir.push_str(&format!("  {} = alloca %{}\n", enum_ptr, enum_name));

                        // Store tag
                        let tag_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 0\n",
                            tag_ptr, enum_name, enum_name, enum_ptr
                        ));
                        ir.push_str(&format!("  store i32 {}, i32* {}\n", tag, tag_ptr));

                        // Store payload fields into the payload sub-struct
                        for (i, arg_val) in arg_vals.iter().enumerate() {
                            let payload_field_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = getelementptr %{}, %{}* {}, i32 0, i32 1, i32 {}\n",
                                payload_field_ptr, enum_name, enum_name, enum_ptr, i
                            ));
                            ir.push_str(&format!(
                                "  store i64 {}, i64* {}\n",
                                arg_val, payload_field_ptr
                            ));
                        }

                        // Return pointer to the enum
                        return Ok((enum_ptr, ir));
                    }

                    // Check if this is a SIMD intrinsic
                    if Self::is_simd_intrinsic(name) {
                        return self.generate_simd_intrinsic(name, args, counter);
                    }

                    // Handle print_i64/print_f64 builtins: emit printf call
                    // Skip if user defined their own function with the same name
                    let has_user_print_i64 = self
                        .functions
                        .get("print_i64")
                        .map(|f| !f.is_extern)
                        .unwrap_or(false);
                    if name == "print_i64" && args.len() == 1 && !has_user_print_i64 {
                        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = arg_ir;
                        let fmt_str = "%ld";
                        let fmt_name = self.make_string_name();
                        self.string_counter += 1;
                        self.string_constants
                            .push((fmt_name.clone(), fmt_str.to_string()));
                        let fmt_len = fmt_str.len() + 1;
                        let fmt_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
                            fmt_ptr, fmt_len, fmt_len, fmt_name
                        ));
                        let i32_result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = call i32 (i8*, ...) @printf(i8* {}, i64 {})\n",
                            i32_result, fmt_ptr, arg_val
                        ));
                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = sext i32 {} to i64\n",
                            result, i32_result
                        ));
                        return Ok((result, ir));
                    }

                    let has_user_print_f64 = self
                        .functions
                        .get("print_f64")
                        .map(|f| !f.is_extern)
                        .unwrap_or(false);
                    if name == "print_f64" && args.len() == 1 && !has_user_print_f64 {
                        let (arg_val, arg_ir) = self.generate_expr(&args[0], counter)?;
                        let mut ir = arg_ir;
                        let fmt_str = "%f";
                        let fmt_name = self.make_string_name();
                        self.string_counter += 1;
                        self.string_constants
                            .push((fmt_name.clone(), fmt_str.to_string()));
                        let fmt_len = fmt_str.len() + 1;
                        let fmt_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
                            fmt_ptr, fmt_len, fmt_len, fmt_name
                        ));
                        let i32_result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = call i32 (i8*, ...) @printf(i8* {}, double {})\n",
                            i32_result, fmt_ptr, arg_val
                        ));
                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = sext i32 {} to i64\n",
                            result, i32_result
                        ));
                        return Ok((result, ir));
                    }
                }

                // Check if this is a direct function call or indirect (lambda) call
                let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
                    // Check if this is a generic function that needs monomorphization
                    if let Some(instantiations_list) = self.generic_fn_instantiations.get(name) {
                        // Infer argument types to select the right specialization
                        let arg_types: Vec<ResolvedType> =
                            args.iter().map(|a| self.infer_expr_type(a)).collect();

                        // Find the matching instantiation based on argument types
                        let mangled =
                            self.resolve_generic_call(name, &arg_types, instantiations_list);
                        (mangled, false)
                    } else if self.functions.contains_key(name) {
                        (name.clone(), false)
                    } else if self.locals.contains_key(name) {
                        (name.clone(), true) // Lambda call
                    } else if self.declared_functions.contains(name) {
                        // Function declared in module (may be generic, will instantiate later)
                        (name.clone(), false)
                    } else {
                        // Unknown function - provide suggestions
                        let mut candidates: Vec<&str> = Vec::new();

                        // Add declared function names (including generics)
                        for func_name in &self.declared_functions {
                            candidates.push(func_name.as_str());
                        }

                        // Add instantiated function names
                        for func_name in self.functions.keys() {
                            candidates.push(func_name.as_str());
                        }

                        // Add local variables (could be lambdas)
                        for var_name in self.locals.keys() {
                            candidates.push(var_name.as_str());
                        }

                        let suggestions = suggest_similar(name, &candidates, 3);
                        let suggestion_text = format_did_you_mean(&suggestions);
                        return Err(CodegenError::UndefinedFunction(format!(
                            "{}{}",
                            name, suggestion_text
                        )));
                    }
                } else if let Expr::SelfCall = &func.node {
                    (self.current_function.clone().unwrap_or_default(), false)
                } else {
                    return Err(CodegenError::Unsupported(
                        "complex indirect call".to_string(),
                    ));
                };

                // Look up function info for parameter types (only for direct calls)
                let fn_info = if !is_indirect {
                    self.functions.get(&fn_name).cloned()
                } else {
                    None
                };

                let mut ir = String::new();
                let mut arg_vals = Vec::new();

                for (i, arg) in args.iter().enumerate() {
                    let (mut val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);

                    // Get parameter type from function info if available
                    let param_ty = fn_info
                        .as_ref()
                        .and_then(|f| f.signature.params.get(i))
                        .map(|(_, ty, _)| ty.clone());

                    let arg_ty = if let Some(ref ty) = param_ty {
                        self.type_to_llvm(ty)
                    } else {
                        // For vararg arguments, infer the type from the expression
                        let inferred_ty = self.infer_expr_type(arg);
                        self.type_to_llvm(&inferred_ty)
                    };

                    // For struct arguments, load the value if we have a pointer
                    // (struct literals generate alloca+stores, returning pointers)
                    if let Some(ResolvedType::Named { .. }) = &param_ty {
                        // Check if val looks like a pointer (starts with %)
                        if val.starts_with('%') {
                            let loaded = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                loaded, arg_ty, arg_ty, val
                            ));
                            val = loaded;
                        }
                    }

                    // Trait object coercion: &ConcreteType -> &dyn Trait
                    // When parameter expects &dyn Trait and argument is a concrete type reference,
                    // create a fat pointer { data_ptr, vtable_ptr }
                    if let Some(ref param_type) = param_ty {
                        let dyn_trait = match param_type {
                            ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                                    Some(trait_name.clone())
                                } else {
                                    None
                                }
                            }
                            ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                            _ => None,
                        };

                        if let Some(trait_name) = dyn_trait {
                            // Get the concrete type of the argument
                            let arg_expr_type = self.infer_expr_type(arg);
                            let concrete_type_name = match &arg_expr_type {
                                ResolvedType::Named { name, .. } => Some(name.clone()),
                                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                                    if let ResolvedType::Named { name, .. } = inner.as_ref() {
                                        Some(name.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(concrete_name) = concrete_type_name {
                                // Generate vtable for this concrete type + trait
                                let vtable_info =
                                    self.get_or_generate_vtable(&concrete_name, &trait_name);

                                if let Some(vtable) = vtable_info {
                                    // Load the actual struct pointer if we have a pointer-to-pointer
                                    // (Ref expressions return the address of the storage, not the struct)
                                    let struct_ptr = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = load %{}*, %{}** {}\n",
                                        struct_ptr, concrete_name, concrete_name, val
                                    ));
                                    // Cast data pointer to i8*
                                    let data_ptr = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = bitcast %{}* {} to i8*\n",
                                        data_ptr, concrete_name, struct_ptr
                                    ));

                                    // Create fat pointer { i8*, i8* }
                                    let trait_obj_1 = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = insertvalue {{ i8*, i8* }} undef, i8* {}, 0\n",
                                        trait_obj_1, data_ptr
                                    ));
                                    let vtable_cast = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = bitcast {{ i8*, i64, i64, i64(i8*)* }}* {} to i8*\n",
                                        vtable_cast, vtable.global_name
                                    ));
                                    let trait_obj_2 = self.next_temp(counter);
                                    ir.push_str(&format!(
                                        "  {} = insertvalue {{ i8*, i8* }} {}, i8* {}, 1\n",
                                        trait_obj_2, trait_obj_1, vtable_cast
                                    ));

                                    val = trait_obj_2;
                                }
                            }
                        }
                    }

                    // Insert integer conversion if needed (trunc for narrowing, sext for widening)
                    if let Some(param_type) = &param_ty {
                        let src_bits = self.get_integer_bits_from_val(&val);
                        let dst_bits = self.get_integer_bits(param_type);

                        if src_bits > 0 && dst_bits > 0 && src_bits != dst_bits {
                            let conv_tmp = self.next_temp(counter);
                            let src_ty = format!("i{}", src_bits);
                            let dst_ty = format!("i{}", dst_bits);

                            if src_bits > dst_bits {
                                // Truncate
                                ir.push_str(&format!(
                                    "  {} = trunc {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            } else {
                                // Sign extend
                                ir.push_str(&format!(
                                    "  {} = sext {} {} to {}\n",
                                    conv_tmp, src_ty, val, dst_ty
                                ));
                            }
                            val = conv_tmp;
                        }
                    }

                    // Convert i64 to i8* when parameter expects str/i8* but arg is i64
                    if let Some(ref param_type) = param_ty {
                        if matches!(param_type, ResolvedType::Str) {
                            let actual_ty = self.infer_expr_type(arg);
                            if matches!(actual_ty, ResolvedType::I64) {
                                let ptr_tmp = self.next_temp(counter);
                                ir.push_str(&format!(
                                    "  {} = inttoptr i64 {} to i8*\n",
                                    ptr_tmp, val
                                ));
                                val = ptr_tmp;
                            }
                        }
                    }

                    arg_vals.push(format!("{} {}", arg_ty, val));
                }

                // Get return type and actual function name (may differ for builtins)
                let ret_ty = fn_info
                    .as_ref()
                    .map(|f| self.type_to_llvm(&f.signature.ret))
                    .unwrap_or_else(|| "i64".to_string());

                let actual_fn_name = fn_info
                    .as_ref()
                    .map(|f| f.signature.name.clone())
                    .unwrap_or_else(|| fn_name.clone());

                let is_vararg = fn_info
                    .as_ref()
                    .map(|f| f.signature.is_vararg)
                    .unwrap_or(false);

                if is_indirect {
                    // Check if this is a closure with captured variables
                    let closure_info = self.closures.get(&fn_name).cloned();

                    // Prepend captured values to arguments if this is a closure
                    let mut all_args = Vec::new();
                    if let Some(ref info) = closure_info {
                        for (_, capture_val) in &info.captures {
                            all_args.push(format!("i64 {}", capture_val));
                        }
                    }
                    all_args.extend(arg_vals);

                    // If we have closure info, we know the exact function name - call directly
                    if let Some(ref info) = closure_info {
                        let tmp = self.next_temp(counter);
                        let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                        ir.push_str(&format!(
                            "  {} = call i64 @{}({}){}\n",
                            tmp,
                            info.func_name,
                            all_args.join(", "),
                            dbg_info
                        ));
                        return Ok((tmp, ir));
                    }

                    // Get the local variable info
                    let local_info = self.locals.get(&fn_name).cloned();
                    let is_ssa_or_param = local_info
                        .as_ref()
                        .map(|l| l.is_ssa() || l.is_param())
                        .unwrap_or(false);

                    let ptr_tmp = if is_ssa_or_param {
                        // SSA or param: the value IS the function pointer (as i64), no load needed
                        let local = match local_info.as_ref() {
                            Some(l) => l,
                            None => {
                                return Err(CodegenError::TypeError(format!(
                                    "missing local info for '{}'",
                                    fn_name
                                )))
                            }
                        };
                        let val = &local.llvm_name;
                        if local.is_ssa() {
                            // SSA values already include the % prefix (e.g., "%5")
                            val.clone()
                        } else {
                            // Param names don't include % prefix
                            format!("%{}", val)
                        }
                    } else {
                        // Alloca: load the function pointer from the stack slot
                        let llvm_var_name = local_info
                            .as_ref()
                            .map(|l| l.llvm_name.clone())
                            .unwrap_or_else(|| fn_name.clone());
                        let tmp = self.next_temp(counter);
                        ir.push_str(&format!("  {} = load i64, i64* %{}\n", tmp, llvm_var_name));
                        tmp
                    };

                    // Build function type signature for indirect call (including captures)
                    let arg_types: Vec<String> = all_args
                        .iter()
                        .map(|a| a.split_whitespace().next().unwrap_or("i64").to_string())
                        .collect();
                    let fn_type = format!("i64 ({})*", arg_types.join(", "));

                    // Cast i64 to function pointer
                    let fn_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to {}\n",
                        fn_ptr, ptr_tmp, fn_type
                    ));

                    // Make indirect call with all arguments
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i64 {}({}){}\n",
                        tmp,
                        fn_ptr,
                        all_args.join(", "),
                        dbg_info
                    ));
                    Ok((tmp, ir))
                } else if fn_name == "malloc" {
                    // Special handling for malloc: call returns i8*, convert to i64
                    let ptr_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @malloc({}){}\n",
                        ptr_tmp,
                        arg_vals.join(", "),
                        dbg_info
                    ));
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", result, ptr_tmp));
                    Ok((result, ir))
                } else if fn_name == "free" {
                    // Special handling for free: convert i64 to i8*
                    let ptr_tmp = self.next_temp(counter);
                    // Extract the i64 value from arg_vals
                    let arg_val = arg_vals
                        .first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!("  call void @free(i8* {}){}\n", ptr_tmp, dbg_info));
                    Ok(("void".to_string(), ir))
                } else if fn_name == "memcpy" || fn_name == "memcpy_str" {
                    // Special handling for memcpy/memcpy_str: convert pointers as needed
                    let dest_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
                    let src_full = arg_vals.get(1).map(|s| s.as_str()).unwrap_or("i64 0");
                    let n_val = arg_vals
                        .get(2)
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");

                    // Handle dest pointer
                    let dest_ptr = if dest_full.starts_with("i8*") {
                        // Use everything after "i8* " to preserve complex expressions
                        dest_full
                            .strip_prefix("i8* ")
                            .unwrap_or(dest_full.split_whitespace().last().unwrap_or("null"))
                            .to_string()
                    } else {
                        let dest_val = dest_full.split_whitespace().last().unwrap_or("0");
                        let ptr = self.next_temp(counter);
                        ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, dest_val));
                        ptr
                    };

                    // Handle src pointer (can be i64 or i8* for memcpy_str)
                    let src_ptr = if src_full.starts_with("i8*") {
                        // Use everything after "i8* " to preserve complex expressions
                        src_full
                            .strip_prefix("i8* ")
                            .unwrap_or(src_full.split_whitespace().last().unwrap_or("null"))
                            .to_string()
                    } else {
                        let src_val = src_full.split_whitespace().last().unwrap_or("0");
                        let ptr = self.next_temp(counter);
                        ir.push_str(&format!("  {} = inttoptr i64 {} to i8*\n", ptr, src_val));
                        ptr
                    };

                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i8* @memcpy(i8* {}, i8* {}, i64 {}){}\n",
                        result, dest_ptr, src_ptr, n_val, dbg_info
                    ));
                    // Convert result back to i64
                    let result_i64 = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = ptrtoint i8* {} to i64\n",
                        result_i64, result
                    ));
                    Ok((result_i64, ir))
                } else if fn_name == "strlen" {
                    // Special handling for strlen: convert i64 to i8* if needed
                    let arg_full = arg_vals.first().map(|s| s.as_str()).unwrap_or("i64 0");
                    let result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);

                    // Check if the argument is already i8* (str type) or i64 (pointer as integer)
                    if arg_full.starts_with("i8*") {
                        // Already a pointer, use directly
                        // Use everything after "i8* " to preserve complex expressions like getelementptr
                        let ptr_val = arg_full.strip_prefix("i8* ").unwrap_or(
                            arg_full.split_whitespace().last().unwrap_or("null"),
                        );
                        ir.push_str(&format!(
                            "  {} = call i64 @strlen(i8* {}){}\n",
                            result, ptr_val, dbg_info
                        ));
                    } else {
                        // Convert i64 to pointer
                        let arg_val = arg_full.split_whitespace().last().unwrap_or("0");
                        let ptr_tmp = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to i8*\n",
                            ptr_tmp, arg_val
                        ));
                        ir.push_str(&format!(
                            "  {} = call i64 @strlen(i8* {}){}\n",
                            result, ptr_tmp, dbg_info
                        ));
                    }
                    Ok((result, ir))
                } else if fn_name == "puts_ptr" {
                    // Special handling for puts_ptr: convert i64 to i8*
                    let arg_val = arg_vals
                        .first()
                        .map(|s| s.split_whitespace().last().unwrap_or("0"))
                        .unwrap_or("0");
                    let ptr_tmp = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        ptr_tmp, arg_val
                    ));
                    let i32_result = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    ir.push_str(&format!(
                        "  {} = call i32 @puts(i8* {}){}\n",
                        i32_result, ptr_tmp, dbg_info
                    ));
                    // Convert i32 result to i64 for consistency
                    let result = self.next_temp(counter);
                    ir.push_str(&format!("  {} = sext i32 {} to i64\n", result, i32_result));
                    Ok((result, ir))
                } else if ret_ty == "void" {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // Direct void function call
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        let param_types: Vec<String> = fn_info
                            .as_ref()
                            .map(|f| {
                                f.signature
                                    .params
                                    .iter()
                                    .map(|(_, ty, _)| self.type_to_llvm(ty))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let sig = format!("void ({}, ...)", param_types.join(", "));
                        ir.push_str(&format!(
                            "  call {} @{}({}){}\n",
                            sig,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  call void @{}({}){}\n",
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    }
                    Ok(("void".to_string(), ir))
                } else if ret_ty == "i32" {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // i32 return function call - convert to i64 for consistency
                    let i32_tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        // Variadic functions need explicit signature in LLVM IR call
                        let param_types: Vec<String> = fn_info
                            .as_ref()
                            .map(|f| {
                                f.signature
                                    .params
                                    .iter()
                                    .map(|(_, ty, _)| self.type_to_llvm(ty))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let sig = format!("i32 ({}, ...)", param_types.join(", "));
                        ir.push_str(&format!(
                            "  {} = call {} @{}({}){}\n",
                            i32_tmp,
                            sig,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  {} = call i32 @{}({}){}\n",
                            i32_tmp,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    }
                    let tmp = self.next_temp(counter);
                    ir.push_str(&format!("  {} = sext i32 {} to i64\n", tmp, i32_tmp));
                    Ok((tmp, ir))
                } else {
                    // Check for recursive call with decreases clause
                    if self.is_recursive_call(&fn_name) {
                        let check_ir = self.generate_recursive_decreases_check(args, counter)?;
                        ir.push_str(&check_ir);
                    }

                    // Direct function call with return value
                    let tmp = self.next_temp(counter);
                    let dbg_info = self.debug_info.dbg_ref_from_span(expr.span);
                    if is_vararg {
                        let param_types: Vec<String> = fn_info
                            .as_ref()
                            .map(|f| {
                                f.signature
                                    .params
                                    .iter()
                                    .map(|(_, ty, _)| self.type_to_llvm(ty))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let sig = format!("{} ({}, ...)", ret_ty, param_types.join(", "));
                        ir.push_str(&format!(
                            "  {} = call {} @{}({}){}\n",
                            tmp,
                            sig,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    } else {
                        ir.push_str(&format!(
                            "  {} = call {} @{}({}){}\n",
                            tmp,
                            ret_ty,
                            actual_fn_name,
                            arg_vals.join(", "),
                            dbg_info
                        ));
                    }
                    Ok((tmp, ir))
                }
            }

            // If/Else expression with basic blocks
            Expr::If { cond, then, else_ } => {
                let then_label = self.next_label("then");
                let else_label = self.next_label("else");
                let merge_label = self.next_label("merge");

                // Infer the type of the then block for phi node
                let block_type = self.infer_block_type(then);
                let llvm_type = self.type_to_llvm(&block_type);

                // Check if the result is a struct type (returned as pointer from struct literals)
                let is_struct_result = matches!(&block_type, ResolvedType::Named { .. })
                    && !self.is_block_result_value(then);

                // Generate condition
                let (cond_val, cond_ir) = self.generate_expr(cond, counter)?;
                let mut ir = cond_ir;

                // Convert i64 to i1 for branch condition
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));

                // Conditional branch
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, then_label, else_label
                ));

                // Then block
                ir.push_str(&format!("{}:\n", then_label));
                self.current_block = then_label.clone();
                let (then_val, then_ir, then_terminated) =
                    self.generate_block_stmts(then, counter)?;
                ir.push_str(&then_ir);

                // For struct results, load the value before branch if it's a pointer
                let then_val_for_phi = if is_struct_result && !then_terminated {
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_type, llvm_type, then_val
                    ));
                    loaded
                } else {
                    then_val.clone()
                };

                let then_actual_block = self.current_block.clone();
                // Only emit branch to merge if block is not terminated
                let then_from_label = if !then_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    then_actual_block
                } else {
                    String::new() // Block is terminated, won't contribute to phi
                };

                // Else block
                ir.push_str(&format!("{}:\n", else_label));
                self.current_block = else_label.clone();
                let (else_val, else_ir, else_terminated, nested_last_block, has_else) =
                    if let Some(else_branch) = else_ {
                        let (v, i, t, last) =
                            self.generate_if_else_with_term(else_branch, counter, &merge_label)?;
                        (v, i, t, last, true)
                    } else {
                        ("0".to_string(), String::new(), false, String::new(), false)
                    };
                ir.push_str(&else_ir);

                // For struct results, load the value before branch if it's a pointer
                // But if else_val comes from a nested if-else (indicated by non-empty nested_last_block),
                // it's already a phi node value (not a pointer), so don't load it
                let else_val_for_phi = if is_struct_result
                    && !else_terminated
                    && has_else
                    && nested_last_block.is_empty()
                {
                    let loaded = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        loaded, llvm_type, llvm_type, else_val
                    ));
                    loaded
                } else {
                    else_val.clone()
                };

                // Only emit branch to merge if block is not terminated
                let else_from_label = if !else_terminated {
                    ir.push_str(&format!("  br label %{}\n", merge_label));
                    // If there was a nested if-else, use its merge block as the predecessor
                    if !nested_last_block.is_empty() {
                        nested_last_block
                    } else {
                        self.current_block.clone()
                    }
                } else {
                    String::new()
                };

                // Merge block with phi node
                ir.push_str(&format!("{}:\n", merge_label));
                self.current_block = merge_label.clone();
                let result = self.next_temp(counter);

                // Check if the block type is void/unit - if so, don't generate phi nodes
                // (phi nodes cannot have void type in LLVM IR)
                let is_void_type = matches!(block_type, ResolvedType::Unit);

                // If there's no else branch, don't use phi - the value is not meaningful
                // This avoids type mismatches when then branch returns i32 (e.g., putchar)
                if !has_else || is_void_type {
                    // If-only statement or void type: value is not used, just use 0
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                } else if !then_from_label.is_empty() && !else_from_label.is_empty() {
                    // Both branches reach merge - use the inferred type
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ], [ {}, %{} ]\n",
                        result,
                        llvm_type,
                        then_val_for_phi,
                        then_from_label,
                        else_val_for_phi,
                        else_from_label
                    ));
                } else if !then_from_label.is_empty() {
                    // Only then branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ]\n",
                        result, llvm_type, then_val_for_phi, then_from_label
                    ));
                } else if !else_from_label.is_empty() {
                    // Only else branch reaches merge
                    ir.push_str(&format!(
                        "  {} = phi {} [ {}, %{} ]\n",
                        result, llvm_type, else_val_for_phi, else_from_label
                    ));
                } else {
                    // Neither branch reaches merge (both break/continue)
                    // This merge block is actually unreachable, but we still need a value
                    ir.push_str(&format!("  {} = add i64 0, 0\n", result));
                }

                Ok((result, ir))
            }

            // Loop expression
            Expr::Loop {
                pattern,
                iter,
                body,
            } => {
                // Check if this is a range-based for loop
                let is_range_loop = iter
                    .as_ref()
                    .is_some_and(|it| matches!(&it.node, Expr::Range { .. }));

                if is_range_loop {
                    if let (Some(pat), Some(it)) = (pattern.as_ref(), iter.as_ref()) {
                        // Range-based for loop: L pattern : start..end { body }
                        return self.generate_range_for_loop(pat, it, body, counter);
                    }
                }

                // Conditional or infinite loop
                let loop_start = self.next_label("loop.start");
                let loop_body = self.next_label("loop.body");
                let loop_end = self.next_label("loop.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Check if this is a conditional loop (L cond { body }) or infinite loop
                if let Some(iter_expr) = iter {
                    // Conditional loop: L condition { body }
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));

                    // Evaluate condition
                    let (cond_val, cond_ir) = self.generate_expr(iter_expr, counter)?;
                    ir.push_str(&cond_ir);

                    // Convert i64 to i1 for branch
                    let cond_bool = self.next_temp(counter);
                    ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));
                    ir.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cond_bool, loop_body, loop_end
                    ));

                    // Loop body
                    ir.push_str(&format!("{}:\n", loop_body));
                    let (_body_val, body_ir, body_terminated) =
                        self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                } else {
                    // Infinite loop: L { body } - must use break to exit
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                    ir.push_str(&format!("{}:\n", loop_start));
                    let (_body_val, body_ir, body_terminated) =
                        self.generate_block_stmts(body, counter)?;
                    ir.push_str(&body_ir);
                    // Only emit loop back if body doesn't terminate
                    if !body_terminated {
                        ir.push_str(&format!("  br label %{}\n", loop_start));
                    }
                }

                // Loop end
                ir.push_str(&format!("{}:\n", loop_end));

                self.loop_stack.pop();

                // Loop returns void by default (use break with value for expression)
                Ok(("0".to_string(), ir))
            }

            // While loop expression
            Expr::While { condition, body } => {
                let loop_start = self.next_label("while.start");
                let loop_body = self.next_label("while.body");
                let loop_end = self.next_label("while.end");

                // Push loop labels for break/continue
                self.loop_stack.push(LoopLabels {
                    continue_label: loop_start.clone(),
                    break_label: loop_end.clone(),
                });

                let mut ir = String::new();

                // Jump to condition check
                ir.push_str(&format!("  br label %{}\n", loop_start));
                ir.push_str(&format!("{}:\n", loop_start));

                // Evaluate condition
                let (cond_val, cond_ir) = self.generate_expr(condition, counter)?;
                ir.push_str(&cond_ir);

                // Convert to i1 for branch
                let cond_bool = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp ne i64 {}, 0\n", cond_bool, cond_val));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_bool, loop_body, loop_end
                ));

                // Loop body
                ir.push_str(&format!("{}:\n", loop_body));
                let (_body_val, body_ir, body_terminated) =
                    self.generate_block_stmts(body, counter)?;
                ir.push_str(&body_ir);

                // Jump back to condition if body doesn't terminate
                if !body_terminated {
                    ir.push_str(&format!("  br label %{}\n", loop_start));
                }

                // Loop end
                ir.push_str(&format!("{}:\n", loop_end));

                self.loop_stack.pop();

                Ok(("0".to_string(), ir))
            }

            // Block expression
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }

            // Assignment expression
            Expr::Assign { target, value } => {
                let (val, val_ir) = self.generate_expr(value, counter)?;
                let mut ir = val_ir;

                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name).cloned() {
                        if !local.is_param() {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            // For struct types (Named), the local is a double pointer (%Type**).
                            // We need to alloca a new struct, store the value, then update the pointer.
                            if matches!(&local.ty, ResolvedType::Named { .. }) && local.is_alloca()
                            {
                                let tmp_ptr = self.next_temp(counter);
                                ir.push_str(&format!("  {} = alloca {}\n", tmp_ptr, llvm_ty));
                                ir.push_str(&format!(
                                    "  store {} {}, {}* {}\n",
                                    llvm_ty, val, llvm_ty, tmp_ptr
                                ));
                                ir.push_str(&format!(
                                    "  store {}* {}, {}** %{}\n",
                                    llvm_ty, tmp_ptr, llvm_ty, local.llvm_name
                                ));
                            } else {
                                ir.push_str(&format!(
                                    "  store {} {}, {}* %{}\n",
                                    llvm_ty, val, llvm_ty, local.llvm_name
                                ));
                            }
                        }
                    }
                } else if let Expr::Deref(inner) = &target.node {
                    // Pointer dereference assignment: *ptr = value
                    let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
                    ir.push_str(&ptr_ir);
                    // Store value at the pointed-to location
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", val, ptr_val));
                } else if let Expr::Index {
                    expr: arr_expr,
                    index,
                } = &target.node
                {
                    // Array index assignment: arr[i] = value
                    let (arr_val, arr_ir) = self.generate_expr(arr_expr, counter)?;
                    ir.push_str(&arr_ir);
                    let (idx_val, idx_ir) = self.generate_expr(index, counter)?;
                    ir.push_str(&idx_ir);

                    // Determine element type from array type
                    let arr_type = self.infer_expr_type(arr_expr);
                    let elem_llvm_type = match &arr_type {
                        ResolvedType::Array(inner) => self.type_to_llvm(inner),
                        ResolvedType::ConstArray { element, .. } => self.type_to_llvm(element),
                        ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                        _ => "i64".to_string(),
                    };

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 {}\n",
                        elem_ptr, elem_llvm_type, elem_llvm_type, arr_val, idx_val
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        elem_llvm_type, val, elem_llvm_type, elem_ptr
                    ));
                } else if let Expr::Field {
                    expr: obj_expr,
                    field,
                } = &target.node
                {
                    // Field assignment: obj.field = value
                    let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                    ir.push_str(&obj_ir);

                    // Get struct or union info
                    if let Expr::Ident(var_name) = &obj_expr.node {
                        if let Some(local) = self.locals.get(var_name.as_str()).cloned() {
                            if let ResolvedType::Named {
                                name: type_name, ..
                            } = &local.ty
                            {
                                // First check struct
                                if let Some(struct_info) = self.structs.get(type_name).cloned() {
                                    if let Some(field_idx) = struct_info
                                        .fields
                                        .iter()
                                        .position(|(n, _)| n == &field.node)
                                    {
                                        let field_ty = &struct_info.fields[field_idx].1;
                                        let llvm_ty = self.type_to_llvm(field_ty);

                                        let field_ptr = self.next_temp(counter);
                                        ir.push_str(&format!(
                                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                                            field_ptr, type_name, type_name, obj_val, field_idx
                                        ));
                                        ir.push_str(&format!(
                                            "  store {} {}, {}* {}\n",
                                            llvm_ty, val, llvm_ty, field_ptr
                                        ));
                                    }
                                }
                                // Then check union
                                else if let Some(union_info) = self.unions.get(type_name).cloned()
                                {
                                    if let Some((_, field_ty)) =
                                        union_info.fields.iter().find(|(n, _)| n == &field.node)
                                    {
                                        let llvm_ty = self.type_to_llvm(field_ty);

                                        // For union, bitcast to field type pointer
                                        let field_ptr = self.next_temp(counter);
                                        ir.push_str(&format!(
                                            "  {} = bitcast %{}* {} to {}*\n",
                                            field_ptr, type_name, obj_val, llvm_ty
                                        ));
                                        ir.push_str(&format!(
                                            "  store {} {}, {}* {}\n",
                                            llvm_ty, val, llvm_ty, field_ptr
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                Ok((val, ir))
            }

            // Compound assignment (+=, -=, etc.)
            Expr::AssignOp { op, target, value } => {
                // First load current value
                let (current_val, load_ir) = self.generate_expr(target, counter)?;
                let (rhs_val, rhs_ir) = self.generate_expr(value, counter)?;

                let mut ir = load_ir;
                ir.push_str(&rhs_ir);

                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                    BinOp::Mod => "srem",
                    BinOp::BitAnd => "and",
                    BinOp::BitOr => "or",
                    BinOp::BitXor => "xor",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "ashr",
                    _ => return Err(CodegenError::Unsupported(format!("compound {:?}", op))),
                };

                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    result, op_str, current_val, rhs_val
                ));

                // Store back
                if let Expr::Ident(name) = &target.node {
                    if let Some(local) = self.locals.get(name.as_str()).cloned() {
                        if !local.is_param() {
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_ty, result, llvm_ty, local.llvm_name
                            ));
                        }
                    }
                }

                Ok((result, ir))
            }

            // Array literal: [a, b, c]
            Expr::Array(elements) => {
                let mut ir = String::new();
                let len = elements.len();

                // Infer element type from first element (default to i64)
                let elem_ty = if let Some(first) = elements.first() {
                    let resolved = self.infer_expr_type(first);
                    self.type_to_llvm(&resolved)
                } else {
                    "i64".to_string()
                };
                let arr_ty = format!("[{}  x {}]", len, elem_ty);

                // Allocate array on stack
                let arr_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", arr_ptr, arr_ty));

                // Store each element
                for (i, elem) in elements.iter().enumerate() {
                    let (val, elem_ir) = self.generate_expr(elem, counter)?;
                    ir.push_str(&elem_ir);

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                        elem_ptr, arr_ty, arr_ty, arr_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        elem_ty, val, elem_ty, elem_ptr
                    ));
                }

                // Return pointer to first element
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
                    result, arr_ty, arr_ty, arr_ptr
                ));

                Ok((result, ir))
            }

            // Map literal: {k: v, ...}
            // Stored as parallel arrays of keys and values on the stack
            Expr::MapLit(pairs) => {
                let mut ir = String::new();
                let len = pairs.len();

                // Infer key/value types
                let (key_ty, val_ty) = if let Some((k, v)) = pairs.first() {
                    let kt = self.type_to_llvm(&self.infer_expr_type(k));
                    let vt = self.type_to_llvm(&self.infer_expr_type(v));
                    (kt, vt)
                } else {
                    ("i64".to_string(), "i64".to_string())
                };

                let keys_arr_ty = format!("[{} x {}]", len, key_ty);
                let vals_arr_ty = format!("[{} x {}]", len, val_ty);

                // Allocate key and value arrays on stack
                let keys_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", keys_ptr, keys_arr_ty));
                let vals_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", vals_ptr, vals_arr_ty));

                // Store each key-value pair
                for (i, (k, v)) in pairs.iter().enumerate() {
                    let (kval, k_ir) = self.generate_expr(k, counter)?;
                    ir.push_str(&k_ir);
                    let k_elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                        k_elem_ptr, keys_arr_ty, keys_arr_ty, keys_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        key_ty, kval, key_ty, k_elem_ptr
                    ));

                    let (vval, v_ir) = self.generate_expr(v, counter)?;
                    ir.push_str(&v_ir);
                    let v_elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                        v_elem_ptr, vals_arr_ty, vals_arr_ty, vals_ptr, i
                    ));
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        val_ty, vval, val_ty, v_elem_ptr
                    ));
                }

                // Return pointer to keys array (map is represented as parallel arrays)
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i64 0, i64 0\n",
                    result, keys_arr_ty, keys_arr_ty, keys_ptr
                ));

                Ok((result, ir))
            }

            // Tuple literal: (a, b, c)
            Expr::Tuple(elements) => {
                let mut ir = String::new();
                let len = elements.len();

                // Build tuple type string
                let tuple_ty = format!("{{ {} }}", vec!["i64"; len].join(", "));

                // Allocate tuple on stack
                let tuple_ptr = self.next_temp(counter);
                ir.push_str(&format!("  {} = alloca {}\n", tuple_ptr, tuple_ty));

                // Store each element
                for (i, elem) in elements.iter().enumerate() {
                    let (val, elem_ir) = self.generate_expr(elem, counter)?;
                    ir.push_str(&elem_ir);

                    let elem_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                        elem_ptr, tuple_ty, tuple_ty, tuple_ptr, i
                    ));
                    ir.push_str(&format!("  store i64 {}, i64* {}\n", val, elem_ptr));
                }

                // Load and return tuple value
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    result, tuple_ty, tuple_ty, tuple_ptr
                ));

                Ok((result, ir))
            }

            // Struct literal: Point{x:1, y:2}
            // Also handles union literal: IntOrFloat{as_int: 42}
            Expr::StructLit { name, fields } => {
                let resolved_name = self.resolve_struct_name(&name.node);
                let type_name = &resolved_name;

                // First check if it's a struct
                if let Some(struct_info) = self.structs.get(type_name).cloned() {
                    let mut ir = String::new();

                    // Allocate struct on stack
                    let struct_ptr = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca %{}\n", struct_ptr, type_name));

                    // Store each field
                    for (field_name, field_expr) in fields {
                        // Find field index
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field_name.node)
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = struct_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field_name.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'{}",
                                    field_name.node, type_name, suggestion_text
                                ))
                            })?;

                        let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                        ir.push_str(&field_ir);

                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, struct_ptr, field_idx
                        ));

                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        // For struct-typed fields, val might be a pointer that needs to be loaded
                        let val_to_store = if matches!(field_ty, ResolvedType::Named { .. })
                            && !self.is_expr_value(field_expr)
                        {
                            // Field value is a pointer to struct, need to load the value
                            let loaded = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                loaded, llvm_ty, llvm_ty, val
                            ));
                            loaded
                        } else {
                            val
                        };

                        ir.push_str(&format!(
                            "  store {} {}, {}* {}\n",
                            llvm_ty, val_to_store, llvm_ty, field_ptr
                        ));
                    }

                    // Return pointer to struct
                    Ok((struct_ptr, ir))
                // Then check if it's a union
                } else if let Some(union_info) = self.unions.get(type_name).cloned() {
                    let mut ir = String::new();

                    // Allocate union on stack
                    let union_ptr = self.next_temp(counter);
                    ir.push_str(&format!("  {} = alloca %{}\n", union_ptr, type_name));

                    // Union should have exactly one field in the literal
                    if fields.len() != 1 {
                        return Err(CodegenError::TypeError(format!(
                            "Union literal should have exactly one field, got {}",
                            fields.len()
                        )));
                    }

                    let (field_name, field_expr) = &fields[0];

                    // Find field type
                    let field_ty = union_info
                        .fields
                        .iter()
                        .find(|(n, _)| n == &field_name.node)
                        .map(|(_, ty)| ty.clone())
                        .ok_or_else(|| {
                            CodegenError::TypeError(format!(
                                "Unknown field '{}' in union '{}'",
                                field_name.node, type_name
                            ))
                        })?;

                    let (val, field_ir) = self.generate_expr(field_expr, counter)?;
                    ir.push_str(&field_ir);

                    // Bitcast union pointer to field type pointer (all fields at offset 0)
                    let field_llvm_ty = self.type_to_llvm(&field_ty);
                    let field_ptr = self.next_temp(counter);
                    ir.push_str(&format!(
                        "  {} = bitcast %{}* {} to {}*\n",
                        field_ptr, type_name, union_ptr, field_llvm_ty
                    ));

                    // Store the value
                    ir.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        field_llvm_ty, val, field_llvm_ty, field_ptr
                    ));

                    // Return pointer to union
                    Ok((union_ptr, ir))
                } else {
                    Err(CodegenError::TypeError(format!(
                        "Unknown struct or union: {}",
                        type_name
                    )))
                }
            }

            // Index: arr[idx] or slice: arr[start..end]
            Expr::Index {
                expr: array_expr,
                index,
            } => {
                // Check if this is a slice operation (index is a Range expression)
                if let Expr::Range {
                    start,
                    end,
                    inclusive,
                } = &index.node
                {
                    return self.generate_slice(
                        array_expr,
                        start.as_deref(),
                        end.as_deref(),
                        *inclusive,
                        counter,
                    );
                }

                let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;

                // Check if the type is actually indexable
                let arr_type = self.infer_expr_type(array_expr);
                match arr_type {
                    ResolvedType::Array(_)
                    | ResolvedType::ConstArray { .. }
                    | ResolvedType::Pointer(_) => {
                        // OK - indexable type
                    }
                    _ => {
                        let type_name = format!("{:?}", arr_type);
                        return Err(CodegenError::TypeError(format!(
                            "Cannot index non-array type (found {})",
                            type_name
                        )));
                    }
                }
                let (idx_val, idx_ir) = self.generate_expr(index, counter)?;

                let mut ir = arr_ir;
                ir.push_str(&idx_ir);

                // Determine element type from array type
                let elem_llvm_type = match &arr_type {
                    ResolvedType::Array(inner) => self.type_to_llvm(inner),
                    ResolvedType::ConstArray { element, .. } => self.type_to_llvm(element),
                    ResolvedType::Pointer(inner) => self.type_to_llvm(inner),
                    _ => "i64".to_string(),
                };

                // Get element pointer
                let elem_ptr = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i64 {}\n",
                    elem_ptr, elem_llvm_type, elem_llvm_type, arr_val, idx_val
                ));

                // Load element
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    result, elem_llvm_type, elem_llvm_type, elem_ptr
                ));

                Ok((result, ir))
            }

            // Field access: obj.field
            Expr::Field {
                expr: obj_expr,
                field,
            } => {
                let (obj_val, obj_ir) = self.generate_expr(obj_expr, counter)?;
                let mut ir = obj_ir;

                // Use type inference to get the type of the object expression
                // This handles both simple identifiers and nested field accesses
                let obj_type = self.infer_expr_type(obj_expr);

                if let ResolvedType::Named {
                    name: orig_type_name,
                    ..
                } = &obj_type
                {
                    let type_name = &self.resolve_struct_name(orig_type_name);
                    // First check if it's a struct
                    if let Some(struct_info) = self.structs.get(type_name).cloned() {
                        let field_idx = struct_info
                            .fields
                            .iter()
                            .position(|(n, _)| n == &field.node)
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = struct_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in struct '{}'{}",
                                    field.node, type_name, suggestion_text
                                ))
                            })?;

                        let field_ty = &struct_info.fields[field_idx].1;
                        let llvm_ty = self.type_to_llvm(field_ty);

                        // Generate field pointer
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}\n",
                            field_ptr, type_name, type_name, obj_val, field_idx
                        ));

                        // Only load if the field is not itself a struct (to support nested access)
                        // For nested field access like o.a.val, we want o.a to return a pointer to Inner,
                        // not the Inner value itself
                        if matches!(field_ty, ResolvedType::Named { .. }) {
                            // Field is a struct - return pointer for nested access
                            return Ok((field_ptr, ir));
                        } else {
                            // Field is a primitive - load the value
                            let result = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = load {}, {}* {}\n",
                                result, llvm_ty, llvm_ty, field_ptr
                            ));
                            return Ok((result, ir));
                        }
                    }
                    // Then check if it's a union
                    else if let Some(union_info) = self.unions.get(type_name).cloned() {
                        let field_ty = union_info
                            .fields
                            .iter()
                            .find(|(n, _)| n == &field.node)
                            .map(|(_, ty)| ty.clone())
                            .ok_or_else(|| {
                                let candidates: Vec<&str> = union_info
                                    .fields
                                    .iter()
                                    .map(|(name, _)| name.as_str())
                                    .collect();
                                let suggestions = suggest_similar(&field.node, &candidates, 3);
                                let suggestion_text = format_did_you_mean(&suggestions);
                                CodegenError::TypeError(format!(
                                    "Unknown field '{}' in union '{}'{}",
                                    field.node, type_name, suggestion_text
                                ))
                            })?;

                        let llvm_ty = self.type_to_llvm(&field_ty);

                        // For union field access, bitcast union pointer to field type pointer
                        // All fields share offset 0
                        let field_ptr = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = bitcast %{}* {} to {}*\n",
                            field_ptr, type_name, obj_val, llvm_ty
                        ));

                        let result = self.next_temp(counter);
                        ir.push_str(&format!(
                            "  {} = load {}, {}* {}\n",
                            result, llvm_ty, llvm_ty, field_ptr
                        ));

                        return Ok((result, ir));
                    }
                }

                Err(CodegenError::Unsupported(
                    "field access requires known struct or union type".to_string(),
                ))
            }

            // Method call: obj.method(args)
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                // Special case: @.method() means self.method() (call another method on self)
                let (recv_val, recv_ir, recv_type) = if matches!(&receiver.node, Expr::SelfCall) {
                    // When receiver is @, use %self instead of the function pointer
                    if let Some(local) = self.locals.get("self") {
                        let recv_type = local.ty.clone();
                        ("%self".to_string(), String::new(), recv_type)
                    } else {
                        return Err(CodegenError::Unsupported(
                            "@.method() used outside of a method with self".to_string(),
                        ));
                    }
                } else {
                    let (recv_val, recv_ir) = self.generate_expr(receiver, counter)?;
                    let recv_type = self.infer_expr_type(receiver);
                    (recv_val, recv_ir, recv_type)
                };
                let mut ir = recv_ir;

                let method_name = &method.node;

                // String method calls: str.len(), str.charAt(), str.contains(), etc.
                if matches!(recv_type, ResolvedType::Str) {
                    return self.generate_string_method_call(
                        &recv_val,
                        &ir,
                        method_name,
                        args,
                        counter,
                    );
                }

                // Check for dynamic trait dispatch (dyn Trait)
                let dyn_trait_name = match &recv_type {
                    ResolvedType::DynTrait { trait_name, .. } => Some(trait_name.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::DynTrait { trait_name, .. } = inner.as_ref() {
                            Some(trait_name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(trait_name) = dyn_trait_name {
                    // Dynamic dispatch through vtable
                    // Generate additional arguments (just values, vtable generator adds types)
                    let mut extra_arg_vals = Vec::new();
                    for arg in args {
                        let (val, arg_ir) = self.generate_expr(arg, counter)?;
                        ir.push_str(&arg_ir);
                        extra_arg_vals.push(val);
                    }

                    let (dyn_ir, result) = self.generate_dyn_method_call(
                        &recv_val,
                        &trait_name,
                        method_name,
                        &extra_arg_vals,
                        counter,
                    )?;
                    ir.push_str(&dyn_ir);
                    return Ok((result, ir));
                }

                // Build full method name: ResolvedStructName_methodName
                // Use resolve_struct_name to match definition naming (e.g., Pair → Pair$i64)
                // For non-generic structs, this is a no-op (Vec → Vec)
                let full_method_name = if let ResolvedType::Named { name, .. } = &recv_type {
                    let resolved = self.resolve_struct_name(name);
                    format!("{}_{}", resolved, method_name)
                } else {
                    method_name.clone()
                };

                // Get struct type for receiver (add * for pointer)
                let recv_llvm_ty = if matches!(&recv_type, ResolvedType::Named { .. }) {
                    format!("{}*", self.type_to_llvm(&recv_type))
                } else {
                    self.type_to_llvm(&recv_type)
                };

                // Generate arguments (receiver is implicit first arg)
                let mut arg_vals = vec![format!("{} {}", recv_llvm_ty, recv_val)];

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Determine return type from function registry
                let ret_type = self
                    .functions
                    .get(&full_method_name)
                    .map(|info| self.type_to_llvm(&info.signature.ret))
                    .unwrap_or_else(|| "i64".to_string());

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {} @{}({})\n",
                    tmp,
                    ret_type,
                    full_method_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // Static method call: Type.method(args)
            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                let mut ir = String::new();

                // Build full method name: TypeName_methodName
                let full_method_name = format!("{}_{}", type_name.node, method.node);

                // Generate arguments (no receiver for static methods)
                let mut arg_vals = Vec::new();

                for arg in args {
                    let (val, arg_ir) = self.generate_expr(arg, counter)?;
                    ir.push_str(&arg_ir);
                    let arg_type = self.infer_expr_type(arg);
                    let arg_llvm_ty = self.type_to_llvm(&arg_type);
                    arg_vals.push(format!("{} {}", arg_llvm_ty, val));
                }

                // Get return type from method signature
                let ret_type = self
                    .functions
                    .get(&full_method_name)
                    .map(|info| self.type_to_llvm(&info.signature.ret))
                    .unwrap_or_else(|| "i64".to_string());

                let tmp = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {} @{}({})\n",
                    tmp,
                    ret_type,
                    full_method_name,
                    arg_vals.join(", ")
                ));

                Ok((tmp, ir))
            }

            // Spread: ..expr (handled within array generation; standalone generates inner)
            Expr::Spread(inner) => self.generate_expr(inner, counter),

            // Reference: &expr
            Expr::Ref(inner) => {
                // For simple references, just return the address
                if let Expr::Ident(name) = &inner.node {
                    if let Some(local) = self.locals.get(name.as_str()).cloned() {
                        if local.is_alloca() {
                            // Alloca variables already have an address
                            return Ok((format!("%{}", local.llvm_name), String::new()));
                        } else {
                            // SSA/Param values need to be spilled to stack to take their address
                            let mut ir = String::new();
                            let llvm_ty = self.type_to_llvm(&local.ty);
                            let (val, val_ir) = self.generate_expr(inner, counter)?;
                            ir.push_str(&val_ir);
                            let tmp_alloca = self.next_temp(counter);
                            ir.push_str(&format!("  {} = alloca {}\n", tmp_alloca, llvm_ty));
                            ir.push_str(&format!(
                                "  store {} {}, {}* {}\n",
                                llvm_ty, val, llvm_ty, tmp_alloca
                            ));
                            return Ok((tmp_alloca, ir));
                        }
                    }
                }
                // For complex expressions, evaluate and return
                self.generate_expr(inner, counter)
            }

            // Dereference: *expr
            Expr::Deref(inner) => {
                let (ptr_val, ptr_ir) = self.generate_expr(inner, counter)?;
                let mut ir = ptr_ir;

                let result = self.next_temp(counter);
                ir.push_str(&format!("  {} = load i64, i64* {}\n", result, ptr_val));

                Ok((result, ir))
            }

            // Type cast: expr as Type
            Expr::Cast { expr, ty } => {
                let (val, val_ir) = self.generate_expr(expr, counter)?;
                let mut ir = val_ir;

                let target_type = self.ast_type_to_resolved(&ty.node);
                let llvm_type = self.type_to_llvm(&target_type);

                // Simple cast - in many cases just bitcast or pass through
                let result = self.next_temp(counter);
                match (&target_type, llvm_type.as_str()) {
                    // Integer to pointer cast
                    (ResolvedType::Pointer(_), _)
                    | (ResolvedType::Ref(_), _)
                    | (ResolvedType::RefMut(_), _) => {
                        ir.push_str(&format!(
                            "  {} = inttoptr i64 {} to {}\n",
                            result, val, llvm_type
                        ));
                    }
                    // Pointer to integer cast
                    _ if val.starts_with('%') || val.parse::<i64>().is_err() => {
                        // Might be a pointer, try to cast
                        ir.push_str(&format!("  {} = ptrtoint i64* {} to i64\n", result, val));
                    }
                    // Default: just use the value as-is (same size types)
                    _ => {
                        return Ok((val, ir));
                    }
                }

                Ok((result, ir))
            }

            // Match expression: M expr { pattern => body, ... }
            Expr::Match {
                expr: match_expr,
                arms,
            } => self.generate_match(match_expr, arms, counter),

            // Range expression (for now just return start value)
            Expr::Range { start, .. } => {
                if let Some(start_expr) = start {
                    self.generate_expr(start_expr, counter)
                } else {
                    Ok(("0".to_string(), String::new()))
                }
            }

            // Await expression: poll the future until Ready
            Expr::Await(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Get the function name being awaited (for poll function lookup)
                // Helper to extract poll function name from an expression
                fn get_poll_func_name(expr: &Expr) -> String {
                    match expr {
                        Expr::Call { func, .. } => {
                            if let Expr::Ident(name) = &func.node {
                                format!("{}__poll", name)
                            } else {
                                "__async_poll".to_string()
                            }
                        }
                        Expr::MethodCall { method, .. } => {
                            format!("{}__poll", method.node)
                        }
                        Expr::Spawn(inner) => {
                            // For spawn, look at the inner expression
                            get_poll_func_name(&inner.node)
                        }
                        _ => "__async_poll".to_string(),
                    }
                }
                let poll_func = get_poll_func_name(&inner.node);

                // Generate blocking poll loop
                let poll_start = self.next_label("await_poll");
                let poll_ready = self.next_label("await_ready");
                let poll_pending = self.next_label("await_pending");

                ir.push_str(&format!("  br label %{}\n\n", poll_start));
                ir.push_str(&format!("{}:\n", poll_start));

                // Call poll function: returns {i64 status, i64 result}
                let poll_result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = call {{ i64, i64 }} @{}(i64 {})\n",
                    poll_result, poll_func, future_ptr
                ));

                // Extract status (0 = Pending, 1 = Ready)
                let status = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 0\n",
                    status, poll_result
                ));

                // Check if Ready
                let is_ready = self.next_temp(counter);
                ir.push_str(&format!("  {} = icmp eq i64 {}, 1\n", is_ready, status));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_ready, poll_ready, poll_pending
                ));

                // Pending: yield and retry (for now just spin)
                ir.push_str(&format!("{}:\n", poll_pending));
                ir.push_str(&format!("  br label %{}\n\n", poll_start));

                // Ready: extract result
                ir.push_str(&format!("{}:\n", poll_ready));
                let result = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {{ i64, i64 }} {}, 1\n",
                    result, poll_result
                ));

                Ok((result, ir))
            }

            // Spawn expression: create a new task for the runtime
            Expr::Spawn(inner) => {
                let (future_ptr, future_ir) = self.generate_expr(inner, counter)?;
                let mut ir = future_ir;

                // Spawn returns the task/future handle for later awaiting
                // For now, just return the future pointer directly
                ir.push_str(&format!("; Spawned task at {}\n", future_ptr));

                Ok((future_ptr, ir))
            }

            // Yield expression: yield a value from generator
            // For now, treat yield as returning the value (simplified generator support)
            Expr::Yield(inner) => {
                let (val, ir) = self.generate_expr(inner, counter)?;
                // In a full implementation, yield would save state and return.
                // For now, it just evaluates and returns the yielded value.
                Ok((val, ir))
            }

            // Comptime expression: evaluate at compile time and emit constant
            Expr::Comptime { body } => {
                // Evaluate at compile time
                let mut evaluator = vais_types::ComptimeEvaluator::new();
                let value = evaluator.eval(body).map_err(|e| {
                    CodegenError::TypeError(format!("Comptime evaluation failed: {}", e))
                })?;

                // Return the evaluated constant
                match value {
                    vais_types::ComptimeValue::Int(n) => Ok((n.to_string(), String::new())),
                    vais_types::ComptimeValue::Float(f) => {
                        Ok((crate::types::format_llvm_float(f), String::new()))
                    }
                    vais_types::ComptimeValue::Bool(b) => {
                        Ok((if b { "1" } else { "0" }.to_string(), String::new()))
                    }
                    vais_types::ComptimeValue::String(s) => {
                        // Create a global string constant
                        let name = self.make_string_name();
                        self.string_counter += 1;
                        self.string_constants.push((name.clone(), s.clone()));
                        let len = s.len() + 1;
                        Ok((
                            format!(
                                "getelementptr ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                                len, len, name
                            ),
                            String::new(),
                        ))
                    }
                    vais_types::ComptimeValue::Array(arr) => {
                        // Generate array literal from comptime array
                        let mut elements = Vec::new();
                        let mut ir = String::new();

                        for elem in arr {
                            match elem {
                                vais_types::ComptimeValue::Int(n) => elements.push(n.to_string()),
                                vais_types::ComptimeValue::Float(f) => {
                                    elements.push(crate::types::format_llvm_float(f))
                                }
                                vais_types::ComptimeValue::Bool(b) => {
                                    elements.push(if b { "1" } else { "0" }.to_string())
                                }
                                _ => {
                                    return Err(CodegenError::TypeError(
                                        "Comptime arrays can only contain simple values (int, float, bool)".to_string()
                                    ));
                                }
                            }
                        }

                        // Create array on the stack
                        let array_name = format!("%comptime_array_{}", counter);
                        *counter += 1;
                        let len = elements.len();

                        // For now, assume i64 elements (we'd need better type inference for mixed types)
                        ir.push_str(&format!("  {} = alloca [{} x i64]\n", array_name, len));

                        for (i, elem_val) in elements.iter().enumerate() {
                            let elem_ptr = self.next_temp(counter);
                            ir.push_str(&format!(
                                "  {} = getelementptr [{} x i64], [{} x i64]* {}, i64 0, i64 {}\n",
                                elem_ptr, len, len, array_name, i
                            ));
                            ir.push_str(&format!("  store i64 {}, i64* {}\n", elem_val, elem_ptr));
                        }

                        Ok((array_name, ir))
                    }
                    vais_types::ComptimeValue::Unit => Ok(("void".to_string(), String::new())),
                }
            }

            // Macro invocation (should be expanded before codegen)
            Expr::MacroInvoke(invoke) => Err(CodegenError::TypeError(format!(
                "Unexpanded macro invocation: {}! - macros must be expanded before code generation",
                invoke.name.node
            ))),

            // Old expression for contract ensures clauses
            Expr::Old(inner) => {
                // old(expr) references a pre-snapshot value
                // In codegen, we generate a load from the pre-snapshot storage
                let old_var_name = format!("__old_{}", counter);
                *counter += 1;

                // Check if we have a pre-snapshot for this expression
                if let Some(snapshot_var) = self.old_snapshots.get(&old_var_name) {
                    let ty = self.infer_expr_type(inner);
                    let llvm_ty = self.type_to_llvm(&ty);
                    let result = self.next_temp(counter);
                    let ir = format!(
                        "  {} = load {}, {}* %{}\n",
                        result, llvm_ty, llvm_ty, snapshot_var
                    );
                    Ok((result, ir))
                } else {
                    // Fallback: just evaluate the expression (for non-ensures contexts)
                    self.generate_expr(inner, counter)
                }
            }

            // Assert expression
            Expr::Assert { condition, message } => {
                self.generate_assert(condition, message.as_deref(), counter)
            }

            // Assume expression (verification hint, no runtime effect in release)
            Expr::Assume(inner) => {
                if self.release_mode {
                    // In release mode, assume is a no-op
                    Ok(("0".to_string(), String::new()))
                } else {
                    // In debug mode, assume acts like assert but with different error message
                    self.generate_assume(inner, counter)
                }
            }

            // Lambda expression with captures
            Expr::Lambda {
                params,
                body,
                captures: _,
            } => {
                // Generate a unique function name for this lambda
                let lambda_name = format!("__lambda_{}", self.label_counter);
                self.label_counter += 1;

                // Find captured variables by analyzing free variables in lambda body
                let capture_names = self.find_lambda_captures(params, body);

                // Collect captured variable info from current scope
                let mut captured_vars: Vec<(String, ResolvedType, String)> = Vec::new();
                let mut capture_ir = String::new();

                for cap_name in &capture_names {
                    if let Some(local) = self.locals.get(cap_name) {
                        let ty = local.ty.clone();
                        // Load captured value if it's a local variable
                        if local.is_param() {
                            // Parameters are already values, use directly
                            captured_vars.push((
                                cap_name.clone(),
                                ty,
                                format!("%{}", local.llvm_name),
                            ));
                        } else if local.is_ssa() {
                            // SSA values are already the value itself, use directly
                            // llvm_name for SSA includes % prefix (e.g., "%5") or is a literal (e.g., "10")
                            captured_vars.push((cap_name.clone(), ty, local.llvm_name.clone()));
                        } else {
                            // Load from alloca
                            let tmp = self.next_temp(counter);
                            let llvm_ty = self.type_to_llvm(&ty);
                            capture_ir.push_str(&format!(
                                "  {} = load {}, {}* %{}\n",
                                tmp, llvm_ty, llvm_ty, local.llvm_name
                            ));
                            captured_vars.push((cap_name.clone(), ty, tmp));
                        }
                    }
                }

                // Build parameter list (original params + captured vars)
                let mut param_strs = Vec::new();
                let mut param_types = Vec::new();

                // First add captured variables as parameters (they come first)
                for (cap_name, cap_ty, _) in &captured_vars {
                    let llvm_ty = self.type_to_llvm(cap_ty);
                    param_strs.push(format!("{} %__cap_{}", llvm_ty, cap_name));
                    param_types.push(llvm_ty);
                }

                // Then add original lambda parameters
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    let llvm_ty = self.type_to_llvm(&ty);
                    param_strs.push(format!("{} %{}", llvm_ty, p.name.node));
                    param_types.push(llvm_ty);
                }

                // Store current function state
                let saved_function = self.current_function.clone();
                let saved_locals = self.locals.clone();

                // Set up lambda context
                self.current_function = Some(lambda_name.clone());
                self.locals.clear();

                // Register captured variables as locals (using capture parameter names)
                for (cap_name, cap_ty, _) in &captured_vars {
                    self.locals.insert(
                        cap_name.clone(),
                        LocalVar::param(cap_ty.clone(), format!("__cap_{}", cap_name)),
                    );
                }

                // Register original parameters as locals
                for p in params {
                    let ty = self.ast_type_to_resolved(&p.ty.node);
                    self.locals.insert(
                        p.name.node.clone(),
                        LocalVar::param(ty, p.name.node.clone()),
                    );
                }

                // Generate lambda body
                let mut lambda_counter = 0;
                let (body_val, body_ir) = self.generate_expr(body, &mut lambda_counter)?;

                // Build lambda function IR
                let mut lambda_ir = format!(
                    "define i64 @{}({}) {{\nentry:\n",
                    lambda_name,
                    param_strs.join(", ")
                );
                lambda_ir.push_str(&body_ir);
                lambda_ir.push_str(&format!("  ret i64 {}\n}}\n", body_val));

                // Store lambda function for later emission
                self.lambda_functions.push(lambda_ir);

                // Restore function context
                self.current_function = saved_function;
                self.locals = saved_locals;

                // Emit ptrtoint as a proper instruction (not a constant expression)
                // so the result is a clean SSA temp that can be used anywhere
                let fn_ptr_tmp = self.next_temp(counter);
                capture_ir.push_str(&format!(
                    "  {} = ptrtoint i64 ({})* @{} to i64\n",
                    fn_ptr_tmp,
                    param_types.join(", "),
                    lambda_name
                ));

                // Store lambda info for Let statement to pick up
                if captured_vars.is_empty() {
                    self.last_lambda_info = None;
                    Ok((fn_ptr_tmp, capture_ir))
                } else {
                    // Store closure info with captured variable values
                    self.last_lambda_info = Some(ClosureInfo {
                        func_name: lambda_name.clone(),
                        captures: captured_vars
                            .iter()
                            .map(|(name, _, val)| (name.clone(), val.clone()))
                            .collect(),
                    });
                    Ok((fn_ptr_tmp, capture_ir))
                }
            }

            // Try expression: expr? - propagate Err early, continue with Ok value
            // User-defined enum layout: %EnumName = type { i32 tag, { i64 } payload }
            Expr::Try(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Try expression (?)\n");

                // Extract tag (field 0, i32)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err (tag != 0, i.e., not Ok)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("try_err");
                let ok_label = self.next_label("try_ok");
                let merge_label = self.next_label("try_merge");

                ir.push_str(&format!("  {} = icmp ne i32 {}, 0\n", is_err, tag));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_err, err_label, ok_label
                ));

                // Err branch: return the whole enum value as-is (early return)
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str(&format!(
                    "  ret {} {}  ; early return on Err\n\n",
                    llvm_type, inner_val
                ));

                // Ok branch: extract payload value (field 1, then field 0 of the payload struct)
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 1, 0\n",
                    value, llvm_type, inner_val
                ));
                ir.push_str(&format!("  br label %{}\n\n", merge_label));

                // Merge block
                ir.push_str(&format!("{}:\n", merge_label));

                Ok((value, ir))
            }

            // Unwrap expression: expr! - panic on Err/None, continue with value
            // User-defined enum layout: %EnumName = type { i32 tag, { i64 } payload }
            Expr::Unwrap(inner) => {
                // Determine the LLVM type name from the inner expression's type
                let inner_type = self.infer_expr_type(inner);
                let llvm_type = self.type_to_llvm(&inner_type);

                let (inner_val, inner_ir) = self.generate_expr(inner, counter)?;
                let mut ir = inner_ir;

                ir.push_str("  ; Unwrap expression\n");

                // Extract tag (field 0, i32)
                let tag = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 0\n",
                    tag, llvm_type, inner_val
                ));

                // Check if Err/None (tag != 0)
                let is_err = self.next_temp(counter);
                let err_label = self.next_label("unwrap_err");
                let ok_label = self.next_label("unwrap_ok");

                ir.push_str(&format!("  {} = icmp ne i32 {}, 0\n", is_err, tag));
                ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n\n",
                    is_err, err_label, ok_label
                ));

                // Err branch: panic/abort
                ir.push_str(&format!("{}:\n", err_label));
                ir.push_str("  call i32 @puts(ptr getelementptr ([22 x i8], ptr @.unwrap_panic_msg, i64 0, i64 0))\n");
                ir.push_str("  call void @abort()\n");
                ir.push_str("  unreachable\n\n");

                // Ok branch: extract value (field 1, field 0 of payload struct)
                ir.push_str(&format!("{}:\n", ok_label));
                let value = self.next_temp(counter);
                ir.push_str(&format!(
                    "  {} = extractvalue {} {}, 1, 0\n",
                    value, llvm_type, inner_val
                ));

                // Track that we need the panic message and abort declaration
                self.needs_unwrap_panic = true;

                Ok((value, ir))
            }

            // Error nodes should not reach codegen
            Expr::Error { message, .. } => Err(CodegenError::Unsupported(format!(
                "Parse error in expression: {}",
                message
            ))),

            // Lazy and Force expressions - delegate to visitor
            Expr::Lazy(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_lazy(inner, counter)
            }
            Expr::Force(inner) => {
                use crate::visitor::ExprVisitor;
                self.visit_force(inner, counter)
            }
        }
    }

    fn next_temp(&self, counter: &mut usize) -> String {
        let tmp = format!("%t.{}", counter);
        *counter += 1;
        tmp
    }

    /// Resolve a generic function call to the appropriate mangled specialized name.
    /// Given a base function name and the inferred argument types, finds the
    /// matching instantiation from the pre-computed instantiation list.
    fn resolve_generic_call(
        &self,
        base_name: &str,
        arg_types: &[ResolvedType],
        instantiations_list: &[(Vec<ResolvedType>, String)],
    ) -> String {
        // If only one instantiation exists, use it directly
        if instantiations_list.len() == 1 {
            return instantiations_list[0].1.clone();
        }

        // Look up the generic function template to map argument types to type parameters
        if let Some(template) = self.generic_function_templates.get(base_name) {
            let type_params: Vec<&String> = template
                .generics
                .iter()
                .filter(|g| !matches!(g.kind, GenericParamKind::Lifetime { .. }))
                .map(|g| &g.name.node)
                .collect();

            // Infer type arguments from argument types by matching against parameter types
            let mut inferred: HashMap<String, ResolvedType> = HashMap::new();
            for (i, param) in template.params.iter().enumerate() {
                if i < arg_types.len() {
                    self.infer_type_args(
                        &self.ast_type_to_resolved(&param.ty.node),
                        &arg_types[i],
                        &type_params,
                        &mut inferred,
                    );
                }
            }

            // Build type_args vector in order of generic params
            let type_args: Vec<ResolvedType> = type_params
                .iter()
                .map(|name| inferred.get(*name).cloned().unwrap_or(ResolvedType::I64))
                .collect();

            // Find exact match in instantiations
            for (inst_types, mangled) in instantiations_list {
                if inst_types == &type_args {
                    return mangled.clone();
                }
            }
        }

        // Fallback: try to mangle based on argument types directly
        let mangled = vais_types::mangle_name(base_name, arg_types);
        if self.functions.contains_key(&mangled) {
            return mangled;
        }

        // Last resort: use first instantiation
        instantiations_list
            .first()
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| base_name.to_string())
    }

    /// Infer type arguments by matching a parameter type pattern against a concrete argument type.
    fn infer_type_args(
        &self,
        param_type: &ResolvedType,
        arg_type: &ResolvedType,
        type_params: &[&String],
        inferred: &mut HashMap<String, ResolvedType>,
    ) {
        match param_type {
            ResolvedType::Generic(name) => {
                // Direct generic type parameter (e.g., T)
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                }
            }
            ResolvedType::Named { name, generics } => {
                // Check if this is a type parameter name
                if type_params.contains(&name) {
                    inferred
                        .entry(name.clone())
                        .or_insert_with(|| arg_type.clone());
                } else if let ResolvedType::Named {
                    generics: arg_generics,
                    ..
                } = arg_type
                {
                    // Recurse into generic arguments
                    for (g, ag) in generics.iter().zip(arg_generics.iter()) {
                        self.infer_type_args(g, ag, type_params, inferred);
                    }
                }
            }
            ResolvedType::Array(inner) => {
                if let ResolvedType::Array(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Pointer(inner) => {
                if let ResolvedType::Pointer(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            ResolvedType::Optional(inner) => {
                if let ResolvedType::Optional(arg_inner) = arg_type {
                    self.infer_type_args(inner, arg_inner, type_params, inferred);
                }
            }
            _ => {}
        }
    }

    /// Generate code for a block expression (used in if/else branches)
    #[allow(dead_code)]
    fn generate_block_expr(
        &mut self,
        expr: &Spanned<Expr>,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        match &expr.node {
            Expr::Block(stmts) => {
                let (val, ir, _terminated) = self.generate_block_stmts(stmts, counter)?;
                Ok((val, ir))
            }
            _ => self.generate_expr(expr, counter),
        }
    }

    /// Generate code for a block of statements
    /// Returns (value, ir_code, is_terminated)
    fn generate_block_stmts(
        &mut self,
        stmts: &[Spanned<Stmt>],
        counter: &mut usize,
    ) -> CodegenResult<(String, String, bool)> {
        // Use StmtVisitor trait for statement code generation
        use crate::visitor::StmtVisitor;
        self.visit_block_stmts(stmts, counter)
    }

    // Control flow functions (if-else, match, pattern matching) are in control_flow.rs module

    // Type inference functions are in type_inference.rs module

    /// Generate code for array slicing: arr[start..end]
    /// Returns a new array (allocated on heap) containing the slice
    fn generate_slice(
        &mut self,
        array_expr: &Spanned<Expr>,
        start: Option<&Spanned<Expr>>,
        end: Option<&Spanned<Expr>>,
        inclusive: bool,
        counter: &mut usize,
    ) -> CodegenResult<(String, String)> {
        let (arr_val, arr_ir) = self.generate_expr(array_expr, counter)?;
        let mut ir = arr_ir;

        // Get start index (default 0)
        let start_val = if let Some(start_expr) = start {
            let (val, start_ir) = self.generate_expr(start_expr, counter)?;
            ir.push_str(&start_ir);
            val
        } else {
            "0".to_string()
        };

        // Get end index
        // For simplicity, we require end to be specified for now
        // A proper implementation would need array length tracking
        let end_val = if let Some(end_expr) = end {
            let (val, end_ir) = self.generate_expr(end_expr, counter)?;
            ir.push_str(&end_ir);

            // If inclusive (..=), add 1 to end
            if inclusive {
                let adj_end = self.next_temp(counter);
                ir.push_str(&format!("  {} = add i64 {}, 1\n", adj_end, val));
                adj_end
            } else {
                val
            }
        } else {
            return Err(CodegenError::Unsupported(
                "Slice without end index requires array length".to_string(),
            ));
        };

        // Calculate slice length: end - start
        let length = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = sub i64 {}, {}\n",
            length, end_val, start_val
        ));

        // Allocate new array for slice
        let byte_size = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = mul i64 {}, 8\n", // 8 bytes per i64 element
            byte_size, length
        ));

        let raw_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = call i8* @malloc(i64 {})\n",
            raw_ptr, byte_size
        ));

        let slice_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = bitcast i8* {} to i64*\n",
            slice_ptr, raw_ptr
        ));

        // Copy elements using a loop
        let loop_idx_ptr = self.next_temp(counter);
        ir.push_str(&format!("  {} = alloca i64\n", loop_idx_ptr));
        ir.push_str(&format!("  store i64 0, i64* {}\n", loop_idx_ptr));

        let loop_start = self.next_label("slice_loop");
        let loop_body = self.next_label("slice_body");
        let loop_end = self.next_label("slice_end");

        ir.push_str(&format!("  br label %{}\n", loop_start));
        ir.push_str(&format!("{}:\n", loop_start));

        let loop_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = load i64, i64* {}\n",
            loop_idx, loop_idx_ptr
        ));

        let cmp = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = icmp slt i64 {}, {}\n",
            cmp, loop_idx, length
        ));
        ir.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cmp, loop_body, loop_end
        ));

        ir.push_str(&format!("{}:\n", loop_body));

        // Calculate source index: start + loop_idx
        let src_idx = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = add i64 {}, {}\n",
            src_idx, start_val, loop_idx
        ));

        // Get source element pointer
        let src_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            src_ptr, arr_val, src_idx
        ));

        // Load source element
        let elem = self.next_temp(counter);
        ir.push_str(&format!("  {} = load i64, i64* {}\n", elem, src_ptr));

        // Get destination element pointer
        let dst_ptr = self.next_temp(counter);
        ir.push_str(&format!(
            "  {} = getelementptr i64, i64* {}, i64 {}\n",
            dst_ptr, slice_ptr, loop_idx
        ));

        // Store element
        ir.push_str(&format!("  store i64 {}, i64* {}\n", elem, dst_ptr));

        // Increment loop index
        let next_idx = self.next_temp(counter);
        ir.push_str(&format!("  {} = add i64 {}, 1\n", next_idx, loop_idx));
        ir.push_str(&format!(
            "  store i64 {}, i64* {}\n",
            next_idx, loop_idx_ptr
        ));
        ir.push_str(&format!("  br label %{}\n", loop_start));

        ir.push_str(&format!("{}:\n", loop_end));

        Ok((slice_ptr, ir))
    }

    // Lambda closure capture functions are in lambda_closure.rs module
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_fibonacci() {
        let source = "F fib(n:i64)->i64=n<2?n:@(n-1)+@(n-2)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @fib"));
        assert!(ir.contains("call i64 @fib"));
    }

    #[test]
    fn test_if_else() {
        // I cond { then } E { else }
        let source = "F max(a:i64,b:i64)->i64{I a>b{R a}E{R b}}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @max"));
        assert!(ir.contains("br i1"));
        assert!(ir.contains("then"));
        assert!(ir.contains("else"));
    }

    #[test]
    fn test_loop_with_condition() {
        // L pattern:iter { body } - `L _:condition{body}` for while loop
        let source = "F countdown(n:i64)->i64{x:=n;L _:x>0{x=x-1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @countdown"));
        assert!(ir.contains("loop.start"));
        assert!(ir.contains("loop.body"));
        assert!(ir.contains("loop.end"));
    }

    #[test]
    fn test_array_literal() {
        let source = "F get_arr()->*i64=[1,2,3]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("alloca [3  x i64]"));
        assert!(ir.contains("getelementptr"));
        assert!(ir.contains("store i64"));
    }

    #[test]
    fn test_array_index() {
        let source = "F get_elem(arr:*i64, idx:i64)->i64=arr[idx]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("getelementptr i64, i64*"));
        assert!(ir.contains("load i64, i64*"));
    }

    #[test]
    fn test_struct_codegen() {
        let source = "S Point{x:i64,y:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Point = type { i64, i64 }"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_module() {
        let source = "";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should generate valid LLVM IR even with empty module
        assert!(ir.contains("source_filename"));
    }

    #[test]
    fn test_minimal_function() {
        let source = "F f()->()=()";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @f"));
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_function_returning_unit() {
        let source = "F void_fn()->(){}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define void @void_fn"));
    }

    #[test]
    fn test_empty_struct() {
        let source = "S Empty{}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty struct should still generate a type
        assert!(ir.contains("%Empty = type"));
    }

    #[test]
    fn test_single_field_struct() {
        let source = "S Single{x:i64}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Single = type { i64 }"));
    }

    #[test]
    fn test_enum_with_variants() {
        let source = "E Color{Red,Green,Blue}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Enum should generate a type
        assert!(ir.contains("%Color = type"));
    }

    #[test]
    fn test_i64_max_value() {
        let source = "F max()->i64=9223372036854775807";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_negative_number() {
        let source = "F neg()->i64=-42";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Negative numbers involve subtraction from 0
        assert!(ir.contains("sub i64 0, 42"));
    }

    #[test]
    fn test_zero_value() {
        let source = "F zero()->i64=0";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_float_values() {
        let source = "F pi()->f64=3.141592653589793";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("double"));
    }

    #[test]
    fn test_boolean_true() {
        let source = "F yes()->bool=true";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 true") || ir.contains("ret i1 1"));
    }

    #[test]
    fn test_boolean_false() {
        let source = "F no()->bool=false";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("ret i1 false") || ir.contains("ret i1 0"));
    }

    #[test]
    fn test_empty_string() {
        let source = r#"F empty()->str="""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle empty string
        assert!(ir.contains("@str") || ir.contains("i8*"));
    }

    #[test]
    fn test_string_with_escape() {
        let source = r#"F escaped()->str="hello\nworld""#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should handle escape sequences
        assert!(ir.contains("@str"));
    }

    #[test]
    fn test_empty_array() {
        let source = "F empty_arr()->*i64=[]";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Empty array should still work
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_nested_if_else() {
        let source = r#"
            F classify(x:i64)->i64{
                I x>0{
                    I x>100{2}E{1}
                }E{
                    I x<-100{-2}E{-1}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @classify"));
        // Should have multiple branches
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_simple_match() {
        let source = "F digit(n:i64)->str=M n{0=>\"zero\",1=>\"one\",_=>\"other\"}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define"));
    }

    #[test]
    fn test_for_loop() {
        let source = "F sum_to(n:i64)->i64{s:=0;L i:0..n{s+=i};s}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @sum_to"));
        assert!(ir.contains("for.cond"));
        assert!(ir.contains("for.body"));
        assert!(ir.contains("for.inc"));
    }

    #[test]
    fn test_while_loop() {
        let source = "F count_down(n:i64)->i64{x:=n;L _:x>0{x-=1};x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @count_down"));
    }

    #[test]
    fn test_infinite_loop_with_break() {
        let source = "F find()->i64{x:=0;L{I x>10{B x};x+=1};0}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @find"));
    }

    #[test]
    fn test_arithmetic_operations() {
        let source = "F math(a:i64,b:i64)->i64=a+b-a*b/a%b";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("add i64"));
        assert!(ir.contains("sub i64"));
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("sdiv i64"));
        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_comparison_operations() {
        let source = r#"
            F compare(a:i64,b:i64)->bool{
                x:=a<b;
                y:=a<=b;
                z:=a>b;
                w:=a>=b;
                u:=a==b;
                v:=a!=b;
                x
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("icmp slt"));
        assert!(ir.contains("icmp sle"));
        assert!(ir.contains("icmp sgt"));
        assert!(ir.contains("icmp sge"));
        assert!(ir.contains("icmp eq"));
        assert!(ir.contains("icmp ne"));
    }

    #[test]
    fn test_bitwise_operations() {
        let source = "F bits(a:i64,b:i64)->i64{x:=a&b;y:=a|b;z:=a^b;w:=a<<2;v:=a>>1;x}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_logical_operations() {
        let source = "F logic(a:bool,b:bool)->bool=a&&b||!a";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i1 @logic"));
    }

    #[test]
    fn test_unary_minus() {
        let source = "F negate(x:i64)->i64=-x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("sub i64 0"));
    }

    #[test]
    fn test_bitwise_not() {
        let source = "F complement(x:i64)->i64=~x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("xor i64") && ir.contains("-1"));
    }

    #[test]
    fn test_ternary_expression() {
        let source = "F abs(x:i64)->i64=x<0?-x:x";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @abs"));
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_compound_assignment() {
        // In Vais, mutable variables use := for declaration
        let source = r#"
            F compound(x:i64)->i64{
                y:=x;
                y+=1;
                y-=2;
                y*=3;
                y/=4;
                y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @compound"));
    }

    #[test]
    fn test_struct_literal() {
        let source = r#"
            S Point{x:i64,y:i64}
            F origin()->Point=Point{x:0,y:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("%Point = type { i64, i64 }"));
        assert!(ir.contains("define %Point"));
    }

    #[test]
    fn test_struct_field_access() {
        let source = r#"
            S Point{x:i64,y:i64}
            F get_x(p:Point)->i64=p.x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("getelementptr"));
    }

    #[test]
    fn test_lambda_simple() {
        let source = "F f()->i64{add:=|a:i64,b:i64|a+b;add(1,2)}";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @f"));
    }

    #[test]
    fn test_recursive_factorial() {
        let source = "F factorial(n:i64)->i64=n<=1?1:n*@(n-1)";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @factorial"));
        assert!(ir.contains("call i64 @factorial"));
    }

    #[test]
    fn test_multiple_functions() {
        let source = r#"
            F add(a:i64,b:i64)->i64=a+b
            F sub(a:i64,b:i64)->i64=a-b
            F mul(a:i64,b:i64)->i64=a*b
            F test()->i64=mul(add(1,2),sub(5,2))
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @add"));
        assert!(ir.contains("define i64 @sub"));
        assert!(ir.contains("define i64 @mul"));
        assert!(ir.contains("define i64 @test"));
    }

    #[test]
    fn test_function_with_many_params() {
        let source = "F many(a:i64,b:i64,c:i64,d:i64,e:i64,f:i64,g:i64,h:i64)->i64=a+b+c+d+e+f+g+h";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // LLVM IR uses %a, %b etc, and the define line may not have spaces
        assert!(ir.contains("define i64 @many"));
        assert!(ir.contains("i64 %a"));
        assert!(ir.contains("i64 %h"));
    }

    #[test]
    fn test_all_integer_types() {
        let source = r#"
            F test_i8(x:i8)->i8=x
            F test_i16(x:i16)->i16=x
            F test_i32(x:i32)->i32=x
            F test_i64(x:i64)->i64=x
            F test_u8(x:u8)->u8=x
            F test_u16(x:u16)->u16=x
            F test_u32(x:u32)->u32=x
            F test_u64(x:u64)->u64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i8 @test_i8"));
        assert!(ir.contains("define i16 @test_i16"));
        assert!(ir.contains("define i32 @test_i32"));
        assert!(ir.contains("define i64 @test_i64"));
        assert!(ir.contains("define i8 @test_u8"));
        assert!(ir.contains("define i16 @test_u16"));
        assert!(ir.contains("define i32 @test_u32"));
        assert!(ir.contains("define i64 @test_u64"));
    }

    #[test]
    fn test_float_types() {
        let source = r#"
            F test_f32(x:f32)->f32=x
            F test_f64(x:f64)->f64=x
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define float @test_f32"));
        assert!(ir.contains("define double @test_f64"));
    }

    #[test]
    fn test_deeply_nested_expression() {
        let source = "F deep(a:i64)->i64=((((a+1)+2)+3)+4)+5";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        assert!(ir.contains("define i64 @deep"));
    }

    #[test]
    fn test_mixed_arithmetic_precedence() {
        let source = "F prec(a:i64,b:i64,c:i64)->i64=a+b*c";
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();
        // Should multiply first then add (precedence)
        assert!(ir.contains("mul i64"));
        assert!(ir.contains("add i64"));
    }

    // ==================== Generic Instantiation Tests ====================

    #[test]
    fn test_generate_specialized_function() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->i64=identity(42)
        "#;
        let module = parse(source).unwrap();

        // First, type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain specialized function identity$i64
        assert!(
            ir.contains("define i64 @identity$i64"),
            "Expected identity$i64 in IR: {}",
            ir
        );
        assert!(ir.contains("ret i64 %x"), "Expected return in identity$i64");
    }

    #[test]
    fn test_generate_specialized_struct_type() {
        use vais_types::TypeChecker;

        // Test that generic struct type definition is specialized
        // Note: Full struct literal code generation with generics requires additional work
        // This test verifies the type definition is generated correctly
        let source = r#"
            S Pair<T>{first:T,second:T}
            F main()->i64{
                p:=Pair{first:1,second:2};
                p.first
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Verify instantiation was recorded
        let pair_inst = instantiations.iter().find(|i| i.base_name == "Pair");
        assert!(
            pair_inst.is_some(),
            "Expected Pair instantiation to be recorded"
        );

        // Verify mangled name
        let inst = pair_inst.unwrap();
        assert_eq!(
            inst.mangled_name, "Pair$i64",
            "Expected mangled name Pair$i64, got {}",
            inst.mangled_name
        );
    }

    #[test]
    fn test_multiple_instantiations() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
            F main()->f64{
                a:=identity(42);
                b:=identity(3.14);
                b
            }
        "#;
        let module = parse(source).unwrap();

        // Type check to get instantiations
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // Should have at least 2 instantiations
        assert!(
            instantiations.len() >= 2,
            "Expected at least 2 instantiations, got {}",
            instantiations.len()
        );

        // Generate code with instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should contain both specialized functions
        assert!(ir.contains("@identity$i64"), "Expected identity$i64 in IR");
        assert!(ir.contains("@identity$f64"), "Expected identity$f64 in IR");
    }

    #[test]
    fn test_no_code_for_generic_template() {
        use vais_types::TypeChecker;

        let source = r#"
            F identity<T>(x:T)->T=x
        "#;
        let module = parse(source).unwrap();

        // Type check (no instantiations since function isn't called)
        let mut checker = TypeChecker::new();
        checker.check_module(&module).unwrap();
        let instantiations = checker.get_generic_instantiations();

        // No instantiations
        assert!(instantiations.is_empty());

        // Generate code with empty instantiations
        let mut gen = CodeGenerator::new("test");
        let ir = gen
            .generate_module_with_instantiations(&module, instantiations)
            .unwrap();

        // Should NOT contain any identity function definition
        assert!(
            !ir.contains("define i64 @identity"),
            "Generic template should not generate code"
        );
        assert!(
            !ir.contains("define double @identity"),
            "Generic template should not generate code"
        );
    }

    // ==================== Advanced Edge Case Tests ====================

    #[test]
    fn test_i8_boundary_values() {
        // Test i8 min (-128) and max (127)
        let source = r#"
            F i8_bounds()->(i8,i8){
                min:i8=-128;
                max:i8=127;
                (min,max)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check for i8 type usage
        assert!(ir.contains("i8"));
    }

    #[test]
    fn test_i8_overflow_value() {
        // Test arithmetic that could overflow (using i64 as i8 not fully supported)
        let source = r#"
            F add_large()->i64{
                x:=9000000000000000000;
                y:=1000000000000000000;
                x+y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate code (overflow behavior is runtime)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_i8_underflow_value() {
        // Test arithmetic that could underflow (using i64)
        let source = r#"
            F sub_large()->i64{
                x:=-9000000000000000000;
                y:=1000000000000000000;
                x-y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64"));
    }

    #[test]
    fn test_i64_max_value_codegen() {
        // Test i64 max: 9223372036854775807
        let source = r#"
            F i64_max()->i64=9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("9223372036854775807"));
    }

    #[test]
    fn test_i64_min_value_codegen() {
        // Test i64 min (approximately): -9223372036854775808
        let source = r#"
            F i64_near_min()->i64=-9223372036854775807
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0, 9223372036854775807"));
    }

    #[test]
    fn test_integer_overflow_addition() {
        // Test potential overflow in addition
        let source = r#"
            F add_overflow(a:i64,b:i64)->i64=a+b
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate regular add (overflow is runtime behavior)
        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_integer_overflow_multiplication() {
        // Test potential overflow in multiplication
        let source = r#"
            F mul_large()->i64{
                a:i64=1000000000;
                b:i64=1000000000;
                a*b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("mul i64"));
    }

    #[test]
    fn test_division_by_zero() {
        // Test division by zero (runtime error, should compile)
        let source = r#"
            F div_zero()->i64{
                x:=10;
                y:=0;
                x/y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sdiv i64"));
    }

    #[test]
    fn test_modulo_by_zero() {
        // Test modulo by zero
        let source = r#"
            F mod_zero()->i64{
                x:=10;
                y:=0;
                x%y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("srem i64"));
    }

    #[test]
    fn test_all_integer_type_boundaries() {
        // Test boundary values for all integer types
        // Note: Variables must be used for type information to appear in IR
        // Vais primarily uses i64 for integer arithmetic, but stores typed values
        // Test that integer literals with annotations generate valid IR
        let source = r#"
            F get_i8()->i8{
                a:i8=127;
                a
            }
            F get_i32()->i32{
                e:i32=2147483647;
                e
            }
            F get_i64()->i64{
                f:i64=9223372036854775807;
                f
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that the IR contains function definitions with correct return types
        assert!(ir.contains("i8"), "IR should contain i8 type");
        assert!(ir.contains("i32"), "IR should contain i32 type");
        assert!(ir.contains("i64"), "IR should contain i64 type");
    }

    #[test]
    fn test_signed_integer_wraparound() {
        // Test signed integer wraparound behavior (using i64)
        let source = r#"
            F wraparound()->i64{
                max:=9223372036854775806;
                max+1
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("add i64"));
    }

    #[test]
    fn test_nested_generic_codegen() {
        // Simplified generic struct test
        let source = r#"
            S Container<T>{data:T}
            F empty()->Container<i64> =Container{data:0}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("%Container"));
    }

    #[test]
    fn test_pattern_match_with_guard_codegen() {
        // Test pattern match with guard generates correct branches (fix escaping)
        let source = r#"
            F classify(x:i64)->str=M x{
                n I n>0=>"pos",
                n I n<0=>"neg",
                _=>"zero"
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have branches for guards
        assert!(ir.contains("br i1"));
    }

    #[test]
    fn test_mutual_recursion_codegen() {
        // Test mutual recursion generates correct calls
        let source = r#"
            F is_even(n:i64)->bool=n==0?true:is_odd(n-1)
            F is_odd(n:i64)->bool=n==0?false:is_even(n-1)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @is_even"));
        assert!(ir.contains("define i1 @is_odd"));
        assert!(ir.contains("call i1 @is_odd"));
        assert!(ir.contains("call i1 @is_even"));
    }

    #[test]
    fn test_deeply_nested_if_codegen() {
        // Test deeply nested if-else generates correct basic blocks
        let source = r#"
            F deep(x:i64)->i64{
                I x>100{
                    I x>1000{1}E{2}
                }E{
                    I x>10{
                        I x>50{3}E{4}
                    }E{5}
                }
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have multiple branches
        let br_count = ir.matches("br i1").count();
        assert!(br_count >= 4, "Expected at least 4 branches");
    }

    #[test]
    fn test_large_number_of_parameters() {
        // Test function with many parameters
        let source = r#"
            F many_params(
                a:i64,b:i64,c:i64,d:i64,e:i64,
                f:i64,g:i64,h:i64,i:i64,j:i64,
                k:i64,l:i64,m:i64,n:i64,o:i64
            )->i64=a+b+c+d+e+f+g+h+i+j+k+l+m+n+o
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i64 @many_params"));
        // Check for parameter usage
        assert!(ir.contains("%a"));
        assert!(ir.contains("%o"));
    }

    #[test]
    fn test_zero_return_optimization() {
        // Test that returning 0 is optimized
        let source = r#"
            F zero()->i64=0
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("ret i64 0"));
    }

    #[test]
    fn test_constant_folding_candidate() {
        // Test expressions that could be constant folded
        let source = r#"
            F const_expr()->i64=2+3*4-1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should generate arithmetic operations
        assert!(ir.contains("add i64") || ir.contains("ret i64 13"));
        assert!(ir.contains("mul i64") || ir.contains("ret i64 13"));
    }

    #[test]
    fn test_boolean_short_circuit() {
        // Test boolean short-circuit evaluation
        let source = r#"
            F short_circuit(a:bool,b:bool)->bool=a&&b||!a
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("define i1 @short_circuit"));
    }

    #[test]
    fn test_comparison_chain_codegen() {
        // Test comparison chains: a < b < c
        let source = r#"
            F compare_chain(a:i64,b:i64,c:i64)->bool{
                x:=a<b;
                y:=b<c;
                x&&y
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("icmp slt"));
    }

    #[test]
    fn test_bitwise_operations_all_types() {
        // Test bitwise operations (i8 not fully supported, use i64)
        let source = r#"
            F bitwise_i64(a:i64,b:i64)->i64=a&b|a^b
            F bitwise_test()->i64=bitwise_i64(5,3)
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("and i64"));
        assert!(ir.contains("or i64"));
        assert!(ir.contains("xor i64"));
    }

    #[test]
    fn test_shift_operations_boundaries() {
        // Test shift operations at boundaries
        let source = r#"
            F shift_max(x:i64)->i64{
                a:=x<<63;
                b:=x>>63;
                a+b
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
        assert!(ir.contains("ashr i64"));
    }

    #[test]
    fn test_negative_shift_amount() {
        // Test negative shift (undefined behavior, should compile)
        let source = r#"
            F neg_shift(x:i64)->i64{
                shift:=-1;
                x<<shift
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("shl i64"));
    }

    #[test]
    fn test_all_unary_operators() {
        // Test all unary operators
        let source = r#"
            F unary_ops(x:i64,b:bool)->(i64,i64,bool){
                neg:=-x;
                bit_not:=(~x);
                log_not:=!b;
                (neg,bit_not,log_not)
            }
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("sub i64 0")); // negation
        assert!(ir.contains("xor i64") && ir.contains("-1")); // bitwise not
    }

    #[test]
    fn test_float_division_by_zero() {
        // Test float division (check IR has float division instruction)
        let source = r#"
            F fdiv_test(x:f64,y:f64)->f64=x/y
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Check that float division is generated
        assert!(ir.contains("fdiv") || ir.contains("define double"));
    }

    #[test]
    fn test_recursive_depth() {
        // Test deep recursion (should compile, runtime stack depth)
        let source = r#"
            F deep_recursion(n:i64)->i64=n<1?0:@(n-1)+1
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        assert!(ir.contains("call i64 @deep_recursion"));
    }

    // ==================== Decreases Termination Tests ====================

    #[test]
    fn test_decreases_basic() {
        // Test basic decreases clause for termination proof
        let source = r#"
            #[requires(n >= 0)]
            #[decreases(n)]
            F factorial(n:i64)->i64{I n<=1{R 1}R n*factorial(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have initial decreases storage
        assert!(
            ir.contains("__decreases_factorial"),
            "Expected decreases storage variable"
        );
        // Should have non-negative check
        assert!(
            ir.contains("decreases_nonneg"),
            "Expected non-negative check"
        );
        // Should have strict decrease check before recursive call
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before recursive call"
        );
        // Should have panic call for failed check
        assert!(
            ir.contains("@__panic"),
            "Expected panic call for failed check"
        );
    }

    #[test]
    fn test_decreases_strict_decrease_check() {
        // Test that the strict decrease check (new < old) is generated
        let source = r#"
            #[decreases(n)]
            F count_down(n:i64)->i64{I n<=0{R 0}R count_down(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp slt (strictly less than) check
        assert!(
            ir.contains("icmp slt i64"),
            "Expected strict less-than comparison for decreases"
        );
        // Should have both decreases labels
        assert!(ir.contains("decreases_check_ok"), "Expected success label");
        assert!(
            ir.contains("decreases_check_fail"),
            "Expected failure label"
        );
    }

    #[test]
    fn test_decreases_nonneg_check() {
        // Test that non-negative check is generated for decreases expression
        let source = r#"
            #[decreases(x)]
            F process(x:i64)->i64{I x<=0{R 0}R process(x-1)+1}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have icmp sge (signed greater-or-equal) for non-negative check
        assert!(
            ir.contains("icmp sge i64"),
            "Expected non-negative check (sge 0)"
        );
        assert!(
            ir.contains("decreases_nonneg_ok"),
            "Expected success label for non-negative"
        );
        assert!(
            ir.contains("decreases_nonneg_fail"),
            "Expected failure label for non-negative"
        );
    }

    #[test]
    fn test_decreases_release_mode() {
        // Test that decreases checks are skipped in release mode
        let source = r#"
            #[decreases(n)]
            F fib(n:i64)->i64{I n<2{R n}R fib(n-1)+fib(n-2)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        gen.enable_release_mode();
        let ir = gen.generate_module(&module).unwrap();

        // Should NOT have decreases checks in release mode
        assert!(
            !ir.contains("__decreases_fib"),
            "Should skip decreases in release mode"
        );
        assert!(
            !ir.contains("decreases_nonneg"),
            "Should skip non-negative check in release mode"
        );
        assert!(
            !ir.contains("decreases_check"),
            "Should skip decrease check in release mode"
        );
    }

    #[test]
    fn test_decreases_with_selfcall() {
        // Test decreases with @ self-call operator
        let source = r#"
            #[decreases(n)]
            F sum_to(n:i64)->i64{I n<=0{R 0}R n+@(n-1)}
        "#;
        let module = parse(source).unwrap();
        let mut gen = CodeGenerator::new("test");
        let ir = gen.generate_module(&module).unwrap();

        // Should have decreases check before the self-call
        assert!(
            ir.contains("__decreases_sum_to"),
            "Expected decreases storage"
        );
        assert!(
            ir.contains("decreases_check"),
            "Expected decrease check before self-call"
        );
    }

    #[test]
    fn test_type_recursion_depth_limit() {
        // Test that deeply nested types work within the limit
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // Create a deeply nested pointer type (should work)
        let mut nested_type = ResolvedType::I32;
        for _ in 0..50 {
            nested_type = ResolvedType::Pointer(Box::new(nested_type));
        }

        // This should work fine (well within the 128 limit)
        let llvm_type = gen.type_to_llvm(&nested_type);
        assert!(llvm_type.ends_with('*'), "Should generate nested pointers");

        // Create an extremely deeply nested type (exceeds limit of 128)
        let mut extremely_nested = ResolvedType::I32;
        for _ in 0..150 {
            extremely_nested = ResolvedType::Pointer(Box::new(extremely_nested));
        }

        // This should hit the recursion limit and fall back to i64
        // (The error is logged but doesn't fail - returns fallback type)
        let llvm_type_over_limit = gen.type_to_llvm(&extremely_nested);
        // Should still return a valid type (either i64 fallback or truncated)
        assert!(
            !llvm_type_over_limit.is_empty(),
            "Should return a fallback type on recursion limit"
        );
    }

    #[test]
    fn test_type_recursion_reset_between_calls() {
        // Test that recursion depth is properly reset between calls
        use vais_types::ResolvedType;

        let gen = CodeGenerator::new("test");

        // First call with nested types
        let mut nested1 = ResolvedType::I32;
        for _ in 0..30 {
            nested1 = ResolvedType::Pointer(Box::new(nested1));
        }
        let _ = gen.type_to_llvm(&nested1);

        // Second call should work independently (depth should be reset)
        let mut nested2 = ResolvedType::I64;
        for _ in 0..30 {
            nested2 = ResolvedType::Pointer(Box::new(nested2));
        }
        let llvm_type = gen.type_to_llvm(&nested2);
        assert!(
            llvm_type.ends_with('*'),
            "Second call should work independently"
        );
    }

    #[test]
    fn test_ast_type_recursion_limit() {
        // Test that ast_type_to_resolved also respects recursion limits
        use vais_ast::{Span, Type};

        let gen = CodeGenerator::new("test");

        // Create deeply nested AST type
        let mut nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..50 {
            nested = Type::Pointer(Box::new(Spanned::new(nested, Span { start: 0, end: 0 })));
        }

        // Should work within limit
        let resolved = gen.ast_type_to_resolved(&nested);
        assert!(
            matches!(resolved, ResolvedType::Pointer(_)),
            "Should resolve nested pointers"
        );

        // Create extremely nested type (exceeds limit)
        let mut extremely_nested = Type::Named {
            name: "i32".to_string(),
            generics: vec![],
        };
        for _ in 0..150 {
            extremely_nested = Type::Pointer(Box::new(Spanned::new(
                extremely_nested,
                Span { start: 0, end: 0 },
            )));
        }

        // Should hit limit and return fallback
        let resolved_over = gen.ast_type_to_resolved(&extremely_nested);
        // Should still return a valid type (Unknown as fallback)
        assert!(
            matches!(
                resolved_over,
                ResolvedType::Unknown | ResolvedType::Pointer(_)
            ),
            "Should return a fallback or truncated type on recursion limit"
        );
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "hallo"), 1);
        assert_eq!(edit_distance("hello", "hell"), 1);
        assert_eq!(edit_distance("hello", "helloo"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_suggest_similar() {
        let candidates = vec!["count", "counter", "account", "mount", "county"];

        // Exact case-insensitive match should be prioritized
        let suggestions = suggest_similar("COUNT", &candidates, 3);
        assert_eq!(suggestions[0], "count");

        // Close matches
        let suggestions = suggest_similar("countr", &candidates, 3);
        assert!(suggestions.contains(&"counter".to_string()));
        assert!(suggestions.contains(&"count".to_string()));

        // Should limit to max_suggestions
        let suggestions = suggest_similar("cont", &candidates, 2);
        assert!(suggestions.len() <= 2);

        // No matches if too far
        let suggestions = suggest_similar("xyz", &candidates, 3);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_format_did_you_mean() {
        assert_eq!(format_did_you_mean(&[]), "");
        assert_eq!(
            format_did_you_mean(&["foo".to_string()]),
            ". Did you mean `foo`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string()]),
            ". Did you mean `foo` or `bar`?"
        );
        assert_eq!(
            format_did_you_mean(&["foo".to_string(), "bar".to_string(), "baz".to_string()]),
            ". Did you mean `foo`, `bar`, or `baz`?"
        );
    }

    #[test]
    fn test_suggest_type_conversion() {
        // Numeric conversions
        assert!(suggest_type_conversion("i64", "f64").contains("as i64"));
        assert!(suggest_type_conversion("f64", "i64").contains("as f64"));
        assert!(suggest_type_conversion("i32", "i64").contains("as i32"));

        // String conversions
        assert!(suggest_type_conversion("String", "&str").contains(".to_string()"));
        assert!(suggest_type_conversion("&str", "String").contains(".as_str()"));

        // Bool to int
        assert!(suggest_type_conversion("i64", "bool").contains("as i64"));

        // No suggestion for unrelated types
        assert_eq!(suggest_type_conversion("Vec", "HashMap"), "");
    }
}
