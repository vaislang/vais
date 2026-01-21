//! Python bindings for Vais compiler
//!
//! Provides Python API for compiling, checking, parsing, and tokenizing Vais source code.

mod token_conv;

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::types::{PyDict, PyList};

use vais_lexer::tokenize as vais_tokenize;
use vais_parser::{parse as vais_parse, ParseError};
use vais_ast::Module;
use vais_types::TypeChecker;
use vais_codegen::{CodeGenerator, TargetTriple};
use vais_codegen::optimize::{optimize_ir, OptLevel};

/// Represents a compilation error
#[pyclass]
#[derive(Clone)]
pub struct Error {
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub span: Option<(usize, usize)>,
    #[pyo3(get)]
    pub error_type: String,
}

#[pymethods]
impl Error {
    fn __repr__(&self) -> String {
        match self.span {
            Some((start, end)) => format!("Error(type='{}', message='{}', span=({}, {}))",
                self.error_type, self.message, start, end),
            None => format!("Error(type='{}', message='{}')", self.error_type, self.message),
        }
    }
}

/// Represents a token
#[pyclass]
#[derive(Clone)]
pub struct TokenInfo {
    #[pyo3(get)]
    pub token_type: String,
    #[pyo3(get)]
    pub span: (usize, usize),
    #[pyo3(get)]
    pub text: Option<String>,
}

#[pymethods]
impl TokenInfo {
    fn __repr__(&self) -> String {
        match &self.text {
            Some(text) => format!("Token(type='{}', span=({}, {}), text='{}')",
                self.token_type, self.span.0, self.span.1, text),
            None => format!("Token(type='{}', span=({}, {}))",
                self.token_type, self.span.0, self.span.1),
        }
    }
}

/// Tokenize Vais source code
///
/// Args:
///     source: The Vais source code as a string
///
/// Returns:
///     A list of Token objects
///
/// Raises:
///     ValueError: If the source code contains lexical errors
#[pyfunction]
fn tokenize(source: String) -> PyResult<Vec<TokenInfo>> {
    let tokens = vais_tokenize(&source)
        .map_err(|e| PyValueError::new_err(format!("Lexer error: {}", e)))?;

    Ok(tokens.iter().map(|st| {
        TokenInfo {
            token_type: token_conv::token_to_string(&st.token),
            span: (st.span.start, st.span.end),
            text: token_conv::token_text(&st.token, &source, &st.span),
        }
    }).collect())
}

/// Serialize AST to Python dict
fn module_to_dict(module: &Module) -> PyResult<PyObject> {
    Python::with_gil(|py| {
        let dict = PyDict::new_bound(py);
        dict.set_item("type", "Module")?;

        let items_list = PyList::empty_bound(py);
        // For now, return simplified representation with item count
        // Full AST serialization would require extensive boilerplate
        dict.set_item("items_count", module.items.len())?;
        dict.set_item("items", &items_list)?;

        Ok(dict.into_any().unbind())
    })
}

/// Parse Vais source code into an AST
///
/// Args:
///     source: The Vais source code as a string
///
/// Returns:
///     A dictionary representing the abstract syntax tree
///
/// Raises:
///     ValueError: If the source code contains syntax errors
#[pyfunction]
fn parse(source: String) -> PyResult<PyObject> {
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
            PyValueError::new_err(error_msg)
        })?;

    module_to_dict(&ast)
}

/// Type check Vais source code
///
/// Args:
///     source: The Vais source code as a string
///
/// Returns:
///     A list of Error objects (empty if no errors)
#[pyfunction]
fn check(source: String) -> PyResult<Vec<Error>> {
    // First parse
    let ast = match vais_parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            let (message, span) = match &e {
                ParseError::UnexpectedToken { found, expected, span } => {
                    (format!("Unexpected token {:?}, expected {}", found, expected), Some((span.start, span.end)))
                }
                ParseError::UnexpectedEof { span } => {
                    ("Unexpected end of file".to_string(), Some((span.start, span.end)))
                }
                ParseError::InvalidExpression => {
                    ("Invalid expression".to_string(), None)
                }
            };
            return Ok(vec![Error {
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
            Ok(vec![Error {
                message: e.to_string(),
                span: None,
                error_type: "TypeError".to_string(),
            }])
        }
    }
}

/// Compile Vais source code to LLVM IR
///
/// Args:
///     source: The Vais source code as a string
///     opt_level: Optional optimization level (0-3, default: 0)
///     module_name: Optional module name (default: "main")
///     target: Optional target triple (default: native)
///
/// Returns:
///     The compiled LLVM IR as a string
///
/// Raises:
///     RuntimeError: If compilation fails
#[pyfunction]
#[pyo3(signature = (source, opt_level=0, module_name=None, target=None))]
fn compile(
    source: String,
    opt_level: u8,
    module_name: Option<String>,
    target: Option<String>,
) -> PyResult<String> {
    let module_name = module_name.unwrap_or_else(|| "main".to_string());

    let target_str = target;

    let target = if let Some(t) = target_str {
        TargetTriple::from_str(&t).unwrap_or(TargetTriple::Native)
    } else {
        TargetTriple::Native
    };

    // Parse
    let ast = vais_parse(&source)
        .map_err(|e| PyRuntimeError::new_err(format!("Parse error: {}", e)))?;

    // Type check
    let mut checker = TypeChecker::new();
    checker.check_module(&ast)
        .map_err(|e| PyRuntimeError::new_err(format!("Type error: {}", e)))?;

    // Generate code
    let mut codegen = CodeGenerator::new_with_target(&module_name, target);
    let raw_ir = codegen.generate_module(&ast)
        .map_err(|e| PyRuntimeError::new_err(format!("Codegen error: {}", e)))?;

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

/// Python module initialization
#[pymodule]
fn vais(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    m.add_class::<Error>()?;
    m.add_class::<TokenInfo>()?;
    Ok(())
}
