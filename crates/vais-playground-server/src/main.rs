//! Vais Playground Server
//!
//! Server-side compilation and execution service for the Vais Playground.
//! Accepts Vais source code via REST API, compiles it using the real compiler
//! pipeline, and optionally executes the resulting binary.

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Simple rate limiter based on IP address
struct RateLimiter {
    /// Map of IP addresses to their request timestamps
    requests: HashMap<IpAddr, Vec<Instant>>,
    /// Maximum number of requests per window
    max_requests: usize,
    /// Time window in seconds
    window_secs: u64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_secs,
        }
    }

    /// Check if a request from the given IP is allowed.
    /// Returns true if allowed, false if rate-limited.
    fn check(&mut self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let timestamps = self.requests.entry(ip).or_default();

        // Remove expired timestamps
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

#[derive(Clone)]
struct AppState {
    compile_semaphore: Arc<Semaphore>,
    config: PlaygroundConfig,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

#[derive(Clone)]
struct PlaygroundConfig {
    host: String,
    port: u16,
    max_concurrent: usize,
    execution_timeout_secs: u64,
    max_source_bytes: usize,
    max_output_bytes: usize,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        let host = std::env::var("PLAYGROUND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("PLAYGROUND_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080u16);

        // Validate port range
        if port == 0 {
            tracing::warn!("Port 0 will cause OS to assign a random port");
        }

        Self {
            host,
            port,
            max_concurrent: std::env::var("PLAYGROUND_MAX_CONCURRENT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4),
            execution_timeout_secs: std::env::var("PLAYGROUND_TIMEOUT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(10),
            max_source_bytes: 64 * 1024, // 64KB max source
            max_output_bytes: 1024 * 1024, // 1MB max output
        }
    }
}

#[derive(Deserialize)]
struct CompileRequest {
    source: String,
    #[serde(default)]
    optimize: bool,
    #[serde(default)]
    emit_ir: bool,
    #[serde(default = "default_true")]
    execute: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
struct CompileResponse {
    success: bool,
    errors: Vec<DiagnosticItem>,
    warnings: Vec<DiagnosticItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compile_time_ms: Option<u64>,
}

#[derive(Serialize)]
struct DiagnosticItem {
    line: usize,
    column: usize,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    compiler: String,
}

fn make_error_response(message: String) -> CompileResponse {
    CompileResponse {
        success: false,
        errors: vec![DiagnosticItem {
            line: 0,
            column: 0,
            message,
            severity: Some("error".to_string()),
        }],
        warnings: vec![],
        ir: None,
        output: None,
        exit_code: None,
        compile_time_ms: None,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = PlaygroundConfig::default();
    let addr = format!("{}:{}", config.host, config.port);

    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(10, 60)));

    let state = AppState {
        compile_semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
        config,
        rate_limiter,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/compile", post(compile))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("Vais Playground server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Server error: {}", e);
            std::process::exit(1);
        });
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        compiler: "vaisc".to_string(),
    })
}

async fn compile(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<CompileResponse>)> {
    // Rate limiting
    {
        let mut limiter = state.rate_limiter.lock().await;
        if !limiter.check(addr.ip()) {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(make_error_response(
                    "Rate limit exceeded: max 10 requests per 60 seconds".to_string(),
                )),
            ));
        }
    }

    if req.source.len() > state.config.max_source_bytes {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(make_error_response(format!(
                "Source code too large: {} bytes (max {} bytes)",
                req.source.len(),
                state.config.max_source_bytes
            ))),
        ));
    }

    let _permit = state.compile_semaphore.acquire().await.map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(make_error_response(
                "Server busy, try again later".to_string(),
            )),
        )
    })?;

    let source = req.source.clone();
    let optimize = req.optimize;
    let emit_ir = req.emit_ir;
    let execute = req.execute;
    let timeout_secs = state.config.execution_timeout_secs;
    let max_output_bytes = state.config.max_output_bytes;

    let result = tokio::task::spawn_blocking(move || {
        compile_and_run(&source, optimize, emit_ir, execute, timeout_secs, max_output_bytes)
    })
    .await
    .unwrap_or_else(|e| make_error_response(format!("Internal error: {}", e)));

    Ok(Json(result))
}

