//! Interactive REPL for Vais
//!
//! Read-Eval-Print-Loop for interactive Vais development.

use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::History;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};
use std::fs;
use std::process::Command;

use vais_codegen::CodeGenerator;
use vais_parser::parse;
use vais_types::TypeChecker;

/// REPL helper with completion, validation, and highlighting
struct ReplHelper {
    keywords: Vec<String>,
    builtins: Vec<String>,
}

impl ReplHelper {
    fn new() -> Self {
        let keywords = vec![
            // Single-letter keywords
            "F", "S", "E", "I", "L", "M", "W", "X", "A", "R", "B", "C", "T", "U", "P",
            // Common keywords
            "mut", "self", "Self", "true", "false", "spawn", "await", "weak", "clone",
            // Primitive types
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
            "f32", "f64", "bool", "str", "char",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        let builtins = vec![
            // I/O functions
            "printf", "putchar", "puts", "puts_ptr", "exit",
            // Memory functions
            "malloc", "free", "memcpy", "load_byte", "store_byte", "load_i64", "store_i64",
            // File functions
            "fopen", "fclose", "fread", "fwrite", "fgetc", "fputc", "fgets", "fputs",
            "fseek", "ftell", "fflush", "feof",
            // String functions
            "strlen", "strcmp", "strncmp",
            // Async functions
            "usleep", "sched_yield",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        ReplHelper { keywords, builtins }
    }
}

impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &line[start..pos];
        if prefix.is_empty() {
            return Ok((pos, Vec::new()));
        }

        let mut candidates = Vec::new();

        // Complete keywords
        for keyword in &self.keywords {
            if keyword.starts_with(prefix) {
                candidates.push(Pair {
                    display: keyword.clone(),
                    replacement: keyword.clone(),
                });
            }
        }

        // Complete built-in functions
        for builtin in &self.builtins {
            if builtin.starts_with(prefix) {
                candidates.push(Pair {
                    display: format!("{}()", builtin),
                    replacement: format!("{}(", builtin),
                });
            }
        }

        Ok((start, candidates))
    }
}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Highlighter for ReplHelper {}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        // Count opening and closing braces
        let open_braces = input.chars().filter(|&c| c == '{').count();
        let close_braces = input.chars().filter(|&c| c == '}').count();

        // Count opening and closing parentheses
        let open_parens = input.chars().filter(|&c| c == '(').count();
        let close_parens = input.chars().filter(|&c| c == ')').count();

        // If braces or parens are unbalanced, input is incomplete
        if open_braces > close_braces || open_parens > close_parens {
            return Ok(ValidationResult::Incomplete);
        }

        Ok(ValidationResult::Valid(None))
    }
}

/// Start the interactive REPL
pub fn run() -> Result<(), String> {
    println!("{}", "Vais REPL v0.0.1".bold().cyan());
    println!("Type expressions to evaluate, or :help for commands");
    println!();

    let helper = ReplHelper::new();
    let mut rl = Editor::new().map_err(|e| format!("Failed to create editor: {}", e))?;
    rl.set_helper(Some(helper));

    // Load history from file (if it exists)
    let history_path = std::env::temp_dir().join(".vais_repl_history");
    let _ = rl.load_history(&history_path);

    let mut definitions: Vec<String> = Vec::new();

    loop {
        // Read input with readline
        let readline = rl.readline("vais> ");

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(input);

                // Handle commands
                if input.starts_with(':') {
                    if handle_command(input, &mut definitions, &rl) {
                        break;
                    }
                    continue;
                }

                // Check if this is a definition (starts with F, S, E, W, X, T)
                let is_definition = input.starts_with("F ")
                    || input.starts_with("S ")
                    || input.starts_with("E ")
                    || input.starts_with("W ")
                    || input.starts_with("X ")
                    || input.starts_with("T ");

                if is_definition {
                    handle_definition(input, &mut definitions);
                } else {
                    handle_expression(input, &definitions);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red().bold(), err);
                break;
            }
        }
    }

    // Save history to file
    let _ = rl.save_history(&history_path);

    Ok(())
}

