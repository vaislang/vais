//! Windows-specific E2E Tests for the Vais Compiler
//!
//! These tests verify Windows-specific compilation scenarios including:
//! - Path separator handling (backslash escaping)
//! - IOCP async patterns
//! - Windows platform detection
//! - File I/O patterns
//! - Windows-specific FFI declarations
//! - Large memory layouts
//!
//! IMPORTANT: These tests run on ALL platforms and only verify that the code
//! compiles correctly. They do NOT require Windows or invoke actual Windows APIs.

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Compile Vais source to LLVM IR string
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker
        .check_module(&module)
        .map_err(|e| format!("Type error: {:?}", e))?;
    let mut gen = CodeGenerator::new("windows_test");
    // Pass resolved function signatures for inferred parameter type support
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Result of running a compiled program
struct RunResult {
    exit_code: i32,
    #[allow(dead_code)]
    stdout: String,
    #[allow(dead_code)]
    stderr: String,
}

/// Compile source, build executable with clang, run it, return exit code + output
fn compile_and_run(source: &str) -> Result<RunResult, String> {
    let ir = compile_to_ir(source)?;

    let tmp_dir =
        tempfile::TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ll_path = tmp_dir.path().join("test.ll");
    let exe_name = if cfg!(target_os = "windows") {
        "test_exe.exe"
    } else {
        "test_exe"
    };
    let exe_path = tmp_dir.path().join(exe_name);

    std::fs::write(&ll_path, &ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    let clang_output = std::process::Command::new("clang")
        .arg(&ll_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-Wno-override-module")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&clang_output.stderr);
        return Err(format!("clang compilation failed:\n{}", stderr));
    }

    let run_output = std::process::Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    let exit_code = run_output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

    Ok(RunResult {
        exit_code,
        stdout,
        stderr,
    })
}

/// Assert that source compiles, runs, and returns the expected exit code
fn assert_exit_code(source: &str, expected: i32) {
    match compile_and_run(source) {
        Ok(result) => {
            assert_eq!(
                result.exit_code, expected,
                "Expected exit code {}, got {}.\nstdout: {}\nstderr: {}",
                expected, result.exit_code, result.stdout, result.stderr
            );
        }
        Err(e) => panic!("Compilation/execution failed: {}", e),
    }
}

/// Assert that IR contains expected string (currently unused but kept for future tests)
#[allow(dead_code)]
fn assert_ir_contains(source: &str, expected: &str) {
    match compile_to_ir(source) {
        Ok(ir) => {
            assert!(
                ir.contains(expected),
                "Expected IR to contain {:?}, but it didn't.\nGenerated IR:\n{}",
                expected,
                ir
            );
        }
        Err(e) => panic!("Compilation failed: {}", e),
    }
}

// ==================== 1. Windows Path Separator Handling ====================

#[test]
fn windows_path_backslash_escaping() {
    // Test that backslashes in string literals are properly escaped
    let source = r#"
F get_windows_path() -> i64 {
    path := "C:\\Users\\Admin\\file.txt"
    # Backslashes must be escaped in Vais strings
    0
}
F main() -> i64 = get_windows_path()
"#;
    assert_exit_code(source, 0);
}

#[test]
fn windows_path_raw_string() {
    // Test path handling with string constants (no extern dependency)
    let source = r#"
F check_windows_path() -> i64 {
    path := "C:\\Program Files\\MyApp\\config.ini"
    # String literal compiles — test passes if code reaches this point
    0
}
F main() -> i64 = check_windows_path()
"#;
    // Verifies Windows path with backslashes compiles and runs correctly
    assert_exit_code(source, 0);
}

// ==================== 2. Path String Concatenation ====================

#[test]
fn windows_path_concatenation() {
    // Test building paths with string operations (no extern dependency)
    let source = r#"
F check_path() -> i64 {
    # Test Windows path string creation
    path := "C:\\Users\\data.txt"
    # String literal compiles — test passes if code reaches this point
    0
}
F main() -> i64 {
    result := check_path()
    result
}
"#;
    // Verifies Windows path string compiles and runs correctly
    assert_exit_code(source, 0);
}

// ==================== 3. Environment Variable Access Pattern ====================

#[test]
fn windows_env_var_pattern() {
    // Test Windows environment variable access pattern
    let source = r#"
# Mock getenv for testing Windows environment variable patterns
F getenv_mock() -> i64 {
    # Simulate getenv("USERPROFILE") returning a pointer
    # In real usage: getenv("USERPROFILE") would be called
    12345678
}

F check_user_profile() -> i64 {
    # On Windows, %USERPROFILE% = C:\Users\Username
    # Test pattern: check if environment variable exists
    profile := getenv_mock()
    I profile == 0 {
        # NULL means not found
        1
    } E {
        # Found - return success
        0
    }
}
F main() -> i64 = check_user_profile()
"#;
    // getenv_mock is a regular Vais function (not extern), returns 12345678 → profile != 0 → 0
    assert_exit_code(source, 0);
}

// ==================== 4. Process Exit Code Handling ====================

#[test]
fn windows_exit_code_handling() {
    // Windows uses same exit code conventions as Unix
    let source = r#"
C EXIT_SUCCESS: i64 = 0
C EXIT_FAILURE: i64 = 1
C EXIT_INVALID_ARGUMENT: i64 = 2

F validate_input(x: i64) -> i64 {
    I x < 0 {
        EXIT_INVALID_ARGUMENT
    } E I x == 0 {
        EXIT_FAILURE
    } E {
        EXIT_SUCCESS
    }
}
F main() -> i64 = validate_input(42)
"#;
    // validate_input(42): x=42 >= 0, x != 0, so EXIT_SUCCESS = 0
    assert_exit_code(source, 0);
}

// ==================== 5. File I/O Pattern (open/read/write/close) ====================

#[test]
fn windows_file_io_pattern() {
    // Test Windows file I/O struct pattern (no extern dependency)
    let source = r#"
S FileHandle {
    ptr: i64,
    is_open: i64
}

X FileHandle {
    F new(handle: i64) -> FileHandle {
        I handle == 0 {
            FileHandle { ptr: 0, is_open: 0 }
        } E {
            FileHandle { ptr: handle, is_open: 1 }
        }
    }

    F close(&self) -> i64 {
        I self.is_open == 1 {
            # Mock close: return success
            0
        } E {
            0
        }
    }
}

F main() -> i64 {
    # Test FileHandle struct with mock handle (simulates fopen returning 42)
    file_open := FileHandle::new(42)
    file_null := FileHandle::new(0)
    # open handle: is_open=1, null handle: is_open=0
    file_open.is_open + file_null.is_open
}
"#;
    // FileHandle::new(42).is_open=1, FileHandle::new(0).is_open=0, so 1 + 0 = 1
    assert_exit_code(source, 1);
}

// ==================== 6. IOCP Async Constants Compilation ====================

#[test]
fn windows_iocp_constants() {
    // Test IOCP (I/O Completion Port) constants used in async_reactor.vais
    let source = r#"
# Windows IOCP platform constants
C PLATFORM_UNKNOWN: i64 = 0
C PLATFORM_MACOS: i64 = 1
C PLATFORM_LINUX: i64 = 2
C PLATFORM_WINDOWS: i64 = 3

# IOCP event types
C REACTOR_READ: i64 = -1
C REACTOR_WRITE: i64 = -2
C REACTOR_TIMER: i64 = -7

# IOCP action flags
C REACTOR_ADD: i64 = 1
C REACTOR_DELETE: i64 = 2
C REACTOR_ONESHOT: i64 = 16

# Max events per GetQueuedCompletionStatusEx call
C REACTOR_MAX_EVENTS: i64 = 64

F main() -> i64 = PLATFORM_WINDOWS
"#;
    // PLATFORM_WINDOWS = 3
    assert_exit_code(source, 3);
}

// ==================== 7. Windows Async Platform Detection ====================

#[test]
fn windows_async_platform_detection() {
    // Test platform detection pattern used in async runtime (no extern dependency)
    let source = r#"
C PLATFORM_MACOS: i64 = 1
C PLATFORM_LINUX: i64 = 2
C PLATFORM_WINDOWS: i64 = 3

# Mock platform detection: returns PLATFORM_WINDOWS
F mock_platform() -> i64 = PLATFORM_WINDOWS

S Reactor {
    platform: i64,
    backend_fd: i64
}

X Reactor {
    F new() -> Reactor {
        platform := mock_platform()
        backend_fd := I platform == PLATFORM_WINDOWS {
            1
        } E I platform == PLATFORM_LINUX {
            2
        } E {
            3
        }
        Reactor { platform: platform, backend_fd: backend_fd }
    }

    F is_windows(&self) -> i64 {
        I self.platform == PLATFORM_WINDOWS { 1 } E { 0 }
    }
}

F main() -> i64 {
    reactor := Reactor::new()
    reactor.is_windows()
}
"#;
    // mock_platform returns PLATFORM_WINDOWS=3, is_windows returns 1
    assert_exit_code(source, 1);
}

// ==================== 8. Network Socket Constants ====================

#[test]
fn windows_socket_constants() {
    // Windows uses Winsock2, but constants are similar to POSIX
    let source = r#"
# Socket types
C SOCK_STREAM: i64 = 1    # TCP
C SOCK_DGRAM: i64 = 2     # UDP

# Address families
C AF_INET: i64 = 2        # IPv4
C AF_INET6: i64 = 23      # IPv6 (Windows uses 23, not 10)

# Socket options
C SOL_SOCKET: i64 = 65535 # Windows-specific value
C SO_REUSEADDR: i64 = 4
C SO_KEEPALIVE: i64 = 8

# Winsock error codes (subset)
C WSAEWOULDBLOCK: i64 = 10035
C WSAECONNRESET: i64 = 10054

S Socket {
    fd: i64,
    family: i64,
    sock_type: i64
}

X Socket {
    F new(family: i64, sock_type: i64) -> Socket {
        Socket { fd: 0, family: family, sock_type: sock_type }
    }

    F is_tcp(&self) -> i64 {
        I self.sock_type == SOCK_STREAM { 1 } E { 0 }
    }
}

F main() -> i64 {
    sock := Socket::new(AF_INET, SOCK_STREAM)
    sock.is_tcp()
}
"#;
    // sock = Socket::new(AF_INET, SOCK_STREAM), is_tcp() checks sock_type == SOCK_STREAM → 1
    assert_exit_code(source, 1);
}

// ==================== 9. Multi-threading Pattern ====================

#[test]
fn windows_threading_pattern() {
    // Test Windows threading struct pattern (no extern dependency)
    let source = r#"
S Thread {
    handle: i64,
    thread_id: i64
}

X Thread {
    F spawn(func_id: i64, arg: i64) -> Thread {
        # Mock: simulate thread creation
        handle := func_id + arg
        tid := 1001
        Thread { handle: handle, thread_id: tid }
    }

    F join(&self) -> i64 {
        # Mock: return the handle as result
        self.handle
    }
}

F worker_func(arg: i64) -> i64 {
    arg * 2
}

F main() -> i64 {
    thread := Thread::spawn(0, 42)
    result := thread.join()
    # handle = 0 + 42 = 42
    result
}
"#;
    // Thread::spawn(0, 42) → handle=42, join → 42
    assert_exit_code(source, 42);
}

// ==================== 10. TLS + Windows Path Combination ====================

#[test]
fn windows_tls_with_paths() {
    // Test TLS certificate loading with Windows paths
    let source = r#"
# Mock TLS functions for testing Windows path patterns
F tls_load_cert_mock() -> i64 {
    # Simulate loading from "C:\\certs\\server.crt"
    0
}

F tls_load_key_mock() -> i64 {
    # Simulate loading from "D:\\ssl\\custom.key"
    0
}

# TLS configuration structure
S TlsConfig {
    verify_peer: i64,
    cert_loaded: i64,
    key_loaded: i64
}

X TlsConfig {
    F new() -> TlsConfig {
        TlsConfig {
            verify_peer: 1,
            cert_loaded: 0,
            key_loaded: 0
        }
    }

    F load_cert(&self) -> i64 {
        # Test Windows cert path pattern
        result := tls_load_cert_mock()
        I result == 0 { 1 } E { 0 }
    }

    F load_key(&self) -> i64 {
        # Test Windows key path pattern
        result := tls_load_key_mock()
        I result == 0 { 1 } E { 0 }
    }
}

F main() -> i64 {
    config := TlsConfig::new()
    cert_ok := config.load_cert()
    key_ok := config.load_key()
    I cert_ok == 1 { 0 } E { 1 }
}
"#;
    // No extern calls in main path — all mock functions are regular Vais functions
    // cert_ok = 1 (tls_load_cert_mock returns 0 → 1), so returns 0
    assert_exit_code(source, 0);
}

// ==================== 11. Structured Logging + File Paths ====================

#[test]
fn windows_logging_with_file_paths() {
    // Test logging system with Windows file paths
    let source = r#"
C LOG_LEVEL_ERROR: i64 = 1
C LOG_LEVEL_WARN: i64 = 2
C LOG_LEVEL_INFO: i64 = 3
C LOG_LEVEL_DEBUG: i64 = 4

# Mock logging functions for testing Windows path patterns
F log_open_mock(append: i64) -> i64 {
    # Simulate opening "C:\\logs\\app.log"
    0
}

F log_set_level_mock(level: i64) -> i64 {
    0
}

S LogConfig {
    level: i64,
    append_mode: i64,
    is_open: i64
}

X LogConfig {
    F new() -> LogConfig {
        LogConfig {
            level: LOG_LEVEL_INFO,
            append_mode: 1,
            is_open: 0
        }
    }

    F set_level(&self, level: i64) -> LogConfig {
        LogConfig {
            level: level,
            append_mode: self.append_mode,
            is_open: self.is_open
        }
    }

    F open(&self) -> i64 {
        # Test Windows log path pattern
        result := log_open_mock(self.append_mode)
        log_set_level_mock(self.level)
        result
    }
}

F main() -> i64 {
    config := LogConfig::new()
    config2 := config.set_level(LOG_LEVEL_DEBUG)
    result := config2.open()
    I result == 0 { 0 } E { 1 }
}
"#;
    // No extern calls — all mock functions, result=0 → returns 0
    assert_exit_code(source, 0);
}

// ==================== 12. Compression + File I/O ====================

#[test]
fn windows_compression_with_file_io() {
    // Test compression operations with Windows file paths
    let source = r#"
# Compression algorithms
C COMPRESS_NONE: i64 = 0
C COMPRESS_GZIP: i64 = 1
C COMPRESS_DEFLATE: i64 = 2
C COMPRESS_ZSTD: i64 = 3

# Compression levels
C COMPRESS_FAST: i64 = 1
C COMPRESS_DEFAULT: i64 = 6
C COMPRESS_BEST: i64 = 9

# Mock compression function for testing Windows path patterns
F compress_file_mock(algorithm: i64, level: i64) -> i64 {
    # Simulate compressing "C:\\data\\large_file.bin"
    # to "C:\\data\\large_file.bin.gz"
    0
}

S CompressConfig {
    algorithm: i64,
    level: i64
}

X CompressConfig {
    F new() -> CompressConfig {
        CompressConfig {
            algorithm: COMPRESS_GZIP,
            level: COMPRESS_DEFAULT
        }
    }

    F with_algorithm(&self, algo: i64) -> CompressConfig {
        CompressConfig {
            algorithm: algo,
            level: self.level
        }
    }

    F compress(&self) -> i64 {
        # Test Windows file path pattern for compression
        result := compress_file_mock(self.algorithm, self.level)
        result
    }
}

F main() -> i64 {
    config := CompressConfig::new()
    config2 := config.with_algorithm(COMPRESS_ZSTD)
    result := config2.compress()
    0
}
"#;
    // No extern calls — all mock functions, main returns 0
    assert_exit_code(source, 0);
}

// ==================== 13. Cross-platform Conditional Compilation ====================

#[test]
fn windows_cross_platform_conditional() {
    // Test cross-platform code patterns (no extern dependency)
    let source = r#"
C PLATFORM_WINDOWS: i64 = 3
C PLATFORM_LINUX: i64 = 2
C PLATFORM_MACOS: i64 = 1

# Mock platform detection
F mock_get_platform() -> i64 = PLATFORM_MACOS

F create_directory_mock(platform: i64) -> i64 {
    0
}

S PathConfig {
    platform: i64,
    initialized: i64
}

X PathConfig {
    F new() -> PathConfig {
        PathConfig {
            platform: mock_get_platform(),
            initialized: 0
        }
    }

    F initialize(&self) -> i64 {
        result := create_directory_mock(self.platform)
        result
    }

    F is_windows(&self) -> i64 {
        I self.platform == PLATFORM_WINDOWS { 1 } E { 0 }
    }
}

F main() -> i64 {
    config := PathConfig::new()
    result := config.initialize()
    is_win := config.is_windows()
    # mock returns MACOS=1, is_windows → 0, initialize → 0
    result + is_win
}
"#;
    // initialize returns 0, is_windows returns 0 (MACOS != WINDOWS), so 0 + 0 = 0
    assert_exit_code(source, 0);
}

// ==================== 14. Windows Registry FFI Declaration Pattern ====================

#[test]
fn windows_registry_ffi_pattern() {
    // Test Windows Registry constants and struct patterns (no extern dependency)
    let source = r#"
# Registry root keys (HKEY) - using decimal values
C HKEY_CLASSES_ROOT: i64 = 2147483648
C HKEY_CURRENT_USER: i64 = 2147483649
C HKEY_LOCAL_MACHINE: i64 = 2147483650
C HKEY_USERS: i64 = 2147483651

# Registry access rights
C KEY_READ: i64 = 131097
C KEY_WRITE: i64 = 131078
C KEY_ALL_ACCESS: i64 = 983103

# Registry value types
C REG_SZ: i64 = 1        # String
C REG_DWORD: i64 = 4     # 32-bit number

# Mock registry open — simulates success (returns 0)
F mock_reg_open(root: i64, access: i64) -> i64 {
    I root == 0 { 1 } E { 0 }
}

S RegistryKey {
    handle: i64,
    is_open: i64
}

X RegistryKey {
    F open(root: i64, access: i64) -> RegistryKey {
        result := mock_reg_open(root, access)
        I result == 0 {
            RegistryKey { handle: root, is_open: 1 }
        } E {
            RegistryKey { handle: 0, is_open: 0 }
        }
    }

    F close(&self) -> i64 {
        I self.is_open == 1 { 0 } E { 1 }
    }
}

F main() -> i64 {
    key := RegistryKey::open(HKEY_LOCAL_MACHINE, KEY_READ)
    # HKEY_LOCAL_MACHINE != 0, so mock returns 0 (success) → is_open=1
    key.is_open
}
"#;
    // mock_reg_open(HKEY_LOCAL_MACHINE, KEY_READ) → 0 (success), is_open = 1
    assert_exit_code(source, 1);
}

// ==================== 15. Large Struct Array Pattern (Memory Layout) ====================

#[test]
fn windows_large_struct_array() {
    // Test large structure arrays (important for memory layout on Windows)
    let source = r#"
# Large structure representing Windows OVERLAPPED structure
S OverlappedIO {
    internal: i64,
    internal_high: i64,
    offset: i64,
    offset_high: i64,
    event_handle: i64,
    user_data: i64,
    buffer_ptr: i64,
    buffer_size: i64
}

# IOCP event buffer
S IOCPEventBuffer {
    events: i64,
    capacity: i64,
    count: i64
}

X IOCPEventBuffer {
    F new(capacity: i64) -> IOCPEventBuffer {
        # In real code, would allocate array of OverlappedIO structures
        # Calculate size: capacity * size_of(OverlappedIO)
        # OverlappedIO has 8 i64 fields = 8 * 8 = 64 bytes
        size := capacity * 64
        events_ptr := malloc(size)
        IOCPEventBuffer {
            events: events_ptr,
            capacity: capacity,
            count: 0
        }
    }

    F get_event(&self, index: i64) -> i64 {
        I index >= self.count {
            0
        } E {
            # Would return pointer to events[index]
            1
        }
    }
}

# Large configuration structure
S AsyncIOConfig {
    thread_count: i64,
    event_buffer_size: i64,
    timeout_ms: i64,
    max_concurrent_ops: i64,
    enable_logging: i64,
    retry_count: i64,
    retry_delay_ms: i64,
    buffer_size: i64,
    use_scatter_gather: i64
}

X AsyncIOConfig {
    F default() -> AsyncIOConfig {
        AsyncIOConfig {
            thread_count: 4,
            event_buffer_size: 64,
            timeout_ms: 1000,
            max_concurrent_ops: 256,
            enable_logging: 1,
            retry_count: 3,
            retry_delay_ms: 100,
            buffer_size: 4096,
            use_scatter_gather: 0
        }
    }

    F get_log_path(&self) -> i64 {
        # Would return "C:\\logs\\async_io.log" in real usage
        # Testing Windows path pattern compilation
        0
    }
}

F main() -> i64 {
    config := AsyncIOConfig::default()
    buffer := IOCPEventBuffer::new(config.event_buffer_size)
    event := buffer.get_event(0)
    0
}
"#;
    // malloc is available through libc, main returns 0
    assert_exit_code(source, 0);
}

// ==================== Bonus: Complex Windows Integration Test ====================

#[test]
fn windows_complex_integration() {
    // Comprehensive test combining multiple Windows patterns (no extern dependency)
    let source = r#"
C PLATFORM_WINDOWS: i64 = 3
C EXIT_SUCCESS: i64 = 0
C EXIT_FAILURE: i64 = 1

# Mock functions
F mock_platform() -> i64 = PLATFORM_WINDOWS
F mock_open_config() -> i64 = 42

S WindowsApp {
    platform: i64,
    has_config: i64,
    is_initialized: i64
}

X WindowsApp {
    F new() -> WindowsApp {
        WindowsApp {
            platform: mock_platform(),
            has_config: 0,
            is_initialized: 0
        }
    }

    F check_config(&self) -> i64 {
        handle := mock_open_config()
        I handle == 0 { 0 } E { 1 }
    }

    F initialize(&self) -> i64 {
        I self.platform != PLATFORM_WINDOWS {
            EXIT_FAILURE
        } E {
            has_cfg := self.check_config()
            I has_cfg == 0 {
                EXIT_FAILURE
            } E {
                EXIT_SUCCESS
            }
        }
    }
}

F main() -> i64 {
    app := WindowsApp::new()
    result := app.initialize()
    result
}
"#;
    // mock_platform=WINDOWS, mock_open_config=42 (non-null) → has_cfg=1 → EXIT_SUCCESS=0
    assert_exit_code(source, 0);
}
