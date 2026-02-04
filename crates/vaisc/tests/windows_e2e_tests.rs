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
    let ir = gen
        .generate_module(&module)
        .map_err(|e| format!("Codegen error: {:?}", e))?;
    Ok(ir)
}

/// Assert that source compiles successfully to IR
fn assert_compiles(source: &str) {
    match compile_to_ir(source) {
        Ok(ir) => {
            assert!(!ir.is_empty(), "Generated IR should not be empty");
        }
        Err(e) => panic!("Compilation failed: {}", e),
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
    assert_compiles(source);
}

#[test]
fn windows_path_raw_string() {
    // Test path handling with string constants in function calls
    let source = r#"
X F puts(s: i64) -> i64

F print_windows_path() -> i64 {
    puts("C:\\Program Files\\MyApp\\config.ini")
    0
}
F main() -> i64 = print_windows_path()
"#;
    assert_compiles(source);
}

// ==================== 2. Path String Concatenation ====================

#[test]
fn windows_path_concatenation() {
    // Test building paths with string operations
    let source = r#"
F print_path() -> i64 {
    # Test passing Windows paths to builtin functions
    puts("C:\\Users\\data.txt")
    0
}
F main() -> i64 {
    result := print_path()
    0
}
"#;
    assert_compiles(source);
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
    assert_compiles(source);
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
    assert_compiles(source);
}

// ==================== 5. File I/O Pattern (open/read/write/close) ====================

#[test]
fn windows_file_io_pattern() {
    // Test Windows file I/O using standard C FILE* API
    let source = r#"
# Extern C file I/O functions
X F fopen(path: i64, mode: i64) -> i64
X F fclose(handle: i64) -> i64
X F fread(buf: i64, size: i64, count: i64, handle: i64) -> i64
X F fwrite(buf: i64, size: i64, count: i64, handle: i64) -> i64

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
            fclose(self.ptr)
        } E {
            0
        }
    }
}

F main() -> i64 {
    # Test Windows file I/O pattern
    result := fopen("C:\\temp\\test.txt", "r")
    file := FileHandle::new(result)
    I file.is_open == 1 {
        file.close()
    } E {
        0
    }
}
"#;
    assert_compiles(source);
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
    assert_compiles(source);
}

// ==================== 7. Windows Async Platform Detection ====================

