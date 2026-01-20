//! Interactive REPL for Vais
//!
//! Read-Eval-Print-Loop for interactive Vais development.

use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use vais_codegen::CodeGenerator;
use vais_parser::parse;
use vais_types::TypeChecker;

/// Start the interactive REPL
pub fn run() -> Result<(), String> {
    println!("{}", "Vais REPL v0.0.1".bold().cyan());
    println!("Type expressions to evaluate, or :help for commands");
    println!();

    let mut history: Vec<String> = Vec::new();
    let mut definitions: Vec<String> = Vec::new();

    loop {
        // Print prompt
        print!("{} ", "vais>".green().bold());
        let _ = io::stdout().flush();

        // Read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle commands
        if input.starts_with(':') {
            if handle_command(input, &mut definitions, &history) {
                break;
            }
            continue;
        }

        // Add to history
        history.push(input.to_string());

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

    Ok(())
}

/// Handle REPL commands (returns true if should exit)
fn handle_command(input: &str, definitions: &mut Vec<String>, history: &[String]) -> bool {
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
