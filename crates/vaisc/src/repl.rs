//! Interactive REPL for Vais
//!
//! Read-Eval-Print-Loop for interactive Vais development.
//! Supports JIT compilation when the `jit` feature is enabled.

use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::History;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};
#[cfg(not(feature = "jit"))]
use std::fs;
#[cfg(not(feature = "jit"))]
use std::process::Command;
#[cfg(not(feature = "jit"))]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(not(feature = "jit"))]
use vais_codegen::CodeGenerator;
use vais_parser::parse;
use vais_types::TypeChecker;

#[cfg(feature = "jit")]
use std::collections::HashMap;
#[cfg(feature = "jit")]
use vais_jit::JitCompiler;

/// Atomic counter for generating unique temporary file names (TOCTOU mitigation)
#[cfg(not(feature = "jit"))]
static REPL_COUNTER: AtomicU32 = AtomicU32::new(0);

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
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
            "bool", "str", "char",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        let builtins = vec![
            // I/O functions
            "printf",
            "putchar",
            "puts",
            "puts_ptr",
            "exit",
            // Memory functions
            "malloc",
            "free",
            "memcpy",
            "load_byte",
            "store_byte",
            "load_i64",
            "store_i64",
            // File functions
            "fopen",
            "fclose",
            "fread",
            "fwrite",
            "fgetc",
            "fputc",
            "fgets",
            "fputs",
            "fseek",
            "ftell",
            "fflush",
            "feof",
            // String functions
            "strlen",
            "strcmp",
            "strncmp",
            // Async functions
            "usleep",
            "sched_yield",
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

/// REPL state for JIT mode (execution tracking and cache management)
#[cfg(feature = "jit")]
struct ReplState {
    /// Function execution counts
    function_counts: HashMap<String, u64>,
    /// Total expression evaluations
    expression_count: u64,
    /// Cache dirty flag (true if definitions changed)
    cache_dirty: bool,
}

#[cfg(feature = "jit")]
impl ReplState {
    fn new() -> Self {
        ReplState {
            function_counts: HashMap::new(),
            expression_count: 0,
            cache_dirty: false,
        }
    }

    fn clear(&mut self) {
        self.function_counts.clear();
        self.expression_count = 0;
        self.cache_dirty = true;
    }

    fn mark_dirty(&mut self) {
        self.cache_dirty = true;
    }

    fn increment_expression(&mut self) {
        self.expression_count += 1;
    }
}

/// Start the interactive REPL
pub fn run() -> Result<(), String> {
    #[cfg(feature = "jit")]
    println!("{}", "Vais REPL v0.0.1 (JIT enabled)".bold().cyan());
    #[cfg(not(feature = "jit"))]
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

    #[cfg(feature = "jit")]
    let mut jit = JitCompiler::new().map_err(|e| format!("Failed to initialize JIT: {}", e))?;

    #[cfg(feature = "jit")]
    let mut state = ReplState::new();

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
                    #[cfg(feature = "jit")]
                    if handle_command_jit(input, &mut definitions, &rl, &mut jit, &mut state) {
                        break;
                    }
                    #[cfg(not(feature = "jit"))]
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
                    #[cfg(feature = "jit")]
                    handle_definition_jit(input, &mut definitions, &mut state);
                    #[cfg(not(feature = "jit"))]
                    handle_definition(input, &mut definitions);
                } else {
                    #[cfg(feature = "jit")]
                    handle_expression_jit(input, &definitions, &mut jit, &mut state);
                    #[cfg(not(feature = "jit"))]
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
#[cfg(not(feature = "jit"))]
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
            println!("  :help, :h       Show this help");
            println!("  :quit, :q       Exit the REPL");
            println!("  :clear          Clear definitions");
            println!("  :defs           Show current definitions");
            println!("  :history        Show input history");
            println!("  :type <expr>    Show type of expression");
            println!("  :disasm <expr>  Show LLVM IR for expression");
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
            println!("  :type 1 + 2                     Show type (i64)");
            println!("  :disasm add(1, 2)               Show LLVM IR");
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
        _ if input.starts_with(":type ") => {
            let expr = input.strip_prefix(":type ").unwrap().trim();
            handle_type_command(expr, definitions);
        }
        _ if input.starts_with(":disasm ") => {
            let expr = input.strip_prefix(":disasm ").unwrap().trim();
            handle_disasm_command(expr, definitions);
        }
        _ => {
            println!("{} Unknown command: {}", "Error:".red().bold(), input);
        }
    }
    false
}

