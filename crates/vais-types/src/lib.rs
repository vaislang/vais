//! Vais 0.0.1 Type System
//!
//! Static type checking with inference for AI-optimized code generation.

use std::collections::HashMap;
use thiserror::Error;
use vais_ast::*;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },

    #[error("Undefined variable: {0}")]
    UndefinedVar(String),

    #[error("Undefined type: {0}")]
    UndefinedType(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Cannot call non-function type: {0}")]
    NotCallable(String),

    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    ArgCount { expected: usize, got: usize },

    #[error("Cannot infer type")]
    CannotInfer,

    #[error("Duplicate definition: {0}")]
    Duplicate(String),

    #[error("Cannot assign to immutable variable: {0}")]
    ImmutableAssign(String),
}

type TypeResult<T> = Result<T, TypeError>;

/// Resolved type in the type system
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    // Primitives
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Str,
    Unit,

    // Compound types
    Array(Box<ResolvedType>),
    Map(Box<ResolvedType>, Box<ResolvedType>),
    Tuple(Vec<ResolvedType>),
    Optional(Box<ResolvedType>),
    Result(Box<ResolvedType>),
    Pointer(Box<ResolvedType>),
    Ref(Box<ResolvedType>),
    RefMut(Box<ResolvedType>),

    // Function type
    Fn {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
    },

    // Named type (struct/enum)
    Named {
        name: String,
        generics: Vec<ResolvedType>,
    },

    // Type variable for inference
    Var(usize),

    // Generic type parameter (e.g., T in F foo<T>)
    Generic(String),

    // Unknown/Error type
    Unknown,
}

impl ResolvedType {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
                | ResolvedType::F32
                | ResolvedType::F64
                | ResolvedType::Generic(_) // Generics are assumed to support numeric ops
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::I128
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
                | ResolvedType::U128
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, ResolvedType::F32 | ResolvedType::F64)
    }
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::I8 => write!(f, "i8"),
            ResolvedType::I16 => write!(f, "i16"),
            ResolvedType::I32 => write!(f, "i32"),
            ResolvedType::I64 => write!(f, "i64"),
            ResolvedType::I128 => write!(f, "i128"),
            ResolvedType::U8 => write!(f, "u8"),
            ResolvedType::U16 => write!(f, "u16"),
            ResolvedType::U32 => write!(f, "u32"),
            ResolvedType::U64 => write!(f, "u64"),
            ResolvedType::U128 => write!(f, "u128"),
            ResolvedType::F32 => write!(f, "f32"),
            ResolvedType::F64 => write!(f, "f64"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Str => write!(f, "str"),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Array(t) => write!(f, "[{}]", t),
            ResolvedType::Map(k, v) => write!(f, "[{}:{}]", k, v),
            ResolvedType::Tuple(ts) => {
                write!(f, "(")?;
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            ResolvedType::Optional(t) => write!(f, "{}?", t),
            ResolvedType::Result(t) => write!(f, "{}!", t),
            ResolvedType::Pointer(t) => write!(f, "*{}", t),
            ResolvedType::Ref(t) => write!(f, "&{}", t),
            ResolvedType::RefMut(t) => write!(f, "&mut {}", t),
            ResolvedType::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")->{}", ret)
            }
            ResolvedType::Named { name, generics } => {
                write!(f, "{}", name)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    for (i, g) in generics.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            ResolvedType::Var(id) => write!(f, "?{}", id),
            ResolvedType::Generic(name) => write!(f, "{}", name),
            ResolvedType::Unknown => write!(f, "?"),
        }
    }
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<(String, ResolvedType, bool)>, // (name, type, is_mut)
    pub ret: ResolvedType,
    pub is_async: bool,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, ResolvedType>,
    pub methods: HashMap<String, FunctionSig>,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: HashMap<String, Vec<ResolvedType>>,
}

/// Trait method signature
#[derive(Debug, Clone)]
pub struct TraitMethodSig {
    pub name: String,
    pub params: Vec<(String, ResolvedType, bool)>, // (name, type, is_mut) - first param is &self
    pub ret: ResolvedType,
    pub has_default: bool,
}

/// Trait definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub generics: Vec<String>,
    pub methods: HashMap<String, TraitMethodSig>,
}

/// Tracks which types implement which traits
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TraitImpl {
    trait_name: String,
    type_name: String,
}

/// Variable info
#[derive(Debug, Clone)]
struct VarInfo {
    ty: ResolvedType,
    is_mut: bool,
}

/// Type checker
pub struct TypeChecker {
    // Type environment
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    functions: HashMap<String, FunctionSig>,
    type_aliases: HashMap<String, ResolvedType>,
    traits: HashMap<String, TraitDef>,
    trait_impls: Vec<TraitImpl>, // (type_name, trait_name) pairs

    // Scope stack
    scopes: Vec<HashMap<String, VarInfo>>,

    // Current function context
    current_fn_ret: Option<ResolvedType>,
    current_fn_name: Option<String>,

    // Current generic parameters (for type resolution)
    current_generics: Vec<String>,

    // Current generic bounds (maps generic param name to trait bounds)
    current_generic_bounds: HashMap<String, Vec<String>>,

    // Type variable counter for inference
    #[allow(dead_code)]
    next_type_var: usize,

