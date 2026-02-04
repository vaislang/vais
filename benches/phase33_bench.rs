//! Phase 33 Performance Benchmarks (Stages 1-7)
//!
//! Measures compilation speed for Phase 33 features:
//! 1. TLS/HTTPS compilation (Stage 1)
//! 2. Async reactor compilation (Stage 2)
//! 3. Logging library compilation (Stage 4)
//! 4. Compression library compilation (Stage 5)
//! 5. Combined Phase 33 feature compilation

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ============================================================
// 1. TLS/HTTPS Compilation Speed (Phase 33 Stage 1)
// ============================================================

const TLS_SOURCE: &str = r#"
# TLS/HTTPS standard library constants and types

C TLS_VERSION_1_2: i64 = 771
C TLS_VERSION_1_3: i64 = 772
C TLS_HANDSHAKE_CLIENT_HELLO: i64 = 1
C TLS_HANDSHAKE_SERVER_HELLO: i64 = 2

S TlsConfig {
    version: i64,
    cert_path: i64,
    key_path: i64,
    verify_peer: i64,
}

S TlsConnection {
    socket_fd: i64,
    state: i64,
    cipher_suite: i64,
}

X F tls_init(config: TlsConfig) -> i64
X F tls_connect(host: i64, port: i64, config: TlsConfig) -> TlsConnection
X F tls_accept(socket_fd: i64, config: TlsConfig) -> TlsConnection
X F tls_read(conn: TlsConnection, buffer: i64, size: i64) -> i64
X F tls_write(conn: TlsConnection, data: i64, size: i64) -> i64
X F tls_close(conn: TlsConnection) -> i64

F create_tls_config(verify: i64) -> TlsConfig {
    config := TlsConfig {
        version: TLS_VERSION_1_3,
        cert_path: 0,
        key_path: 0,
        verify_peer: verify,
    }
    R config
}

F main() -> i64 {
    config := create_tls_config(1)
    R config.version
}
"#;

const TLS_LARGE_SOURCE: &str = r#"
# Larger TLS/HTTPS code for scaling test

C TLS_VERSION_1_0: i64 = 769
C TLS_VERSION_1_1: i64 = 770
C TLS_VERSION_1_2: i64 = 771
C TLS_VERSION_1_3: i64 = 772

C TLS_HANDSHAKE_CLIENT_HELLO: i64 = 1
C TLS_HANDSHAKE_SERVER_HELLO: i64 = 2
C TLS_HANDSHAKE_CERTIFICATE: i64 = 11
C TLS_HANDSHAKE_SERVER_KEY_EXCHANGE: i64 = 12
C TLS_HANDSHAKE_CERTIFICATE_REQUEST: i64 = 13
C TLS_HANDSHAKE_SERVER_HELLO_DONE: i64 = 14
C TLS_HANDSHAKE_CERTIFICATE_VERIFY: i64 = 15
C TLS_HANDSHAKE_CLIENT_KEY_EXCHANGE: i64 = 16
C TLS_HANDSHAKE_FINISHED: i64 = 20

S TlsConfig {
    version: i64,
    cert_path: i64,
    key_path: i64,
    verify_peer: i64,
    cipher_suites: i64,
    alpn_protocols: i64,
}

S TlsConnection {
    socket_fd: i64,
    state: i64,
    cipher_suite: i64,
    protocol_version: i64,
    session_id: i64,
}

S TlsHandshake {
    msg_type: i64,
    length: i64,
    data: i64,
}

X F tls_init(config: TlsConfig) -> i64
X F tls_connect(host: i64, port: i64, config: TlsConfig) -> TlsConnection
X F tls_accept(socket_fd: i64, config: TlsConfig) -> TlsConnection
X F tls_read(conn: TlsConnection, buffer: i64, size: i64) -> i64
X F tls_write(conn: TlsConnection, data: i64, size: i64) -> i64
X F tls_close(conn: TlsConnection) -> i64
X F tls_get_peer_certificate(conn: TlsConnection) -> i64
X F tls_verify_certificate(cert: i64) -> i64

F create_tls_config(verify: i64) -> TlsConfig {
    config := TlsConfig {
        version: TLS_VERSION_1_3,
        cert_path: 0,
        key_path: 0,
        verify_peer: verify,
        cipher_suites: 0,
        alpn_protocols: 0,
    }
    R config
}