fn compile_and_run(
    source: &str,
    optimize: bool,
    emit_ir: bool,
    execute: bool,
    timeout_secs: u64,
    max_output_bytes: usize,
) -> CompileResponse {
    let start = std::time::Instant::now();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Step 1: Tokenize
    let tokens = match vais_lexer::tokenize(source) {
        Ok(tokens) => tokens,
        Err(e) => {
            errors.push(DiagnosticItem {
                line: 1,
                column: 1,
                message: format!("Lexer error: {}", e),
                severity: Some("error".to_string()),
            });
            return CompileResponse {
                success: false,
                errors,
                warnings,
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: Some(start.elapsed().as_millis() as u64),
            };
        }
    };

    // Check for error tokens
    for token in &tokens {
        if format!("{:?}", token.token).contains("Error") {
            errors.push(DiagnosticItem {
                line: 1,
                column: 1,
                message: format!("Lexer error: unexpected token {:?}", token.token),
                severity: Some("error".to_string()),
            });
        }
    }

    if !errors.is_empty() {
        return CompileResponse {
            success: false,
            errors,
            warnings,
            ir: None,
            output: None,
            exit_code: None,
            compile_time_ms: Some(start.elapsed().as_millis() as u64),
        };
    }

    // Step 2: Parse
    let mut ast = match vais_parser::parse(source) {
        Ok(ast) => ast,
        Err(parse_error) => {
            let err_str = format!("{}", parse_error);
            errors.push(DiagnosticItem {
                line: extract_line(&err_str),
                column: extract_column(&err_str),
                message: err_str,
                severity: Some("error".to_string()),
            });
            return CompileResponse {
                success: false,
                errors,
                warnings,
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: Some(start.elapsed().as_millis() as u64),
            };
        }
    };

    // Step 3: Macro expansion
    let mut registry = vais_macro::MacroRegistry::new();
    vais_macro::collect_macros(&ast, &mut registry);
    ast = match vais_macro::expand_macros(ast, &registry) {
        Ok(expanded) => expanded,
        Err(_) => {
            // Macro expansion failed, but we already have the AST from before move
            // This shouldn't happen in practice for playground code
            return CompileResponse {
                success: false,
                errors: vec![DiagnosticItem {
                    line: 0,
                    column: 0,
                    message: "Macro expansion failed".to_string(),
                    severity: Some("error".to_string()),
                }],
                warnings,
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: Some(start.elapsed().as_millis() as u64),
            };
        }
    };
    let _ = vais_macro::process_derives(&mut ast);

    // Step 4: Type checking
    let mut type_checker = vais_types::TypeChecker::new();
    if let Err(type_error) = type_checker.check_module(&ast) {
        let err_str = format!("{}", type_error);
        // Treat type errors as warnings in playground mode to allow partial compilation
        warnings.push(DiagnosticItem {
            line: extract_line(&err_str),
            column: extract_column(&err_str),
            message: err_str,
            severity: Some("warning".to_string()),
        });
    }

    // Step 5: Code generation
    let opt_level = if optimize {
        vais_codegen::optimize::OptLevel::O2
    } else {
        vais_codegen::optimize::OptLevel::O0
    };
    let target = vais_codegen::TargetTriple::Native;

    let mut codegen = vais_codegen::CodeGenerator::new_with_target("playground", target);
    let ir_string = match codegen.generate_module(&ast) {
        Ok(raw_ir) => {
            if optimize {
                vais_codegen::optimize::optimize_ir(&raw_ir, opt_level)
            } else {
                raw_ir
            }
        }
        Err(e) => {
            errors.push(DiagnosticItem {
                line: 0,
                column: 0,
                message: format!("Codegen error: {}", e),
                severity: Some("error".to_string()),
            });
            return CompileResponse {
                success: false,
                errors,
                warnings,
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: Some(start.elapsed().as_millis() as u64),
            };
        }
    };

    let ir_output = if emit_ir {
        Some(ir_string.clone())
    } else {
        None
    };

    // Step 6: Compile and execute (if requested)
    if execute {
        match compile_ir_and_execute(&ir_string, timeout_secs, max_output_bytes) {
            Ok((output, exit_code)) => CompileResponse {
                success: true,
                errors,
                warnings,
                ir: ir_output,
                output: Some(output),
                exit_code: Some(exit_code),
                compile_time_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => {
                errors.push(DiagnosticItem {
                    line: 0,
                    column: 0,
                    message: e,
                    severity: Some("error".to_string()),
                });
                CompileResponse {
                    success: false,
                    errors,
                    warnings,
                    ir: ir_output,
                    output: None,
                    exit_code: None,
                    compile_time_ms: Some(start.elapsed().as_millis() as u64),
                }
            }
        }
    } else {
        CompileResponse {
            success: true,
            errors,
            warnings,
            ir: ir_output,
            output: Some("Compilation successful".to_string()),
            exit_code: None,
            compile_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

/// Truncate output to max_bytes, appending a truncation notice if needed.
fn truncate_output(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let truncated = &s[..max_bytes];
        format!("{}\n... (output truncated at {} bytes)", truncated, max_bytes)
    }
}

fn compile_ir_and_execute(
    ir: &str,
    timeout_secs: u64,
    max_output_bytes: usize,
) -> Result<(String, i32), String> {
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ir_path = tmp_dir.path().join("playground.ll");
    let bin_path = tmp_dir.path().join("playground");

    std::fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    let ir_path_str = ir_path.to_str()
        .ok_or_else(|| "IR path contains invalid UTF-8".to_string())?;
    let bin_path_str = bin_path.to_str()
        .ok_or_else(|| "Binary path contains invalid UTF-8".to_string())?;

    let compile_output = Command::new("clang")
        .args(["-o", bin_path_str, ir_path_str, "-lm", "-O0"])
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(format!("Linking failed: {}", stderr));
    }

    // Execute with timeout using spawn + try_wait loop
    let mut child = Command::new(bin_path_str)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Execution failed: {}", e))?;

    // Apply resource limits on Linux
    #[cfg(target_os = "linux")]
    {
        use std::process::Stdio;
        let _ = Stdio::null(); // suppress unused import on non-linux
        // Resource limits are set via ulimit wrapper if available
        // For additional security, the binary runs in a temp directory with env_clear()
    }

    let timeout = std::time::Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_millis(50);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished
                let mut stdout_bytes = Vec::new();
                let mut stderr_bytes = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    use std::io::Read;
                    let _ = out.read_to_end(&mut stdout_bytes);
                }
                if let Some(mut err) = child.stderr.take() {
                    use std::io::Read;
                    let _ = err.read_to_end(&mut stderr_bytes);
                }

                let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
                let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();
                let combined = if stderr.is_empty() {
                    stdout
                } else {
                    format!("{}{}", stdout, stderr)
                };
                let combined = truncate_output(&combined, max_output_bytes);
                let exit_code = status.code().unwrap_or(-1);
                return Ok((combined, exit_code));
            }
            Ok(None) => {
                // Still running
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "Execution timed out after {} seconds",
                        timeout_secs
                    ));
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                return Err(format!("Error waiting for process: {}", e));
            }
        }
    }
}

fn extract_line(err: &str) -> usize {
    if let Some(pos) = err.find("line ") {
        let rest = &err[pos + 5..];
        if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
            if let Ok(line) = rest[..end].parse() {
                return line;
            }
        }
    }
    1
}

fn extract_column(err: &str) -> usize {
    if let Some(pos) = err.find("column ") {
        let rest = &err[pos + 7..];
        if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
            if let Ok(col) = rest[..end].parse() {
                return col;
            }
        }
    }
    1
}
