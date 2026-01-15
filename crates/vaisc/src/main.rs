//! Vais Compiler CLI
//!
//! The `vaisc` command compiles Vais source files to LLVM IR or native binaries.

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};

use vais_codegen::CodeGenerator;
use vais_lexer::tokenize;
use vais_parser::parse;
use vais_types::TypeChecker;

#[derive(Parser)]
#[command(name = "vaisc")]
#[command(author = "Vais Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Vais compiler - AI-optimized systems programming language")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input source file (.vais)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (default: input with .ll extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Emit LLVM IR only (don't compile to binary)
    #[arg(long)]
    emit_ir: bool,

    /// Show tokens (lexer output)
    #[arg(long)]
    show_tokens: bool,

    /// Show AST (parser output)
    #[arg(long)]
    show_ast: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Vais source file
    Build {
        /// Input source file
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Emit LLVM IR only
        #[arg(long)]
        emit_ir: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,
    },

    /// Run a Vais source file
    Run {
        /// Input source file
        input: PathBuf,

        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Check a Vais source file for errors
    Check {
        /// Input source file
        input: PathBuf,
    },

    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Build { input, output, emit_ir, opt_level }) => {
            cmd_build(&input, output, emit_ir, opt_level, cli.verbose)
        }
        Some(Commands::Run { input, args }) => {
            cmd_run(&input, &args, cli.verbose)
        }
        Some(Commands::Check { input }) => {
            cmd_check(&input, cli.verbose)
        }
        Some(Commands::Version) => {
            println!("{} {}", "vaisc".bold(), env!("CARGO_PKG_VERSION"));
            println!("Vais 2.0 - AI-optimized systems programming language");
            Ok(())
        }
        None => {
            // Direct file compilation
            if let Some(input) = cli.input {
                cmd_build(&input, cli.output, cli.emit_ir, 0, cli.verbose)
            } else {
                println!("{}", "Usage: vaisc <FILE.vais> or vaisc build <FILE.vais>".yellow());
                println!("Run 'vaisc --help' for more information.");
                Ok(())
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "error".red().bold(), e);
        exit(1);
    }
}

fn cmd_build(
    input: &PathBuf,
    output: Option<PathBuf>,
    emit_ir: bool,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {}", "Compiling".green().bold(), input.display());
    }

    // Tokenize
    let tokens = tokenize(&source)
        .map_err(|e| format!("Lexer error: {}", e))?;

    if verbose {
        println!("  {} tokens", tokens.len());
    }

    // Parse
    let ast = parse(&source)
        .map_err(|e| format!("Parser error: {}", e))?;

    if verbose {
        println!("  {} items", ast.items.len());
    }

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    if verbose {
        println!("  {}", "Type check passed".green());
    }

    // Generate LLVM IR
    let module_name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = CodeGenerator::new(module_name);
    let ir = codegen.generate_module(&ast)
        .map_err(|e| format!("Codegen error: {}", e))?;

    // Determine output path
    let ir_path = output.clone().unwrap_or_else(|| {
        input.with_extension("ll")
    });

    // Write IR
    fs::write(&ir_path, &ir)
        .map_err(|e| format!("Cannot write '{}': {}", ir_path.display(), e))?;

    if verbose || emit_ir {
        println!("{} {}", "Wrote".green().bold(), ir_path.display());
    }

    // If not emit_ir only, compile to binary
    if !emit_ir {
        let bin_path = output.unwrap_or_else(|| {
            input.with_extension("")
        });

        compile_ir_to_binary(&ir_path, &bin_path, opt_level, verbose)?;
    }

    Ok(())
}

fn compile_ir_to_binary(
    ir_path: &PathBuf,
    bin_path: &PathBuf,
    opt_level: u8,
    verbose: bool,
) -> Result<(), String> {
    // Try clang first, then llc + ld
    let opt_flag = format!("-O{}", opt_level.min(3));

    let status = Command::new("clang")
        .args([
            &opt_flag,
            "-Wno-override-module", // Suppress warning when clang sets target triple
            "-o", bin_path.to_str().unwrap(),
            ir_path.to_str().unwrap(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            if verbose {
                println!("{} {}", "Compiled".green().bold(), bin_path.display());
            } else {
                println!("{}", bin_path.display());
            }
            Ok(())
        }
        Ok(s) => {
            Err(format!("clang exited with code {}", s.code().unwrap_or(-1)))
        }
        Err(_) => {
            Err("clang not found. Install LLVM/clang or use --emit-ir to output LLVM IR only.".to_string())
        }
    }
}

fn cmd_run(input: &PathBuf, args: &[String], verbose: bool) -> Result<(), String> {
    // Build first
    let bin_path = input.with_extension("");
    cmd_build(input, Some(bin_path.clone()), false, 0, verbose)?;

    // Run the binary
    if verbose {
        println!("{} {}", "Running".green().bold(), bin_path.display());
    }

    let status = Command::new(&bin_path)
        .args(args)
        .status()
        .map_err(|e| format!("Cannot run '{}': {}", bin_path.display(), e))?;

    if !status.success() {
        return Err(format!("Program exited with code {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}

fn cmd_check(input: &PathBuf, verbose: bool) -> Result<(), String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", input.display(), e))?;

    if verbose {
        println!("{} {}", "Checking".green().bold(), input.display());
    }

    // Tokenize
    let _tokens = tokenize(&source)
        .map_err(|e| format!("Lexer error: {}", e))?;

    // Parse
    let ast = parse(&source)
        .map_err(|e| format!("Parser error: {}", e))?;

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    println!("{} No errors found", "OK".green().bold());
    Ok(())
}
