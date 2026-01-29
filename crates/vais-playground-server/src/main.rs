//! Vais Playground Server
//!
//! Server-side compilation and execution service for the Vais Playground.
//! Accepts Vais source code via REST API, compiles it using the real compiler
//! pipeline, and optionally executes the resulting binary.

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    compile_semaphore: Arc<Semaphore>,
    config: PlaygroundConfig,
}

#[derive(Clone)]
struct PlaygroundConfig {
    host: String,
    port: u16,
    max_concurrent: usize,
    _execution_timeout_secs: u64,
    max_source_bytes: usize,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("PLAYGROUND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PLAYGROUND_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            max_concurrent: std::env::var("PLAYGROUND_MAX_CONCURRENT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4),
            _execution_timeout_secs: std::env::var("PLAYGROUND_TIMEOUT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(10),
            max_source_bytes: 64 * 1024, // 64KB max source
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

    let state = AppState {
        compile_semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
        config,
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

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
    Json(req): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<CompileResponse>)> {
    if req.source.len() > state.config.max_source_bytes {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(CompileResponse {
                success: false,
                errors: vec![DiagnosticItem {
                    line: 0,
                    column: 0,
                    message: format!(
                        "Source code too large: {} bytes (max {} bytes)",
                        req.source.len(),
                        state.config.max_source_bytes
                    ),
                    severity: Some("error".to_string()),
                }],
                warnings: vec![],
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: None,
            }),
        ));
    }

    let _permit = state.compile_semaphore.acquire().await.map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CompileResponse {
                success: false,
                errors: vec![DiagnosticItem {
                    line: 0,
                    column: 0,
                    message: "Server busy, try again later".to_string(),
                    severity: Some("error".to_string()),
                }],
                warnings: vec![],
                ir: None,
                output: None,
                exit_code: None,
                compile_time_ms: None,
            }),
        )
    })?;

    let source = req.source.clone();
    let optimize = req.optimize;
    let emit_ir = req.emit_ir;
    let execute = req.execute;

    let result = tokio::task::spawn_blocking(move || {
        compile_and_run(&source, optimize, emit_ir, execute)
    })
    .await
    .unwrap_or_else(|e| CompileResponse {
        success: false,
        errors: vec![DiagnosticItem {
            line: 0,
            column: 0,
            message: format!("Internal error: {}", e),
            severity: Some("error".to_string()),
        }],
        warnings: vec![],
        ir: None,
        output: None,
        exit_code: None,
        compile_time_ms: None,
    });

    Ok(Json(result))
}

fn compile_and_run(
    source: &str,
    optimize: bool,
    emit_ir: bool,
    execute: bool,
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
        match compile_ir_and_execute(&ir_string) {
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

fn compile_ir_and_execute(ir: &str) -> Result<(String, i32), String> {
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let ir_path = tmp_dir.path().join("playground.ll");
    let bin_path = tmp_dir.path().join("playground");

    std::fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    let compile_output = Command::new("clang")
        .args([
            "-o",
            bin_path.to_str().unwrap(),
            ir_path.to_str().unwrap(),
            "-lm",
            "-O0",
        ])
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(format!("Linking failed: {}", stderr));
    }

    let exec_output = Command::new(bin_path.to_str().unwrap())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output();

    match exec_output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout
            } else {
                format!("{}{}", stdout, stderr)
            };
            let exit_code = output.status.code().unwrap_or(-1);
            Ok((combined, exit_code))
        }
        Err(e) => Err(format!("Execution failed: {}", e)),
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
