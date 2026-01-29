//! Vais DAP Server Entry Point
//!
//! Starts the Debug Adapter Protocol server for Vais debugging.

use std::io::{self};
use std::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use vais_dap::DapServer;

#[derive(Debug)]
struct Args {
    /// Use TCP socket instead of stdio
    port: Option<u16>,
    /// Log level (trace, debug, info, warn, error)
    log_level: Level,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            port: None,
            log_level: Level::INFO,
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--port" | "-p" => {
                if let Some(port_str) = iter.next() {
                    if let Ok(port) = port_str.parse() {
                        args.port = Some(port);
                    }
                }
            }
            "--log-level" | "-l" => {
                if let Some(level_str) = iter.next() {
                    args.log_level = match level_str.to_lowercase().as_str() {
                        "trace" => Level::TRACE,
                        "debug" => Level::DEBUG,
                        "info" => Level::INFO,
                        "warn" => Level::WARN,
                        "error" => Level::ERROR,
                        _ => Level::INFO,
                    };
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("vais-dap {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => {}
        }
    }

    args
}

fn print_help() {
    println!(
        r#"vais-dap - Debug Adapter Protocol server for Vais

USAGE:
    vais-dap [OPTIONS]

OPTIONS:
    -p, --port <PORT>       Start server on TCP port (default: stdio)
    -l, --log-level <LEVEL> Set log level (trace, debug, info, warn, error)
    -h, --help              Print help information
    -v, --version           Print version information

EXAMPLES:
    # Start with stdio (for IDE integration)
    vais-dap

    # Start with TCP socket for remote debugging
    vais-dap --port 4711
"#
    );
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = parse_args();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting vais-dap server v{}", env!("CARGO_PKG_VERSION"));

    match args.port {
        Some(port) => run_tcp_server(port).await,
        None => run_stdio_server().await,
    }
}

async fn run_stdio_server() -> io::Result<()> {
    info!("Running in stdio mode");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut server = DapServer::new();
    server.run(stdin, stdout).await
}

async fn run_tcp_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    info!("DAP server listening on port {}", port);

    loop {
        let (stream, addr) = listener.accept()?;
        info!("Client connected from {}", addr);

        let stream = tokio::net::TcpStream::from_std(stream)?;
        let (reader, writer) = stream.into_split();

        let mut server = DapServer::new();
        if let Err(e) = server.run(reader, writer).await {
            tracing::error!("Session error: {}", e);
        }

        info!("Client disconnected");
    }
}