    // Type substitutions
    substitutions: HashMap<usize, ResolvedType>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: Vec::new(),
            scopes: vec![HashMap::new()],
            current_fn_ret: None,
            current_fn_name: None,
            current_generics: Vec::new(),
            current_generic_bounds: HashMap::new(),
            next_type_var: 0,
            substitutions: HashMap::new(),
        };
        checker.register_builtins();
        checker
    }

    /// Register built-in functions (libc wrappers)
    fn register_builtins(&mut self) {
        // printf: (str, ...) -> i32
        self.functions.insert(
            "printf".to_string(),
            FunctionSig {
                name: "printf".to_string(),
                generics: vec![],
                params: vec![("format".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                is_async: false,
            },
        );

        // puts: (str) -> i32
        self.functions.insert(
            "puts".to_string(),
            FunctionSig {
                name: "puts".to_string(),
                generics: vec![],
                params: vec![("s".to_string(), ResolvedType::Str, false)],
                ret: ResolvedType::I32,
                is_async: false,
            },
        );

        // putchar: (i32) -> i32
        self.functions.insert(
            "putchar".to_string(),
            FunctionSig {
                name: "putchar".to_string(),
                generics: vec![],
                params: vec![("c".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::I32,
                is_async: false,
            },
        );

        // malloc: (size: i64) -> i64 (pointer as integer for simplicity)
        self.functions.insert(
            "malloc".to_string(),
            FunctionSig {
                name: "malloc".to_string(),
                generics: vec![],
                params: vec![("size".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // free: (ptr: i64) -> ()
        self.functions.insert(
            "free".to_string(),
            FunctionSig {
                name: "free".to_string(),
                generics: vec![],
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::Unit,
                is_async: false,
            },
        );

        // exit: (code: i32) -> void (noreturn, but typed as Unit)
        self.functions.insert(
            "exit".to_string(),
            FunctionSig {
                name: "exit".to_string(),
                generics: vec![],
                params: vec![("code".to_string(), ResolvedType::I32, false)],
                ret: ResolvedType::Unit,
                is_async: false,
            },
        );

        // memcpy: (dest, src, n) -> i64
        self.functions.insert(
            "memcpy".to_string(),
            FunctionSig {
                name: "memcpy".to_string(),
                generics: vec![],
                params: vec![
                    ("dest".to_string(), ResolvedType::I64, false),
                    ("src".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // strlen: (s) -> i64
        self.functions.insert(
            "strlen".to_string(),
            FunctionSig {
                name: "strlen".to_string(),
                generics: vec![],
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // puts_ptr: (s) -> i32
        self.functions.insert(
            "puts_ptr".to_string(),
            FunctionSig {
                name: "puts_ptr".to_string(),
                generics: vec![],
                params: vec![("s".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                is_async: false,
            },
        );

        // load_byte: (ptr) -> i64
        self.functions.insert(
            "load_byte".to_string(),
            FunctionSig {
                name: "load_byte".to_string(),
                generics: vec![],
                params: vec![("ptr".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // store_byte: (ptr, val) -> ()
        self.functions.insert(
            "store_byte".to_string(),
            FunctionSig {
                name: "store_byte".to_string(),
                generics: vec![],
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("val".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::Unit,
                is_async: false,
            },
        );

        // ===== File I/O functions =====

        // fopen: (path, mode) -> FILE* (as i64)
        self.functions.insert(
            "fopen".to_string(),
            FunctionSig {
                name: "fopen".to_string(),
                generics: vec![],
                params: vec![
                    ("path".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fclose: (stream) -> i32
        self.functions.insert(
            "fclose".to_string(),
            FunctionSig {
                name: "fclose".to_string(),
                generics: vec![],
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I32,
                is_async: false,
            },
        );

        // fread: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fread".to_string(),
            FunctionSig {
                name: "fread".to_string(),
                generics: vec![],
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fwrite: (ptr, size, count, stream) -> i64
        self.functions.insert(
            "fwrite".to_string(),
            FunctionSig {
                name: "fwrite".to_string(),
                generics: vec![],
                params: vec![
                    ("ptr".to_string(), ResolvedType::I64, false),
                    ("size".to_string(), ResolvedType::I64, false),
                    ("count".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fgetc: (stream) -> i64 (returns -1 on EOF)
        self.functions.insert(
            "fgetc".to_string(),
            FunctionSig {
                name: "fgetc".to_string(),
                generics: vec![],
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fputc: (c, stream) -> i64
        self.functions.insert(
            "fputc".to_string(),
            FunctionSig {
                name: "fputc".to_string(),
                generics: vec![],
                params: vec![
                    ("c".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fgets: (str, n, stream) -> i64 (char*)
        self.functions.insert(
            "fgets".to_string(),
            FunctionSig {
                name: "fgets".to_string(),
                generics: vec![],
                params: vec![
                    ("str".to_string(), ResolvedType::I64, false),
                    ("n".to_string(), ResolvedType::I64, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fputs: (str, stream) -> i64
        self.functions.insert(
            "fputs".to_string(),
            FunctionSig {
                name: "fputs".to_string(),
                generics: vec![],
                params: vec![
                    ("str".to_string(), ResolvedType::Str, false),
                    ("stream".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fseek: (stream, offset, origin) -> i64
        self.functions.insert(
            "fseek".to_string(),
            FunctionSig {
                name: "fseek".to_string(),
                generics: vec![],
                params: vec![
                    ("stream".to_string(), ResolvedType::I64, false),
                    ("offset".to_string(), ResolvedType::I64, false),
                    ("origin".to_string(), ResolvedType::I64, false),
                ],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // ftell: (stream) -> i64
        self.functions.insert(
            "ftell".to_string(),
            FunctionSig {
                name: "ftell".to_string(),
                generics: vec![],
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // fflush: (stream) -> i64
        self.functions.insert(
            "fflush".to_string(),
            FunctionSig {
                name: "fflush".to_string(),
                generics: vec![],
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );

        // feof: (stream) -> i64
        self.functions.insert(
            "feof".to_string(),
            FunctionSig {
                name: "feof".to_string(),
                generics: vec![],
                params: vec![("stream".to_string(), ResolvedType::I64, false)],
                ret: ResolvedType::I64,
                is_async: false,
            },
        );
    }

    /// Type check a module
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        // First pass: collect all type definitions
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.register_function(f)?,
                Item::Struct(s) => self.register_struct(s)?,
                Item::Enum(e) => self.register_enum(e)?,
                Item::TypeAlias(t) => self.register_type_alias(t)?,
                Item::Use(_use_stmt) => {
                    // Use statements are handled at the compiler level (AST merging)
                    // by the time we reach type checking, all imports are already resolved
                }
                Item::Trait(t) => self.register_trait(t)?,
                Item::Impl(impl_block) => {
                    // Register impl methods to the target type
                    self.register_impl(impl_block)?;
                }
            }
        }

        // Second pass: check function bodies
        for item in &module.items {
            match &item.node {
                Item::Function(f) => self.check_function(f)?,
                Item::Impl(impl_block) => {
                    // Check impl method bodies
                    for method in &impl_block.methods {
                        self.check_impl_method(&impl_block.target_type.node, &method.node)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Set current generics with their bounds for type resolution
    fn set_generics(&mut self, generics: &[GenericParam]) -> (Vec<String>, HashMap<String, Vec<String>>) {
        let prev_generics = std::mem::replace(
            &mut self.current_generics,
            generics.iter().map(|g| g.name.node.clone()).collect(),
        );
        let prev_bounds = std::mem::replace(
            &mut self.current_generic_bounds,
            generics
                .iter()
                .map(|g| {
                    (
                        g.name.node.clone(),
                        g.bounds.iter().map(|b| b.node.clone()).collect(),
                    )
                })
                .collect(),
        );
        (prev_generics, prev_bounds)
    }

    /// Restore previous generics
    fn restore_generics(&mut self, prev_generics: Vec<String>, prev_bounds: HashMap<String, Vec<String>>) {
        self.current_generics = prev_generics;
        self.current_generic_bounds = prev_bounds;
    }

    /// Register a function signature
    fn register_function(&mut self, f: &Function) -> TypeResult<()> {
        let name = f.name.node.clone();
        if self.functions.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        // Set current generics for type resolution
        let (prev_generics, prev_bounds) = self.set_generics(&f.generics);

        let params: Vec<_> = f
            .params
            .iter()
            .map(|p| {
                let ty = self.resolve_type(&p.ty.node);
                (p.name.node.clone(), ty, p.is_mut)
            })
            .collect();

        let ret = f
            .ret_type
            .as_ref()
            .map(|t| self.resolve_type(&t.node))
            .unwrap_or(ResolvedType::Unit);

        // Restore previous generics
        self.restore_generics(prev_generics, prev_bounds);

        self.functions.insert(
            name.clone(),
            FunctionSig {
                name,
                generics: f.generics.iter().map(|g| g.name.node.clone()).collect(),
                params,
                ret,
                is_async: f.is_async,
            },
        );

        Ok(())
    }

    /// Register a struct
    fn register_struct(&mut self, s: &Struct) -> TypeResult<()> {
        let name = s.name.node.clone();
        if self.structs.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let mut fields = HashMap::new();
        for field in &s.fields {
            fields.insert(field.name.node.clone(), self.resolve_type(&field.ty.node));
        }

        let mut methods = HashMap::new();
        for method in &s.methods {
            let params: Vec<_> = method
                .node
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .node
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                method.node.name.node.clone(),
                FunctionSig {
                    name: method.node.name.node.clone(),
                    generics: method.node.generics.iter().map(|g| g.name.node.clone()).collect(),
                    params,
                    ret,
                    is_async: method.node.is_async,
                },
            );
        }

        self.structs.insert(
            name.clone(),
            StructDef {
                name,
                generics: s.generics.iter().map(|g| g.name.node.clone()).collect(),
                fields,
                methods,
            },
        );

        Ok(())
    }

    /// Register an enum
    fn register_enum(&mut self, e: &Enum) -> TypeResult<()> {
        let name = e.name.node.clone();
        if self.enums.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let mut variants = HashMap::new();
        for variant in &e.variants {
            let types = match &variant.fields {
                VariantFields::Unit => vec![],
                VariantFields::Tuple(ts) => ts.iter().map(|t| self.resolve_type(&t.node)).collect(),
                VariantFields::Struct(_) => vec![], // TODO: Handle struct variants
            };
            variants.insert(variant.name.node.clone(), types);
        }

        self.enums.insert(
            name.clone(),
            EnumDef {
                name,
                generics: e.generics.iter().map(|g| g.name.node.clone()).collect(),
                variants,
            },
        );

        Ok(())
    }

    /// Register impl block methods to the target type
    fn register_impl(&mut self, impl_block: &Impl) -> TypeResult<()> {
        // Get the type name
        let type_name = match &impl_block.target_type.node {
            Type::Named { name, .. } => name.clone(),
            _ => return Ok(()), // Skip non-named types for now
        };

        // Check if struct exists
        if !self.structs.contains_key(&type_name) {
            return Ok(()); // Struct not registered yet, skip
        }

        // If implementing a trait, validate the impl
        if let Some(trait_name) = &impl_block.trait_name {
            let trait_name_str = trait_name.node.clone();

            // Check trait exists
            if !self.traits.contains_key(&trait_name_str) {
                return Err(TypeError::UndefinedType(format!("trait {}", trait_name_str)));
            }

            // Record that this type implements this trait
            self.trait_impls.push(TraitImpl {
                trait_name: trait_name_str.clone(),
                type_name: type_name.clone(),
            });

            // Validate that all required trait methods are implemented
            if let Some(trait_def) = self.traits.get(&trait_name_str).cloned() {
                let impl_method_names: std::collections::HashSet<_> = impl_block
                    .methods
                    .iter()
                    .map(|m| m.node.name.node.clone())
                    .collect();

                for (method_name, trait_method) in &trait_def.methods {
                    if !trait_method.has_default && !impl_method_names.contains(method_name) {
                        return Err(TypeError::Mismatch {
                            expected: format!("implementation of method '{}' from trait '{}'", method_name, trait_name_str),
                            found: "missing".to_string(),
                        });
                    }
                }
            }
        }

        // Collect method signatures first (to avoid borrow issues)
        let mut method_sigs = Vec::new();
        for method in &impl_block.methods {
            let params: Vec<_> = method
                .node
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .node
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit);

            method_sigs.push((
                method.node.name.node.clone(),
                FunctionSig {
                    name: method.node.name.node.clone(),
                    generics: method.node.generics.iter().map(|g| g.name.node.clone()).collect(),
                    params,
                    ret,
                    is_async: method.node.is_async,
                },
            ));
        }

        // Now insert methods into the struct
        if let Some(struct_def) = self.structs.get_mut(&type_name) {
            for (name, sig) in method_sigs {
                struct_def.methods.insert(name, sig);
            }
        }

        Ok(())
    }

    /// Register a trait definition
    fn register_trait(&mut self, t: &vais_ast::Trait) -> TypeResult<()> {
        let name = t.name.node.clone();
        if self.traits.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        // Set current generics for type resolution
        let (prev_generics, prev_bounds) = self.set_generics(&t.generics);

        let mut methods = HashMap::new();
        for method in &t.methods {
            let params: Vec<_> = method
                .params
                .iter()
                .map(|p| {
                    let ty = self.resolve_type(&p.ty.node);
                    (p.name.node.clone(), ty, p.is_mut)
                })
                .collect();

            let ret = method
                .ret_type
                .as_ref()
                .map(|rt| self.resolve_type(&rt.node))
                .unwrap_or(ResolvedType::Unit);

            methods.insert(
                method.name.node.clone(),
                TraitMethodSig {
                    name: method.name.node.clone(),
                    params,
                    ret,
                    has_default: method.default_body.is_some(),
                },
            );
        }

        self.restore_generics(prev_generics, prev_bounds);

        self.traits.insert(
            name.clone(),
            TraitDef {
                name,
                generics: t.generics.iter().map(|g| g.name.node.clone()).collect(),
                methods,
            },
        );

        Ok(())
    }

    /// Register a type alias
    fn register_type_alias(&mut self, t: &TypeAlias) -> TypeResult<()> {
        let name = t.name.node.clone();
        if self.type_aliases.contains_key(&name) {
            return Err(TypeError::Duplicate(name));
        }

        let resolved = self.resolve_type(&t.ty.node);
        self.type_aliases.insert(name, resolved);

        Ok(())
    }

    /// Check a function body
    fn check_function(&mut self, f: &Function) -> TypeResult<()> {
        self.push_scope();

        // Set current generic parameters
        let (prev_generics, prev_bounds) = self.set_generics(&f.generics);

        // Add parameters to scope
        for param in &f.params {
            let ty = self.resolve_type(&param.ty.node);
            self.define_var(&param.name.node, ty, param.is_mut);
        }

        // Set current function context
        self.current_fn_ret = Some(
            f.ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit),
        );
        self.current_fn_name = Some(f.name.node.clone());

        // Check body
        let body_type = match &f.body {
            FunctionBody::Expr(expr) => self.check_expr(expr)?,
            FunctionBody::Block(stmts) => self.check_block(stmts)?,
        };

        // Check return type
        let expected_ret = self.current_fn_ret.clone().unwrap();
        self.unify(&expected_ret, &body_type)?;

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.restore_generics(prev_generics, prev_bounds);
        self.pop_scope();

        Ok(())
    }

    /// Check an impl method body
    fn check_impl_method(&mut self, target_type: &Type, method: &Function) -> TypeResult<()> {
        self.push_scope();

        // Get the type name for self
        let self_type_name = match target_type {
            Type::Named { name, .. } => name.clone(),
            _ => return Ok(()), // Skip non-named types
        };

        // Set current generic parameters
        let (prev_generics, prev_bounds) = self.set_generics(&method.generics);

        // Add parameters to scope
        for param in &method.params {
            // Handle &self parameter specially
            if param.name.node == "self" {
                // self is a reference to the target type
                let self_ty = ResolvedType::Ref(Box::new(ResolvedType::Named {
                    name: self_type_name.clone(),
                    generics: vec![],
                }));
                self.define_var("self", self_ty, param.is_mut);
            } else {
                let ty = self.resolve_type(&param.ty.node);
                self.define_var(&param.name.node, ty, param.is_mut);
            }
        }

        // Set current function context
        self.current_fn_ret = Some(
            method
                .ret_type
                .as_ref()
                .map(|t| self.resolve_type(&t.node))
                .unwrap_or(ResolvedType::Unit),
        );
        self.current_fn_name = Some(format!("{}::{}", self_type_name, method.name.node));

        // Check body
        let body_type = match &method.body {
            FunctionBody::Expr(expr) => self.check_expr(expr)?,
            FunctionBody::Block(stmts) => self.check_block(stmts)?,
        };

        // Check return type
        let expected_ret = self.current_fn_ret.clone().unwrap();
        self.unify(&expected_ret, &body_type)?;

        self.current_fn_ret = None;
        self.current_fn_name = None;
        self.restore_generics(prev_generics, prev_bounds);
        self.pop_scope();

        Ok(())
    }

    /// Check a block of statements
    fn check_block(&mut self, stmts: &[Spanned<Stmt>]) -> TypeResult<ResolvedType> {
        let mut last_type = ResolvedType::Unit;

        for stmt in stmts {
            last_type = self.check_stmt(stmt)?;
        }

        Ok(last_type)
    }

    /// Check a statement
    fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> TypeResult<ResolvedType> {
        match &stmt.node {
            Stmt::Let {
                name,
                ty,
                value,
                is_mut,
            } => {
                let value_type = self.check_expr(value)?;
                let var_type = if let Some(ty) = ty {
                    let expected = self.resolve_type(&ty.node);
                    self.unify(&expected, &value_type)?;
                    expected
                } else {
                    value_type
                };
                self.define_var(&name.node, var_type, *is_mut);
                Ok(ResolvedType::Unit)
            }
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Return(expr) => {
                let ret_type = if let Some(expr) = expr {
                    self.check_expr(expr)?
                } else {
                    ResolvedType::Unit
                };
                if let Some(expected) = self.current_fn_ret.clone() {
                    self.unify(&expected, &ret_type)?;
                }
                Ok(ResolvedType::Unit)
            }
            Stmt::Break(_) | Stmt::Continue => Ok(ResolvedType::Unit),
        }
    }

    /// Check an expression
    fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        match &expr.node {
            Expr::Int(_) => Ok(ResolvedType::I64),
            Expr::Float(_) => Ok(ResolvedType::F64),
            Expr::Bool(_) => Ok(ResolvedType::Bool),
            Expr::String(_) => Ok(ResolvedType::Str),
            Expr::Unit => Ok(ResolvedType::Unit),

            Expr::Ident(name) => self.lookup_var_or_err(name),

            Expr::SelfCall => {
                // @ refers to current function
                if let Some(name) = &self.current_fn_name {
                    if let Some(sig) = self.functions.get(name) {
                        return Ok(ResolvedType::Fn {
                            params: sig.params.iter().map(|(_, t, _)| t.clone()).collect(),
                            ret: Box::new(sig.ret.clone()),
                        });
                    }
                }
                Err(TypeError::UndefinedFunction("@".to_string()))
            }

            Expr::Binary { op, left, right } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                        if !left_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::Eq | BinOp::Neq => {
                        self.unify(&left_type, &right_type)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(&left_type, &ResolvedType::Bool)?;
                        self.unify(&right_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        if !left_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: left_type.to_string(),
                            });
                        }
                        self.unify(&left_type, &right_type)?;
                        Ok(left_type)
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let inner_type = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            return Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: inner_type.to_string(),
                            });
                        }
                        Ok(inner_type)
                    }
                    UnaryOp::Not => {
                        self.unify(&inner_type, &ResolvedType::Bool)?;
                        Ok(ResolvedType::Bool)
                    }
                    UnaryOp::BitNot => {
                        if !inner_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: inner_type.to_string(),
                            });
                        }
                        Ok(inner_type)
                    }
                }
            }

            Expr::Ternary { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                let then_type = self.check_expr(then)?;
                let else_type = self.check_expr(else_)?;
                self.unify(&then_type, &else_type)?;

                Ok(then_type)
            }

            Expr::If { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    self.unify(&then_type, &else_type)?;
                    Ok(then_type)
                } else {
                    Ok(ResolvedType::Unit)
                }
            }

            Expr::Loop { pattern, iter, body } => {
                self.push_scope();

                if let (Some(pattern), Some(iter)) = (pattern, iter) {
                    let iter_type = self.check_expr(iter)?;
                    // TODO: Proper iterator type inference
                    if let ResolvedType::Array(elem_type) = iter_type {
                        if let Pattern::Ident(name) = &pattern.node {
                            self.define_var(name, *elem_type, false);
                        }
                    }
                }

                self.check_block(body)?;
                self.pop_scope();

                Ok(ResolvedType::Unit)
            }

            Expr::Match { expr, arms } => {
                let expr_type = self.check_expr(expr)?;
                let mut result_type: Option<ResolvedType> = None;

                for arm in arms {
                    self.push_scope();

                    // Register pattern bindings in scope
                    self.register_pattern_bindings(&arm.pattern, &expr_type)?;

                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_type = self.check_expr(guard)?;
                        self.unify(&ResolvedType::Bool, &guard_type)?;
                    }

                    let arm_type = self.check_expr(&arm.body)?;
                    self.pop_scope();

                    if let Some(ref prev) = result_type {
                        self.unify(prev, &arm_type)?;
                    } else {
                        result_type = Some(arm_type);
                    }
                }

                Ok(result_type.unwrap_or(ResolvedType::Unit))
            }

            Expr::Call { func, args } => {
                let func_type = self.check_expr(func)?;

                match func_type {
                    ResolvedType::Fn { params, ret } => {
                        if params.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: params.len(),
                                got: args.len(),
                            });
                        }

                        for (param_type, arg) in params.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        Ok(*ret)
                    }
                    _ => Err(TypeError::NotCallable(func_type.to_string())),
                }
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver_type = self.check_expr(receiver)?;

                if let ResolvedType::Named { name, .. } = &receiver_type {
                    if let Some(struct_def) = self.structs.get(name).cloned() {
                        if let Some(method_sig) = struct_def.methods.get(&method.node).cloned() {
                            // Skip self parameter
                            let param_types: Vec<_> =
                                method_sig.params.iter().skip(1).map(|(_, t, _)| t.clone()).collect();

                            if param_types.len() != args.len() {
                                return Err(TypeError::ArgCount {
                                    expected: param_types.len(),
                                    got: args.len(),
                                });
                            }

                            for (param_type, arg) in param_types.iter().zip(args) {
                                let arg_type = self.check_expr(arg)?;
                                self.unify(param_type, &arg_type)?;
                            }

                            return Ok(method_sig.ret.clone());
                        }
                    }
                }

                Err(TypeError::UndefinedFunction(method.node.clone()))
            }

            Expr::StaticMethodCall {
                type_name,
                method,
                args,
            } => {
                // Static method call: Type.method(args)
                if let Some(struct_def) = self.structs.get(&type_name.node).cloned() {
                    if let Some(method_sig) = struct_def.methods.get(&method.node).cloned() {
                        // For static methods, don't skip first param (no self)
                        // But if the first param is self, skip it for backwards compat
                        let param_types: Vec<_> = if method_sig.params.first().map(|(n, _, _)| n == "self").unwrap_or(false) {
                            method_sig.params.iter().skip(1).map(|(_, t, _)| t.clone()).collect()
                        } else {
                            method_sig.params.iter().map(|(_, t, _)| t.clone()).collect()
                        };

                        if param_types.len() != args.len() {
                            return Err(TypeError::ArgCount {
                                expected: param_types.len(),
                                got: args.len(),
                            });
                        }

                        for (param_type, arg) in param_types.iter().zip(args) {
                            let arg_type = self.check_expr(arg)?;
                            self.unify(param_type, &arg_type)?;
                        }

                        return Ok(method_sig.ret.clone());
                    }
                }

                Err(TypeError::UndefinedFunction(format!("{}::{}", type_name.node, method.node)))
            }

            Expr::Field { expr: inner, field } => {
                let inner_type = self.check_expr(inner)?;

                // Handle both direct Named types and references to Named types
                let type_name = match &inner_type {
                    ResolvedType::Named { name, .. } => Some(name.clone()),
                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                        if let ResolvedType::Named { name, .. } = inner.as_ref() {
                            Some(name.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(name) = type_name {
                    if let Some(struct_def) = self.structs.get(&name) {
                        if let Some(field_type) = struct_def.fields.get(&field.node) {
                            return Ok(field_type.clone());
                        }
                    }
                }

                Err(TypeError::UndefinedVar(field.node.clone()))
            }

            Expr::Index { expr: inner, index } => {
                let inner_type = self.check_expr(inner)?;
                let index_type = self.check_expr(index)?;

                // Check if this is a slice operation (index is a Range)
                let is_slice = matches!(index.node, Expr::Range { .. });

                match inner_type {
                    ResolvedType::Array(elem_type) => {
                        if is_slice {
                            // Slice returns a pointer to array elements
                            Ok(ResolvedType::Pointer(elem_type))
                        } else if !index_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                            });
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    ResolvedType::Map(key_type, value_type) => {
                        self.unify(&key_type, &index_type)?;
                        Ok(*value_type)
                    }
                    // Pointers can be indexed like arrays
                    ResolvedType::Pointer(elem_type) => {
                        if is_slice {
                            // Slice of pointer returns a pointer
                            Ok(ResolvedType::Pointer(elem_type))
                        } else if !index_type.is_integer() {
                            return Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                            });
                        } else {
                            Ok(*elem_type)
                        }
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "indexable type".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    let var = self.fresh_type_var();
                    // Array literals decay to pointers in Vais
                    return Ok(ResolvedType::Pointer(Box::new(var)));
                }

                let first_type = self.check_expr(&exprs[0])?;
                for expr in &exprs[1..] {
                    let t = self.check_expr(expr)?;
                    self.unify(&first_type, &t)?;
                }

                // Array literals produce pointers to first element
                Ok(ResolvedType::Pointer(Box::new(first_type)))
            }

            Expr::Tuple(exprs) => {
                let types: Result<Vec<_>, _> = exprs.iter().map(|e| self.check_expr(e)).collect();
                Ok(ResolvedType::Tuple(types?))
            }

            Expr::StructLit { name, fields } => {
                if let Some(struct_def) = self.structs.get(&name.node).cloned() {
                    for (field_name, value) in fields {
                        let value_type = self.check_expr(value)?;
                        if let Some(expected_type) = struct_def.fields.get(&field_name.node).cloned() {
                            self.unify(&expected_type, &value_type)?;
                        } else {
                            return Err(TypeError::UndefinedVar(field_name.node.clone()));
                        }
                    }
                    Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: vec![],
                    })
                } else {
                    Err(TypeError::UndefinedType(name.node.clone()))
                }
            }

            Expr::Range { .. } => {
                // TODO: Implement range type
                Ok(ResolvedType::Named {
                    name: "Range".to_string(),
                    generics: vec![ResolvedType::I64],
                })
            }

            Expr::Block(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }

            Expr::Await(inner) => {
                // TODO: Proper async type checking
                self.check_expr(inner)
            }

            Expr::Try(inner) => {
                let inner_type = self.check_expr(inner)?;
                if let ResolvedType::Result(ok_type) = inner_type {
                    Ok(*ok_type)
                } else {
                    Err(TypeError::Mismatch {
                        expected: "Result type".to_string(),
                        found: inner_type.to_string(),
                    })
                }
            }

            Expr::Unwrap(inner) => {
                let inner_type = self.check_expr(inner)?;
                match inner_type {
                    ResolvedType::Optional(inner) | ResolvedType::Result(inner) => Ok(*inner),
                    _ => Err(TypeError::Mismatch {
                        expected: "Optional or Result".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Ref(inner) => {
                let inner_type = self.check_expr(inner)?;
                Ok(ResolvedType::Ref(Box::new(inner_type)))
            }

            Expr::Deref(inner) => {
                let inner_type = self.check_expr(inner)?;
                match inner_type {
                    ResolvedType::Ref(t) | ResolvedType::RefMut(t) | ResolvedType::Pointer(t) => {
                        Ok(*t)
                    }
                    _ => Err(TypeError::Mismatch {
                        expected: "reference or pointer".to_string(),
                        found: inner_type.to_string(),
                    }),
                }
            }

            Expr::Assign { target, value } => {
                // Check target is mutable
                if let Expr::Ident(name) = &target.node {
                    let var_info = self.lookup_var_info(name)?;
                    if !var_info.is_mut {
                        return Err(TypeError::ImmutableAssign(name.clone()));
                    }
                }

                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::AssignOp { op: _, target, value } => {
                // Similar to assign
                if let Expr::Ident(name) = &target.node {
                    let var_info = self.lookup_var_info(name)?;
                    if !var_info.is_mut {
                        return Err(TypeError::ImmutableAssign(name.clone()));
                    }
                }

                let target_type = self.check_expr(target)?;
                let value_type = self.check_expr(value)?;
                self.unify(&target_type, &value_type)?;
                Ok(ResolvedType::Unit)
            }

            Expr::Lambda { params, body, captures: _ } => {
                // Find free variables (captures) before entering lambda scope
                let param_names: std::collections::HashSet<_> = params.iter()
                    .map(|p| p.name.node.clone())
                    .collect();
                let free_vars = self.find_free_vars_in_expr(body, &param_names);

                // Verify all captured variables exist in current scope
                for var in &free_vars {
                    if self.lookup_var(var).is_none() {
                        return Err(TypeError::UndefinedVar(var.clone()));
                    }
                }

                self.push_scope();

                // Define captured variables in lambda scope
                for var in &free_vars {
                    if let Some((ty, is_mut)) = self.lookup_var_with_mut(var) {
                        self.define_var(var, ty, is_mut);
                    }
                }

                let param_types: Vec<_> = params
                    .iter()
                    .map(|p| {
                        let ty = self.resolve_type(&p.ty.node);
                        self.define_var(&p.name.node, ty.clone(), p.is_mut);
                        ty
                    })
                    .collect();

                let ret_type = self.check_expr(body)?;
                self.pop_scope();

                Ok(ResolvedType::Fn {
                    params: param_types,
                    ret: Box::new(ret_type),
                })
            }

            Expr::Spawn(inner) => {
                let inner_type = self.check_expr(inner)?;
                // For now, spawn is synchronous and returns the inner value directly
                // Future: Return Task<T> type for proper async handling
                Ok(inner_type)
            }
        }
    }

    /// Check if-else branch
    fn check_if_else(&mut self, branch: &IfElse) -> TypeResult<ResolvedType> {
        match branch {
            IfElse::ElseIf(cond, then, else_) => {
                let cond_type = self.check_expr(cond)?;
                self.unify(&cond_type, &ResolvedType::Bool)?;

                self.push_scope();
                let then_type = self.check_block(then)?;
                self.pop_scope();

                if let Some(else_branch) = else_ {
                    let else_type = self.check_if_else(else_branch)?;
                    self.unify(&then_type, &else_type)?;
                }

                Ok(then_type)
            }
            IfElse::Else(stmts) => {
                self.push_scope();
                let result = self.check_block(stmts);
                self.pop_scope();
                result
            }
        }
    }

    /// Resolve AST type to internal type
    fn resolve_type(&self, ty: &Type) -> ResolvedType {
        match ty {
            Type::Named { name, generics } => {
                let resolved_generics: Vec<_> =
                    generics.iter().map(|g| self.resolve_type(&g.node)).collect();

                match name.as_str() {
                    "i8" => ResolvedType::I8,
                    "i16" => ResolvedType::I16,
                    "i32" => ResolvedType::I32,
                    "i64" => ResolvedType::I64,
                    "i128" => ResolvedType::I128,
                    "u8" => ResolvedType::U8,
                    "u16" => ResolvedType::U16,
                    "u32" => ResolvedType::U32,
                    "u64" => ResolvedType::U64,
                    "u128" => ResolvedType::U128,
                    "f32" => ResolvedType::F32,
                    "f64" => ResolvedType::F64,
                    "bool" => ResolvedType::Bool,
                    "str" => ResolvedType::Str,
                    _ => {
                        // Check if it's a generic type parameter
                        if self.current_generics.contains(name) {
                            ResolvedType::Generic(name.clone())
                        } else if let Some(alias) = self.type_aliases.get(name) {
                            alias.clone()
                        } else {
                            ResolvedType::Named {
                                name: name.clone(),
                                generics: resolved_generics,
                            }
                        }
                    }
                }
            }
            Type::Array(inner) => ResolvedType::Array(Box::new(self.resolve_type(&inner.node))),
            Type::Map(key, value) => ResolvedType::Map(
                Box::new(self.resolve_type(&key.node)),
                Box::new(self.resolve_type(&value.node)),
            ),
            Type::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.resolve_type(&t.node)).collect())
            }
            Type::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.resolve_type(&inner.node)))
            }
            Type::Result(inner) => ResolvedType::Result(Box::new(self.resolve_type(&inner.node))),
            Type::Pointer(inner) => ResolvedType::Pointer(Box::new(self.resolve_type(&inner.node))),
            Type::Ref(inner) => ResolvedType::Ref(Box::new(self.resolve_type(&inner.node))),
            Type::RefMut(inner) => ResolvedType::RefMut(Box::new(self.resolve_type(&inner.node))),
            Type::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.resolve_type(&p.node)).collect(),
                ret: Box::new(self.resolve_type(&ret.node)),
            },
            Type::Unit => ResolvedType::Unit,
            Type::Infer => self.fresh_type_var(),
        }
    }

    /// Unify two types
    fn unify(&mut self, expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()> {
        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        match (&expected, &found) {
            // Type variables can unify with anything
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            // Generic type parameters match with any type (type erasure)
            (ResolvedType::Generic(_), _) | (_, ResolvedType::Generic(_)) => Ok(()),
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a), ResolvedType::Result(b)) => self.unify(a, b),
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Allow implicit integer type conversions (widening and narrowing)
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => Ok(()),
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
            }),
        }
    }

    /// Check if type is an integer type
    fn is_integer_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
        )
    }

    /// Apply substitutions to a type
    fn apply_substitutions(&self, ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Var(id) => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            ResolvedType::Array(inner) => {
                ResolvedType::Array(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Optional(inner) => {
                ResolvedType::Optional(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Result(inner) => {
                ResolvedType::Result(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Ref(inner) => {
                ResolvedType::Ref(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::RefMut(inner) => {
                ResolvedType::RefMut(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Pointer(inner) => {
                ResolvedType::Pointer(Box::new(self.apply_substitutions(inner)))
            }
            ResolvedType::Tuple(types) => {
                ResolvedType::Tuple(types.iter().map(|t| self.apply_substitutions(t)).collect())
            }
            ResolvedType::Fn { params, ret } => ResolvedType::Fn {
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                ret: Box::new(self.apply_substitutions(ret)),
            },
            _ => ty.clone(),
        }
    }

    /// Create a fresh type variable
    fn fresh_type_var(&self) -> ResolvedType {
        // Note: This should be mutable, but for simplicity we'll use a workaround
        ResolvedType::Var(0)
    }

    // === Scope management ===

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: &str, ty: ResolvedType, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), VarInfo { ty, is_mut });
        }
    }

    /// Register pattern bindings in the current scope
    fn register_pattern_bindings(
        &mut self,
        pattern: &Spanned<Pattern>,
        expr_type: &ResolvedType,
    ) -> TypeResult<()> {
        match &pattern.node {
            Pattern::Wildcard => Ok(()),
            Pattern::Ident(name) => {
                // Bind the identifier to the matched expression's type
                self.define_var(name, expr_type.clone(), false);
                Ok(())
            }
            Pattern::Literal(_) => Ok(()), // Literals don't bind variables
            Pattern::Tuple(patterns) => {
                if let ResolvedType::Tuple(types) = expr_type {
                    for (pat, ty) in patterns.iter().zip(types.iter()) {
                        self.register_pattern_bindings(pat, ty)?;
                    }
                } else {
                    // If type doesn't match, still try to bind with unknown types
                    for pat in patterns {
                        self.register_pattern_bindings(pat, &ResolvedType::Unknown)?;
                    }
                }
                Ok(())
            }
            Pattern::Struct { fields, .. } => {
                // For struct patterns, each field name becomes a binding
                for (field_name, sub_pattern) in fields {
                    if let Some(sub_pat) = sub_pattern {
                        self.register_pattern_bindings(sub_pat, &ResolvedType::Unknown)?;
                    } else {
                        // Shorthand: `Point { x, y }` binds x and y
                        self.define_var(&field_name.node, ResolvedType::Unknown, false);
                    }
                }
                Ok(())
            }
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    self.register_pattern_bindings(field, &ResolvedType::Unknown)?;
                }
                Ok(())
            }
            Pattern::Range { .. } => Ok(()), // Ranges don't bind variables
            Pattern::Or(patterns) => {
                // For or patterns, all patterns must bind the same variables
                // For now, just process the first one
                if let Some(first) = patterns.first() {
                    self.register_pattern_bindings(first, expr_type)?;
                }
                Ok(())
            }
        }
    }

    fn lookup_var(&self, name: &str) -> Option<ResolvedType> {
        self.lookup_var_info(name).ok().map(|v| v.ty)
    }

    fn lookup_var_with_mut(&self, name: &str) -> Option<(ResolvedType, bool)> {
        self.lookup_var_info(name).ok().map(|v| (v.ty, v.is_mut))
    }

    fn lookup_var_or_err(&self, name: &str) -> TypeResult<ResolvedType> {
        self.lookup_var_info(name).map(|v| v.ty)
    }

    fn lookup_var_info(&self, name: &str) -> TypeResult<VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Ok(info.clone());
            }
        }

        // Check if it's a function
        if let Some(sig) = self.functions.get(name) {
            return Ok(VarInfo {
                ty: ResolvedType::Fn {
                    params: sig.params.iter().map(|(_, t, _)| t.clone()).collect(),
                    ret: Box::new(sig.ret.clone()),
                },
                is_mut: false,
            });
        }

        Err(TypeError::UndefinedVar(name.to_string()))
    }

    /// Find free variables in an expression that are not in bound_vars
    fn find_free_vars_in_expr(&self, expr: &Spanned<Expr>, bound_vars: &std::collections::HashSet<String>) -> Vec<String> {
        let mut free_vars = Vec::new();
        self.collect_free_vars(&expr.node, bound_vars, &mut free_vars);
        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        free_vars.retain(|v| seen.insert(v.clone()));
        free_vars
    }

    fn collect_free_vars(&self, expr: &Expr, bound: &std::collections::HashSet<String>, free: &mut Vec<String>) {
        match expr {
            Expr::Ident(name) => {
                if !bound.contains(name) && self.lookup_var(name).is_some() {
                    free.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_free_vars(&left.node, bound, free);
                self.collect_free_vars(&right.node, bound, free);
            }
            Expr::Unary { expr, .. } => {
                self.collect_free_vars(&expr.node, bound, free);
            }
            Expr::Call { func, args } => {
                self.collect_free_vars(&func.node, bound, free);
                for arg in args {
                    self.collect_free_vars(&arg.node, bound, free);
                }
            }
            Expr::If { cond, then, else_ } => {
                self.collect_free_vars(&cond.node, bound, free);
                // then is Vec<Spanned<Stmt>>
                let mut local_bound = bound.clone();
                for stmt in then {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
                if let Some(else_br) = else_ {
                    self.collect_if_else_free_vars(else_br, bound, free);
                }
            }
            Expr::Block(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_free_vars(&receiver.node, bound, free);
                for arg in args {
                    self.collect_free_vars(&arg.node, bound, free);
                }
            }
            Expr::Field { expr, .. } => {
                self.collect_free_vars(&expr.node, bound, free);
            }
            Expr::Index { expr, index } => {
                self.collect_free_vars(&expr.node, bound, free);
                self.collect_free_vars(&index.node, bound, free);
            }
            Expr::Array(elems) => {
                for e in elems {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::StructLit { fields, .. } => {
                for (_, e) in fields {
                    self.collect_free_vars(&e.node, bound, free);
                }
            }
            Expr::Assign { target, value } => {
                self.collect_free_vars(&target.node, bound, free);
                self.collect_free_vars(&value.node, bound, free);
            }
            Expr::AssignOp { target, value, .. } => {
                self.collect_free_vars(&target.node, bound, free);
                self.collect_free_vars(&value.node, bound, free);
            }
            Expr::Lambda { params, body, .. } => {
                let mut inner_bound = bound.clone();
                for p in params {
                    inner_bound.insert(p.name.node.clone());
                }
                self.collect_free_vars(&body.node, &inner_bound, free);
            }
            Expr::Ref(inner) | Expr::Deref(inner) |
            Expr::Try(inner) | Expr::Unwrap(inner) | Expr::Await(inner) |
            Expr::Spawn(inner) => {
                self.collect_free_vars(&inner.node, bound, free);
            }
            Expr::Loop { body, pattern, iter } => {
                // iter expression runs in current scope
                if let Some(it) = iter {
                    self.collect_free_vars(&it.node, bound, free);
                }
                // body is Vec<Spanned<Stmt>>, pattern may introduce bindings
                let mut local_bound = bound.clone();
                if let Some(pat) = pattern {
                    self.collect_pattern_bindings(&pat.node, &mut local_bound);
                }
                for stmt in body {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
            Expr::Match { expr, arms } => {
                self.collect_free_vars(&expr.node, bound, free);
                for arm in arms {
                    // Pattern bindings create new scope
                    let mut arm_bound = bound.clone();
                    self.collect_pattern_bindings(&arm.pattern.node, &mut arm_bound);
                    if let Some(guard) = &arm.guard {
                        self.collect_free_vars(&guard.node, &arm_bound, free);
                    }
                    self.collect_free_vars(&arm.body.node, &arm_bound, free);
                }
            }
            // Literals and other expressions don't contain free variables
            _ => {}
        }
    }

    fn collect_pattern_bindings(&self, pattern: &Pattern, bound: &mut std::collections::HashSet<String>) {
        match pattern {
            Pattern::Ident(name) => { bound.insert(name.clone()); }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, pat) in fields {
                    if let Some(p) = pat {
                        self.collect_pattern_bindings(&p.node, bound);
                    }
                }
            }
            Pattern::Variant { fields, .. } => {
                for p in fields {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    self.collect_pattern_bindings(&p.node, bound);
                }
            }
            _ => {}
        }
    }

    fn collect_if_else_free_vars(&self, if_else: &IfElse, bound: &std::collections::HashSet<String>, free: &mut Vec<String>) {
        match if_else {
            IfElse::ElseIf(cond, then_stmts, else_) => {
                self.collect_free_vars(&cond.node, bound, free);
                let mut local_bound = bound.clone();
                for stmt in then_stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
                if let Some(else_br) = else_ {
                    self.collect_if_else_free_vars(else_br, bound, free);
                }
            }
            IfElse::Else(stmts) => {
                let mut local_bound = bound.clone();
                for stmt in stmts {
                    match &stmt.node {
                        Stmt::Let { name, value, .. } => {
                            self.collect_free_vars(&value.node, &local_bound, free);
                            local_bound.insert(name.node.clone());
                        }
                        Stmt::Expr(e) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Return(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        Stmt::Break(Some(e)) => self.collect_free_vars(&e.node, &local_bound, free),
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_parser::parse;

    #[test]
    fn test_simple_function() {
        let source = "F add(a:i64,b:i64)->i64=a+b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "F add(a:i64,b:str)->i64=a+b";
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_struct() {
        let source = r#"
            S Point{x:f64,y:f64}
            F make_point()->Point=Point{x:1.0,y:2.0}
        "#;
        let module = parse(source).unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }
}
