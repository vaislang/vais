//! Vais CLI
//!
//! Command-line interface for the Vais language.

mod error_format;
mod package;

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "vais")]
#[command(author = "Vais Team")]
#[command(version = "0.1.0")]
#[command(about = "Vais - Vibe AI Script", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an Vais file and check for syntax/type errors
    Check {
        /// The Vais file to check
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Parse an Vais file and print the AST
    Ast {
        /// The Vais file to parse
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Tokenize an Vais file and print tokens
    Tokens {
        /// The Vais file to tokenize
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Compile and execute an expression
    Eval {
        /// The expression to evaluate
        #[arg(value_name = "EXPR")]
        expr: String,
    },

    /// Compile an Vais file to IR
    Compile {
        /// The Vais file to compile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Execute an Vais file
    Run {
        /// The Vais file to execute
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
        /// The Vais file to compile
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
        /// The Vais file to execute
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Skip type checking
        #[arg(long)]
        no_typecheck: bool,
    },

    // === Package Manager Commands ===

    /// Initialize a new Vais project
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

    /// Search for packages in the registry
    Search {
        /// Search query (matches name, description, keywords)
        #[arg(value_name = "QUERY")]
        query: String,
    },

    /// Show package information
    Info {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,
    },

    /// Update dependencies to latest compatible versions
    Update {
        /// Specific package to update (default: all)
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },

    // === Development Tools ===

    /// Format Vais source code
    Format {
        /// The Vais file to format
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

    /// Profile Vais program execution
    Profile {
        /// The Vais file to profile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Debug Vais program with interactive debugger
    Debug {
        /// The Vais file to debug
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Function to debug (default: __main__)
        #[arg(short, long)]
        func: Option<String>,

        /// Set breakpoint at instruction (function:instruction)
        #[arg(short, long)]
        breakpoint: Vec<String>,
    },

    /// Generate documentation for Vais files
    Doc {
        /// The Vais file or directory to document
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output directory (default: ./docs)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (markdown, html, json)
        #[arg(long, default_value = "markdown")]
        format: String,
    },

    /// Run tests in Vais files
    Test {
        /// File or directory to test (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Filter tests by name pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
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
        Commands::Search { query } => {
            cmd_search(&query);
        }
        Commands::Info { package } => {
            cmd_info(&package);
        }
        Commands::Update { package } => {
            cmd_update(package.as_deref());
        }
        Commands::Format { file, write, check, indent, max_width } => {
            cmd_format(&file, write, check, indent, max_width);
        }
        Commands::Profile { file, format } => {
            cmd_profile(&file, &format);
        }
        Commands::Debug { file, func, breakpoint } => {
            cmd_debug(&file, func.as_deref(), &breakpoint);
        }
        Commands::Doc { path, output, format } => {
            cmd_doc(&path, output.as_ref(), &format);
        }
        Commands::Test { path, filter, verbose } => {
            cmd_test(path.as_ref(), filter.as_deref(), verbose);
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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Syntax error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    let func_count = program.items.iter().filter(|i| matches!(i, vais_ast::Item::Function(_))).count();
    let expr_count = program.items.iter().filter(|i| matches!(i, vais_ast::Item::Expr(_))).count();

    // Type check
    match vais_typeck::check(&program) {
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

    match vais_parser::parse(&source) {
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

    let tokens = match vais_lexer::tokenize(&source) {
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
    let parsed = match vais_parser::parse_expr(expr) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Create a program with just this expression
    let program = vais_ast::Program {
        items: vec![vais_ast::Item::Expr(parsed)],
        span: vais_lexer::Span::new(0, expr.len()),
    };

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Execute
    match vais_vm::execute_function(functions, "__main__", vec![]) {
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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
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
    }).collect::<Vec<_>>()).unwrap_or_else(|e| {
        eprintln!("Failed to serialize JSON: {}", e);
        std::process::exit(1);
    });

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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
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
    let args: Vec<vais_ir::Value> = parse_args_json(args_json);

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
    match vais_vm::execute_function(functions, &target_func, args) {
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
fn run_with_jit(functions: Vec<vais_lowering::CompiledFunction>, func_name: &str, args: Vec<vais_ir::Value>, show_stats: bool) {
    use vais_vm::{JitVm, JitConfig};

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

/// Convert serde_json::Value to vais_ir::Value
fn json_to_value(json: serde_json::Value) -> vais_ir::Value {
    match json {
        serde_json::Value::Null => vais_ir::Value::Void,
        serde_json::Value::Bool(b) => vais_ir::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                vais_ir::Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                vais_ir::Value::Float(f)
            } else {
                vais_ir::Value::Void
            }
        }
        serde_json::Value::String(s) => vais_ir::Value::String(s),
        serde_json::Value::Array(arr) => {
            vais_ir::Value::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }
            vais_ir::Value::Map(map)
        }
    }
}

/// Parse JSON array string into Value vector
fn parse_args_json(json: &str) -> Vec<vais_ir::Value> {
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
    use rustyline::error::ReadlineError;
    use rustyline::DefaultEditor;

    println!("Vais REPL v0.1.0");
    println!("Type expressions to evaluate, ':help' for commands, ':quit' to exit.");
    println!("Use Ctrl+D to exit, Up/Down for history.\n");

    // Setup rustyline editor with history
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize readline: {}", e);
            return;
        }
    };

    // Load history from file
    let history_file = dirs::data_dir()
        .map(|d| d.join("vais").join("repl_history"))
        .unwrap_or_else(|| PathBuf::from(".vais_history"));

    if let Some(parent) = history_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = rl.load_history(&history_file);

    // Store defined functions across inputs
    let mut all_functions: Vec<vais_lowering::CompiledFunction> = Vec::new();

    // Multiline input buffer
    let mut multiline_buffer = String::new();
    let mut in_multiline = false;

    loop {
        let prompt = if in_multiline { "....> " } else { "vais> " };

        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim();

                // Check for multiline continuation
                if let Some(stripped) = line.strip_suffix('\\') {
                    multiline_buffer.push_str(stripped);
                    multiline_buffer.push(' ');
                    in_multiline = true;
                    continue;
                }

                // Check for block start (unclosed braces)
                let full_input = if in_multiline {
                    multiline_buffer.push_str(line);
                    let result = multiline_buffer.clone();
                    multiline_buffer.clear();
                    in_multiline = false;
                    result
                } else {
                    line.to_string()
                };

                if full_input.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(&full_input);

                // Handle REPL commands
                if full_input.starts_with(':') {
                    if !handle_repl_command(&full_input, &mut all_functions, &history_file) {
                        break; // :quit was entered
                    }
                    continue;
                }

                // Execute the input
                execute_repl_input(&full_input, &mut all_functions);
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - cancel current input
                if in_multiline {
                    multiline_buffer.clear();
                    in_multiline = false;
                    println!("^C");
                } else {
                    println!("^C (use :quit or Ctrl+D to exit)");
                }
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!("\nGoodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_file);
}

/// Handle REPL commands (returns false if should quit)
fn handle_repl_command(
    input: &str,
    all_functions: &mut Vec<vais_lowering::CompiledFunction>,
    history_file: &Path,
) -> bool {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).copied();

    match cmd {
        ":quit" | ":q" | ":exit" => {
            println!("Goodbye!");
            return false;
        }
        ":help" | ":h" => {
            println!("Commands:");
            println!("  :help, :h         Show this help");
            println!("  :quit, :q         Exit REPL");
            println!("  :list, :l         List defined functions");
            println!("  :clear            Clear all defined functions");
            println!("  :type <expr>      Show type of expression");
            println!("  :ast <expr>       Show AST of expression");
            println!("  :save <file>      Save session to file");
            println!("  :load <file>      Load and execute file");
            println!("  :history          Show command history");
            println!();
            println!("Multiline input:");
            println!("  End a line with \\ to continue on the next line");
            println!();
            println!("Examples:");
            println!("  1 + 2 * 3           Simple arithmetic");
            println!("  add(a, b) = a + b   Define a function");
            println!("  add(3, 4)           Call a function");
            println!("  [1,2,3].@(_ * 2)    Map operation");
            println!("  async fetch() = ... Async function");
        }
        ":list" | ":l" => {
            if all_functions.is_empty() {
                println!("No functions defined.");
            } else {
                println!("Defined functions:");
                for f in all_functions.iter() {
                    if f.name != "__main__" {
                        println!("  {}({})", f.name, f.params.join(", "));
                    }
                }
            }
        }
        ":clear" => {
            all_functions.clear();
            println!("Cleared all functions.");
        }
        ":type" | ":t" => {
            if let Some(expr_str) = arg {
                show_type(expr_str);
            } else {
                println!("Usage: :type <expression>");
            }
        }
        ":ast" => {
            if let Some(expr_str) = arg {
                show_ast(expr_str);
            } else {
                println!("Usage: :ast <expression>");
            }
        }
        ":save" => {
            if let Some(file) = arg {
                save_session(all_functions, file);
            } else {
                println!("Usage: :save <filename>");
            }
        }
        ":load" => {
            if let Some(file) = arg {
                load_file(file, all_functions);
            } else {
                println!("Usage: :load <filename>");
            }
        }
        ":history" => {
            println!("History file: {}", history_file.display());
            if let Ok(content) = fs::read_to_string(history_file) {
                for (i, line) in content.lines().rev().take(20).enumerate() {
                    println!("  {:2}. {}", i + 1, line);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            println!("Type :help for available commands");
        }
    }
    true
}

/// Execute REPL input
fn execute_repl_input(input: &str, all_functions: &mut Vec<vais_lowering::CompiledFunction>) {
    match vais_parser::parse(input) {
        Ok(program) => {
            // Type check first
            if let Err(e) = vais_typeck::check(&program) {
                eprintln!("Type error: {}", e);
                return;
            }

            let mut lowerer = vais_lowering::Lowerer::new();
            match lowerer.lower_program(&program) {
                Ok(mut functions) => {
                    // Separate new function definitions from __main__
                    let mut main_func = None;
                    for func in functions.drain(..) {
                        if func.name == "__main__" {
                            main_func = Some(func);
                        } else {
                            // Update or add function definition
                            let name = func.name.clone();
                            all_functions.retain(|f| f.name != name);
                            all_functions.push(func);
                        }
                    }

                    // If there's a __main__, execute it
                    if let Some(main) = main_func {
                        let mut exec_functions = all_functions.clone();
                        exec_functions.push(main);

                        match vais_vm::execute_function(exec_functions, "__main__", vec![]) {
                            Ok(result) => {
                                if !matches!(result, vais_ir::Value::Void) {
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

/// Show type of expression
fn show_type(expr_str: &str) {
    match vais_parser::parse_expr(expr_str) {
        Ok(expr) => {
            let program = vais_ast::Program {
                items: vec![vais_ast::Item::Expr(expr)],
                span: vais_lexer::Span::new(0, expr_str.len()),
            };

            match vais_typeck::infer_type(&program) {
                Ok(ty) => println!("{}", ty),
                Err(e) => eprintln!("Type error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}

/// Show AST of expression
fn show_ast(expr_str: &str) {
    match vais_parser::parse_expr(expr_str) {
        Ok(expr) => {
            println!("{:#?}", expr);
        }
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
        }
    }
}

/// Save session to file
fn save_session(functions: &[vais_lowering::CompiledFunction], filename: &str) {
    let mut content = String::new();
    content.push_str("// Vais REPL Session\n\n");

    for f in functions {
        if f.name != "__main__" {
            content.push_str(&format!("// Function: {}\n", f.name));
            content.push_str(&format!("{}({}) = ...\n\n", f.name, f.params.join(", ")));
        }
    }

    match fs::write(filename, &content) {
        Ok(_) => println!("Saved session to {}", filename),
        Err(e) => eprintln!("Failed to save: {}", e),
    }
}

/// Load and execute file
fn load_file(filename: &str, all_functions: &mut Vec<vais_lowering::CompiledFunction>) {
    match fs::read_to_string(filename) {
        Ok(source) => {
            match vais_parser::parse(&source) {
                Ok(program) => {
                    let mut lowerer = vais_lowering::Lowerer::new();
                    match lowerer.lower_program(&program) {
                        Ok(functions) => {
                            let count = functions.iter().filter(|f| f.name != "__main__").count();
                            for func in functions {
                                if func.name != "__main__" {
                                    let name = func.name.clone();
                                    all_functions.retain(|f| f.name != name);
                                    all_functions.push(func);
                                }
                            }
                            println!("Loaded {} function(s) from {}", count, filename);
                        }
                        Err(e) => eprintln!("Lowering error: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Parse error: {:?}", e),
            }
        }
        Err(e) => eprintln!("Failed to read file: {}", e),
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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Type check (unless skipped)
    if !no_typecheck {
        if let Err(e) = vais_typeck::check(&program) {
            eprintln!("Type error: {}", e);
            eprintln!("(Use --no-typecheck to skip type checking)");
            std::process::exit(1);
        }
    }

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
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
    functions: &[vais_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate C code
    let c_code = match vais_codegen::generate_c(functions) {
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
    functions: &[vais_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate WAT code
    let wat_code = match vais_codegen::generate_wat(functions) {
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
    functions: &[vais_lowering::CompiledFunction],
) {
    use std::process::Command;

    // Generate LLVM IR code
    let llvm_ir = match vais_codegen::generate_llvm_ir(functions) {
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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Syntax error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Type check (optional)
    if !no_typecheck {
        if let Err(e) = vais_typeck::check(&program) {
            eprintln!("Type error: {:?}", e);
            std::process::exit(1);
        }
    }

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    // JIT compile and execute
    match vais_codegen::jit_execute(&functions) {
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

    // Check if vais.toml already exists
    if project_path.join("vais.toml").exists() {
        eprintln!("Error: vais.toml already exists in {}", project_path.display());
        std::process::exit(1);
    }

    match package::init_project(&project_path, name) {
        Ok(()) => {
            let manifest = package::Manifest::load(&project_path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load manifest: {}", e);
                std::process::exit(1);
            });
            println!("Initialized Vais project '{}'", manifest.package.name);
            println!();
            println!("Created files:");
            println!("  - vais.toml");
            println!("  - src/main.vais");
            println!("  - .gitignore");
            println!();
            println!("Run your project with:");
            println!("  cd {}", project_path.display());
            println!("  vais run src/main.vais");
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
            eprintln!("Error: No vais.toml found in current directory or any parent");
            eprintln!("Run 'vais init' to create a new project");
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
            eprintln!("Error: No vais.toml found in current directory or any parent");
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
            eprintln!("Error: No vais.toml found in current directory or any parent");
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
                    println!("  vais publish");
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
                eprintln!("Error: No vais.toml found in current directory or any parent");
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
            eprintln!("Error: No vais.toml found in {} or any parent", project_path.display());
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
        eprintln!("Bump the version in vais.toml and try again");
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
            let _ = fs::copy(project_root.join("vais.toml"), pkg_path.join("vais.toml"));

            println!("Published {} v{} to local registry", name, version);
            println!();
            println!("Other projects can use it with:");
            println!("  vais add {} --version {}", name, version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_search(query: &str) {
    let registry = match package::Registry::local() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match registry.search(query) {
        Ok(results) => {
            if results.is_empty() {
                println!("No packages found matching '{}'", query);
                return;
            }

            println!("Found {} package(s) matching '{}':\n", results.len(), query);
            for pkg in results {
                let versions_str = if pkg.versions.is_empty() {
                    "no versions".to_string()
                } else {
                    pkg.versions.first().map(|v| v.version.clone()).unwrap_or_default()
                };

                println!("  {} ({})", pkg.name, versions_str);
                if !pkg.description.is_empty() {
                    println!("    {}", pkg.description);
                }
                if !pkg.keywords.is_empty() {
                    println!("    Keywords: {}", pkg.keywords.join(", "));
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_info(package_name: &str) {
    let registry = match package::Registry::local() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Check if package exists
    let versions = match registry.list_versions(package_name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if versions.is_empty() {
        eprintln!("Package '{}' not found in registry", package_name);
        std::process::exit(1);
    }

    println!("Package: {}", package_name);
    println!();

    // Try to load manifest from latest version
    let latest = &versions[0];
    let pkg_path = registry.get_package_path(package_name, latest);

    if let Ok(manifest) = package::Manifest::load(&pkg_path) {
        if !manifest.package.description.is_empty() {
            println!("Description: {}", manifest.package.description);
        }
        if !manifest.package.authors.is_empty() {
            println!("Authors: {}", manifest.package.authors.join(", "));
        }
        if let Some(license) = &manifest.package.license {
            println!("License: {}", license);
        }
        if let Some(repo) = &manifest.package.repository {
            println!("Repository: {}", repo);
        }
        if !manifest.package.keywords.is_empty() {
            println!("Keywords: {}", manifest.package.keywords.join(", "));
        }
        println!();

        if !manifest.dependencies.is_empty() {
            println!("Dependencies:");
            for (name, dep) in &manifest.dependencies {
                match dep {
                    package::Dependency::Version(v) => println!("  {} = \"{}\"", name, v),
                    package::Dependency::Detailed(d) => {
                        println!("  {} = \"{}\"", name, d.version.as_deref().unwrap_or("*"));
                    }
                }
            }
            println!();
        }
    }

    println!("Available versions:");
    for (i, ver) in versions.iter().enumerate() {
        let marker = if i == 0 { " (latest)" } else { "" };
        println!("  {}{}", ver, marker);
    }
}

fn cmd_update(package_name: Option<&str>) {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let project_root = match package::find_project_root(&cwd) {
        Some(p) => p,
        None => {
            eprintln!("Error: No vais.toml found in current directory or any parent");
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

    let registry = match package::Registry::local() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut updated = 0;

    // Update specific package or all
    let deps_to_update: Vec<String> = if let Some(name) = package_name {
        if manifest.dependencies.contains_key(name) || manifest.dev_dependencies.contains_key(name) {
            vec![name.to_string()]
        } else {
            eprintln!("Package '{}' not found in dependencies", name);
            std::process::exit(1);
        }
    } else {
        manifest.dependencies.keys()
            .chain(manifest.dev_dependencies.keys())
            .cloned()
            .collect()
    };

    println!("Checking for updates...\n");

    for name in deps_to_update {
        let is_dev = manifest.dev_dependencies.contains_key(&name);
        let current_dep = if is_dev {
            manifest.dev_dependencies.get(&name)
        } else {
            manifest.dependencies.get(&name)
        };

        let current_version = current_dep.map(|d| match d {
            package::Dependency::Version(v) => v.clone(),
            package::Dependency::Detailed(d) => d.version.clone().unwrap_or_else(|| "*".to_string()),
        }).unwrap_or_else(|| "*".to_string());

        // Get latest version
        if let Ok(Some(latest)) = registry.get_latest_version(&name) {
            if current_version != latest && current_version != "*" {
                println!("  {} {} -> {}", name, current_version, latest);

                // Update manifest
                let new_dep = package::Dependency::Version(latest);
                if is_dev {
                    manifest.dev_dependencies.insert(name, new_dep);
                } else {
                    manifest.dependencies.insert(name, new_dep);
                }
                updated += 1;
            } else {
                println!("  {} {} (up to date)", name, current_version);
            }
        } else {
            println!("  {} {} (not in registry)", name, current_version);
        }
    }

    if updated > 0 {
        if let Err(e) = manifest.save(&project_root) {
            eprintln!("\nError saving manifest: {}", e);
            std::process::exit(1);
        }
        println!("\nUpdated {} package(s)", updated);
    } else {
        println!("\nAll packages are up to date");
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
    let config = vais_tools::formatter::FormatConfig {
        indent_width: indent,
        max_line_width: max_width,
        ..Default::default()
    };
    let mut formatter = vais_tools::Formatter::with_config(config);

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
            eprintln!("Run 'vais format {}' to format it", path.display());
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

fn cmd_debug(path: &PathBuf, func_name: Option<&str>, breakpoints: &[String]) {
    use rustyline::error::ReadlineError;
    use rustyline::DefaultEditor;
    #[allow(unused_imports)]
    use vais_tools::debugger::{Debugger, DebugEvent, DebugState};

    let source = match read_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Parse
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Type check
    if let Err(e) = vais_typeck::check(&program) {
        eprintln!("Type error: {}", e);
        std::process::exit(1);
    }

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
    let functions = match lowerer.lower_program(&program) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Lowering error: {:?}", e);
            std::process::exit(1);
        }
    };

    if functions.is_empty() {
        eprintln!("No functions to debug");
        std::process::exit(1);
    }

    // Initialize debugger
    let mut debugger = Debugger::new();
    debugger.load_functions(functions.clone());

    // Set breakpoints from command line
    for bp_str in breakpoints {
        let parts: Vec<&str> = bp_str.split(':').collect();
        if parts.len() == 2 {
            if let Ok(instr) = parts[1].parse::<usize>() {
                let id = debugger.set_breakpoint(parts[0], instr);
                println!("Breakpoint #{} set at {}:{}", id, parts[0], instr);
            }
        } else {
            eprintln!("Invalid breakpoint format: {} (use function:instruction)", bp_str);
        }
    }

    // Determine entry function
    let entry = func_name
        .map(|s| s.to_string())
        .or_else(|| functions.iter().find(|f| f.name == "__main__").map(|f| f.name.clone()))
        .unwrap_or_else(|| functions[0].name.clone());

    println!("Vais Debugger");
    println!("Debugging: {} -> {}", path.display(), entry);
    println!("Type 'help' for commands\n");

    // Start debug session
    if let Err(e) = debugger.start(&entry, vec![]) {
        eprintln!("Failed to start debugger: {}", e);
        std::process::exit(1);
    }

    // Interactive debug loop
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize readline: {}", e);
            return;
        }
    };

    loop {
        let state = debugger.state();
        let prompt = match state {
            DebugState::NotStarted => "(not started) dbg> ",
            DebugState::Running => "(running) dbg> ",
            DebugState::Paused => "(paused) dbg> ",
            DebugState::Stepping => "(stepping) dbg> ",
            DebugState::Finished => "(finished) dbg> ",
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);
                let parts: Vec<&str> = line.split_whitespace().collect();
                let cmd = parts[0];

                match cmd {
                    "help" | "h" => {
                        println!("Commands:");
                        println!("  step, s           Step to next instruction");
                        println!("  continue, c       Continue to next breakpoint");
                        println!("  into, i           Step into function");
                        println!("  out, o            Step out of function");
                        println!("  break, b <f:n>    Set breakpoint at function:instruction");
                        println!("  delete, d <id>    Delete breakpoint");
                        println!("  list, l           List breakpoints");
                        println!("  locals            Show local variables");
                        println!("  stack             Show call stack");
                        println!("  instr             Show current instruction");
                        println!("  info              Show debug summary");
                        println!("  reset             Reset debugger");
                        println!("  quit, q           Exit debugger");
                    }
                    "step" | "s" => {
                        if let Some(event) = debugger.step() {
                            print_debug_event(&event);
                        }
                    }
                    "continue" | "c" => {
                        if let Some(event) = debugger.continue_execution() {
                            print_debug_event(&event);
                        }
                    }
                    "into" | "i" => {
                        if let Some(event) = debugger.step_into() {
                            print_debug_event(&event);
                        }
                    }
                    "out" | "o" => {
                        if let Some(event) = debugger.step_out() {
                            print_debug_event(&event);
                        }
                    }
                    "break" | "b" => {
                        if parts.len() >= 2 {
                            let bp_parts: Vec<&str> = parts[1].split(':').collect();
                            if bp_parts.len() == 2 {
                                if let Ok(instr) = bp_parts[1].parse::<usize>() {
                                    let id = debugger.set_breakpoint(bp_parts[0], instr);
                                    println!("Breakpoint #{} set at {}:{}", id, bp_parts[0], instr);
                                }
                            } else {
                                println!("Usage: break function:instruction");
                            }
                        } else {
                            println!("Usage: break function:instruction");
                        }
                    }
                    "delete" | "d" => {
                        if parts.len() >= 2 {
                            if let Ok(id) = parts[1].parse::<usize>() {
                                if debugger.remove_breakpoint(id) {
                                    println!("Deleted breakpoint #{}", id);
                                } else {
                                    println!("Breakpoint #{} not found", id);
                                }
                            }
                        } else {
                            println!("Usage: delete <breakpoint-id>");
                        }
                    }
                    "list" | "l" => {
                        let bps = debugger.list_breakpoints();
                        if bps.is_empty() {
                            println!("No breakpoints set");
                        } else {
                            println!("Breakpoints:");
                            for bp in bps {
                                let status = if bp.enabled { "enabled" } else { "disabled" };
                                println!(
                                    "  #{}: {}:{} [{}] hits={}",
                                    bp.id, bp.function, bp.instruction, status, bp.hit_count
                                );
                            }
                        }
                    }
                    "locals" => {
                        if let Some(locals) = debugger.locals() {
                            if locals.is_empty() {
                                println!("No local variables");
                            } else {
                                println!("Local variables:");
                                for (name, value) in locals {
                                    println!("  {} = {}", name, value);
                                }
                            }
                        } else {
                            println!("No active frame");
                        }
                    }
                    "stack" => {
                        let stack = debugger.call_stack();
                        if stack.is_empty() {
                            println!("Call stack empty");
                        } else {
                            println!("Call stack:");
                            for (i, frame) in stack.iter().rev().enumerate() {
                                println!(
                                    "  #{}: {} (ip: {})",
                                    i, frame.function, frame.instruction_pointer
                                );
                            }
                        }
                    }
                    "instr" => {
                        if let Some(instr) = debugger.current_instruction() {
                            println!("Current instruction: {:?}", instr);
                        } else {
                            println!("No current instruction");
                        }
                    }
                    "info" => {
                        println!("{}", debugger.summary());
                    }
                    "reset" => {
                        debugger.reset();
                        if let Err(e) = debugger.start(&entry, vec![]) {
                            eprintln!("Failed to restart: {}", e);
                        } else {
                            println!("Debugger reset, ready to run");
                        }
                    }
                    "quit" | "q" => {
                        println!("Goodbye!");
                        break;
                    }
                    _ => {
                        println!("Unknown command: {}. Type 'help' for commands.", cmd);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("\nGoodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn print_debug_event(event: &vais_tools::debugger::DebugEvent) {
    match event {
        vais_tools::debugger::DebugEvent::BreakpointHit { breakpoint_id, function } => {
            println!("Breakpoint #{} hit at {}", breakpoint_id, function);
        }
        vais_tools::debugger::DebugEvent::StepComplete { function, instruction } => {
            println!("Step complete: {}:{}", function, instruction);
        }
        vais_tools::debugger::DebugEvent::FunctionEnter { name } => {
            println!("Entering function: {}", name);
        }
        vais_tools::debugger::DebugEvent::FunctionExit { name, result } => {
            println!("Exiting function: {} => {}", name, result);
        }
        vais_tools::debugger::DebugEvent::Error { message } => {
            eprintln!("Error: {}", message);
        }
        vais_tools::debugger::DebugEvent::Finished { result } => {
            println!("Execution finished: {}", result);
        }
    }
}

fn cmd_doc(path: &PathBuf, output: Option<&PathBuf>, format: &str) {
    #[allow(unused_imports)]
    use vais_tools::docgen::{DocGenerator, DocFormat};

    // Determine output format
    let doc_format = DocFormat::parse(format).unwrap_or_else(|| {
        eprintln!("Unknown format: {}. Using markdown.", format);
        DocFormat::Markdown
    });

    // Check if path is a file or directory
    if path.is_dir() {
        // Process all .vais files in directory
        let output_dir = output.cloned().unwrap_or_else(|| path.join("docs"));

        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            std::process::exit(1);
        }

        let mut count = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "vais").unwrap_or(false)
                    && generate_doc_for_file(&file_path, &output_dir, doc_format)
                {
                    count += 1;
                }
            }
        }

        if count == 0 {
            println!("No .vais files found in {}", path.display());
        } else {
            println!("Generated documentation for {} file(s) in {}", count, output_dir.display());

            // Generate index file
            generate_index(&output_dir, doc_format);
        }
    } else {
        // Process single file
        let output_dir = output.cloned().unwrap_or_else(|| {
            path.parent()
                .map(|p| p.join("docs"))
                .unwrap_or_else(|| PathBuf::from("docs"))
        });

        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            std::process::exit(1);
        }

        if generate_doc_for_file(path, &output_dir, doc_format) {
            println!("Generated documentation in {}", output_dir.display());
        }
    }
}

fn generate_doc_for_file(
    path: &Path,
    output_dir: &Path,
    format: vais_tools::docgen::DocFormat,
) -> bool {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path.display(), e);
            return false;
        }
    };

    // Parse
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error in {}: {:?}", path.display(), e);
            return false;
        }
    };

    // Generate documentation
    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");

    let generator = vais_tools::DocGenerator::new(format);
    let doc_content = generator.generate(&program, module_name);

    // Write output file
    let output_file = output_dir.join(format!("{}.{}", module_name, format.extension()));

    if let Err(e) = fs::write(&output_file, &doc_content) {
        eprintln!("Failed to write {}: {}", output_file.display(), e);
        return false;
    }

    println!("  {} -> {}", path.display(), output_file.display());
    true
}

fn generate_index(output_dir: &Path, format: vais_tools::docgen::DocFormat) {
    use vais_tools::docgen::DocFormat;

    let ext = format.extension();
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == ext).unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if name != "index" {
                        files.push(name.to_string());
                    }
                }
            }
        }
    }

    files.sort();

    let index_content = match format {
        DocFormat::Markdown => {
            let mut content = String::from("# Vais Documentation Index\n\n");
            content.push_str("## Modules\n\n");
            for name in &files {
                content.push_str(&format!("- [{}](./{}.{})\n", name, name, ext));
            }
            content
        }
        DocFormat::Html => {
            let mut content = String::from("<!DOCTYPE html>\n<html><head><title>Vais Documentation</title>");
            content.push_str("<style>body{font-family:sans-serif;max-width:900px;margin:0 auto;padding:2rem;}</style>");
            content.push_str("</head><body>\n<h1>Vais Documentation Index</h1>\n<h2>Modules</h2>\n<ul>\n");
            for name in &files {
                content.push_str(&format!("<li><a href=\"{}.{}\">{}</a></li>\n", name, ext, name));
            }
            content.push_str("</ul>\n</body></html>");
            content
        }
        DocFormat::Json => {
            let modules_json: Vec<String> = files
                .iter()
                .map(|n| format!("\"{}\"", n))
                .collect();
            format!("{{\"modules\": [{}]}}", modules_json.join(", "))
        }
    };

    let index_file = output_dir.join(format!("index.{}", ext));
    let _ = fs::write(&index_file, &index_content);
    println!("  Generated index: {}", index_file.display());
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
    let program = match vais_parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Resolve modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let program = match vais_parser::resolve_modules(program, base_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Module error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Lower to IR
    let mut lowerer = vais_lowering::Lowerer::new();
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
    let mut profiler = vais_tools::Profiler::new();
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

/// Test runner for Vais files
fn cmd_test(path: Option<&PathBuf>, filter: Option<&str>, verbose: bool) {
    let target_path = path.cloned().unwrap_or_else(|| PathBuf::from("."));

    // Collect .vais files
    let mut vais_files = Vec::new();
    if target_path.is_file() {
        vais_files.push(target_path.clone());
    } else if target_path.is_dir() {
        collect_vais_files(&target_path, &mut vais_files);
    } else {
        eprintln!("Path not found: {}", target_path.display());
        std::process::exit(1);
    }

    if vais_files.is_empty() {
        println!("No .vais files found");
        return;
    }

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut test_results: Vec<(String, String, bool, Option<String>)> = Vec::new();

    for file_path in &vais_files {
        let source = match read_file(file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path.display(), e);
                continue;
            }
        };

        // Parse
        let program = match vais_parser::parse(&source) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Parse error in {}: {:?}", file_path.display(), e);
                continue;
            }
        };

        // Find test functions
        let test_funcs: Vec<_> = program.items.iter()
            .filter_map(|item| {
                if let vais_ast::Item::Function(func) = item {
                    if func.is_test {
                        // Apply filter if provided
                        if let Some(f) = filter {
                            if !func.name.contains(f) {
                                return None;
                            }
                        }
                        return Some(func.clone());
                    }
                }
                None
            })
            .collect();

        if test_funcs.is_empty() {
            continue;
        }

        // Resolve modules
        let base_dir = file_path.parent().unwrap_or(std::path::Path::new("."));
        let program = match vais_parser::resolve_modules(program, base_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Module error in {}: {:?}", file_path.display(), e);
                continue;
            }
        };

        // Lower to IR
        let mut lowerer = vais_lowering::Lowerer::new();
        let functions = match lowerer.lower_program(&program) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Lowering error in {}: {:?}", file_path.display(), e);
                continue;
            }
        };

        // Create VM
        let mut vm = vais_vm::Vm::new();
        vm.load_functions(functions);

        // Run test functions
        for test_func in &test_funcs {
            total_tests += 1;
            let file_name = file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if verbose {
                print!("  Running {}::{} ... ", file_name, test_func.name);
            }

            match vm.call_function(&test_func.name, vec![]) {
                Ok(result) => {
                    // Test passes if result is truthy or is Void (no assertion failure)
                    let passed = match &result {
                        vais_ir::Value::Bool(false) => false,
                        vais_ir::Value::Error(e) => {
                            test_results.push((file_name.to_string(), test_func.name.clone(), false, Some(e.clone())));
                            failed_tests += 1;
                            if verbose {
                                println!("FAILED ({})", e);
                            }
                            continue;
                        }
                        _ => true,
                    };

                    if passed {
                        passed_tests += 1;
                        test_results.push((file_name.to_string(), test_func.name.clone(), true, None));
                        if verbose {
                            println!("ok");
                        }
                    } else {
                        failed_tests += 1;
                        test_results.push((file_name.to_string(), test_func.name.clone(), false, Some("returned false".to_string())));
                        if verbose {
                            println!("FAILED (returned false)");
                        }
                    }
                }
                Err(e) => {
                    failed_tests += 1;
                    test_results.push((file_name.to_string(), test_func.name.clone(), false, Some(format!("{:?}", e))));
                    if verbose {
                        println!("FAILED ({:?})", e);
                    }
                }
            }
        }
    }

    // Print summary
    println!();
    println!("test result: {}. {} passed; {} failed; {} total",
        if failed_tests == 0 { "ok" } else { "FAILED" },
        passed_tests,
        failed_tests,
        total_tests
    );

    // Print failed tests details
    if !verbose && failed_tests > 0 {
        println!();
        println!("failures:");
        for (file, test, passed, err) in &test_results {
            if !passed {
                println!("  {}::{}", file, test);
                if let Some(e) = err {
                    println!("    error: {}", e);
                }
            }
        }
    }

    if failed_tests > 0 {
        std::process::exit(1);
    }
}

/// Recursively collect .vais files
fn collect_vais_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_vais_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("vais") {
                files.push(path);
            }
        }
    }
}
