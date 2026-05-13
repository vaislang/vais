//! Core WIT (WebAssembly Interface Types) type definitions

use std::fmt;

/// WIT (WebAssembly Interface Types) representation
#[derive(Debug, Clone, PartialEq)]
pub enum WitType {
    /// Primitive types
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
    Char,
    String,

    /// Container types
    List(Box<WitType>),
    Option_(Box<WitType>),
    Result_ {
        ok: Option<Box<WitType>>,
        err: Option<Box<WitType>>,
    },
    Tuple(Vec<WitType>),

    /// Named types
    Record(String),
    Variant(String),
    Enum(String),
    Flags(String),
    Resource(String),

    /// Custom types
    Named(String),
}

impl fmt::Display for WitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitType::Bool => write!(f, "bool"),
            WitType::U8 => write!(f, "u8"),
            WitType::U16 => write!(f, "u16"),
            WitType::U32 => write!(f, "u32"),
            WitType::U64 => write!(f, "u64"),
            WitType::S8 => write!(f, "s8"),
            WitType::S16 => write!(f, "s16"),
            WitType::S32 => write!(f, "s32"),
            WitType::S64 => write!(f, "s64"),
            WitType::F32 => write!(f, "f32"),
            WitType::F64 => write!(f, "f64"),
            WitType::Char => write!(f, "char"),
            WitType::String => write!(f, "string"),
            WitType::List(inner) => write!(f, "list<{}>", inner),
            WitType::Option_(inner) => write!(f, "option<{}>", inner),
            WitType::Result_ {
                ok: None,
                err: None,
            } => write!(f, "result"),
            WitType::Result_ {
                ok: Some(ok),
                err: None,
            } => write!(f, "result<{}>", ok),
            WitType::Result_ {
                ok: None,
                err: Some(err),
            } => write!(f, "result<_, {}>", err),
            WitType::Result_ {
                ok: Some(ok),
                err: Some(err),
            } => write!(f, "result<{}, {}>", ok, err),
            WitType::Tuple(types) => {
                write!(f, "tuple<")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ">")
            }
            WitType::Record(name) => write!(f, "{}", name),
            WitType::Variant(name) => write!(f, "{}", name),
            WitType::Enum(name) => write!(f, "{}", name),
            WitType::Flags(name) => write!(f, "{}", name),
            WitType::Resource(name) => write!(f, "{}", name),
            WitType::Named(name) => write!(f, "{}", name),
        }
    }
}

/// WIT record field
#[derive(Debug, Clone)]
pub struct WitField {
    pub name: String,
    pub ty: WitType,
    pub docs: Option<String>,
}

/// WIT record definition
#[derive(Debug, Clone)]
pub struct WitRecord {
    pub name: String,
    pub fields: Vec<WitField>,
    pub docs: Option<String>,
}

/// WIT variant case
#[derive(Debug, Clone)]
pub struct WitVariantCase {
    pub name: String,
    pub ty: Option<WitType>,
    pub docs: Option<String>,
}

/// WIT variant definition
#[derive(Debug, Clone)]
pub struct WitVariant {
    pub name: String,
    pub cases: Vec<WitVariantCase>,
    pub docs: Option<String>,
}

/// WIT enum case
#[derive(Debug, Clone)]
pub struct WitEnumCase {
    pub name: String,
    pub docs: Option<String>,
}

/// WIT enum definition
#[derive(Debug, Clone)]
pub struct WitEnum {
    pub name: String,
    pub cases: Vec<WitEnumCase>,
    pub docs: Option<String>,
}

/// WIT flags definition
#[derive(Debug, Clone)]
pub struct WitFlags {
    pub name: String,
    pub flags: Vec<String>,
    pub docs: Option<String>,
}

/// WIT function parameter
#[derive(Debug, Clone)]
pub struct WitParam {
    pub name: String,
    pub ty: WitType,
}

/// WIT function result
#[derive(Debug, Clone)]
pub enum WitResult {
    Named(Vec<WitParam>),
    Anon(WitType),
}

/// WIT function definition
#[derive(Debug, Clone)]
pub struct WitFunction {
    pub name: String,
    pub params: Vec<WitParam>,
    pub results: Option<WitResult>,
    pub docs: Option<String>,
}

/// WIT type definition
#[derive(Debug, Clone)]
pub enum WitTypeDefinition {
    Record(WitRecord),
    Variant(WitVariant),
    Enum(WitEnum),
    Flags(WitFlags),
    Type { name: String, ty: WitType },
}
