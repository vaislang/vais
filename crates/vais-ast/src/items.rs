//! Top-level module and item definitions

use crate::aggregate_types::{Enum, Struct, Union};
use crate::aliases::{TraitAlias, TypeAlias};
use crate::constants::{ConstDef, GlobalDef};
use crate::extern_block::ExternBlock;
use crate::function::Function;
use crate::infrastructure::Spanned;
use crate::macros::MacroDef;
use crate::traits::{Impl, Trait};

/// Top-level module containing all program items.
///
/// A module represents a complete Vais source file and contains
/// all top-level definitions (functions, structs, enums, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// List of top-level items in this module
    pub items: Vec<Spanned<Item>>,
    /// Per-module item indices (module_path → item indices in `items`)
    /// Populated during import resolution for per-module codegen.
    /// None when not using per-module compilation.
    pub modules_map: Option<std::collections::HashMap<std::path::PathBuf, Vec<usize>>>,
}

/// Top-level item definitions in a module.
///
/// Represents the various kinds of declarations that can appear at module level.
/// Vais uses single-letter keywords for token efficiency.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Function definition: `F name(params)->ret=expr` or `F name(params)->ret{...}`
    Function(Function),
    /// Struct definition: `S Name{fields}`
    Struct(Struct),
    /// Enum definition: `E Name{variants}`
    Enum(Enum),
    /// Union definition: `O Name{fields}` (untagged, C-style)
    Union(Union),
    /// Type alias: `T Name=Type`
    TypeAlias(TypeAlias),
    /// Trait alias: `T Name = TraitA + TraitB`
    TraitAlias(TraitAlias),
    /// Import statement: `U module` or `U module::{items}`
    Use(Use),
    /// Trait definition: `W Name { methods }` (W = "What" interface)
    Trait(Trait),
    /// Implementation block: `X Type: Trait { methods }` (X = "eXtend")
    Impl(Impl),
    /// Macro definition: `macro name! { rules }`
    Macro(MacroDef),
    /// Extern block: `N "C" { declarations }`
    ExternBlock(ExternBlock),
    /// Constant definition: `C NAME: Type = value`
    Const(ConstDef),
    /// Global variable definition: `G name: Type = value`
    Global(GlobalDef),
    /// Error recovery node - represents an item that failed to parse
    /// Used for continuing parsing after errors to report multiple errors at once.
    Error {
        /// Error message describing what went wrong
        message: String,
        /// Tokens that were skipped during recovery
        skipped_tokens: Vec<String>,
    },
}

/// Use/Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Vec<Spanned<String>>,
    pub alias: Option<Spanned<String>>,
    /// Selective import items: `U mod.Item` or `U mod.{A, B}`
    /// None means import the entire module (wildcard)
    pub items: Option<Vec<Spanned<String>>>,
}