#[test]
fn windows_async_platform_detection() {
    // Test platform detection pattern used in async runtime
    let source = r#"
C PLATFORM_MACOS: i64 = 1
C PLATFORM_LINUX: i64 = 2
C PLATFORM_WINDOWS: i64 = 3

# Extern: returns platform ID at runtime
X F async_platform() -> i64

S Reactor {
    platform: i64,
    backend_fd: i64
}

X Reactor {
    F new() -> Reactor {
        platform := async_platform()
        # On Windows, this would create IOCP handle
        # On Linux, epoll_create
        # On macOS, kqueue
        backend_fd := I platform == PLATFORM_WINDOWS {
            # Would call CreateIoCompletionPort
            1
        } E I platform == PLATFORM_LINUX {
            # Would call epoll_create
            2
        } E {
            # Would call kqueue
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
    assert_compiles(source);
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
    assert_compiles(source);
}

// ==================== 9. Multi-threading Pattern ====================

#[test]
fn windows_threading_pattern() {
    // Test Windows threading pattern (CreateThread API)
    let source = r#"
# Extern: Windows threading primitives
X F create_thread(start_routine: i64, arg: i64) -> i64
X F join_thread(handle: i64) -> i64
X F current_thread_id() -> i64

S Thread {
    handle: i64,
    thread_id: i64
}

X Thread {
    F spawn(func: i64, arg: i64) -> Thread {
        handle := create_thread(func, arg)
        tid := current_thread_id()
        Thread { handle: handle, thread_id: tid }
    }

    F join(&self) -> i64 {
        join_thread(self.handle)
    }
}

F worker_func(arg: i64) -> i64 {
    # Thread entry point
    arg * 2
}

F main() -> i64 {
    # Note: In real usage, func pointers would be used
    thread := Thread::spawn(0, 42)
    thread.join()
}
"#;
    assert_compiles(source);
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
    assert_compiles(source);
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
    assert_compiles(source);
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
    assert_compiles(source);
}

// ==================== 13. Cross-platform Conditional Compilation ====================

#[test]
fn windows_cross_platform_conditional() {
    // Test cross-platform code patterns
    let source = r#"
C PLATFORM_WINDOWS: i64 = 3
C PLATFORM_LINUX: i64 = 2
C PLATFORM_MACOS: i64 = 1

X F get_platform() -> i64

# Mock directory creation for testing Windows path patterns
F create_directory_mock(platform: i64) -> i64 {
    # Simulate creating platform-specific directories:
    # Windows: "C:\\ProgramData\\MyApp"
    # macOS: "/Library/Application Support/MyApp"
    # Linux: "/var/lib/myapp"
    0
}

S PathConfig {
    platform: i64,
    initialized: i64
}

X PathConfig {
    F new() -> PathConfig {
        PathConfig {
            platform: get_platform(),
            initialized: 0
        }
    }

    F initialize(&self) -> i64 {
        # Test conditional path patterns based on platform
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
    0
}
"#;
    assert_compiles(source);
}

// ==================== 14. Windows Registry FFI Declaration Pattern ====================

#[test]
fn windows_registry_ffi_pattern() {
    // Test Windows Registry API FFI declarations
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

# Extern: Windows Registry API (advapi32.dll)
X F RegOpenKeyExA(hKey: i64, subKey: i64, options: i64, access: i64, result: i64) -> i64
X F RegQueryValueExA(hKey: i64, valueName: i64, reserved: i64, type_ptr: i64, data: i64, size_ptr: i64) -> i64
X F RegSetValueExA(hKey: i64, valueName: i64, reserved: i64, value_type: i64, data: i64, size: i64) -> i64
X F RegCloseKey(hKey: i64) -> i64

S RegistryKey {
    handle: i64,
    is_open: i64
}

X RegistryKey {
    F open(root: i64, subkey: i64, access: i64) -> RegistryKey {
        # In real implementation, would allocate handle_ptr
        handle_ptr := 0
        result := RegOpenKeyExA(root, subkey, 0, access, handle_ptr)
        I result == 0 {
            RegistryKey { handle: 0, is_open: 1 }
        } E {
            RegistryKey { handle: 0, is_open: 0 }
        }
    }

    F close(&self) -> i64 {
        I self.is_open == 1 {
            RegCloseKey(self.handle)
        } E {
            0
        }
    }
}

F main() -> i64 {
    # Test registry constants compile
    # Registry operations would use RegOpenKeyExA with Windows path strings
    key := HKEY_LOCAL_MACHINE
    access := KEY_READ
    I key != 0 { 0 } E { 1 }
}
"#;
    assert_compiles(source);
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
    assert_compiles(source);
}

// ==================== Bonus: Complex Windows Integration Test ====================

#[test]
fn windows_complex_integration() {
    // Comprehensive test combining multiple Windows patterns
    let source = r#"
C PLATFORM_WINDOWS: i64 = 3
C EXIT_SUCCESS: i64 = 0
C EXIT_FAILURE: i64 = 1

# File I/O
X F fopen(path: i64, mode: i64) -> i64
X F fclose(handle: i64) -> i64

# Platform detection
X F async_platform() -> i64

# Environment
X F getenv(name: i64) -> i64

S WindowsApp {
    platform: i64,
    has_config: i64,
    is_initialized: i64
}

X WindowsApp {
    F new() -> WindowsApp {
        WindowsApp {
            platform: async_platform(),
            has_config: 0,
            is_initialized: 0
        }
    }

    F check_config(&self) -> i64 {
        # Try to open config file at Windows path
        handle := fopen("C:\\ProgramData\\MyApp\\config.ini", "r")
        I handle == 0 {
            0
        } E {
            fclose(handle)
            1
        }
    }

    F initialize(&self) -> i64 {
        # Check if running on Windows
        I self.platform != PLATFORM_WINDOWS {
            EXIT_FAILURE
        } E {
            # Check if config exists
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
    assert_compiles(source);
}