/// Handle REPL commands (returns true if should exit)
fn handle_command(
    input: &str,
    definitions: &mut Vec<String>,
    rl: &Editor<ReplHelper, rustyline::history::DefaultHistory>,
) -> bool {
    match input {
        ":quit" | ":q" | ":exit" => {
            println!("Goodbye!");
            return true;
        }
        ":help" | ":h" => {
            println!("{}", "Commands:".bold());
            println!("  :help, :h     Show this help");
            println!("  :quit, :q     Exit the REPL");
            println!("  :clear        Clear definitions");
            println!("  :defs         Show current definitions");
            println!("  :history      Show input history");
            println!();
            println!("{}", "Features:".bold());
            println!("  - Multiline input: Unclosed braces/parens continue to next line");
            println!("  - History: Use up/down arrows to navigate (max 100 entries)");
            println!("  - Tab completion: Press Tab for keyword/function suggestions");
            println!();
            println!("{}", "Examples:".bold());
            println!("  F add(a:i64,b:i64)->i64=a+b    Define a function");
            println!("  add(2, 3)                       Call a function");
            println!("  1 + 2 * 3                       Evaluate expression");
        }
        ":clear" => {
            definitions.clear();
            println!("Definitions cleared");
        }
        ":defs" => {
            if definitions.is_empty() {
                println!("No definitions");
            } else {
                println!("{}", "Current definitions:".bold());
                for def in definitions.iter() {
                    println!("  {}", def);
                }
            }
        }
        ":history" => {
            let history = rl.history();
            if history.is_empty() {
                println!("No history");
            } else {
                for (i, item) in history.iter().enumerate() {
                    println!("{:3}  {}", i + 1, item);
                }
            }
        }
        _ => {
            println!("{} Unknown command: {}", "Error:".red().bold(), input);
        }
    }
    false
}

/// Handle a definition (F, S, E, W, X, T)
fn handle_definition(input: &str, definitions: &mut Vec<String>) {
    match parse(input) {
        Ok(ast) => {
            let mut checker = TypeChecker::new();
            // First register all previous definitions
            for def in definitions.iter() {
                if let Ok(prev_ast) = parse(def) {
                    let _ = checker.check_module(&prev_ast);
                }
            }
            // Then check the new definition
            match checker.check_module(&ast) {
                Ok(_) => {
                    definitions.push(input.to_string());
                    println!("{}", "Defined".green());
                }
                Err(e) => {
                    println!("{} {}", "Type error:".red().bold(), e);
                }
            }
        }
        Err(e) => {
            println!("{} {}", "Parse error:".red().bold(), e);
        }
    }
}

/// Handle an expression evaluation
fn handle_expression(input: &str, definitions: &[String]) {
    let mut source = String::new();
    for def in definitions {
        source.push_str(def);
        source.push('\n');
    }
    source.push_str(&format!("F __repl_main()->i64{{{}}}", input));

    match evaluate_expr(&source) {
        Ok(result) => {
            println!("{} {}", "=>".cyan(), result);
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
        }
    }
}

/// Evaluate a REPL expression by compiling and running it
fn evaluate_expr(source: &str) -> Result<String, String> {
    // Parse
    let ast = parse(source).map_err(|e| format!("Parse error: {}", e))?;

    // Type check
    let mut checker = TypeChecker::new();
    checker
        .check_module(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    // Generate IR
    let mut codegen = CodeGenerator::new("repl");
    let ir = codegen
        .generate_module(&ast)
        .map_err(|e| format!("Codegen error: {}", e))?;

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let ir_path = temp_dir.join("vais_repl.ll");
    let bin_path = temp_dir.join("vais_repl");

    fs::write(&ir_path, &ir).map_err(|e| format!("Cannot write temp file: {}", e))?;

    // Compile with clang
    let bin_path_str = bin_path.to_str()
        .ok_or("Invalid UTF-8 in binary path")?;
    let ir_path_str = ir_path.to_str()
        .ok_or("Invalid UTF-8 in IR path")?;

    let status = Command::new("clang")
        .args([
            "-O0",
            "-Wno-override-module",
            "-o",
            bin_path_str,
            ir_path_str,
        ])
        .output()
        .map_err(|_| "clang not found")?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(format!("Compilation failed: {}", stderr));
    }

    // Run and capture output
    let output = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("Cannot run: {}", e))?;

    // The result is the exit code (for simple integer expressions)
    let exit_code = output.status.code().unwrap_or(0);

    // Also capture stdout if any
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.is_empty() {
        Ok(format!("{}", exit_code))
    } else {
        Ok(format!("{}\nReturn: {}", stdout.trim(), exit_code))
    }
}
