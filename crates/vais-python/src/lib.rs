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

/// Result of a compilation
#[pyclass]
#[derive(Clone)]
pub struct CompileResult {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub ir: Option<String>,
    #[pyo3(get)]
    pub errors: Vec<Error>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl CompileResult {
    fn __repr__(&self) -> String {
        if self.success {
            format!("CompileResult(success=True, ir_length={})",
                self.ir.as_ref().map(|s| s.len()).unwrap_or(0))
        } else {
            format!("CompileResult(success=False, errors={})", self.errors.len())
        }
    }

    /// Get the LLVM IR if compilation succeeded
    fn get_ir(&self) -> PyResult<String> {
        self.ir.clone()
            .ok_or_else(|| PyRuntimeError::new_err("Compilation failed, no IR available"))
    }
}

/// Result of running compiled code
#[pyclass]
#[derive(Clone)]
pub struct RunResult {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub exit_code: Option<i32>,
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
    #[pyo3(get)]
    pub errors: Vec<Error>,
}

#[pymethods]
impl RunResult {
    fn __repr__(&self) -> String {
        if self.success {
            format!("RunResult(success=True, exit_code={:?})", self.exit_code)
        } else {
            format!("RunResult(success=False, errors={})", self.errors.len())
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
        let dict = PyDict::new(py);
        dict.set_item("type", "Module")?;

        let items_list = PyList::empty(py);
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

/// Compile Vais source code to LLVM IR (returns CompileResult)
///
/// Args:
///     source: The Vais source code as a string
///     opt_level: Optional optimization level (0-3, default: 0)
///     module_name: Optional module name (default: "main")
///     target: Optional target triple (default: native)
///
/// Returns:
///     CompileResult containing the IR and any errors/warnings
#[pyfunction]
#[pyo3(signature = (source, opt_level=0, module_name=None, target=None))]
fn compile_to_result(
    source: String,
    opt_level: u8,
    module_name: Option<String>,
    target: Option<String>,
) -> PyResult<CompileResult> {
    let module_name = module_name.unwrap_or_else(|| "main".to_string());
    let mut errors = Vec::new();
    let warnings = Vec::new();

    let target_str = target;

    let target = if let Some(t) = target_str {
        TargetTriple::parse(&t).unwrap_or(TargetTriple::Native)
    } else {
        TargetTriple::Native
    };

    // Parse
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
            errors.push(Error {
                message,
                span,
                error_type: "ParseError".to_string(),
            });
            return Ok(CompileResult {
                success: false,
                ir: None,
                errors,
                warnings,
            });
        }
    };

    // Type check
    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_module(&ast) {
        errors.push(Error {
            message: e.to_string(),
            span: None,
            error_type: "TypeError".to_string(),
        });
        return Ok(CompileResult {
            success: false,
            ir: None,
            errors,
            warnings,
        });
    }

    // Generate code
    let mut codegen = CodeGenerator::new_with_target(&module_name, target);
    let raw_ir = match codegen.generate_module(&ast) {
        Ok(ir) => ir,
        Err(e) => {
            errors.push(Error {
                message: e.to_string(),
                span: None,
                error_type: "CodegenError".to_string(),
            });
            return Ok(CompileResult {
                success: false,
                ir: None,
                errors,
                warnings,
            });
        }
    };

    // Optimize
    let opt = match opt_level {
        0 => OptLevel::O0,
        1 => OptLevel::O1,
        2 => OptLevel::O2,
        _ => OptLevel::O3,
    };
    let ir = optimize_ir(&raw_ir, opt);

    Ok(CompileResult {
        success: true,
        ir: Some(ir),
        errors,
        warnings,
    })
}

/// Compile Vais source code to LLVM IR (legacy function, raises on error)
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
    let result = compile_to_result(source, opt_level, module_name, target)?;
    if result.success {
        result.get_ir()
    } else {
        let error_msgs: Vec<String> = result.errors.iter()
            .map(|e| format!("{}: {}", e.error_type, e.message))
            .collect();
        Err(PyRuntimeError::new_err(error_msgs.join("\n")))
    }
}

/// Compile and run Vais source code
///
/// Args:
///     source: The Vais source code as a string
///     opt_level: Optional optimization level (0-3, default: 0)
///
/// Returns:
///     RunResult containing execution results
///
/// Note:
///     This function currently returns a placeholder as JIT execution
///     requires additional runtime setup. Use compile() to get IR and
///     execute it separately with your preferred LLVM toolchain.
#[pyfunction]
#[pyo3(signature = (source, opt_level=0))]
fn compile_and_run(
    source: String,
    opt_level: u8,
) -> PyResult<RunResult> {
    // First compile
    let compile_result = compile_to_result(source, opt_level, None, None)?;

    if !compile_result.success {
        return Ok(RunResult {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            errors: compile_result.errors,
        });
    }

    // Note: Actual JIT execution would require vais-jit integration
    // For now, we return a message indicating this is not yet implemented
    Ok(RunResult {
        success: false,
        exit_code: None,
        stdout: String::new(),
        stderr: "JIT execution not yet available in Python bindings. Use compile() to get IR and execute separately.".to_string(),
        errors: vec![Error {
            message: "JIT execution requires additional runtime setup".to_string(),
            span: None,
            error_type: "NotImplemented".to_string(),
        }],
    })
}

/// VaisCompiler class - stateful compiler instance
///
/// Provides an object-oriented interface to the Vais compiler with
/// configurable compilation settings.
#[pyclass]
pub struct VaisCompiler {
    opt_level: u8,
    module_name: String,
    target: Option<String>,
}

#[pymethods]
impl VaisCompiler {
    /// Create a new VaisCompiler instance
    ///
    /// Args:
    ///     opt_level: Optimization level 0-3 (default: 0)
    ///     module_name: Module name (default: "main")
    ///     target: Target triple (default: None for native)
    #[new]
    #[pyo3(signature = (opt_level=0, module_name=None, target=None))]
    fn new(opt_level: u8, module_name: Option<String>, target: Option<String>) -> Self {
        VaisCompiler {
            opt_level,
            module_name: module_name.unwrap_or_else(|| "main".to_string()),
            target,
        }
    }

