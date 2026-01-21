//! Node.js bindings for Vais compiler
//!
//! Provides JavaScript/TypeScript API for compiling, checking, parsing, and tokenizing Vais source code.

mod token_conv;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use vais_lexer::tokenize as vais_tokenize;
use vais_parser::{parse as vais_parse, ParseError};
use vais_types::TypeChecker;
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_codegen::optimize::{optimize_ir, OptLevel};

/// Represents a compilation error
#[napi(object)]
#[derive(Clone)]
pub struct VaisError {
    pub message: String,
    pub span: Option<VaisSpan>,
    pub error_type: String,
}

/// Represents a source location span
#[napi(object)]
#[derive(Clone)]
pub struct VaisSpan {
    pub start: u32,
    pub end: u32,
}

/// Represents a token
#[napi(object)]
#[derive(Clone)]
pub struct VaisToken {
    pub token_type: String,
    pub span: VaisSpan,
    pub text: Option<String>,
}

/// Compilation options
#[napi(object)]
#[derive(Clone, Default)]
pub struct CompileOptions {
    /// Optimization level (0-3)
    pub opt_level: Option<u8>,
    /// Module name
    pub module_name: Option<String>,
    /// Target triple (e.g., "wasm32-unknown-unknown")
    pub target: Option<String>,
}

/// Tokenize Vais source code
///
/// # Arguments
///
/// * `source` - The Vais source code as a string
///
/// # Returns
///
/// An array of token objects
///
/// # Errors
///
/// Throws an error if the source code contains lexical errors
#[napi]
pub fn tokenize(source: String) -> Result<Vec<VaisToken>> {
    let tokens = vais_tokenize(&source)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Lexer error: {}", e)))?;

    Ok(tokens.iter().map(|st| {
        VaisToken {
            token_type: token_conv::token_to_string(&st.token),
            span: VaisSpan {
                start: st.span.start as u32,
                end: st.span.end as u32,
            },
            text: token_conv::token_text(&st.token, &source, &st.span),
        }
    }).collect())
}

/// Parse Vais source code into an AST
///
/// # Arguments
///
/// * `source` - The Vais source code as a string
///
/// # Returns
///
/// A JavaScript object representing the abstract syntax tree
///
/// # Errors
///
/// Throws an error if the source code contains syntax errors
#[napi]
pub fn parse(env: Env, source: String) -> Result<Object> {
    let ast = vais_parse(&source)
        .map_err(|e| {
            let error_msg = match &e {
                ParseError::UnexpectedToken { found, expected, span } => {
                    format!("Unexpected token {:?} at {:?}, expected {}", found, span, expected)
                }
                ParseError::UnexpectedEof { span } => {
                    format!("Unexpected end of file at {:?}", span)
                }
                ParseError::InvalidExpression => {
                    "Invalid expression".to_string()
                }
            };
            Error::new(Status::InvalidArg, error_msg)
        })?;

    // Create simplified AST representation
    let mut obj = env.create_object()?;
    obj.set_named_property("type", env.create_string("Module")?)?;
    obj.set_named_property("items_count", env.create_int32(ast.items.len() as i32)?)?;

    // For now, return simplified representation
    // Full AST serialization would require extensive boilerplate
    let items_arr = env.create_array(0)?;
    obj.set_named_property("items", items_arr)?;

    Ok(obj)
}

/// Type check Vais source code
///
/// # Arguments
///
/// * `source` - The Vais source code as a string
///
/// # Returns
///
/// An array of error objects (empty if no errors)
#[napi]
pub fn check(source: String) -> Result<Vec<VaisError>> {
    // First parse
    let ast = match vais_parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            let (message, span) = match &e {
                ParseError::UnexpectedToken { found, expected, span } => {
                    (format!("Unexpected token {:?}, expected {}", found, expected),
                     Some(VaisSpan { start: span.start as u32, end: span.end as u32 }))
                }
                ParseError::UnexpectedEof { span } => {
                    ("Unexpected end of file".to_string(),
                     Some(VaisSpan { start: span.start as u32, end: span.end as u32 }))
                }
                ParseError::InvalidExpression => {
                    ("Invalid expression".to_string(), None)
                }
            };
            return Ok(vec![VaisError {
                message,
                span,
                error_type: "ParseError".to_string(),
            }]);
        }
    };

    // Type check
    let mut checker = TypeChecker::new();
    match checker.check_module(&ast) {
        Ok(_) => Ok(vec![]),
        Err(e) => {
            Ok(vec![VaisError {
                message: e.to_string(),
                span: None,
                error_type: "TypeError".to_string(),
            }])
        }
    }
}

/// Compile Vais source code to LLVM IR
///
/// # Arguments
///
/// * `source` - The Vais source code as a string
/// * `options` - Optional compilation options
///
/// # Returns
///
/// The compiled LLVM IR as a string
///
/// # Errors
///
/// Throws an error if compilation fails
#[napi]
pub fn compile(source: String, options: Option<CompileOptions>) -> Result<String> {
    let opts = options.unwrap_or_default();

    // Parse options
    let opt_level = opts.opt_level.unwrap_or(0);
    let module_name = opts.module_name.unwrap_or_else(|| "main".to_string());
    let target = if let Some(t) = opts.target {
        TargetTriple::from_str(&t).unwrap_or(TargetTriple::Native)
    } else {
        TargetTriple::Native
    };

    // Parse
    let ast = vais_parse(&source)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Parse error: {}", e)))?;

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Type error: {}", e)))?;

    // Generate code
    let mut codegen = CodeGenerator::new_with_target(&module_name, target);
    let raw_ir = codegen.generate_module(&ast)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Codegen error: {}", e)))?;

    // Optimize
    let opt = match opt_level {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        _ => OptLevel::O3,
    };
    let ir = optimize_ir(&raw_ir, opt);

    Ok(ir)
}