F create_client_hello() -> TlsHandshake {
    handshake := TlsHandshake {
        msg_type: TLS_HANDSHAKE_CLIENT_HELLO,
        length: 256,
        data: 0,
    }
    R handshake
}

F create_server_hello() -> TlsHandshake {
    handshake := TlsHandshake {
        msg_type: TLS_HANDSHAKE_SERVER_HELLO,
        length: 128,
        data: 0,
    }
    R handshake
}

F validate_handshake(handshake: TlsHandshake) -> i64 {
    I handshake.msg_type == TLS_HANDSHAKE_CLIENT_HELLO {
        R 1
    }
    I handshake.msg_type == TLS_HANDSHAKE_SERVER_HELLO {
        R 1
    }
    R 0
}

F main() -> i64 {
    config := create_tls_config(1)
    client_hello := create_client_hello()
    server_hello := create_server_hello()
    valid := validate_handshake(client_hello)
    R valid + config.version
}
"#;

fn bench_tls_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_tls");

    // Benchmark small TLS source
    let bytes = TLS_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("tls_basic_lex", |b| {
        b.iter(|| tokenize(black_box(TLS_SOURCE)))
    });

    group.bench_function("tls_basic_parse", |b| {
        b.iter(|| parse(black_box(TLS_SOURCE)))
    });

    group.bench_function("tls_basic_typecheck", |b| {
        b.iter(|| {
            let ast = parse(black_box(TLS_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    group.bench_function("tls_basic_codegen", |b| {
        b.iter(|| {
            let ast = parse(black_box(TLS_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("tls_bench");
            codegen.generate_module(&ast)
        })
    });

    // Benchmark larger TLS source
    let large_bytes = TLS_LARGE_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(large_bytes));

    group.bench_function("tls_large_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(TLS_LARGE_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("tls_large_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 2. Async Reactor Compilation Speed (Phase 33 Stage 2)
// ============================================================

const ASYNC_REACTOR_SOURCE: &str = r#"
# Async reactor platform detection and event loop

C PLATFORM_LINUX: i64 = 1
C PLATFORM_MACOS: i64 = 2
C PLATFORM_WINDOWS: i64 = 3

C EVENT_READ: i64 = 1
C EVENT_WRITE: i64 = 2
C EVENT_ERROR: i64 = 4

S EventLoop {
    platform: i64,
    epoll_fd: i64,
    kqueue_fd: i64,
    iocp_handle: i64,
}

S Event {
    fd: i64,
    events: i64,
    user_data: i64,
}

X F detect_platform() -> i64
X F epoll_create(size: i64) -> i64
X F kqueue_create() -> i64
X F iocp_create() -> i64
X F event_loop_add(loop: EventLoop, fd: i64, events: i64) -> i64
X F event_loop_wait(loop: EventLoop, events: i64, max_events: i64, timeout: i64) -> i64

F create_event_loop() -> EventLoop {
    platform := detect_platform()
    epoll_val := platform == PLATFORM_LINUX ? epoll_create(1024) : 0
    kqueue_val := platform == PLATFORM_MACOS ? kqueue_create() : 0
    iocp_val := platform == PLATFORM_WINDOWS ? iocp_create() : 0

    loop := EventLoop {
        platform: platform,
        epoll_fd: epoll_val,
        kqueue_fd: kqueue_val,
        iocp_handle: iocp_val,
    }
    R loop
}

F register_event(loop: EventLoop, fd: i64, events: i64) -> i64 {
    result := event_loop_add(loop, fd, events)
    R result
}

F main() -> i64 {
    loop := create_event_loop()
    result := register_event(loop, 42, EVENT_READ)
    R result
}
"#;

const ASYNC_REACTOR_LARGE_SOURCE: &str = r#"
# Larger async reactor with task scheduling

C PLATFORM_LINUX: i64 = 1
C PLATFORM_MACOS: i64 = 2
C PLATFORM_WINDOWS: i64 = 3
C PLATFORM_BSD: i64 = 4

C EVENT_READ: i64 = 1
C EVENT_WRITE: i64 = 2
C EVENT_ERROR: i64 = 4
C EVENT_HUP: i64 = 8
C EVENT_EDGE_TRIGGER: i64 = 16

C TASK_STATE_PENDING: i64 = 0
C TASK_STATE_READY: i64 = 1
C TASK_STATE_RUNNING: i64 = 2
C TASK_STATE_COMPLETED: i64 = 3

S EventLoop {
    platform: i64,
    epoll_fd: i64,
    kqueue_fd: i64,
    iocp_handle: i64,
    task_queue: i64,
}

S Event {
    fd: i64,
    events: i64,
    user_data: i64,
}

S Task {
    id: i64,
    state: i64,
    waker: i64,
    future: i64,
}

X F detect_platform() -> i64
X F epoll_create(size: i64) -> i64
X F kqueue_create() -> i64
X F iocp_create() -> i64
X F event_loop_add(loop: EventLoop, fd: i64, events: i64) -> i64
X F event_loop_wait(loop: EventLoop, events: i64, max_events: i64, timeout: i64) -> i64
X F task_queue_create() -> i64
X F task_queue_push(queue: i64, task: Task) -> i64
X F task_queue_pop(queue: i64) -> Task

F create_event_loop() -> EventLoop {
    platform := detect_platform()
    epoll_val := platform == PLATFORM_LINUX ? epoll_create(1024) : 0
    kqueue_val := platform == PLATFORM_MACOS ? kqueue_create() : 0
    iocp_val := platform == PLATFORM_WINDOWS ? iocp_create() : 0
    queue_val := task_queue_create()

    loop := EventLoop {
        platform: platform,
        epoll_fd: epoll_val,
        kqueue_fd: kqueue_val,
        iocp_handle: iocp_val,
        task_queue: queue_val,
    }
    R loop
}

F register_event(loop: EventLoop, fd: i64, events: i64) -> i64 {
    result := event_loop_add(loop, fd, events)
    R result
}

F create_task(id: i64) -> Task {
    task := Task {
        id: id,
        state: TASK_STATE_PENDING,
        waker: 0,
        future: 0,
    }
    R task
}

F schedule_task(loop: EventLoop, task: Task) -> i64 {
    result := task_queue_push(loop.task_queue, task)
    R result
}

F run_task(task: Task) -> Task {
    updated := Task {
        id: task.id,
        state: TASK_STATE_COMPLETED,
        waker: task.waker,
        future: task.future,
    }
    R updated
}

F main() -> i64 {
    loop := create_event_loop()
    task := create_task(1)
    schedule_task(loop, task)
    result := register_event(loop, 42, EVENT_READ)
    R result
}
"#;

fn bench_async_reactor_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_async_reactor");

    // Benchmark basic async reactor
    let bytes = ASYNC_REACTOR_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("reactor_basic_lex", |b| {
        b.iter(|| tokenize(black_box(ASYNC_REACTOR_SOURCE)))
    });

    group.bench_function("reactor_basic_parse", |b| {
        b.iter(|| parse(black_box(ASYNC_REACTOR_SOURCE)))
    });

    group.bench_function("reactor_basic_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(ASYNC_REACTOR_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("reactor_bench");
            codegen.generate_module(&ast)
        })
    });

    // Benchmark larger reactor with task scheduling
    let large_bytes = ASYNC_REACTOR_LARGE_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(large_bytes));

    group.bench_function("reactor_large_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(ASYNC_REACTOR_LARGE_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("reactor_large_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 3. Logging Library Compilation Speed (Phase 33 Stage 4)
// ============================================================

const LOGGING_SOURCE: &str = r#"
# Structured logging library

C LOG_LEVEL_TRACE: i64 = 0
C LOG_LEVEL_DEBUG: i64 = 1
C LOG_LEVEL_INFO: i64 = 2
C LOG_LEVEL_WARN: i64 = 3
C LOG_LEVEL_ERROR: i64 = 4

S Logger {
    name: i64,
    level: i64,
    target: i64,
}

S LogEntry {
    level: i64,
    timestamp: i64,
    message: i64,
    file: i64,
    line: i64,
}

X F log_init(name: i64, level: i64) -> Logger
X F log_write(logger: Logger, entry: LogEntry) -> i64
X F log_flush(logger: Logger) -> i64
X F get_timestamp() -> i64

F create_logger(name: i64) -> Logger {
    logger := log_init(name, LOG_LEVEL_INFO)
    R logger
}

F log_info(logger: Logger, message: i64) -> i64 {
    entry := LogEntry {
        level: LOG_LEVEL_INFO,
        timestamp: get_timestamp(),
        message: message,
        file: 0,
        line: 0,
    }
    result := log_write(logger, entry)
    R result
}

F log_error(logger: Logger, message: i64) -> i64 {
    entry := LogEntry {
        level: LOG_LEVEL_ERROR,
        timestamp: get_timestamp(),
        message: message,
        file: 0,
        line: 0,
    }
    result := log_write(logger, entry)
    R result
}

F main() -> i64 {
    logger := create_logger(0)
    log_info(logger, 100)
    log_error(logger, 200)
    log_flush(logger)
    R 0
}
"#;

fn bench_logging_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_logging");

    let bytes = LOGGING_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("logging_lex", |b| {
        b.iter(|| tokenize(black_box(LOGGING_SOURCE)))
    });

    group.bench_function("logging_parse", |b| {
        b.iter(|| parse(black_box(LOGGING_SOURCE)))
    });

    group.bench_function("logging_typecheck", |b| {
        b.iter(|| {
            let ast = parse(black_box(LOGGING_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    group.bench_function("logging_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(LOGGING_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("logging_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 4. Compression Library Compilation Speed (Phase 33 Stage 5)
// ============================================================

const COMPRESSION_SOURCE: &str = r#"
# Compression library (gzip, zlib, brotli)

C COMPRESSION_GZIP: i64 = 1
C COMPRESSION_ZLIB: i64 = 2
C COMPRESSION_BROTLI: i64 = 3

C COMPRESSION_LEVEL_FAST: i64 = 1
C COMPRESSION_LEVEL_BALANCED: i64 = 6
C COMPRESSION_LEVEL_BEST: i64 = 9

S CompressionConfig {
    algorithm: i64,
    level: i64,
    buffer_size: i64,
}

S CompressionResult {
    data: i64,
    size: i64,
    checksum: i64,
}

X F compress_gzip(data: i64, size: i64, level: i64) -> CompressionResult
X F compress_zlib(data: i64, size: i64, level: i64) -> CompressionResult
X F compress_brotli(data: i64, size: i64, level: i64) -> CompressionResult
X F decompress_gzip(data: i64, size: i64) -> CompressionResult
X F decompress_zlib(data: i64, size: i64) -> CompressionResult
X F decompress_brotli(data: i64, size: i64) -> CompressionResult

F create_compression_config(algo: i64) -> CompressionConfig {
    config := CompressionConfig {
        algorithm: algo,
        level: COMPRESSION_LEVEL_BALANCED,
        buffer_size: 4096,
    }
    R config
}

F compress_data(data: i64, size: i64, config: CompressionConfig) -> CompressionResult {
    I config.algorithm == COMPRESSION_GZIP {
        result := compress_gzip(data, size, config.level)
        R result
    }
    I config.algorithm == COMPRESSION_ZLIB {
        result := compress_zlib(data, size, config.level)
        R result
    }
    I config.algorithm == COMPRESSION_BROTLI {
        result := compress_brotli(data, size, config.level)
        R result
    }
    empty := CompressionResult { data: 0, size: 0, checksum: 0 }
    R empty
}

F decompress_data(data: i64, size: i64, config: CompressionConfig) -> CompressionResult {
    I config.algorithm == COMPRESSION_GZIP {
        result := decompress_gzip(data, size)
        R result
    }
    I config.algorithm == COMPRESSION_ZLIB {
        result := decompress_zlib(data, size)
        R result
    }
    I config.algorithm == COMPRESSION_BROTLI {
        result := decompress_brotli(data, size)
        R result
    }
    empty := CompressionResult { data: 0, size: 0, checksum: 0 }
    R empty
}

F main() -> i64 {
    config := create_compression_config(COMPRESSION_GZIP)
    compressed := compress_data(1000, 512, config)
    decompressed := decompress_data(compressed.data, compressed.size, config)
    R decompressed.size
}
"#;

fn bench_compression_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_compression");

    let bytes = COMPRESSION_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("compression_lex", |b| {
        b.iter(|| tokenize(black_box(COMPRESSION_SOURCE)))
    });

    group.bench_function("compression_parse", |b| {
        b.iter(|| parse(black_box(COMPRESSION_SOURCE)))
    });

    group.bench_function("compression_typecheck", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMPRESSION_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    group.bench_function("compression_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMPRESSION_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("compression_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 5. Combined Phase 33 Features Compilation Speed
// ============================================================

const COMBINED_PHASE33_SOURCE: &str = r#"
# Combined Phase 33 features: TLS + Async + Logging + Compression

# TLS constants
C TLS_VERSION_1_3: i64 = 772

# Async constants
C PLATFORM_LINUX: i64 = 1
C EVENT_READ: i64 = 1

# Logging constants
C LOG_LEVEL_INFO: i64 = 2
C LOG_LEVEL_ERROR: i64 = 4

# Compression constants
C COMPRESSION_GZIP: i64 = 1
C COMPRESSION_LEVEL_BALANCED: i64 = 6

# Structs
S TlsConfig {
    version: i64,
    verify_peer: i64,
}

S EventLoop {
    platform: i64,
    epoll_fd: i64,
}

S Logger {
    name: i64,
    level: i64,
}

S CompressionConfig {
    algorithm: i64,
    level: i64,
}

S HttpsServer {
    tls: TlsConfig,
    event_loop: EventLoop,
    logger: Logger,
    compression: CompressionConfig,
}

# Extern functions
X F tls_init(config: TlsConfig) -> i64
X F epoll_create(size: i64) -> i64
X F log_init(name: i64, level: i64) -> Logger
X F compress_gzip(data: i64, size: i64, level: i64) -> i64

F create_tls_config() -> TlsConfig {
    config := TlsConfig {
        version: TLS_VERSION_1_3,
        verify_peer: 1,
    }
    R config
}

F create_event_loop() -> EventLoop {
    epoll_val := epoll_create(1024)
    loop := EventLoop {
        platform: PLATFORM_LINUX,
        epoll_fd: epoll_val,
    }
    R loop
}

F create_logger() -> Logger {
    logger := log_init(0, LOG_LEVEL_INFO)
    R logger
}

F create_compression_config() -> CompressionConfig {
    config := CompressionConfig {
        algorithm: COMPRESSION_GZIP,
        level: COMPRESSION_LEVEL_BALANCED,
    }
    R config
}

F create_https_server() -> HttpsServer {
    tls := create_tls_config()
    loop := create_event_loop()
    logger := create_logger()
    compression := create_compression_config()

    server := HttpsServer {
        tls: tls,
        event_loop: loop,
        logger: logger,
        compression: compression,
    }
    R server
}

F init_server(server: HttpsServer) -> i64 {
    tls_result := tls_init(server.tls)
    R tls_result
}

F main() -> i64 {
    server := create_https_server()
    result := init_server(server)
    R result
}
"#;

fn bench_combined_phase33_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_combined");

    let bytes = COMBINED_PHASE33_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("combined_lex", |b| {
        b.iter(|| tokenize(black_box(COMBINED_PHASE33_SOURCE)))
    });

    group.bench_function("combined_parse", |b| {
        b.iter(|| parse(black_box(COMBINED_PHASE33_SOURCE)))
    });

    group.bench_function("combined_typecheck", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMBINED_PHASE33_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    group.bench_function("combined_codegen", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMBINED_PHASE33_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("combined_bench");
            codegen.generate_module(&ast)
        })
    });

    group.bench_function("combined_full_pipeline", |b| {
        b.iter(|| {
            let _tokens = tokenize(black_box(COMBINED_PHASE33_SOURCE)).expect("lex");
            let ast = parse(black_box(COMBINED_PHASE33_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("combined_full_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// Comparison: Phase 33 vs Basic Code
// ============================================================

const BASIC_SOURCE: &str = r#"
F add(x: i64, y: i64) -> i64 = x + y
F mul(x: i64, y: i64) -> i64 = x * y
F main() -> i64 = add(mul(2, 3), 4)
"#;

fn bench_phase33_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase33_comparison");

    // Basic source (baseline)
    group.bench_function("basic_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(BASIC_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("basic_bench");
            codegen.generate_module(&ast)
        })
    });

    // TLS source
    group.bench_function("tls_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(TLS_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("tls_bench");
            codegen.generate_module(&ast)
        })
    });

    // Async reactor source
    group.bench_function("reactor_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(ASYNC_REACTOR_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("reactor_bench");
            codegen.generate_module(&ast)
        })
    });

    // Combined source
    group.bench_function("combined_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMBINED_PHASE33_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("combined_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// Main
// ============================================================

criterion_group!(
    benches,
    bench_tls_compilation,
    bench_async_reactor_compilation,
    bench_logging_compilation,
    bench_compression_compilation,
    bench_combined_phase33_compilation,
    bench_phase33_comparison,
);

criterion_main!(benches);
