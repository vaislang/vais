//! Type expressions (named ast_types to avoid conflict with the `types` module name)

use crate::infrastructure::Spanned;
use crate::constants::ConstExpr;
use crate::expressions::Expr;

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type: `i64`, `String`, `Vec<T>`
    Named {
        name: String,
        generics: Vec<Spanned<Type>>,
    },
    /// Function pointer type: `fn(A, B) -> C`
    FnPtr {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
        is_vararg: bool,
    },
    /// Array: `[T]`
    Array(Box<Spanned<Type>>),
    /// Const-sized array: `[T; N]` where N is a const expression
    ConstArray {
        element: Box<Spanned<Type>>,
        size: ConstExpr,
    },
    /// Map: `[K:V]`
    Map(Box<Spanned<Type>>, Box<Spanned<Type>>),
    /// Tuple: `(T1, T2, ...)`
    Tuple(Vec<Spanned<Type>>),
    /// Optional: `T?`
    Optional(Box<Spanned<Type>>),
    /// Result: `T!`
    Result(Box<Spanned<Type>>),
    /// Pointer: `*T`
    Pointer(Box<Spanned<Type>>),
    /// Reference: `&T` or `&'a T` (with lifetime)
    Ref(Box<Spanned<Type>>),
    /// Mutable reference: `&mut T` or `&'a mut T` (with lifetime)
    RefMut(Box<Spanned<Type>>),
    /// Immutable slice: `&[T]` — fat pointer (ptr, len)
    Slice(Box<Spanned<Type>>),
    /// Mutable slice: `&mut [T]` — fat pointer (ptr, len)
    SliceMut(Box<Spanned<Type>>),
    /// Reference with explicit lifetime: `&'a T`
    RefLifetime {
        lifetime: String,
        inner: Box<Spanned<Type>>,
    },
    /// Mutable reference with explicit lifetime: `&'a mut T`
    RefMutLifetime {
        lifetime: String,
        inner: Box<Spanned<Type>>,
    },
    /// Lazy type: `Lazy<T>` - Deferred evaluation thunk
    Lazy(Box<Spanned<Type>>),
    /// Function type: `(A,B)->C`
    Fn {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
    },
    /// Unit type: `()`
    Unit,
    /// Inferred type (for internal use)
    Infer,
    /// Dynamic trait object: `dyn Trait` or `dyn Trait<T>`
    /// Used for runtime polymorphism via vtable dispatch.
    DynTrait {
        trait_name: String,
        generics: Vec<Spanned<Type>>,
    },
    /// Associated type: `<T as Trait>::Item` or `Self::Item`
    /// GAT support: `<T as Trait>::Item<'a, B>` with generic arguments
    Associated {
        base: Box<Spanned<Type>>,
        trait_name: Option<String>, // None for Self::Item
        assoc_name: String,
        /// GAT generic arguments (e.g., <'a, i64> in Self::Item<'a, i64>)
        generics: Vec<Spanned<Type>>,
    },
    /// Linear type: `linear T` - must be used exactly once
    Linear(Box<Spanned<Type>>),
    /// Affine type: `affine T` - can be used at most once
    Affine(Box<Spanned<Type>>),
    /// Existential type: `X Trait` or `X Trait + Trait2` in return position
    /// Represents an opaque return type that implements the given trait bounds.
    /// Resolved to concrete type during monomorphization.
    ImplTrait { bounds: Vec<Spanned<String>> },
    /// Dependent type (Refinement type): `{x: T | predicate}`
    /// A type `T` refined by a predicate that must hold for all values.
    /// Example: `{n: i64 | n > 0}` (positive integers)
    Dependent {
        /// The bound variable name (e.g., "n" in {n: i64 | n > 0})
        var_name: String,
        /// The base type being refined
        base: Box<Spanned<Type>>,
        /// The predicate expression that must evaluate to bool
        predicate: Box<Spanned<Expr>>,
    },
}
