//! AOEL CLI
//!
//! Command-line interface for the AOEL language.

use clap::{Parser, Subcommand};
use std::collections::HashMap;
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

    /// Compile an AOEL file to IR
    Compile {
        /// The AOEL file to compile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0 = none, 1 = basic, 2 = aggressive)
        #[arg(short = 'O', long = "opt-level", default_value = "1")]
        opt_level: u8,
    },

    /// Execute an AOEL file
    Run {
        /// The AOEL file to execute
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Input values as JSON (e.g., '{"x": 10, "name": "test"}')
        #[arg(short, long, default_value = "{}")]
        input: String,
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
        Commands::Compile { file, output, opt_level } => {
            compile_file(&file, output.as_ref(), opt_level);
        }
        Commands::Run { file, input } => {
            run_file(&file, &input);
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

fn compile_file(path: &PathBuf, output: Option<&PathBuf>, opt_level: u8) {
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
    if let Err(errors) = aoel_typeck::check(&unit) {
        for error in &errors {
            eprintln!("{}", error.report(&source, &filename));
        }
        eprintln!("\n✗ {} type error(s) found", errors.len());
        std::process::exit(1);
    }

    // Lower to IR
    let mut module = aoel_ir::lower(&unit);

    // Apply optimizations
    let level = match opt_level {
        0 => aoel_ir::OptLevel::None,
        1 => aoel_ir::OptLevel::Basic,
        _ => aoel_ir::OptLevel::Aggressive,
    };
    aoel_ir::optimize(&mut module, level);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&module).unwrap_or_else(|e| {
        eprintln!("Failed to serialize IR: {}", e);
        std::process::exit(1);
    });

    // Output
    match output {
        Some(out_path) => {
            fs::write(out_path, &json).unwrap_or_else(|e| {
                eprintln!("Failed to write output file: {}", e);
                std::process::exit(1);
            });
            println!("✓ Compiled {} to {} (opt-level: {})", path.display(), out_path.display(), opt_level);
        }
        None => {
            println!("{}", json);
        }
    }
}

fn run_file(path: &PathBuf, input_json: &str) {
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
    if let Err(errors) = aoel_typeck::check(&unit) {
        for error in &errors {
            eprintln!("{}", error.report(&source, &filename));
        }
        eprintln!("\n✗ {} type error(s) found", errors.len());
        std::process::exit(1);
    }

    // Lower to IR
    let module = aoel_ir::lower(&unit);

    // Parse input JSON
    let inputs: HashMap<String, aoel_ir::Value> = parse_input_json(input_json);

    // Execute
    match aoel_vm::execute(&module, inputs) {
        Ok(outputs) => {
            println!("✓ Executed {}", path.display());
            println!("\nOutputs:");
            for (name, value) in &outputs {
                println!("  {}: {}", name, value);
            }
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Parse JSON input string into Value HashMap
fn parse_input_json(json: &str) -> HashMap<String, aoel_ir::Value> {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse input JSON: {}", e);
        std::process::exit(1);
    });

    let mut result = HashMap::new();

    if let serde_json::Value::Object(obj) = parsed {
        for (key, value) in obj {
            result.insert(key, json_to_value(value));
        }
    }

    result
}

/// Convert serde_json::Value to aoel_ir::Value
fn json_to_value(json: serde_json::Value) -> aoel_ir::Value {
    match json {
        serde_json::Value::Null => aoel_ir::Value::Void,
        serde_json::Value::Bool(b) => aoel_ir::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                aoel_ir::Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                aoel_ir::Value::Float(f)
            } else {
                aoel_ir::Value::Void
            }
        }
        serde_json::Value::String(s) => aoel_ir::Value::String(s),
        serde_json::Value::Array(arr) => {
            aoel_ir::Value::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }
            aoel_ir::Value::Map(map)
        }
    }
}
