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
    /// Parse an AOEL file and check for syntax/type errors
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

    /// Compile and execute an expression
    Eval {
        /// The expression to evaluate
        #[arg(value_name = "EXPR")]
        expr: String,
    },

    /// Compile an AOEL file to IR
    Compile {
        /// The AOEL file to compile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Execute an AOEL file
    Run {
        /// The AOEL file to execute
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function to call (default: __main__ or first function)
        #[arg(short, long)]
        func: Option<String>,

        /// Arguments as JSON array (e.g., '[1, 2, 3]')
        #[arg(short, long, default_value = "[]")]
        args: String,
    },

    /// Start interactive REPL
    Repl,

    /// Compile to native executable
    Build {
        /// The AOEL file to compile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file (default: input file name without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Keep generated C file
        #[arg(long)]
        keep_c: bool,

        /// Skip type checking
        #[arg(long)]
        no_typecheck: bool,

        /// Target format (c, wasm)
        #[arg(long, default_value = "c")]
        target: String,
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
        Commands::Eval { expr } => {
            eval_expr(&expr);
        }
        Commands::Compile { file, output } => {
            compile_file(&file, output.as_ref());
        }
        Commands::Run { file, func, args } => {
            run_file(&file, func.as_deref(), &args);
        }
        Commands::Repl => {
            repl();
        }
        Commands::Build { file, output, keep_c, no_typecheck, target } => {
            build_file(&file, output.as_ref(), keep_c, no_typecheck, &target);
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

    // Parse
    let program = match aoel_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Syntax error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match aoel_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    let func_count = program.items.iter().filter(|i| matches!(i, aoel_ast::Item::Function(_))).count();
    let expr_count = program.items.iter().filter(|i| matches!(i, aoel_ast::Item::Expr(_))).count();

    // Type check
    match aoel_typeck::check(&program) {
        Ok(()) => {
            println!("✓ {} passed all checks", path.display());
            println!("  Functions: {}", func_count);
            println!("  Expressions: {}", expr_count);
        }
        Err(e) => {
            eprintln!("Type error: {}", e);
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
        Ok(program) => {
            println!("{:#?}", program);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
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

    let tokens = match aoel_lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lex error: {:?}", e);
            std::process::exit(1);
        }
    };

    for token in tokens {
        println!(
            "{:4}..{:4}  {:20} {:?}",
            token.span.start,
            token.span.end,
            format!("{:?}", token.kind),
            token.text
        );
    }
}

fn eval_expr(expr: &str) {
    // Parse expression
    let parsed = match aoel_parser::parse_expr(expr) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Create a program with just this expression
    let program = aoel_ast::Program {
        items: vec![aoel_ast::Item::Expr(parsed)],
        span: aoel_lexer::Span::new(0, expr.len()),
    };

    // Lower to IR
    let mut lowerer = aoel_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Execute
    match aoel_vm::execute_function(functions, "__main__", vec![]) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_file(path: &PathBuf, output: Option<&PathBuf>) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Parse
    let program = match aoel_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match aoel_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Lower to IR
    let mut lowerer = aoel_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Serialize IR
    let json = serde_json::to_string_pretty(&functions.iter().map(|f| {
        serde_json::json!({
            "name": f.name,
            "params": f.params,
            "instructions": f.instructions.iter().map(|i| format!("{:?}", i.opcode)).collect::<Vec<_>>()
        })
    }).collect::<Vec<_>>()).unwrap();

    match output {
        Some(out_path) => {
            fs::write(out_path, &json).unwrap_or_else(|e| {
                eprintln!("Failed to write output file: {}", e);
                std::process::exit(1);
            });
            println!("✓ Compiled {} to {}", path.display(), out_path.display());
        }
        None => {
            println!("{}", json);
        }
    }
}

fn run_file(path: &PathBuf, func_name: Option<&str>, args_json: &str) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Parse
    let program = match aoel_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match aoel_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Lower to IR
    let mut lowerer = aoel_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    if functions.is_empty() {
        eprintln!("No functions to execute");
        std::process::exit(1);
    }

    // Parse arguments
    let args: Vec<aoel_ir::Value> = parse_args_json(args_json);

    // Determine which function to call
    let target_func = func_name
        .map(|s| s.to_string())
        .or_else(|| {
            functions.iter().find(|f| f.name == "__main__").map(|f| f.name.clone())
        })
        .unwrap_or_else(|| functions[0].name.clone());

    // Execute
    match aoel_vm::execute_function(functions, &target_func, args) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
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

/// Parse JSON array string into Value vector
fn parse_args_json(json: &str) -> Vec<aoel_ir::Value> {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse arguments JSON: {}", e);
        std::process::exit(1);
    });

    match parsed {
        serde_json::Value::Array(arr) => {
            arr.into_iter().map(json_to_value).collect()
        }
        _ => {
            eprintln!("Arguments must be a JSON array");
            std::process::exit(1);
        }
    }
}

fn repl() {
    use std::io::{self, Write};

    println!("AOEL REPL v0.1.0");
    println!("Type expressions to evaluate, ':help' for commands, ':quit' to exit.\n");

    // Store defined functions across inputs
    let mut all_functions: Vec<aoel_lowering::CompiledFunction> = Vec::new();

    loop {
        print!("aoel> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle REPL commands
        if input.starts_with(':') {
            match input {
                ":quit" | ":q" | ":exit" => {
                    println!("Goodbye!");
                    break;
                }
                ":help" | ":h" => {
                    println!("Commands:");
                    println!("  :help, :h     Show this help");
                    println!("  :quit, :q     Exit REPL");
                    println!("  :list, :l     List defined functions");
                    println!("  :clear        Clear all defined functions");
                    println!("\nExamples:");
                    println!("  1 + 2 * 3           Simple arithmetic");
                    println!("  add(a, b) = a + b   Define a function");
                    println!("  add(3, 4)           Call a function");
                    println!("  [1,2,3].@(_ * 2)    Map operation");
                    continue;
                }
                ":list" | ":l" => {
                    if all_functions.is_empty() {
                        println!("No functions defined.");
                    } else {
                        println!("Defined functions:");
                        for f in &all_functions {
                            if f.name != "__main__" {
                                println!("  {}({})", f.name, f.params.join(", "));
                            }
                        }
                    }
                    continue;
                }
                ":clear" => {
                    all_functions.clear();
                    println!("Cleared all functions.");
                    continue;
                }
                _ => {
                    eprintln!("Unknown command: {}", input);
                    continue;
                }
            }
        }

        // Try to parse as a full program (may contain function definitions)
        match aoel_parser::parse(input) {
            Ok(program) => {
                let mut lowerer = aoel_lowering::Lowerer::new();
                match lowerer.lower_program(&program) {
                    Ok(mut functions) => {
                        // Separate new function definitions from __main__
                        let mut main_func = None;
                        for func in functions.drain(..) {
                            if func.name == "__main__" {
                                main_func = Some(func);
                            } else {
                                // Update or add function definition
                                all_functions.retain(|f| f.name != func.name);
                                all_functions.push(func);
                            }
                        }

                        // If there's a __main__, execute it
                        if let Some(main) = main_func {
                            let mut exec_functions = all_functions.clone();
                            exec_functions.push(main);

                            match aoel_vm::execute_function(exec_functions, "__main__", vec![]) {
                                Ok(result) => {
                                    if !matches!(result, aoel_ir::Value::Void) {
                                        println!("=> {}", result);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Runtime error: {}", e);
                                }
                            }
                        } else {
                            // Just function definition(s), no expression to evaluate
                            println!("Function defined.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Lowering error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
            }
        }
    }
}

fn build_file(path: &PathBuf, output: Option<&PathBuf>, keep_c: bool, no_typecheck: bool, target: &str) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Parse
    let program = match aoel_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match aoel_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Type check (unless skipped)
    if !no_typecheck {
        if let Err(e) = aoel_typeck::check(&program) {
            eprintln!("Type error: {}", e);
            eprintln!("(Use --no-typecheck to skip type checking)");
            std::process::exit(1);
        }
    }

    // Lower to IR
    let mut lowerer = aoel_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    if functions.is_empty() {
        eprintln!("No functions to compile");
        std::process::exit(1);
    }

    match target.to_lowercase().as_str() {
        "c" => build_c_target(path, output, keep_c, &functions),
        "wasm" | "wat" => build_wasm_target(path, output, &functions),
        _ => {
            eprintln!("Unknown target: {}. Supported targets: c, wasm", target);
            std::process::exit(1);
        }
    }
}

fn build_c_target(
    path: &PathBuf,
    output: Option<&PathBuf>,
    keep_c: bool,
    functions: &[aoel_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate C code
    let c_code = match aoel_codegen::generate_c(functions) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Determine output paths
    let c_file = path.with_extension("c");
    let exe_file = output.cloned().unwrap_or_else(|| {
        if cfg!(windows) {
            path.with_extension("exe")
        } else {
            path.with_extension("")
        }
    });

    // Write C file
    if let Err(e) = fs::write(&c_file, &c_code) {
        eprintln!("Failed to write C file: {}", e);
        std::process::exit(1);
    }

    println!("Generated C code: {}", c_file.display());

    // Compile with system C compiler
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());

    let status = Command::new(&compiler)
        .arg("-O2")
        .arg("-o")
        .arg(&exe_file)
        .arg(&c_file)
        .arg("-lm")  // Link math library
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Compiled {} to {}", path.display(), exe_file.display());

            // Clean up C file unless --keep-c
            if !keep_c {
                let _ = fs::remove_file(&c_file);
            }
        }
        Ok(s) => {
            eprintln!("C compiler failed with exit code: {:?}", s.code());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run C compiler '{}': {}", compiler, e);
            eprintln!("Make sure you have a C compiler installed (gcc, clang, or set CC env var)");
            std::process::exit(1);
        }
    }
}

fn build_wasm_target(
    path: &PathBuf,
    output: Option<&PathBuf>,
    functions: &[aoel_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate WAT code
    let wat_code = match aoel_codegen::generate_wat(functions) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Determine output paths
    let wat_file = path.with_extension("wat");
    let wasm_file = output.cloned().unwrap_or_else(|| path.with_extension("wasm"));

    // Write WAT file
    if let Err(e) = fs::write(&wat_file, &wat_code) {
        eprintln!("Failed to write WAT file: {}", e);
        std::process::exit(1);
    }

    println!("Generated WAT code: {}", wat_file.display());

    // Try to compile to WASM using wat2wasm if available
    let status = Command::new("wat2wasm")
        .arg(&wat_file)
        .arg("-o")
        .arg(&wasm_file)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Compiled {} to {}", path.display(), wasm_file.display());
            println!("  (WAT file kept: {})", wat_file.display());
        }
        Ok(s) => {
            eprintln!("wat2wasm failed with exit code: {:?}", s.code());
            eprintln!("WAT file generated: {}", wat_file.display());
            eprintln!("You can compile it manually with: wat2wasm {} -o {}", wat_file.display(), wasm_file.display());
        }
        Err(_) => {
            println!("✓ Generated WAT file: {}", wat_file.display());
            println!("");
            println!("Note: wat2wasm not found. To compile to WASM binary:");
            println!("  1. Install wabt: https://github.com/WebAssembly/wabt");
            println!("  2. Run: wat2wasm {} -o {}", wat_file.display(), wasm_file.display());
            println!("");
            println!("Or use the WAT file directly with a WASM runtime that supports text format.");
        }
    }
}