    /// Compile source code to LLVM IR
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     CompileResult with IR and any errors
    fn compile(&self, source: String) -> PyResult<CompileResult> {
        compile_to_result(
            source,
            self.opt_level,
            Some(self.module_name.clone()),
            self.target.clone(),
        )
    }

    /// Compile source code to LLVM IR (raises on error)
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     The compiled LLVM IR as a string
    ///
    /// Raises:
    ///     RuntimeError: If compilation fails
    fn compile_ir(&self, source: String) -> PyResult<String> {
        compile(
            source,
            self.opt_level,
            Some(self.module_name.clone()),
            self.target.clone(),
        )
    }

    /// Compile and run source code
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     RunResult with execution results
    fn run(&self, source: String) -> PyResult<RunResult> {
        compile_and_run(source, self.opt_level)
    }

    /// Tokenize source code
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     List of Token objects
    fn tokenize(&self, source: String) -> PyResult<Vec<TokenInfo>> {
        tokenize(source)
    }

    /// Parse source code to AST
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     Dictionary representing the AST
    fn parse(&self, source: String) -> PyResult<PyObject> {
        parse(source)
    }

    /// Type check source code
    ///
    /// Args:
    ///     source: The Vais source code
    ///
    /// Returns:
    ///     List of Error objects (empty if no errors)
    fn check(&self, source: String) -> PyResult<Vec<Error>> {
        check(source)
    }

    /// Set optimization level
    ///
    /// Args:
    ///     level: Optimization level 0-3
    fn set_opt_level(&mut self, level: u8) {
        self.opt_level = level;
    }

    /// Set module name
    ///
    /// Args:
    ///     name: Module name
    fn set_module_name(&mut self, name: String) {
        self.module_name = name;
    }

    /// Set target triple
    ///
    /// Args:
    ///     target: Target triple (e.g., "wasm32-unknown-unknown")
    fn set_target(&mut self, target: Option<String>) {
        self.target = target;
    }

    /// Get current optimization level
    #[getter]
    fn get_opt_level(&self) -> u8 {
        self.opt_level
    }

    /// Get current module name
    #[getter]
    fn get_module_name(&self) -> String {
        self.module_name.clone()
    }

    /// Get current target
    #[getter]
    fn get_target(&self) -> Option<String> {
        self.target.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "VaisCompiler(opt_level={}, module_name='{}', target={:?})",
            self.opt_level, self.module_name, self.target
        )
    }
}

/// Python module initialization
#[pymodule]
fn vais(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    // Functions
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(compile_to_result, m)?)?;
    m.add_function(wrap_pyfunction!(compile_and_run, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;

    // Classes
    m.add_class::<Error>()?;
    m.add_class::<TokenInfo>()?;
    m.add_class::<CompileResult>()?;
    m.add_class::<RunResult>()?;
    m.add_class::<VaisCompiler>()?;

    // Module metadata
    m.add("__version__", "0.0.1")?;
    m.add("__doc__", "Python bindings for the Vais compiler")?;

    Ok(())
}
