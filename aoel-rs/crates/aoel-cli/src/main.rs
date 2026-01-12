//! AOEL CLI
//!
//! Command-line interface for the AOEL language.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aoel")]
#[command(author = "AOEL Team")]
#[command(version = "0.1.0")]
#[command(about = "AOEL - AI-Optimized Executable Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an AOEL file and check for syntax errors
    Check {
        /// The AOEL file to check
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Parse an AOEL file and print the AST
    Ast {
        /// The AOEL file to parse
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Tokenize an AOEL file and print tokens
    Tokens {
        /// The AOEL file to tokenize
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { file } => {
            check_file(&file);
        }
        Commands::Ast { file } => {
            print_ast(&file);
        }
        Commands::Tokens { file } => {
            print_tokens(&file);
        }
    }
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))
}

fn check_file(path: &PathBuf) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let filename = path.to_string_lossy();

    // Parse
    let unit = match aoel_parser::parse(&source) {
        Ok(unit) => unit,
        Err(e) => {
            eprintln!("{}", e.report(&source, &filename));
            std::process::exit(1);
        }
    };

    // Type check
    match aoel_typeck::check(&unit) {
        Ok(()) => {
            println!("✓ {} passed all checks", path.display());
            println!("  Unit: {} {}", unit.header.kind.as_str(), unit.full_name());
            if let Some(version) = unit.version() {
                println!("  Version: {}", version.to_string());
            }
            println!("  Input fields: {}", unit.input.fields.len());
            println!("  Output fields: {}", unit.output.fields.len());
            println!("  Flow nodes: {}", unit.flow.nodes.len());
            println!("  Flow edges: {}", unit.flow.edges.len());
            println!("  Constraints: {}", unit.constraint.constraints.len());
            println!("  Verify entries: {}", unit.verify.entries.len());
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error.report(&source, &filename));
            }
            eprintln!("\n✗ {} type error(s) found", errors.len());
            std::process::exit(1);
        }
    }
}

fn print_ast(path: &PathBuf) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match aoel_parser::parse(&source) {
        Ok(unit) => {
            println!("{:#?}", unit);
        }
        Err(e) => {
            let filename = path.to_string_lossy();
            eprintln!("{}", e.report(&source, &filename));
            std::process::exit(1);
        }
    }
}

fn print_tokens(path: &PathBuf) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let lexer = aoel_lexer::Lexer::new(&source);

    for token in lexer {
        println!(
            "{:4}..{:4}  {:20} {:?}",
            token.span.start,
            token.span.end,
            format!("{:?}", token.kind),
            token.text
        );
    }
}
