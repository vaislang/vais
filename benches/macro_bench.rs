//! Phase 34 Stage 8: Macro-level Benchmarks
//!
//! Measures compilation speed for real-world, application-level Vais code:
//! 1. CLI Tool - 100+ line command-line application
//! 2. HTTP Server - 150+ line network server
//! 3. Data Pipeline - 100+ line CSV processing pipeline
//! 4. Combined Application - 300+ line full application
//! 5. Scaling Test - Measures how compilation scales with source size

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

// ============================================================
// 1. CLI Tool Application (100+ lines)
// ============================================================

const CLI_TOOL_SOURCE: &str = r#"
# CLI Tool - Command-line argument parser and processor

C CLI_MAX_ARGS: i64 = 32
C CLI_ARG_SIZE: i64 = 256
C EXIT_SUCCESS: i64 = 0
C EXIT_FAILURE: i64 = 1

S CliArgs {
    count: i64,
    program_name: i64,
    verbose: i64,
    help_flag: i64,
}

S CliCommand {
    name: i64,
    description: i64,
    handler: i64,
}

X F strcmp(a: str, b: str) -> i64
X F strlen(s: str) -> i64
X F malloc(size: i64) -> i64
X F free_mem(ptr: i64) -> i64
X F printf(format: str, value: i64) -> i64
X F puts(text: str) -> i64
X F exit_program(code: i64) -> i64

F cli_args_new() -> CliArgs {
    args := CliArgs {
        count: 0,
        program_name: 0,
        verbose: 0,
        help_flag: 0,
    }
    R args
}

F cli_args_parse(argc: i64, argv: str) -> CliArgs {
    args := cli_args_new()

    I argc > 0 {
        args = CliArgs {
            count: argc,
            program_name: 1,
            verbose: 0,
            help_flag: 0,
        }
    }

    R args
}

F cli_args_has_flag(args: CliArgs, flag: str) -> i64 {
    I strcmp(flag, "--verbose") == 0 {
        R args.verbose
    }

    I strcmp(flag, "--help") == 0 {
        R args.help_flag
    }

    R 0
}

F cli_print_help() -> i64 {
    puts("Usage: cli-tool [OPTIONS] <COMMAND>")
    puts("")
    puts("Options:")
    puts("  --help       Show this help message")
    puts("  --verbose    Enable verbose output")
    puts("")
    puts("Commands:")
    puts("  process      Process input files")
    puts("  analyze      Analyze data")
    puts("  report       Generate report")
    puts("")
    R 0
}

F cli_command_process(args: CliArgs) -> i64 {
    I args.verbose == 1 {
        puts("Processing with verbose output...")
    } E {
        puts("Processing...")
    }

    puts("Process complete!")
    R EXIT_SUCCESS
}

F cli_command_analyze(args: CliArgs) -> i64 {
    puts("Analyzing data...")

    total := 0
    i := 0
    L {
        I i >= 10 {
            R EXIT_SUCCESS
        }

        total = total + i
        i = i + 1

        I i >= 10 {
            R EXIT_SUCCESS
        }
    }

    printf("Analysis complete. Total: %d\n", total)
    R EXIT_SUCCESS
}

F cli_command_report(args: CliArgs) -> i64 {
    puts("Generating report...")

    items := 5
    processed := 0

    L {
        I processed >= items {
            R EXIT_SUCCESS
        }

        I args.verbose == 1 {
            printf("Processing item %d\n", processed)
        }

        processed = processed + 1

        I processed >= items {
            R EXIT_SUCCESS
        }
    }

    puts("Report generated!")
    R EXIT_SUCCESS
}

F cli_execute_command(cmd: str, args: CliArgs) -> i64 {
    I strcmp(cmd, "process") == 0 {
        R cli_command_process(args)
    }

    I strcmp(cmd, "analyze") == 0 {
        R cli_command_analyze(args)
    }

    I strcmp(cmd, "report") == 0 {
        R cli_command_report(args)
    }

    puts("Error: Unknown command")
    R EXIT_FAILURE
}

F main(argc: i64, argv: str) -> i64 {
    args := cli_args_parse(argc, argv)

    I cli_args_has_flag(args, "--help") == 1 {
        R cli_print_help()
    }

    I argc < 2 {
        cli_print_help()
        R EXIT_FAILURE
    }

    command := "process"
    result := cli_execute_command(command, args)

    R result
}
"#;