/// Handle a definition (F, S, E, W, X, T)
#[cfg(not(feature = "jit"))]
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

/// Handle :type command - show the type of an expression
fn handle_type_command(expr: &str, definitions: &[String]) {
    // Build source with a wrapper function to infer the type
    let mut source = String::new();
    for def in definitions {
        source.push_str(def);
        source.push('\n');
    }
    source.push_str(&format!("F __repl_type_check()->_={{{}}}", expr));

    match parse(&source) {
        Ok(ast) => {
            let mut checker = TypeChecker::new();
            match checker.check_module(&ast) {
                Ok(_) => {
                    // Get the return type of __repl_type_check
                    if let Some(sig) = checker.get_function("__repl_type_check") {
                        println!("{} {}", "Type:".cyan().bold(), format_type(&sig.ret));
                    } else {
                        println!("{} Could not determine type", "Error:".red().bold());
                    }
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

/// Handle :disasm command - show LLVM IR for an expression
#[cfg(not(feature = "jit"))]
fn handle_disasm_command(expr: &str, definitions: &[String]) {
    let mut source = String::new();
    for def in definitions {
        source.push_str(def);
        source.push('\n');
    }
    source.push_str(&format!("F __repl_disasm()->i64{{{}}}", expr));

    match parse(&source) {
        Ok(ast) => {
            let mut checker = TypeChecker::new();
            match checker.check_module(&ast) {
                Ok(_) => {
                    let mut codegen = CodeGenerator::new("repl_disasm");
                    match codegen.generate_module(&ast) {
                        Ok(ir) => {
                            // Extract just the __repl_disasm function
                            println!("{}", "LLVM IR:".cyan().bold());
                            let mut in_func = false;
                            for line in ir.lines() {
                                if line.contains("define") && line.contains("@__repl_disasm") {
                                    in_func = true;
                                }
                                if in_func {
                                    println!("{}", line);
                                    if line.trim() == "}" {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("{} {}", "Codegen error:".red().bold(), e);
                        }
                    }
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

/// Format a ResolvedType for display
fn format_type(ty: &vais_types::ResolvedType) -> String {
    use vais_types::ResolvedType::*;
    match ty {
        I8 => "i8".to_string(),
        I16 => "i16".to_string(),
        I32 => "i32".to_string(),
        I64 => "i64".to_string(),
        I128 => "i128".to_string(),
        U8 => "u8".to_string(),
        U16 => "u16".to_string(),
        U32 => "u32".to_string(),
        U64 => "u64".to_string(),
        U128 => "u128".to_string(),
        F32 => "f32".to_string(),
        F64 => "f64".to_string(),
        Bool => "bool".to_string(),
        Str => "str".to_string(),
        Unit => "()".to_string(),
        Never => "!".to_string(),
        Pointer(inner) => format!("*{}", format_type(inner)),
        Ref(inner) => format!("&{}", format_type(inner)),
        RefMut(inner) => format!("&mut {}", format_type(inner)),
        Slice(inner) => format!("&[{}]", format_type(inner)),
        SliceMut(inner) => format!("&mut [{}]", format_type(inner)),
        Array(inner) => format!("[{}]", format_type(inner)),
        ConstArray { element, size } => format!("[{}; {:?}]", format_type(element), size),
        Optional(inner) => format!("Option<{}>", format_type(inner)),
        Result(ok, err) => format!("Result<{}, {}>", format_type(ok), format_type(err)),
        Map(key, val) => format!("Map<{}, {}>", format_type(key), format_type(val)),
        Range(inner) => format!("Range<{}>", format_type(inner)),
        Future(inner) => format!("Future<{}>", format_type(inner)),
        Named { name, generics } if generics.is_empty() => name.clone(),
        Named { name, generics } => {
            let args: Vec<_> = generics.iter().map(format_type).collect();
            format!("{}<{}>", name, args.join(", "))
        }
        Fn { params, ret, .. } => {
            let param_strs: Vec<_> = params.iter().map(format_type).collect();
            format!("fn({}) -> {}", param_strs.join(", "), format_type(ret))
        }
        FnPtr {
            params,
            ret,
            is_vararg,
            ..
        } => {
            let param_strs: Vec<_> = params.iter().map(format_type).collect();
            let vararg = if *is_vararg { ", ..." } else { "" };
            format!(
                "fn({}{}) -> {}",
                param_strs.join(", "),
                vararg,
                format_type(ret)
            )
        }
        Generic(name) => name.clone(),
        ConstGeneric(name) => format!("const {}", name),
        Tuple(elems) => {
            let elem_strs: Vec<_> = elems.iter().map(format_type).collect();
            format!("({})", elem_strs.join(", "))
        }
        Vector { element, lanes } => format!("<{} x {}>", lanes, format_type(element)),
        DynTrait {
            trait_name,
            generics,
        } if generics.is_empty() => format!("dyn {}", trait_name),
        DynTrait {
            trait_name,
            generics,
        } => {
            let args: Vec<_> = generics.iter().map(format_type).collect();
            format!("dyn {}<{}>", trait_name, args.join(", "))
        }
        Var(id) => format!("?{}", id),
        Unknown => "unknown".to_string(),
        Associated {
            base,
            trait_name,
            assoc_name,
            generics,
        } => {
            let base_str = if let Some(tn) = trait_name {
                format!("<{} as {}>::{}", format_type(base), tn, assoc_name)
            } else {
                format!("{}::{}", format_type(base), assoc_name)
            };
            // Add GAT generic arguments if present
            if generics.is_empty() {
                base_str
            } else {
                let gen_strs: Vec<String> = generics.iter().map(format_type).collect();
                format!("{}<{}>", base_str, gen_strs.join(", "))
            }
        }
        Linear(inner) => format!("linear {}", format_type(inner)),
        Affine(inner) => format!("affine {}", format_type(inner)),
        Dependent {
            var_name,
            base,
            predicate,
        } => {
            format!("{{{}: {} | {}}}", var_name, format_type(base), predicate)
        }
        RefLifetime { lifetime, inner } => format!("&'{} {}", lifetime, format_type(inner)),
        RefMutLifetime { lifetime, inner } => format!("&'{} mut {}", lifetime, format_type(inner)),
        Lifetime(name) => format!("'{}", name),
        Lazy(inner) => format!("Lazy<{}>", format_type(inner)),
        ImplTrait { bounds } => format!("impl {}", bounds.join(" + ")),
        HigherKinded { name, arity } => {
            let holes = (0..*arity).map(|_| "_").collect::<Vec<_>>().join(", ");
            format!("{}<{}>", name, holes)
        }
    }
}

/// Handle an expression evaluation (clang-based, used when JIT is not enabled)
#[cfg(not(feature = "jit"))]
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

/// Evaluate a REPL expression by compiling and running it (clang-based)
#[cfg(not(feature = "jit"))]
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

    // Write to temp file (atomic counter + PID for uniqueness, TOCTOU mitigation)
    let counter = REPL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_dir = std::env::temp_dir();
    let ir_path = temp_dir.join(format!("vais_repl_{}_{}.ll", std::process::id(), counter));
    let bin_path = temp_dir.join(format!("vais_repl_{}_{}", std::process::id(), counter));

    fs::write(&ir_path, &ir).map_err(|e| format!("Cannot write temp file: {}", e))?;

    // Compile with clang
    let bin_path_str = bin_path.to_str().ok_or("Invalid UTF-8 in binary path")?;
    let ir_path_str = ir_path.to_str().ok_or("Invalid UTF-8 in IR path")?;

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

// ============== JIT-enabled REPL functions ==============

/// Handle REPL commands (JIT version) - returns true if should exit
#[cfg(feature = "jit")]
fn handle_command_jit(
    input: &str,
    definitions: &mut Vec<String>,
    rl: &Editor<ReplHelper, rustyline::history::DefaultHistory>,
    jit: &mut JitCompiler,
    state: &mut ReplState,
) -> bool {
    match input {
        ":quit" | ":q" | ":exit" => {
            println!("Goodbye!");
            return true;
        }
        ":help" | ":h" => {
            println!("{}", "Commands:".bold());
            println!("  :help, :h         Show this help");
            println!("  :quit, :q         Exit the REPL");
            println!("  :clear            Clear definitions and reset JIT");
            println!("  :defs             Show current definitions");
            println!("  :history          Show input history");
            println!("  :type <expr>      Show type of expression");
            println!("  :profile          Show function execution statistics");
            println!("  :jit-stats        Show JIT engine status");
            println!("  :tier <func>      Show optimization tier for function");
            println!();
            println!("{}", "Features:".bold());
            println!("  - JIT compilation (Cranelift backend)");
            println!("  - Multiline input: Unclosed braces/parens continue to next line");
            println!("  - History: Use up/down arrows to navigate (max 100 entries)");
            println!("  - Tab completion: Press Tab for keyword/function suggestions");
            println!();
            println!("{}", "Examples:".bold());
            println!("  F add(a:i64,b:i64)->i64{{a+b}}    Define a function");
            println!("  add(2, 3)                       Call a function");
            println!("  1 + 2 * 3                       Evaluate expression");
            println!("  :type 1 + 2                     Show type (i64)");
            println!("  :profile                        Show execution stats");
        }
        ":clear" => {
            definitions.clear();
            state.clear();
            if let Err(e) = jit.clear() {
                println!("{} Failed to reset JIT: {}", "Warning:".yellow().bold(), e);
            }
            println!("Definitions and JIT state cleared");
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
        ":profile" => {
            handle_profile_command(state, definitions);
        }
        ":jit-stats" => {
            handle_jit_stats_command(state, definitions);
        }
        _ if input.starts_with(":tier ") => {
            let func_name = input.strip_prefix(":tier ").unwrap().trim();
            handle_tier_command(func_name, definitions);
        }
        _ if input.starts_with(":type ") => {
            let expr = input.strip_prefix(":type ").unwrap().trim();
            handle_type_command(expr, definitions);
        }
        _ => {
            println!("{} Unknown command: {}", "Error:".red().bold(), input);
        }
    }
    false
}

/// Handle definition in JIT mode (track cache dirty state)
#[cfg(feature = "jit")]
fn handle_definition_jit(input: &str, definitions: &mut Vec<String>, state: &mut ReplState) {
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
                    state.mark_dirty();

                    // Extract function name for tracking
                    if input.starts_with("F ") {
                        if let Some(name_end) = input.find('(') {
                            let name = input[2..name_end].trim();
                            state.function_counts.insert(name.to_string(), 0);
                        }
                    }

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

/// Handle :profile command - show function execution statistics
#[cfg(feature = "jit")]
fn handle_profile_command(state: &ReplState, _definitions: &[String]) {
    println!("{}", "Execution Profile:".bold().cyan());
    println!();

    if state.function_counts.is_empty() {
        println!("  No functions defined");
    } else {
        println!("{:30} {}", "Function".bold(), "Executions".bold());
        println!("{}", "-".repeat(50));

        let mut sorted_funcs: Vec<_> = state.function_counts.iter().collect();
        sorted_funcs.sort_by(|a, b| b.1.cmp(a.1));

        for (name, count) in sorted_funcs {
            println!("{:30} {}", name, count);
        }
    }

    println!();
    println!("Total expressions evaluated: {}", state.expression_count);
}

/// Handle :jit-stats command - show JIT engine status
#[cfg(feature = "jit")]
fn handle_jit_stats_command(state: &ReplState, definitions: &[String]) {
    println!("{}", "JIT Engine Status:".bold().cyan());
    println!();
    println!("  Mode:                  {}", "Cranelift JIT".green());
    println!(
        "  Defined functions:     {}",
        definitions.iter().filter(|d| d.starts_with("F ")).count()
    );
    println!("  Expressions evaluated: {}", state.expression_count);
    println!(
        "  Cache state:           {}",
        if state.cache_dirty {
            "dirty".yellow()
        } else {
            "clean".green()
        }
    );
    println!();
    println!("{}", "Optimization Tiers:".bold());
    println!("  All functions:         {}", "Baseline JIT".cyan());
    println!();
    println!("{}", "Note:".bold());
    println!("  Tier information is tracked at JIT engine level");
    println!("  Use :tier <func> to check specific function optimization");
}

/// Handle :tier command - show function optimization tier
#[cfg(feature = "jit")]
fn handle_tier_command(func_name: &str, definitions: &[String]) {
    // Check if function is defined
    let func_exists = definitions.iter().any(|def| {
        if def.starts_with("F ") {
            if let Some(name_end) = def.find('(') {
                let name = def[2..name_end].trim();
                return name == func_name;
            }
        }
        false
    });

    if !func_exists {
        println!(
            "{} Function '{}' not defined",
            "Error:".red().bold(),
            func_name
        );
        println!("Use :defs to see available functions");
        return;
    }

    println!(
        "{}",
        format!("Tier info for '{}':", func_name).bold().cyan()
    );
    println!();
    println!("  Current tier:     {}", "Baseline JIT".cyan());
    println!("  Backend:          {}", "Cranelift".green());
    println!("  Status:           {}", "Compiled".green());
    println!();
    println!("{}", "Note:".bold());
    println!("  Detailed tier profiling (hot path score, deopt count, loop iterations)");
    println!("  is tracked internally by the JIT engine's tiered compilation system.");
}

/// Handle an expression evaluation using JIT compilation
#[cfg(feature = "jit")]
fn handle_expression_jit(
    input: &str,
    definitions: &[String],
    jit: &mut JitCompiler,
    state: &mut ReplState,
) {
    let mut source = String::new();
    for def in definitions {
        source.push_str(def);
        source.push('\n');
    }
    source.push_str(&format!("F __repl_main()->i64{{{}}}", input));

    match evaluate_expr_jit(&source, jit, state) {
        Ok(result) => {
            state.increment_expression();
            println!("{} {}", "=>".cyan(), result);
        }
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
            if e.contains("JIT") {
                println!("{} Try :clear to reset JIT state", "Hint:".yellow().bold());
            }
        }
    }
}

/// Evaluate a REPL expression using JIT compilation
#[cfg(feature = "jit")]
fn evaluate_expr_jit(
    source: &str,
    jit: &mut JitCompiler,
    state: &mut ReplState,
) -> Result<String, String> {
    // Parse
    let ast = parse(source).map_err(|e| format!("Parse error: {}", e))?;

    // Type check
    let mut checker = TypeChecker::new();
    checker
        .check_module(&ast)
        .map_err(|e| format!("Type error: {}", e))?;

    // Only reset JIT if cache is dirty (definitions changed)
    if state.cache_dirty {
        jit.clear().map_err(|e| format!("JIT reset error: {}", e))?;
        state.cache_dirty = false;
    }

    // JIT compile and run (with graceful degradation)
    let result = jit.compile_and_run_main(&ast).map_err(|e| {
        // Mark cache as dirty on JIT failure to trigger reset on next attempt
        state.cache_dirty = true;
        format!("JIT compilation failed: {}", e)
    })?;

    Ok(format!("{}", result))
}
