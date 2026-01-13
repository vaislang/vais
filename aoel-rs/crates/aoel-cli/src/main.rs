//! AOEL CLI
//!
//! Command-line interface for the AOEL language.

mod package;

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

        /// Enable JIT compilation (requires --features jit)
        #[arg(long)]
        jit: bool,

        /// Show JIT/profiling statistics after execution
        #[arg(long)]
        stats: bool,
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

        /// Target format (c, wasm, llvm)
        #[arg(long, default_value = "c")]
        target: String,
    },

    /// JIT compile and execute using Cranelift (requires --features cranelift)
    Jit {
        /// The AOEL file to execute
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Skip type checking
        #[arg(long)]
        no_typecheck: bool,
    },

    // === Package Manager Commands ===

    /// Initialize a new AOEL project
    Init {
        /// Project directory (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,
    },

    /// Add a dependency to the project
    Add {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// Package version (default: latest)
        #[arg(long, default_value = "*")]
        version: String,

        /// Add as dev dependency
        #[arg(long)]
        dev: bool,

        /// Local path to package
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Remove a dependency from the project
    Remove {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,
    },

    /// Install project dependencies
    Install,

    /// List installed packages
    List {
        /// Show all available packages in registry
        #[arg(long)]
        available: bool,
    },

    /// Publish package to registry (local only for now)
    Publish {
        /// Package path (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },

    // === Development Tools ===

    /// Format AOEL source code
    Format {
        /// The AOEL file to format
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Write formatted output back to file (default: print to stdout)
        #[arg(short, long)]
        write: bool,

        /// Check if file is formatted (exit with error if not)
        #[arg(short, long)]
        check: bool,

        /// Indentation width (default: 4)
        #[arg(long, default_value = "4")]
        indent: usize,

        /// Maximum line width (default: 100)
        #[arg(long, default_value = "100")]
        max_width: usize,
    },

    /// Profile AOEL program execution
    Profile {
        /// The AOEL file to profile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
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
        Commands::Run { file, func, args, jit, stats } => {
            run_file(&file, func.as_deref(), &args, jit, stats);
        }
        Commands::Repl => {
            repl();
        }
        Commands::Build { file, output, keep_c, no_typecheck, target } => {
            build_file(&file, output.as_ref(), keep_c, no_typecheck, &target);
        }
        Commands::Jit { file, no_typecheck } => {
            jit_run(&file, no_typecheck);
        }
        Commands::Init { path, name } => {
            cmd_init(path.as_ref(), name.as_deref());
        }
        Commands::Add { package, version, dev, path } => {
            cmd_add(&package, &version, dev, path.as_ref());
        }
        Commands::Remove { package } => {
            cmd_remove(&package);
        }
        Commands::Install => {
            cmd_install();
        }
        Commands::List { available } => {
            cmd_list(available);
        }
        Commands::Publish { path } => {
            cmd_publish(path.as_ref());
        }
        Commands::Format { file, write, check, indent, max_width } => {
            cmd_format(&file, write, check, indent, max_width);
        }
        Commands::Profile { file, format } => {
            cmd_profile(&file, &format);
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

fn run_file(path: &PathBuf, func_name: Option<&str>, args_json: &str, use_jit: bool, _show_stats: bool) {
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

    // Execute with JIT or interpreter
    #[cfg(feature = "jit")]
    if use_jit {
        run_with_jit(functions, &target_func, args, show_stats);
        return;
    }

    #[cfg(not(feature = "jit"))]
    if use_jit {
        eprintln!("JIT support not enabled. Build with: cargo build --features jit");
        std::process::exit(1);
    }

    // Execute with interpreter
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

#[cfg(feature = "jit")]
fn run_with_jit(functions: Vec<aoel_lowering::CompiledFunction>, func_name: &str, args: Vec<aoel_ir::Value>, show_stats: bool) {
    use aoel_vm::{JitVm, JitConfig};

    let mut vm = JitVm::with_config(JitConfig {
        enabled: true,
        auto_jit: true,
        profiling: true,
        threshold: 10, // Lower threshold for CLI execution
    });

    vm.load_functions(functions);

    match vm.call_function(func_name, args) {
        Ok(result) => {
            println!("{}", result);

            if show_stats {
                vm.print_jit_stats();
                vm.print_profile_stats();
            }
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
        "c" => build_c_target(path, output.map(|p| p.as_path()), keep_c, &functions),
        "wasm" | "wat" => build_wasm_target(path, output.map(|p| p.as_path()), &functions),
        "llvm" | "ll" => build_llvm_target(path, output.map(|p| p.as_path()), &functions),
        _ => {
            eprintln!("Unknown target: {}. Supported targets: c, wasm, llvm", target);
            std::process::exit(1);
        }
    }
}

fn build_c_target(
    path: &Path,
    output: Option<&Path>,
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
    let exe_file = output.map(Path::to_path_buf).unwrap_or_else(|| {
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
    path: &Path,
    output: Option<&Path>,
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
    let wasm_file = output.map(Path::to_path_buf).unwrap_or_else(|| path.with_extension("wasm"));

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
            println!();
            println!("Note: wat2wasm not found. To compile to WASM binary:");
            println!("  1. Install wabt: https://github.com/WebAssembly/wabt");
            println!("  2. Run: wat2wasm {} -o {}", wat_file.display(), wasm_file.display());
            println!();
            println!("Or use the WAT file directly with a WASM runtime that supports text format.");
        }
    }
}

fn build_llvm_target(
    path: &Path,
    output: Option<&Path>,
    functions: &[aoel_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate LLVM IR code
    let llvm_ir = match aoel_codegen::generate_llvm_ir(functions) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("Code generation error: {}", e);
            std::process::exit(1);
        }
    };

    // Determine output paths
    let ll_file = path.with_extension("ll");
    let exe_file = output.map(Path::to_path_buf).unwrap_or_else(|| {
        if cfg!(windows) {
            path.with_extension("exe")
        } else {
            path.with_extension("")
        }
    });

    // Write LLVM IR file
    if let Err(e) = fs::write(&ll_file, &llvm_ir) {
        eprintln!("Failed to write LLVM IR file: {}", e);
        std::process::exit(1);
    }

    println!("Generated LLVM IR: {}", ll_file.display());

    // Try to compile using clang
    let status = Command::new("clang")
        .arg("-O2")
        .arg("-o")
        .arg(&exe_file)
        .arg(&ll_file)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("✓ Compiled {} to {}", path.display(), exe_file.display());
            // Keep the .ll file for debugging
            println!("  (LLVM IR file kept: {})", ll_file.display());
        }
        Ok(s) => {
            eprintln!("clang failed with exit code: {:?}", s.code());
            eprintln!("LLVM IR file generated: {}", ll_file.display());
            eprintln!("You can compile it manually with: clang {} -o {}", ll_file.display(), exe_file.display());
        }
        Err(_) => {
            // Try llc + cc as fallback
            println!("clang not found, trying llc...");

            let obj_file = path.with_extension("o");
            let llc_status = Command::new("llc")
                .arg("-filetype=obj")
                .arg("-o")
                .arg(&obj_file)
                .arg(&ll_file)
                .status();

            match llc_status {
                Ok(s) if s.success() => {
                    // Link with system linker
                    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
                    let link_status = Command::new(&compiler)
                        .arg("-o")
                        .arg(&exe_file)
                        .arg(&obj_file)
                        .status();

                    match link_status {
                        Ok(s) if s.success() => {
                            println!("✓ Compiled {} to {}", path.display(), exe_file.display());
                            let _ = fs::remove_file(&obj_file);
                        }
                        _ => {
                            eprintln!("Linking failed. Object file: {}", obj_file.display());
                        }
                    }
                }
                _ => {
                    println!("✓ Generated LLVM IR file: {}", ll_file.display());
                    println!();
                    println!("Note: Neither clang nor llc found. To compile to native binary:");
                    println!("  1. Install LLVM: https://llvm.org/");
                    println!("  2. Run: clang {} -o {}", ll_file.display(), exe_file.display());
                    println!();
                    println!("Or use llc to compile to object file:");
                    println!("  llc -filetype=obj {} -o {}.o", ll_file.display(), path.display());
                }
            }
        }
    }
}

fn jit_run(path: &PathBuf, no_typecheck: bool) {
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

    // Type check (optional)
    if !no_typecheck {
        if let Err(e) = aoel_typeck::check(&program) {
            eprintln!("Type error: {:?}", e);
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

    // JIT compile and execute
    match aoel_codegen::jit_execute(&functions) {
        Ok(result) => {
            // Decode and print result
            let tag = (result & 0xFF) as u8;
            let value = result >> 8;
            match tag {
                0 => println!("void"),
                1 => println!("{}", if value != 0 { "true" } else { "false" }),
                2 => println!("{}", value),
                3 => {
                    // Float - approximate decoding
                    let bits = (result as u64) >> 8;
                    let f = f64::from_bits(bits << 8);
                    println!("{}", f);
                }
                _ => println!("{}", result),
            }
        }
        Err(e) => {
            eprintln!("JIT error: {}", e);
            eprintln!();
            eprintln!("Note: Cranelift JIT requires the 'cranelift' feature.");
            eprintln!("Build with: cargo build --release --features cranelift");
            std::process::exit(1);
        }
    }
}

// === Package Manager Commands ===

fn cmd_init(path: Option<&PathBuf>, name: Option<&str>) {
    let project_path = path
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    // Create directory if it doesn't exist
    if !project_path.exists() {
        if let Err(e) = fs::create_dir_all(&project_path) {
            eprintln!("Failed to create directory: {}", e);
            std::process::exit(1);
        }
    }

    // Check if aoel.toml already exists
    if project_path.join("aoel.toml").exists() {
        eprintln!("Error: aoel.toml already exists in {}", project_path.display());
        std::process::exit(1);
    }

    match package::init_project(&project_path, name) {
        Ok(()) => {
            let manifest = package::Manifest::load(&project_path).unwrap();
            println!("Initialized AOEL project '{}'", manifest.package.name);
            println!();
            println!("Created files:");
            println!("  - aoel.toml");
            println!("  - src/main.aoel");
            println!("  - .gitignore");
            println!();
            println!("Run your project with:");
            println!("  cd {}", project_path.display());
            println!("  aoel run src/main.aoel");
        }
        Err(e) => {
            eprintln!("Failed to initialize project: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_add(package_name: &str, version: &str, dev: bool, local_path: Option<&PathBuf>) {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let project_root = match package::find_project_root(&cwd) {
        Some(p) => p,
        None => {
            eprintln!("Error: No aoel.toml found in current directory or any parent");
            eprintln!("Run 'aoel init' to create a new project");
            std::process::exit(1);
        }
    };

    let mut manifest = match package::Manifest::load(&project_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // If local path specified, use path dependency
    if let Some(path) = local_path {
        let dep = package::Dependency::Detailed(package::DetailedDependency {
            version: if version == "*" { None } else { Some(version.to_string()) },
            path: Some(path.to_string_lossy().to_string()),
            git: None,
            branch: None,
            tag: None,
        });

        if dev {
            manifest.dev_dependencies.insert(package_name.to_string(), dep);
        } else {
            manifest.dependencies.insert(package_name.to_string(), dep);
        }
    } else {
        manifest.add_dependency(package_name, version, dev);
    }

    if let Err(e) = manifest.save(&project_root) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let dep_type = if dev { "dev-dependency" } else { "dependency" };
    println!("Added {} '{}' ({})", dep_type, package_name, version);
}

fn cmd_remove(package_name: &str) {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let project_root = match package::find_project_root(&cwd) {
        Some(p) => p,
        None => {
            eprintln!("Error: No aoel.toml found in current directory or any parent");
            std::process::exit(1);
        }
    };

    let mut manifest = match package::Manifest::load(&project_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if manifest.remove_dependency(package_name) {
        if let Err(e) = manifest.save(&project_root) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("Removed dependency '{}'", package_name);
    } else {
        eprintln!("Error: Dependency '{}' not found", package_name);
        std::process::exit(1);
    }
}

fn cmd_install() {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let project_root = match package::find_project_root(&cwd) {
        Some(p) => p,
        None => {
            eprintln!("Error: No aoel.toml found in current directory or any parent");
            std::process::exit(1);
        }
    };

    let manifest = match package::Manifest::load(&project_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let registry = match package::Registry::local() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let total_deps = manifest.dependencies.len() + manifest.dev_dependencies.len();
    if total_deps == 0 {
        println!("No dependencies to install");
        return;
    }

    let mut installed = 0;
    let mut lock = package::LockFile::load(&project_root).unwrap_or_default();

    println!("Installing {} dependencies...", total_deps);

    // Process dependencies
    for (name, dep) in &manifest.dependencies {
        match dep {
            package::Dependency::Version(ver) => {
                if registry.has_package(name, ver) {
                    println!("  {} {} (from registry)", name, ver);
                    installed += 1;
                    lock.packages.push(package::LockedPackage {
                        name: name.clone(),
                        version: ver.clone(),
                        checksum: None,
                        source: Some("registry".to_string()),
                    });
                } else {
                    println!("  {} {} (not found in registry)", name, ver);
                }
            }
            package::Dependency::Detailed(detail) => {
                if let Some(path) = &detail.path {
                    println!("  {} (path: {})", name, path);
                    installed += 1;
                    lock.packages.push(package::LockedPackage {
                        name: name.clone(),
                        version: detail.version.clone().unwrap_or_else(|| "local".to_string()),
                        checksum: None,
                        source: Some(format!("path:{}", path)),
                    });
                } else if let Some(git) = &detail.git {
                    println!("  {} (git: {})", name, git);
                    // Git dependencies not yet supported
                }
            }
        }
    }

    // Process dev dependencies
    for (name, dep) in &manifest.dev_dependencies {
        match dep {
            package::Dependency::Version(ver) => {
                if registry.has_package(name, ver) {
                    println!("  {} {} [dev] (from registry)", name, ver);
                    installed += 1;
                } else {
                    println!("  {} {} [dev] (not found in registry)", name, ver);
                }
            }
            package::Dependency::Detailed(detail) => {
                if let Some(path) = &detail.path {
                    println!("  {} [dev] (path: {})", name, path);
                    installed += 1;
                }
            }
        }
    }

    // Save lock file
    if let Err(e) = lock.save(&project_root) {
        eprintln!("Warning: Failed to save lock file: {}", e);
    }

    println!();
    println!("Installed {} of {} dependencies", installed, total_deps);
}

fn cmd_list(available: bool) {
    if available {
        // List packages in registry
        let registry = match package::Registry::local() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        match registry.list_packages() {
            Ok(packages) => {
                if packages.is_empty() {
                    println!("No packages in registry");
                    println!();
                    println!("Publish a package with:");
                    println!("  aoel publish");
                } else {
                    println!("Available packages:");
                    for pkg in packages {
                        println!("  - {}", pkg);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // List project dependencies
        let cwd = std::env::current_dir().expect("Failed to get current directory");
        let project_root = match package::find_project_root(&cwd) {
            Some(p) => p,
            None => {
                eprintln!("Error: No aoel.toml found in current directory or any parent");
                std::process::exit(1);
            }
        };

        let manifest = match package::Manifest::load(&project_root) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        println!("{} v{}", manifest.package.name, manifest.package.version);
        println!();

        if manifest.dependencies.is_empty() && manifest.dev_dependencies.is_empty() {
            println!("No dependencies");
            return;
        }

        if !manifest.dependencies.is_empty() {
            println!("Dependencies:");
            for (name, dep) in &manifest.dependencies {
                match dep {
                    package::Dependency::Version(v) => println!("  {} = \"{}\"", name, v),
                    package::Dependency::Detailed(d) => {
                        if let Some(path) = &d.path {
                            println!("  {} (path: {})", name, path);
                        } else if let Some(git) = &d.git {
                            println!("  {} (git: {})", name, git);
                        } else {
                            println!("  {} = \"{}\"", name, d.version.as_deref().unwrap_or("*"));
                        }
                    }
                }
            }
        }

        if !manifest.dev_dependencies.is_empty() {
            println!();
            println!("Dev Dependencies:");
            for (name, dep) in &manifest.dev_dependencies {
                match dep {
                    package::Dependency::Version(v) => println!("  {} = \"{}\"", name, v),
                    package::Dependency::Detailed(d) => {
                        if let Some(path) = &d.path {
                            println!("  {} (path: {})", name, path);
                        } else {
                            println!("  {} = \"{}\"", name, d.version.as_deref().unwrap_or("*"));
                        }
                    }
                }
            }
        }
    }
}

fn cmd_publish(path: Option<&PathBuf>) {
    let project_path = path
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let project_root = match package::find_project_root(&project_path) {
        Some(p) => p,
        None => {
            eprintln!("Error: No aoel.toml found in {} or any parent", project_path.display());
            std::process::exit(1);
        }
    };

    let manifest = match package::Manifest::load(&project_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let registry = match package::Registry::local() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let name = &manifest.package.name;
    let version = &manifest.package.version;

    // Check if already published
    if registry.has_package(name, version) {
        eprintln!("Error: Package {} v{} already exists in registry", name, version);
        eprintln!("Bump the version in aoel.toml and try again");
        std::process::exit(1);
    }

    // Get source directory (src/ or project root)
    let src_dir = if project_root.join("src").exists() {
        project_root.join("src")
    } else {
        project_root.clone()
    };

    match registry.install_from_path(name, version, &src_dir) {
        Ok(()) => {
            // Also copy manifest
            let pkg_path = registry.get_package_path(name, version);
            let _ = fs::copy(project_root.join("aoel.toml"), pkg_path.join("aoel.toml"));

            println!("Published {} v{} to local registry", name, version);
            println!();
            println!("Other projects can use it with:");
            println!("  aoel add {} --version {}", name, version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

// === Development Tools Commands ===

fn cmd_format(path: &PathBuf, write: bool, check: bool, indent: usize, max_width: usize) {
    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Create formatter with config
    let config = aoel_tools::formatter::FormatConfig {
        indent_width: indent,
        max_line_width: max_width,
        ..Default::default()
    };
    let mut formatter = aoel_tools::Formatter::with_config(config);

    // Format the source
    let formatted = match formatter.format_source(&source) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Format error: {}", e);
            std::process::exit(1);
        }
    };

    if check {
        // Check mode: exit with error if not formatted
        if source != formatted {
            eprintln!("File {} is not properly formatted", path.display());
            eprintln!("Run 'aoel format {}' to format it", path.display());
            std::process::exit(1);
        }
        println!("File {} is properly formatted", path.display());
    } else if write {
        // Write mode: write back to file
        if source == formatted {
            println!("File {} is already formatted", path.display());
        } else {
            if let Err(e) = fs::write(path, &formatted) {
                eprintln!("Failed to write file: {}", e);
                std::process::exit(1);
            }
            println!("Formatted {}", path.display());
        }
    } else {
        // Print mode: print to stdout
        print!("{}", formatted);
    }
}

fn cmd_profile(path: &PathBuf, output_format: &str) {
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
        eprintln!("No functions to profile");
        std::process::exit(1);
    }

    // Profile execution
    let mut profiler = aoel_tools::Profiler::new();
    let result = profiler.profile(functions);

    // Output result
    match output_format.to_lowercase().as_str() {
        "json" => {
            println!("{}", result.to_json());
        }
        _ => {
            println!("{}", result.summary());
        }
    }
}