fn bench_cli_tool(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_cli_tool");

    let bytes = CLI_TOOL_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("bench_cli_lex", |b| {
        b.iter(|| tokenize(black_box(CLI_TOOL_SOURCE)))
    });

    group.bench_function("bench_cli_parse", |b| {
        b.iter(|| parse(black_box(CLI_TOOL_SOURCE)))
    });

    group.bench_function("bench_cli_typecheck", |b| {
        b.iter(|| {
            let ast = parse(black_box(CLI_TOOL_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast)
        })
    });

    group.bench_function("bench_cli_codegen", |b| {
        b.iter(|| {
            let ast = parse(black_box(CLI_TOOL_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("cli_bench");
            codegen.generate_module(&ast)
        })
    });

    group.bench_function("bench_cli_full", |b| {
        b.iter(|| {
            let _tokens = tokenize(black_box(CLI_TOOL_SOURCE)).expect("lex");
            let ast = parse(black_box(CLI_TOOL_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("cli_full_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 2. HTTP Server Application (150+ lines)
// ============================================================

const HTTP_SERVER_SOURCE: &str = r#"
# HTTP Server - Simple HTTP/1.1 server implementation

C HTTP_PORT: i64 = 8080
C HTTP_MAX_CONNECTIONS: i64 = 128
C HTTP_BUFFER_SIZE: i64 = 4096

C HTTP_METHOD_GET: i64 = 1
C HTTP_METHOD_POST: i64 = 2
C HTTP_METHOD_PUT: i64 = 3
C HTTP_METHOD_DELETE: i64 = 4

C HTTP_STATUS_OK: i64 = 200
C HTTP_STATUS_NOT_FOUND: i64 = 404
C HTTP_STATUS_SERVER_ERROR: i64 = 500

S HttpRequest {
    method: i64,
    path: i64,
    headers: i64,
    body: i64,
    body_length: i64,
}

S HttpResponse {
    status_code: i64,
    headers: i64,
    body: i64,
    body_length: i64,
}

S HttpServer {
    socket_fd: i64,
    port: i64,
    running: i64,
    connection_count: i64,
}

X F socket_create(domain: i64, type_val: i64, protocol: i64) -> i64
X F socket_bind(sockfd: i64, port: i64) -> i64
X F socket_listen(sockfd: i64, backlog: i64) -> i64
X F socket_accept(sockfd: i64) -> i64
X F socket_recv(sockfd: i64, buffer: i64, size: i64) -> i64
X F socket_send(sockfd: i64, data: i64, size: i64) -> i64
X F socket_close(sockfd: i64) -> i64
X F malloc(size: i64) -> i64
X F free_mem(ptr: i64) -> i64
X F strcmp(a: str, b: str) -> i64
X F strlen(s: str) -> i64
X F sprintf(buffer: i64, format: str, value: i64) -> i64
X F printf(format: str, value: i64) -> i64
X F puts(text: str) -> i64

F http_request_new() -> HttpRequest {
    req := HttpRequest {
        method: 0,
        path: 0,
        headers: 0,
        body: 0,
        body_length: 0,
    }
    R req
}

F http_request_free(req: HttpRequest) -> i64 {
    I req.headers != 0 {
        free_mem(req.headers)
    }
    I req.body != 0 {
        free_mem(req.body)
    }
    R 0
}

F http_request_parse(buffer: i64, size: i64) -> HttpRequest {
    req := http_request_new()

    req = HttpRequest {
        method: HTTP_METHOD_GET,
        path: buffer,
        headers: 0,
        body: 0,
        body_length: 0,
    }

    R req
}

F http_response_new(status: i64) -> HttpResponse {
    resp := HttpResponse {
        status_code: status,
        headers: 0,
        body: 0,
        body_length: 0,
    }
    R resp
}

F http_response_free(resp: HttpResponse) -> i64 {
    I resp.headers != 0 {
        free_mem(resp.headers)
    }
    I resp.body != 0 {
        free_mem(resp.body)
    }
    R 0
}

F http_response_set_body(resp: HttpResponse, body: str) -> HttpResponse {
    body_len := 30

    updated := HttpResponse {
        status_code: resp.status_code,
        headers: resp.headers,
        body: 1,
        body_length: body_len,
    }

    R updated
}

F http_server_new(port: i64) -> HttpServer {
    sockfd := socket_create(2, 1, 0)

    server := HttpServer {
        socket_fd: sockfd,
        port: port,
        running: 0,
        connection_count: 0,
    }

    R server
}

F http_server_start(server: HttpServer) -> HttpServer {
    socket_bind(server.socket_fd, server.port)
    socket_listen(server.socket_fd, HTTP_MAX_CONNECTIONS)

    printf("HTTP server listening on port %d\n", server.port)

    updated := HttpServer {
        socket_fd: server.socket_fd,
        port: server.port,
        running: 1,
        connection_count: 0,
    }

    R updated
}

F http_server_handle_connection(server: HttpServer, client_fd: i64) -> i64 {
    buffer := malloc(HTTP_BUFFER_SIZE)

    bytes_received := socket_recv(client_fd, buffer, HTTP_BUFFER_SIZE)

    I bytes_received > 0 {
        req := http_request_parse(buffer, bytes_received)
        resp := http_handle_request(req)
        http_send_response(client_fd, resp)
        http_request_free(req)
        http_response_free(resp)
    }

    free_mem(buffer)
    socket_close(client_fd)

    R 0
}

F http_handle_request(req: HttpRequest) -> HttpResponse {
    I req.method == HTTP_METHOD_GET {
        R http_handle_get(req)
    }

    I req.method == HTTP_METHOD_POST {
        R http_handle_post(req)
    }

    resp := http_response_new(HTTP_STATUS_NOT_FOUND)
    R resp
}

F http_handle_get(req: HttpRequest) -> HttpResponse {
    resp := http_response_new(HTTP_STATUS_OK)
    resp = http_response_set_body(resp, "Hello from Vais HTTP Server!")
    R resp
}

F http_handle_post(req: HttpRequest) -> HttpResponse {
    resp := http_response_new(HTTP_STATUS_OK)
    resp = http_response_set_body(resp, "POST received")
    R resp
}

F http_send_response(client_fd: i64, resp: HttpResponse) -> i64 {
    response_buffer := malloc(HTTP_BUFFER_SIZE)

    response_len := 0
    I resp.status_code == HTTP_STATUS_OK {
        sprintf(response_buffer, "HTTP/1.1 200 OK\r\n\r\n", 0)
        response_len = 25
    } E {
        sprintf(response_buffer, "HTTP/1.1 404 Not Found\r\n\r\n", 0)
        response_len = 30
    }

    socket_send(client_fd, response_buffer, response_len)

    I resp.body != 0 {
        socket_send(client_fd, resp.body, resp.body_length)
    }

    free_mem(response_buffer)
    R 0
}

F http_server_run(server: HttpServer) -> i64 {
    max_connections := 10
    count := 0

    L {
        I count >= max_connections {
            R 0
        }

        client_fd := socket_accept(server.socket_fd)

        I client_fd > 0 {
            http_server_handle_connection(server, client_fd)
        }

        count = count + 1

        I count >= max_connections {
            R 0
        }
    }

    R 0
}

F main() -> i64 {
    server := http_server_new(HTTP_PORT)
    server = http_server_start(server)

    I server.running == 1 {
        http_server_run(server)
    }

    socket_close(server.socket_fd)
    R 0
}
"#;

fn bench_http_server(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_http_server");

    let bytes = HTTP_SERVER_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("bench_http_lex", |b| {
        b.iter(|| tokenize(black_box(HTTP_SERVER_SOURCE)))
    });

    group.bench_function("bench_http_parse", |b| {
        b.iter(|| parse(black_box(HTTP_SERVER_SOURCE)))
    });

    group.bench_function("bench_http_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(HTTP_SERVER_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("http_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 3. Data Pipeline Application (100+ lines)
// ============================================================

const DATA_PIPELINE_SOURCE: &str = r#"
# Data Pipeline - CSV processing with transformation and aggregation

C CSV_MAX_LINE_SIZE: i64 = 1024
C CSV_MAX_FIELDS: i64 = 32

S CsvReader {
    file_handle: i64,
    line_buffer: i64,
    current_line: i64,
}

S Transformer {
    filter_enabled: i64,
    filter_threshold: i64,
    aggregate_enabled: i64,
}

S AggregateStats {
    total_count: i64,
    sum: i64,
    min_value: i64,
    max_value: i64,
}

S Pipeline {
    reader: CsvReader,
    transformer: Transformer,
    stats: AggregateStats,
}

X F fopen(path: str, mode: str) -> i64
X F fclose(file: i64) -> i64
X F fgets(buffer: i64, size: i64, file: i64) -> i64
X F malloc(size: i64) -> i64
X F free_mem(ptr: i64) -> i64
X F atoi(s: str) -> i64
X F printf(format: str, value: i64) -> i64
X F puts(text: str) -> i64

F csv_reader_new(path: str) -> CsvReader {
    handle := fopen(path, "r")
    buffer := malloc(CSV_MAX_LINE_SIZE)

    reader := CsvReader {
        file_handle: handle,
        line_buffer: buffer,
        current_line: 0,
    }

    R reader
}

F csv_reader_free(reader: CsvReader) -> i64 {
    I reader.file_handle != 0 {
        fclose(reader.file_handle)
    }
    I reader.line_buffer != 0 {
        free_mem(reader.line_buffer)
    }
    R 0
}

F csv_reader_read_line(reader: CsvReader) -> i64 {
    I reader.file_handle == 0 {
        R 0
    }

    result := fgets(reader.line_buffer, CSV_MAX_LINE_SIZE, reader.file_handle)
    R result
}

F transformer_new() -> Transformer {
    trans := Transformer {
        filter_enabled: 0,
        filter_threshold: 0,
        aggregate_enabled: 0,
    }
    R trans
}

F transformer_set_filter(trans: Transformer, threshold: i64) -> Transformer {
    updated := Transformer {
        filter_enabled: 1,
        filter_threshold: threshold,
        aggregate_enabled: trans.aggregate_enabled,
    }
    R updated
}

F transformer_should_keep(trans: Transformer, value: i64) -> i64 {
    I trans.filter_enabled == 0 {
        R 1
    }

    I value > trans.filter_threshold {
        R 1
    }

    R 0
}

F aggregate_stats_new() -> AggregateStats {
    stats := AggregateStats {
        total_count: 0,
        sum: 0,
        min_value: 999999,
        max_value: 0,
    }
    R stats
}

F aggregate_stats_update(stats: AggregateStats, value: i64) -> AggregateStats {
    new_count := stats.total_count + 1
    new_sum := stats.sum + value

    new_min := stats.min_value
    I value < stats.min_value {
        new_min = value
    }

    new_max := stats.max_value
    I value > stats.max_value {
        new_max = value
    }

    updated := AggregateStats {
        total_count: new_count,
        sum: new_sum,
        min_value: new_min,
        max_value: new_max,
    }

    R updated
}

F aggregate_stats_average(stats: AggregateStats) -> i64 {
    I stats.total_count == 0 {
        R 0
    }

    avg := stats.sum / stats.total_count
    R avg
}

F pipeline_new(input_path: str) -> Pipeline {
    reader := csv_reader_new(input_path)
    trans := transformer_new()
    trans = transformer_set_filter(trans, 30)
    stats := aggregate_stats_new()

    pipeline := Pipeline {
        reader: reader,
        transformer: trans,
        stats: stats,
    }

    R pipeline
}

F pipeline_free(pipeline: Pipeline) -> i64 {
    csv_reader_free(pipeline.reader)
    R 0
}

F pipeline_execute(pipeline: Pipeline) -> AggregateStats {
    current_stats := pipeline.stats

    max_rows := 100
    i := 0

    L {
        I i >= max_rows {
            R current_stats
        }

        line_result := csv_reader_read_line(pipeline.reader)

        I line_result == 0 {
            R current_stats
        }

        value := 42
        should_keep := transformer_should_keep(pipeline.transformer, value)

        I should_keep == 1 {
            current_stats = aggregate_stats_update(current_stats, value)
        }

        i = i + 1

        I i >= max_rows {
            R current_stats
        }
    }

    R current_stats
}

F pipeline_print_stats(stats: AggregateStats) -> i64 {
    puts("=== Pipeline Statistics ===")
    printf("Total count: %d\n", stats.total_count)
    printf("Sum: %d\n", stats.sum)
    printf("Average: %d\n", aggregate_stats_average(stats))
    printf("Min: %d\n", stats.min_value)
    printf("Max: %d\n", stats.max_value)
    R 0
}

F main() -> i64 {
    pipeline := pipeline_new("data.csv")

    puts("Starting data pipeline...")
    stats := pipeline_execute(pipeline)

    pipeline_print_stats(stats)
    pipeline_free(pipeline)

    R 0
}
"#;

fn bench_data_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_data_pipeline");

    let bytes = DATA_PIPELINE_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("bench_data_lex", |b| {
        b.iter(|| tokenize(black_box(DATA_PIPELINE_SOURCE)))
    });

    group.bench_function("bench_data_parse", |b| {
        b.iter(|| parse(black_box(DATA_PIPELINE_SOURCE)))
    });

    group.bench_function("bench_data_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(DATA_PIPELINE_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("data_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 4. Combined Application (300+ lines)
// ============================================================

const COMBINED_SOURCE: &str = r#"
# Combined Application - CLI + HTTP Server + Data Pipeline

C EXIT_SUCCESS: i64 = 0
C EXIT_FAILURE: i64 = 1
C HTTP_PORT: i64 = 8080
C CSV_MAX_LINE_SIZE: i64 = 1024

S CliArgs {
    count: i64,
    verbose: i64,
}

S HttpServer {
    socket_fd: i64,
    port: i64,
    running: i64,
}

S CsvReader {
    file_handle: i64,
    line_buffer: i64,
}

S AggregateStats {
    total_count: i64,
    sum: i64,
    min_value: i64,
    max_value: i64,
}

S Application {
    args: CliArgs,
    server: HttpServer,
    reader: CsvReader,
    stats: AggregateStats,
}

X F strcmp(a: str, b: str) -> i64
X F printf(format: str, value: i64) -> i64
X F puts(text: str) -> i64
X F malloc(size: i64) -> i64
X F free_mem(ptr: i64) -> i64
X F fopen(path: str, mode: str) -> i64
X F fclose(file: i64) -> i64
X F fgets(buffer: i64, size: i64, file: i64) -> i64
X F socket_create(domain: i64, type_val: i64, protocol: i64) -> i64
X F socket_bind(sockfd: i64, port: i64) -> i64
X F socket_listen(sockfd: i64, backlog: i64) -> i64
X F socket_close(sockfd: i64) -> i64

F cli_args_new() -> CliArgs {
    args := CliArgs {
        count: 0,
        verbose: 0,
    }
    R args
}

F cli_args_parse(argc: i64) -> CliArgs {
    args := cli_args_new()

    I argc > 1 {
        args = CliArgs {
            count: argc,
            verbose: 1,
        }
    }

    R args
}

F cli_print_help() -> i64 {
    puts("Usage: app [OPTIONS] <COMMAND>")
    puts("  --help       Show help")
    puts("  --verbose    Verbose output")
    puts("")
    puts("Commands:")
    puts("  server       Start HTTP server")
    puts("  process      Process CSV data")
    R 0
}

F http_server_new(port: i64) -> HttpServer {
    sockfd := socket_create(2, 1, 0)

    server := HttpServer {
        socket_fd: sockfd,
        port: port,
        running: 0,
    }

    R server
}

F http_server_start(server: HttpServer) -> HttpServer {
    socket_bind(server.socket_fd, server.port)
    socket_listen(server.socket_fd, 128)

    printf("Server listening on port %d\n", server.port)

    updated := HttpServer {
        socket_fd: server.socket_fd,
        port: server.port,
        running: 1,
    }

    R updated
}

F http_server_stop(server: HttpServer) -> i64 {
    I server.socket_fd != 0 {
        socket_close(server.socket_fd)
    }
    R 0
}

F csv_reader_new(path: str) -> CsvReader {
    handle := fopen(path, "r")
    buffer := malloc(CSV_MAX_LINE_SIZE)

    reader := CsvReader {
        file_handle: handle,
        line_buffer: buffer,
    }

    R reader
}

F csv_reader_free(reader: CsvReader) -> i64 {
    I reader.file_handle != 0 {
        fclose(reader.file_handle)
    }
    I reader.line_buffer != 0 {
        free_mem(reader.line_buffer)
    }
    R 0
}

F csv_reader_read_line(reader: CsvReader) -> i64 {
    I reader.file_handle == 0 {
        R 0
    }

    result := fgets(reader.line_buffer, CSV_MAX_LINE_SIZE, reader.file_handle)
    R result
}

F aggregate_stats_new() -> AggregateStats {
    stats := AggregateStats {
        total_count: 0,
        sum: 0,
        min_value: 999999,
        max_value: 0,
    }
    R stats
}

F aggregate_stats_update(stats: AggregateStats, value: i64) -> AggregateStats {
    new_count := stats.total_count + 1
    new_sum := stats.sum + value

    new_min := stats.min_value
    I value < stats.min_value {
        new_min = value
    }

    new_max := stats.max_value
    I value > stats.max_value {
        new_max = value
    }

    updated := AggregateStats {
        total_count: new_count,
        sum: new_sum,
        min_value: new_min,
        max_value: new_max,
    }

    R updated
}

F aggregate_stats_print(stats: AggregateStats) -> i64 {
    puts("=== Statistics ===")
    printf("Count: %d\n", stats.total_count)
    printf("Sum: %d\n", stats.sum)
    printf("Min: %d\n", stats.min_value)
    printf("Max: %d\n", stats.max_value)
    R 0
}

F app_new(argc: i64) -> Application {
    args := cli_args_parse(argc)
    server := http_server_new(HTTP_PORT)
    reader := csv_reader_new("data.csv")
    stats := aggregate_stats_new()

    app := Application {
        args: args,
        server: server,
        reader: reader,
        stats: stats,
    }

    R app
}

F app_free(app: Application) -> i64 {
    http_server_stop(app.server)
    csv_reader_free(app.reader)
    R 0
}

F app_run_server(app: Application) -> i64 {
    server := http_server_start(app.server)

    I app.args.verbose == 1 {
        puts("Server started in verbose mode")
    }

    R 0
}

F app_process_data(app: Application) -> AggregateStats {
    current_stats := app.stats

    max_rows := 50
    i := 0

    L {
        I i >= max_rows {
            R current_stats
        }

        line_result := csv_reader_read_line(app.reader)

        I line_result == 0 {
            R current_stats
        }

        value := i * 10
        current_stats = aggregate_stats_update(current_stats, value)

        I app.args.verbose == 1 {
            printf("Processed line %d\n", i)
        }

        i = i + 1

        I i >= max_rows {
            R current_stats
        }
    }

    R current_stats
}

F app_execute_command(app: Application, command: str) -> i64 {
    I strcmp(command, "server") == 0 {
        R app_run_server(app)
    }

    I strcmp(command, "process") == 0 {
        stats := app_process_data(app)
        aggregate_stats_print(stats)
        R EXIT_SUCCESS
    }

    puts("Unknown command")
    R EXIT_FAILURE
}

F main(argc: i64, argv: str) -> i64 {
    I argc < 2 {
        cli_print_help()
        R EXIT_FAILURE
    }

    app := app_new(argc)

    command := "process"
    result := app_execute_command(app, command)

    app_free(app)

    R result
}
"#;

fn bench_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_combined");

    let bytes = COMBINED_SOURCE.len() as u64;
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("bench_combined_full", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMBINED_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("combined_bench");
            codegen.generate_module(&ast)
        })
    });

    group.bench_function("bench_combined_codegen", |b| {
        b.iter(|| {
            let ast = parse(black_box(COMBINED_SOURCE)).expect("parse");
            let mut checker = TypeChecker::new();
            checker.check_module(&ast).expect("typecheck");
            let mut codegen = CodeGenerator::new("combined_codegen_bench");
            codegen.generate_module(&ast)
        })
    });

    group.finish();
}

// ============================================================
// 5. Scaling Test - Source Size Impact
// ============================================================

fn generate_scaling_source(lines: usize) -> String {
    let mut source = String::new();
    source.push_str("# Scaling test source\n\n");
    source.push_str("X F printf(format: str, value: i64) -> i64\n");
    source.push_str("X F puts(text: str) -> i64\n\n");

    // Generate struct definitions
    for i in 0..lines/10 {
        source.push_str(&format!(
            "S Struct{} {{\n    field1: i64,\n    field2: i64,\n}}\n\n",
            i
        ));
    }

    // Generate function definitions
    for i in 0..lines/10 {
        source.push_str(&format!(
            "F func{}(x: i64) -> i64 {{\n    y := x + {}\n    R y\n}}\n\n",
            i, i
        ));
    }

    // Generate main function
    source.push_str("F main() -> i64 {\n");
    source.push_str("    total := 0\n");

    for i in 0..lines/20 {
        source.push_str(&format!("    total = total + func{}({})\n", i % (lines/10), i));
    }

    source.push_str("    printf(\"Total: %d\\n\", total)\n");
    source.push_str("    R total\n");
    source.push_str("}\n");

    source
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("macro_scaling");

    let sizes = vec![50, 100, 200, 400];

    for size in sizes {
        let source = generate_scaling_source(size);
        let bytes = source.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(BenchmarkId::new("lines", size), &source, |b, src| {
            b.iter(|| {
                let ast = parse(black_box(src)).expect("parse");
                let mut checker = TypeChecker::new();
                checker.check_module(&ast).expect("typecheck");
                let mut codegen = CodeGenerator::new("scaling_bench");
                codegen.generate_module(&ast)
            })
        });
    }

    group.finish();
}

// ============================================================
// Main
// ============================================================

criterion_group!(
    benches,
    bench_cli_tool,
    bench_http_server,
    bench_data_pipeline,
    bench_combined,
    bench_scaling,
);

criterion_main!(benches);
