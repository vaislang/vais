# Vais Compiler Architecture

**Version**: 0.0.1
**Last Updated**: 2026-01-21
**Status**: Production-Ready

## Table of Contents

1. [Overview](#overview)
2. [Compiler Pipeline](#compiler-pipeline)
3. [Crate Architecture](#crate-architecture)
4. [Data Flow Diagram](#data-flow-diagram)
5. [Core Design Decisions](#core-design-decisions)
6. [Type System and Inference](#type-system-and-inference)
7. [Generic Monomorphization](#generic-monomorphization)
8. [Code Generation Strategy](#code-generation-strategy)
9. [Optimization Pipeline](#optimization-pipeline)
10. [Developer Tools](#developer-tools)
11. [Plugin System](#plugin-system)
12. [LSP Architecture](#lsp-architecture)
13. [Internationalization](#internationalization)
14. [Testing Strategy](#testing-strategy)
15. [Performance Considerations](#performance-considerations)

---

## Overview

Vais is an AI-optimized systems programming language designed for token efficiency and native performance. The compiler is built in Rust using a multi-phase pipeline architecture, targeting LLVM IR for native code generation.

### Key Features

- **Token-efficient syntax**: Single-letter keywords (F, S, E, I, L, M, etc.)
- **Expression-oriented**: Everything is an expression, reducing boilerplate
- **Static typing with inference**: Hindley-Milner type inference minimizes annotations
- **Native performance**: LLVM backend generates optimized machine code
- **Modern features**: Generics, traits, async/await, closures, pattern matching
- **Developer-friendly**: LSP, REPL, formatter, debugger, comprehensive tooling

### Technology Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust 2021 Edition |
| Lexer | [logos](https://github.com/maciejhirsz/logos) (v0.14) |
| Parser | Recursive descent (hand-written) |
| Type System | Hindley-Milner with extensions |
| Backend | LLVM IR → Clang |
| LSP | tower-lsp framework |
| Testing | cargo test + Criterion benchmarks |

---

## Compiler Pipeline

The Vais compiler uses a traditional multi-phase pipeline with clear separation of concerns:

```
┌──────────────────────────────────────────────────────────────────┐
│                        Source Code (.vais)                        │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 1: LEXICAL ANALYSIS                                         │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ vais-lexer                                                  │   │
│ │ • logos-based tokenization                                  │   │
│ │ • Single-letter keyword recognition                         │   │
│ │ • Span tracking for error reporting                         │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Vec<SpannedToken>
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 2: SYNTAX ANALYSIS                                          │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ vais-parser                                                 │   │
│ │ • Recursive descent parsing                                 │   │
│ │ • AST construction (vais-ast)                               │   │
│ │ • Expression-first grammar                                  │   │
│ │ • Error recovery with span information                      │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Module (AST)
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Plugin Hook: Transform Plugins                                    │
│ • Modify AST before type checking                                 │
│ • Custom syntax desugaring                                        │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 3: SEMANTIC ANALYSIS                                        │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ vais-types                                                  │   │
│ │ • Hindley-Milner type inference                             │   │
│ │ • Generic type parameter resolution                         │   │
│ │ • Trait constraint checking                                 │   │
│ │ • Pattern exhaustiveness checking                           │   │
│ │ • Generic instantiation tracking                            │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Typed AST + GenericInstantiations
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 4: CODE GENERATION                                          │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ vais-codegen                                                │   │
│ │ • LLVM IR text generation                                   │   │
│ │ • Monomorphization (generic specialization)                 │   │
│ │ • Memory management code insertion                          │   │
│ │ • DWARF debug metadata generation (optional)                │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  │ LLVM IR (.ll)
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 5: OPTIMIZATION                                             │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ vais-codegen/optimize                                       │   │
│ │ • Constant folding                                          │   │
│ │ • Dead code elimination                                     │   │
│ │ • Common subexpression elimination                          │   │
│ │ • Loop invariant code motion                                │   │
│ │ • Function inlining (O3)                                    │   │
│ │ • Strength reduction                                        │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  │ Optimized LLVM IR
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Plugin Hook: Optimize Plugins                                     │
│ • Custom LLVM IR optimizations                                    │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│ Phase 6: NATIVE CODE GENERATION                                   │
│ ┌────────────────────────────────────────────────────────────┐   │
│ │ clang (external)                                            │   │
│ │ • LLVM IR → object code                                     │   │
│ │ • Platform-specific optimizations                           │   │
│ │ • Linking with standard library                             │   │
│ └────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Native Binary (a.out)                        │
└──────────────────────────────────────────────────────────────────┘
```

### Pipeline Characteristics

- **Single-pass type checking**: No need for multiple passes due to forward declarations
- **Lazy monomorphization**: Only instantiate generic functions that are actually called
- **Incremental error reporting**: Each phase reports errors with source locations
- **Memory efficiency**: Streaming token processing, no backtracking

---

## Crate Architecture

The Vais compiler is organized into 9 workspace crates, each with a single responsibility:

```
vais/
├── crates/
│   ├── vais-ast/          # Abstract Syntax Tree definitions
│   ├── vais-lexer/        # Tokenization (logos-based)
│   ├── vais-parser/       # Recursive descent parser
│   ├── vais-types/        # Type checker & inference engine
│   ├── vais-codegen/      # LLVM IR generator
│   ├── vais-lsp/          # Language Server Protocol
│   ├── vais-i18n/         # Internationalization
│   ├── vais-plugin/       # Plugin system infrastructure
│   └── vaisc/             # CLI compiler & REPL
├── std/                   # Standard library (.vais files)
├── benches/               # Criterion performance benchmarks
└── examples/              # Example programs & plugins
```

### Crate Dependency Graph

```
        ┌─────────────┐
        │  vais-ast   │  (Core data structures)
        └──────┬──────┘
               │
       ┌───────┴──────────┬──────────────┐
       ▼                  ▼              ▼
┌─────────────┐    ┌──────────┐   ┌────────────┐
│ vais-lexer  │    │ vais-i18n│   │vais-plugin │
└──────┬──────┘    └─────┬────┘   └─────┬──────┘
       │                 │              │
       ▼                 │              │
┌─────────────┐          │              │
│vais-parser  │          │              │
└──────┬──────┘          │              │
       │                 │              │
       ▼                 │              │
┌─────────────┐          │              │
│ vais-types  │◄─────────┘              │
└──────┬──────┘                         │
       │                                │
       ▼                                │
┌─────────────┐                         │
│vais-codegen │                         │
└──────┬──────┘                         │
       │                                │
       └────────┬────────┬──────────────┘
                │        │              │
                ▼        ▼              ▼
            ┌────────┐ ┌─────┐    ┌────────┐
            │ vaisc  │ │ LSP │    │ Plugin │
            └────────┘ └─────┘    └────────┘
```

### Crate Descriptions

#### 1. **vais-ast** (Core Data Structures)

**Lines of Code**: ~800
**Dependencies**: None (foundational crate)

Defines all AST node types and structures:

```rust
pub struct Module { pub items: Vec<Spanned<Item>> }
pub enum Item { Function(Function), Struct(Struct), Enum(Enum), ... }
pub struct Function { pub name: String, pub params: Vec<Param>, ... }
pub struct Span { pub start: usize, pub end: usize }
```

**Key types**:
- `Module`, `Item`, `Function`, `Struct`, `Enum`, `Trait`, `Impl`
- `Expr`, `Stmt`, `Pattern`, `Type`
- `Span`, `Spanned<T>` for source location tracking
- `Attribute` for metadata annotations

**Design rationale**:
- Zero dependencies for fast compilation
- Rich span information for error reporting
- Expression-oriented: statements are expressions

#### 2. **vais-lexer** (Tokenization)

**Lines of Code**: ~600
**Dependencies**: logos (v0.14)

Tokenizes source code into a stream of classified tokens:

```rust
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[token("F", priority = 3)] Function,
    #[token("S", priority = 3)] Struct,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Ident(String),
    #[regex(r"\d+")] IntLit(i64),
    // ... 60+ token types
}

pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
```

**Features**:
- Single-letter keyword tokenization (F=function, S=struct, etc.)
- Regex-based patterns for identifiers, literals, operators
- Priority-based disambiguation (keywords > identifiers)
- Zero-copy string slicing via spans
- Comment and whitespace skipping

**Performance**: ~5µs per 100 tokens (logos state machine)

#### 3. **vais-parser** (Syntax Analysis)

**Lines of Code**: ~2,700
**Dependencies**: vais-ast, vais-lexer, vais-i18n

Recursive descent parser with single-token lookahead:

```rust
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn parse(&mut self) -> ParseResult<Module> { ... }
    fn parse_function(&mut self) -> ParseResult<Function> { ... }
    fn parse_expr(&mut self) -> ParseResult<Expr> { ... }
    fn parse_expr_with_precedence(&mut self, min_prec: u8) -> ParseResult<Expr> { ... }
}
```

**Key algorithms**:
- **Pratt parsing** for expressions (operator precedence climbing)
- **Predictive parsing** for statements and declarations
- **Error recovery** with synchronization tokens (`;`, `}`, `EOF`)

**Grammar characteristics**:
- LL(1) with local LL(2) for disambiguation
- Expression-first: `F add(a,b)=a+b` (expression form)
- Block-optional: `F add(a,b){R a+b}` (block form)

#### 4. **vais-types** (Type System)

**Lines of Code**: ~3,400
**Dependencies**: vais-ast, vais-i18n

Implements Hindley-Milner type inference with extensions:

```rust
pub struct TypeChecker {
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, EnumDef>,
    functions: HashMap<String, FunctionSig>,
    traits: HashMap<String, TraitDef>,
    trait_impls: Vec<TraitImpl>,

    scopes: Vec<HashMap<String, VarInfo>>,
    current_generics: Vec<String>,
    substitutions: HashMap<usize, ResolvedType>,

    // Monomorphization tracking
    generic_instantiations: Vec<GenericInstantiation>,
}
```

**Type inference algorithm**:
1. **Constraint generation**: Walk AST, generate type equations
2. **Unification**: Solve equations using Robinson's algorithm
3. **Substitution**: Apply solved types back to AST
4. **Generalization**: Convert inferred types to generic schemes

**Extensions beyond H-M**:
- **Trait constraints**: `F sort<T:Ord>(xs:Vec<T>)`
- **Associated types**: `W Iterator{T Item; F next()->Option<Item>}`
- **Generic bounds**: Multi-trait bounds, lifetime elision
- **Pattern exhaustiveness**: Ensures match arms cover all cases

**Monomorphization**:
- Tracks generic instantiations during type checking
- Records `GenericInstantiation { base_name, type_args, mangled_name }`
- Defers code generation until all type parameters are known

#### 5. **vais-codegen** (Code Generation)

**Lines of Code**: ~4,800
**Dependencies**: vais-ast, vais-types

Generates LLVM IR text from typed AST:

```rust
pub struct CodeGenerator {
    module_name: String,
    functions: HashMap<String, FunctionInfo>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,

    locals: HashMap<String, LocalVar>,
    label_counter: usize,
    loop_stack: Vec<LoopLabels>,
    string_constants: Vec<(String, String)>,

    // Generic support
    generic_instantiations: Vec<GenericInstantiation>,
    specialized_functions: HashSet<String>,

    // Debug info
    debug_builder: Option<DebugInfoBuilder>,
}
```

**Code generation strategy**:
- **SSA form**: All values are immutable (LLVM requirement)
- **Stack allocation**: Local variables via `alloca` in entry block
- **Heap allocation**: Explicit `malloc`/`free` calls (no GC)
- **Calling convention**: C ABI for FFI compatibility

**Modules** (internal organization):
- `lib.rs`: Main generator, module/function generation
- `types.rs`: Type conversion (Vais → LLVM)
- `expr.rs`: Expression code generation
- `stmt.rs`: Statement code generation
- `builtins.rs`: Built-in functions (print, malloc, etc.)
- `optimize.rs`: IR optimization passes
- `debug.rs`: DWARF debug info generation
- `formatter.rs`: AST pretty-printing (for `vaisc fmt`)

#### 6. **vais-lsp** (Language Server)

**Lines of Code**: ~1,300
**Dependencies**: vais-ast, vais-parser, vais-types, tower-lsp

Implements Language Server Protocol for IDE integration:

```rust
pub struct VaisLanguageServer {
    client: Client,
    symbol_cache: Arc<Mutex<SymbolCache>>,
    document_map: Arc<Mutex<HashMap<Url, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for VaisLanguageServer {
    async fn initialize(...) -> Result<InitializeResult> { ... }
    async fn did_change(...) { ... }
    async fn completion(...) -> Result<CompletionResponse> { ... }
    async fn goto_definition(...) -> Result<GotoDefinitionResponse> { ... }
    async fn hover(...) -> Result<Hover> { ... }
    async fn references(...) -> Result<Vec<Location>> { ... }
    async fn rename(...) -> Result<WorkspaceEdit> { ... }
}
```

**Features**:
- **Diagnostics**: Real-time syntax and type errors
- **Auto-completion**: Keywords, types, functions, methods
- **Go-to-definition**: Jump to declaration
- **Hover**: Type information and documentation
- **Find references**: All uses of a symbol
- **Rename**: Safe refactoring across files
- **Semantic tokens**: Syntax highlighting

**Performance optimizations**:
- **Symbol indexing cache**: Avoid re-parsing unchanged files
- **Incremental parsing**: Only re-analyze changed ranges
- **Async processing**: Non-blocking request handling

#### 7. **vais-i18n** (Internationalization)

**Lines of Code**: ~350
**Dependencies**: serde_json, once_cell

JSON-based localization for error messages:

```rust
pub struct I18nEngine {
    locale: Locale,
    messages: HashMap<String, String>,
}

pub fn get(key: &str, vars: &[(&str, &str)]) -> String {
    ENGINE.with(|engine| engine.translate(key, vars))
}
```

**Supported languages**:
- English (en) - default
- Korean (ko)
- Japanese (ja)

**Message format** (locales/en.json):
```json
{
  "type.E001.title": "Type Mismatch",
  "type.E001.message": "Expected type {expected}, found {found}",
  "type.E001.help": "Try converting {found} to {expected}"
}
```

**CLI integration**:
```bash
vaisc --locale ko check file.vais
export VAIS_LANG=ja && vaisc build file.vais
```

#### 8. **vais-plugin** (Plugin System)

**Lines of Code**: ~800
**Dependencies**: libloading, vais-ast, serde

Dynamic plugin loading infrastructure:

```rust
pub trait Plugin {
    fn info(&self) -> PluginInfo;
    fn as_any(&self) -> &dyn Any;
}

pub enum PluginType {
    Lint,       // Check code, return diagnostics
    Transform,  // Modify AST before type checking
    Optimize,   // Custom LLVM IR passes
    Codegen,    // Generate additional files
}

pub struct PluginRegistry {
    plugins: Vec<LoadedPlugin>,
}

impl PluginRegistry {
    pub fn run_lint(&self, module: &Module) -> Vec<Diagnostic> { ... }
    pub fn run_transform(&self, module: Module) -> Result<Module> { ... }
    pub fn run_optimize(&self, ir: &str, level: OptLevel) -> Result<String> { ... }
}
```

**Configuration** (vais-plugins.toml):
```toml
[plugins]
path = ["./plugins/naming-convention.dylib"]

[plugins.config]
naming-convention = { enforce_snake_case = true }
```

**Plugin lifecycle**:
1. **Discovery**: Load from `vais-plugins.toml` or `--plugin` flag
2. **Initialization**: Call `create_plugin()` FFI function
3. **Execution**: Call appropriate trait method
4. **Cleanup**: Drop on compiler exit

#### 9. **vaisc** (CLI & REPL)

**Lines of Code**: ~1,400
**Dependencies**: All above crates, clap, rustyline

Command-line interface and interactive REPL:

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

enum Commands {
    Build { input: PathBuf, opt_level: u8, debug: bool },
    Run { input: PathBuf, args: Vec<String> },
    Check { input: PathBuf },
    Repl,
    Fmt { input: PathBuf, check: bool, indent: usize },
    Doc { input: PathBuf, output: PathBuf, format: String },
}
```

**Subcommands**:
- `vaisc build file.vais -O2 -g`: Compile with optimization + debug info
- `vaisc run file.vais -- arg1 arg2`: Compile and execute
- `vaisc check file.vais`: Type-check only (no codegen)
- `vaisc repl`: Interactive REPL with history
- `vaisc fmt file.vais --check`: Format checking
- `vaisc doc std/ -o docs/`: Documentation generation

**REPL features**:
- **Multi-line input**: Bracket/brace balancing
- **History**: Arrow keys, persistent across sessions
- **Tab completion**: Keywords + built-in functions
- **Commands**: `:help`, `:clear`, `:load`, `:quit`

---

## Data Flow Diagram

### Complete Data Flow (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Source: "F add(a,b)=a+b"                      │
└──────────────────────────────────────┬──────────────────────────────────┘
                                       │
                                       │ String
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ LEXER (vais-lexer)                                                        │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ Input: "F add(a,b)=a+b"                                               │ │
│ │                                                                       │ │
│ │ Logos State Machine:                                                  │ │
│ │   'F' → Token::Function       @ span 0..1                             │ │
│ │   ' ' → skip (whitespace)                                             │ │
│ │   'add' → Token::Ident("add") @ span 2..5                             │ │
│ │   '(' → Token::LParen         @ span 5..6                             │ │
│ │   'a' → Token::Ident("a")     @ span 6..7                             │ │
│ │   ',' → Token::Comma          @ span 7..8                             │ │
│ │   'b' → Token::Ident("b")     @ span 8..9                             │ │
│ │   ')' → Token::RParen         @ span 9..10                            │ │
│ │   '=' → Token::Eq             @ span 10..11                           │ │
│ │   'a' → Token::Ident("a")     @ span 11..12                           │ │
│ │   '+' → Token::Plus           @ span 12..13                           │ │
│ │   'b' → Token::Ident("b")     @ span 13..14                           │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ Vec<SpannedToken>
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ PARSER (vais-parser)                                                      │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ parse_module()                                                        │ │
│ │   ├─ parse_item()                                                     │ │
│ │   │   ├─ expect(Token::Function)                                      │ │
│ │   │   ├─ parse_ident() → "add"                                        │ │
│ │   │   ├─ parse_params() → [Param{name:"a"}, Param{name:"b"}]         │ │
│ │   │   ├─ parse_type_annotation() → None (inferred)                    │ │
│ │   │   └─ parse_expr() → BinaryOp {                                    │ │
│ │   │         op: Plus,                                                 │ │
│ │   │         left: Variable("a"),                                      │ │
│ │   │         right: Variable("b")                                      │ │
│ │   │       }                                                           │ │
│ │   │                                                                   │ │
│ │ Output AST:                                                           │ │
│ │   Module {                                                            │ │
│ │     items: [                                                          │ │
│ │       Spanned {                                                       │ │
│ │         node: Item::Function(Function {                               │ │
│ │           name: "add",                                                │ │
│ │           params: [                                                   │ │
│ │             Param { name: "a", ty: Type::Infer },                     │ │
│ │             Param { name: "b", ty: Type::Infer }                      │ │
│ │           ],                                                          │ │
│ │           ret_type: Type::Infer,                                      │ │
│ │           body: Expr::BinaryOp { ... }                                │ │
│ │         }),                                                           │ │
│ │         span: 0..14                                                   │ │
│ │       }                                                               │ │
│ │     ]                                                                 │ │
│ │   }                                                                   │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ Module (AST)
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ TYPE CHECKER (vais-types)                                                 │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ check_module()                                                        │ │
│ │   ├─ check_function("add")                                            │ │
│ │   │   ├─ fresh_type_var() → ?T0, ?T1 for params                       │ │
│ │   │   ├─ check_expr(BinaryOp):                                        │ │
│ │   │   │   ├─ check_expr(Variable("a")) → ?T0                          │ │
│ │   │   │   ├─ check_expr(Variable("b")) → ?T1                          │ │
│ │   │   │   ├─ unify(?T0, i64) [Plus requires numeric]                 │ │
│ │   │   │   ├─ unify(?T1, i64)                                          │ │
│ │   │   │   └─ return i64                                               │ │
│ │   │   └─ unify(ret_type, i64)                                         │ │
│ │   │                                                                   │ │
│ │ Resolved Types:                                                       │ │
│ │   Function "add":                                                     │ │
│ │     params: [i64, i64]                                                │ │
│ │     returns: i64                                                      │ │
│ │                                                                       │ │
│ │ Output:                                                               │ │
│ │   FunctionSig {                                                       │ │
│ │     name: "add",                                                      │ │
│ │     params: [ResolvedType::I64, ResolvedType::I64],                  │ │
│ │     ret: ResolvedType::I64,                                           │ │
│ │     generics: []                                                      │ │
│ │   }                                                                   │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ Typed AST + FunctionSigs
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ CODE GENERATOR (vais-codegen)                                             │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ generate_module()                                                     │ │
│ │   ├─ emit_prelude() → declare built-ins (malloc, printf, etc.)       │ │
│ │   ├─ generate_function("add"):                                        │ │
│ │   │   ├─ emit: "define i64 @add(i64 %a, i64 %b) {"                   │ │
│ │   │   ├─ generate_expr(BinaryOp):                                     │ │
│ │   │   │   ├─ generate_expr(Variable("a")) → "%a"                      │ │
│ │   │   │   ├─ generate_expr(Variable("b")) → "%b"                      │ │
│ │   │   │   ├─ emit: "%0 = add i64 %a, %b"                              │ │
│ │   │   │   └─ return "%0"                                              │ │
│ │   │   └─ emit: "ret i64 %0"                                           │ │
│ │   │       emit: "}"                                                   │ │
│ │   │                                                                   │ │
│ │ Generated LLVM IR:                                                    │ │
│ │   ; Built-in declarations                                             │ │
│ │   declare i8* @malloc(i64)                                            │ │
│ │   declare void @free(i8*)                                             │ │
│ │                                                                       │ │
│ │   define i64 @add(i64 %a, i64 %b) {                                   │ │
│ │     %0 = add i64 %a, %b                                               │ │
│ │     ret i64 %0                                                        │ │
│ │   }                                                                   │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ LLVM IR String
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ OPTIMIZER (vais-codegen/optimize)                                         │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ optimize_ir(ir, OptLevel::O2)                                         │ │
│ │   ├─ constant_folding() → fold constant arithmetic                    │ │
│ │   ├─ dead_store_elimination() → remove unused stores                  │ │
│ │   ├─ branch_optimization() → simplify branches                        │ │
│ │   ├─ strength_reduction() → replace expensive ops                     │ │
│ │   ├─ common_subexpression_elimination() → eliminate duplicates        │ │
│ │   └─ dead_code_elimination() → remove unreachable code                │ │
│ │                                                                       │ │
│ │ (In this simple case, no changes needed)                              │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ Optimized LLVM IR
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ CLANG (External Compiler)                                                 │
│ ┌──────────────────────────────────────────────────────────────────────┐ │
│ │ clang -x ir add.ll -o add -O2                                         │ │
│ │   ├─ Parse LLVM IR                                                    │ │
│ │   ├─ LLVM optimization passes                                         │ │
│ │   ├─ Machine code generation (x86_64/arm64)                           │ │
│ │   └─ Linking                                                          │ │
│ │                                                                       │ │
│ │ Machine Code (x86_64):                                                │ │
│ │   add:                                                                │ │
│ │     lea eax, [rdi + rsi]  ; %rdi=a, %rsi=b, %eax=result              │ │
│ │     ret                                                               │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       │ Native Binary
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                            Executable: ./add                              │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Core Design Decisions

### 1. LLVM IR as Backend

**Decision**: Generate LLVM IR text and compile via `clang`, rather than using `inkwell` (LLVM Rust bindings).

**Rationale**:
- **Simplicity**: Text generation is easier to debug and inspect
- **Portability**: No LLVM library version dependencies
- **Toolchain integration**: Leverage existing clang optimizations
- **Flexibility**: Easy to emit custom IR patterns

**Trade-offs**:
- Slower compilation (text parsing overhead)
- No compile-time IR validation
- Limited access to LLVM APIs

**Mitigation**:
- Optimization passes are fast enough for target programs
- IR validation via clang error messages
- Future: Option to use inkwell for production builds

### 2. Monomorphization over Type Erasure

**Decision**: Use monomorphization (specialization) for generics, not type erasure or boxing.

**Rationale**:
- **Zero-cost abstraction**: No runtime overhead for generics
- **Type safety**: Compile-time type checking, no casting
- **Performance**: Specialized code can be optimized per type
- **Predictability**: No hidden allocations or indirection

**Implementation**:
1. Type checker tracks `GenericInstantiation` during inference
2. Code generator creates specialized functions: `identity$i64`, `identity$f64`
3. Call sites use mangled names: `call @identity$i64`

**Trade-offs**:
- **Code bloat**: Each instantiation generates new code
- **Compile time**: More functions to generate and optimize

**Mitigation**:
- Lazy instantiation: Only generate called instances
- LLVM linkage optimization: Inline identical functions

**Example**:
```vais
F identity<T>(x:T)->T=x

identity(42)      # generates: define i64 @identity$i64(i64 %x)
identity(3.14)    # generates: define double @identity$f64(double %x)
```

### 3. Expression-Oriented Language

**Decision**: Everything is an expression (if, loop, match, blocks all return values).

**Rationale**:
- **Conciseness**: Reduce boilerplate (no return statements needed)
- **Composability**: Expressions can nest naturally
- **Functional style**: Easier to write pure functions
- **AI-friendly**: Fewer tokens, simpler patterns

**Implementation**:
- Blocks evaluate to their last expression
- `I` (if-else) returns a value: `x=I cond{a}E{b}`
- `L` (loop) with `B` (break) returns a value: `L{I x>10{B x}; x=x+1}`
- `M` (match) arms must return compatible types

**Example**:
```vais
F abs(x:i64)->i64 = I x<0 {-x} E {x}

F collatz(n:i64)->i64 = L {
  I n==1 { B n }
  E { n = I n%2==0 {n/2} E {3*n+1} }
}
```

### 4. Single-Letter Keywords for Token Efficiency

**Decision**: Use single-letter keywords (F, S, E, I, L, M) for core constructs.

**Rationale**:
- **AI optimization**: Reduces token count in LLM contexts (40-60% savings)
- **Typing efficiency**: Less typing for developers
- **Predictable**: Limited keyword set, easy to learn

**Mapping**:
| Keyword | Meaning | Traditional |
|---------|---------|-------------|
| F | Function | `fn`/`def`/`function` |
| S | Struct | `struct`/`record` |
| E | Enum / Else | `enum` / `else` |
| I | If | `if` |
| L | Loop | `loop`/`while` |
| M | Match | `match`/`switch` |
| R | Return | `return` |
| B | Break | `break` |
| C | Continue | `continue` |
| T | Type alias | `type` |
| U | Use/import | `use`/`import` |
| P | Public | `pub`/`public` |
| W | Trait (What) | `trait`/`interface` |
| X | Impl (eXtend) | `impl` |
| A | Async | `async` |

**Trade-offs**:
- Readability: May be harder for beginners (mitigated by LSP hover)
- Ambiguity: `E` used for both Enum and Else (context-disambiguated)

### 5. Hindley-Milner Type Inference

**Decision**: Use H-M inference for type deduction, minimizing annotations.

**Rationale**:
- **Ergonomics**: Reduce type annotations (less boilerplate)
- **Safety**: Still fully statically typed (no `any` or `dynamic`)
- **Predictability**: Local inference, no action-at-a-distance

**Extensions beyond standard H-M**:
- **Trait constraints**: `F sort<T:Ord>(xs:Vec<T>)`
- **Multi-parameter type classes**: Trait with associated types
- **Pattern exhaustiveness**: Ensures match covers all cases

**Limitations**:
- No higher-kinded types (no `Monad<M<_>>`)
- No row polymorphism (no extensible records)
- Recursion requires annotations (future: bidirectional checking)

**Example**:
```vais
# Type annotations inferred:
F map(f,xs) = xs.iter().map(f).collect()
# Inferred: <A,B>(F(A)->B, Vec<A>) -> Vec<B>

# Explicit annotations for clarity:
F map<A,B>(f:F(A)->B, xs:Vec<A>) -> Vec<B> =
  xs.iter().map(f).collect()
```

### 6. Stack Allocation by Default, Explicit Heap

**Decision**: Variables are stack-allocated by default; heap requires explicit `Box<T>`.

**Rationale**:
- **Performance**: Stack allocation is fast (no malloc/free)
- **Predictability**: Memory location is explicit in types
- **Safety**: Ownership rules prevent use-after-free

**Memory model**:
- Primitives: Always copied (i64, f64, bool, etc.)
- Structs: Stack by default, move semantics
- Heap: Explicit via `Box<T>`, `Rc<T>`, or raw pointers
- Arrays: Fixed-size on stack, `Vec<T>` on heap

**Example**:
```vais
F example() {
  x:i64 = 42                # Stack: 8 bytes
  p:Point = Point{x:1,y:2}  # Stack: 16 bytes (move semantics)
  b:Box<Point> = Box.new(p) # Heap: malloc(16)
}
```

---

## Type System and Inference

### Type Representations

#### Internal Type Representation (vais-types)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedType {
    // Primitives
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Bool, Char, String,
    Unit,  // void/()

    // Compound types
    Tuple(Vec<ResolvedType>),
    Array(Box<ResolvedType>, usize),
    Ptr(Box<ResolvedType>),

    // Named types
    Struct(String, Vec<ResolvedType>),  // Name + type args (for generics)
    Enum(String, Vec<ResolvedType>),
    Trait(String),

    // Functions
    Function {
        params: Vec<ResolvedType>,
        ret: Box<ResolvedType>,
        is_async: bool,
    },

    // Generic/inference
    Generic(String),          // Generic parameter (e.g., "T")
    TypeVar(usize),           // Inference variable (?T0, ?T1, ...)
    Infer,                    // Not yet inferred
}
```

### Type Inference Algorithm

**Algorithm**: Hindley-Milner with constraint-based inference

**Steps**:

1. **Constraint Generation**:
   - Walk AST, assign type variables to unknowns
   - Generate constraints: `?T0 = i64`, `?T1 = ?T2 -> i64`

2. **Unification** (Robinson's algorithm):
   - Solve constraints: `unify(t1, t2)`
   - Occurs check: Prevent infinite types (`?T0 = Vec<?T0>`)
   - Substitution: Replace type vars with concrete types

3. **Generalization**:
   - Convert monomorphic types to polytypes
   - Example: `?T0 -> ?T0` becomes `∀T. T -> T`

4. **Instantiation**:
   - When calling generic function, create fresh type vars
   - Example: `identity<T>(x:T)` called with `42` → instantiate `T = i64`

**Code snippet**:
```rust
fn check_expr(&mut self, expr: &Expr) -> TypeResult<ResolvedType> {
    match expr {
        Expr::IntLit(_) => Ok(ResolvedType::I64),
        Expr::Variable(name) => self.lookup_var(name),
        Expr::BinaryOp { op, left, right } => {
            let lt = self.check_expr(left)?;
            let rt = self.check_expr(right)?;
            self.unify(&lt, &rt)?;  // Operands must match

            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => Ok(lt),
                BinOp::Eq | BinOp::Lt | BinOp::Gt => Ok(ResolvedType::Bool),
                // ...
            }
        }
        // ... other cases
    }
}

fn unify(&mut self, t1: &ResolvedType, t2: &ResolvedType) -> TypeResult<()> {
    match (t1, t2) {
        (ResolvedType::TypeVar(id1), ResolvedType::TypeVar(id2)) if id1 == id2 => Ok(()),
        (ResolvedType::TypeVar(id), ty) | (ty, ResolvedType::TypeVar(id)) => {
            if self.occurs_check(*id, ty) {
                return Err(TypeError::InfiniteType);
            }
            self.substitutions.insert(*id, ty.clone());
            Ok(())
        }
        (ResolvedType::I64, ResolvedType::I64) => Ok(()),
        (ResolvedType::Struct(n1, args1), ResolvedType::Struct(n2, args2)) if n1 == n2 => {
            for (a1, a2) in args1.iter().zip(args2) {
                self.unify(a1, a2)?;
            }
            Ok(())
        }
        _ => Err(TypeError::Mismatch { expected: t1.clone(), found: t2.clone() })
    }
}
```

### Generic Type Inference

**Example**: Inferring `Vec.push(v, item)`

```vais
S Vec<T> { data:Ptr<T>, len:i64, cap:i64 }

F Vec.push<T>(self:Ptr<Vec<T>>, item:T) {
  # ... implementation
}

# Usage:
v:Vec<i64> = Vec.new()
Vec.push(v, 42)  # Infer: T = i64 from v's type
```

**Inference process**:
1. `v` has type `Vec<i64>` (annotated)
2. `Vec.push` call: Instantiate `<T>` with fresh var `?T0`
3. First parameter: `Ptr<Vec<?T0>>`
4. Unify with `v`'s type: `Ptr<Vec<?T0>> ~ Ptr<Vec<i64>>`
5. Solve: `?T0 = i64`
6. Second parameter: `item:?T0` → `item:i64`
7. Validate: `42` has type `i64` ✓

**Implementation**:
```rust
fn check_generic_function_call(
    &mut self,
    func_name: &str,
    type_args: &[ResolvedType],
    args: &[Expr],
) -> TypeResult<ResolvedType> {
    let func_sig = self.functions.get(func_name)?;

    // Infer type arguments from call site
    let mut type_map = HashMap::new();
    for (i, param_ty) in func_sig.params.iter().enumerate() {
        let arg_ty = self.check_expr(&args[i])?;
        self.infer_type_args(param_ty, &arg_ty, &mut type_map)?;
    }

    // Apply substitutions
    let specialized_ret = self.substitute_generics(&func_sig.ret, &type_map);

    // Record instantiation for monomorphization
    let type_arg_list: Vec<_> = func_sig.generics.iter()
        .map(|g| type_map.get(g).cloned().unwrap_or(ResolvedType::I64))
        .collect();

    self.generic_instantiations.push(GenericInstantiation {
        base_name: func_name.to_string(),
        type_args: type_arg_list,
        mangled_name: mangle_name(func_name, &type_arg_list),
        kind: InstantiationKind::Function,
    });

    Ok(specialized_ret)
}
```

### Pattern Exhaustiveness Checking

Ensures match expressions cover all possible cases.

**Algorithm**:
1. Enumerate all constructors for matched type
2. Check each arm's pattern against constructors
3. Compute "useful" patterns (not subsumed by previous arms)
4. If any constructor uncovered, report error

**Example**:
```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt:Option<T>, default:T)->T = M opt {
  Some(x) -> x,
  None -> default
}  # ✓ Exhaustive

F bad<T>(opt:Option<T>)->T = M opt {
  Some(x) -> x
}  # ✗ Error: Missing pattern: None
```

**Implementation** (vais-types/exhaustiveness.rs):
```rust
pub struct ExhaustivenessChecker;

impl ExhaustivenessChecker {
    pub fn check_match(
        &self,
        matched_type: &ResolvedType,
        arms: &[MatchArm],
        enums: &HashMap<String, EnumDef>,
    ) -> ExhaustivenessResult {
        let constructors = self.get_constructors(matched_type, enums)?;
        let mut uncovered = constructors.clone();

        for arm in arms {
            let pattern_constructors = self.pattern_constructors(&arm.pattern);
            uncovered.retain(|c| !pattern_constructors.contains(c));
        }

        if uncovered.is_empty() {
            Ok(())
        } else {
            Err(ExhaustivenessError::MissingPatterns(uncovered))
        }
    }
}
```

---

## Generic Monomorphization

**Monomorphization** is the process of generating specialized code for each unique instantiation of a generic function or type.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Type Checker (vais-types)                                        │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ During type inference, track all generic instantiations:    │ │
│ │                                                              │ │
│ │ identity<i64>(42)     → GenericInstantiation {              │ │
│ │                           base_name: "identity",            │ │
│ │                           type_args: [i64],                 │ │
│ │                           mangled_name: "identity$i64"      │ │
│ │                         }                                   │ │
│ │                                                              │ │
│ │ Vec<String>.push(...)  → GenericInstantiation {             │ │
│ │                           base_name: "Vec.push",            │ │
│ │                           type_args: [String],              │ │
│ │                           mangled_name: "Vec$String.push"   │ │
│ │                         }                                   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 │ Vec<GenericInstantiation>
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ Code Generator (vais-codegen)                                    │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ For each GenericInstantiation:                              │ │
│ │                                                              │ │
│ │ 1. Look up original generic function definition             │ │
│ │ 2. Substitute type parameters with concrete types           │ │
│ │ 3. Generate specialized LLVM IR code                        │ │
│ │                                                              │ │
│ │ Example:                                                     │ │
│ │   Generic: F identity<T>(x:T)->T = x                        │ │
│ │                                                              │ │
│ │   Instantiation: identity<i64>                              │ │
│ │   Generated:                                                │ │
│ │     define i64 @identity$i64(i64 %x) {                      │ │
│ │       ret i64 %x                                            │ │
│ │     }                                                       │ │
│ │                                                              │ │
│ │   Instantiation: identity<f64>                              │ │
│ │   Generated:                                                │ │
│ │     define double @identity$f64(double %x) {                │ │
│ │       ret double %x                                         │ │
│ │     }                                                       │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Name Mangling Scheme

To avoid name collisions, specialized functions use mangled names:

**Pattern**: `{base_name}${type1}${type2}...`

**Examples**:
| Generic Call | Mangled Name |
|--------------|--------------|
| `identity<i64>(x)` | `identity$i64` |
| `identity<f64>(x)` | `identity$f64` |
| `Vec<String>.push(...)` | `Vec$String.push` |
| `HashMap<i64,String>.insert(...)` | `HashMap$i64$String.insert` |
| `Option<Vec<i64>>.unwrap()` | `Option$Vec$i64.unwrap` |

**Implementation**:
```rust
pub fn mangle_name(base_name: &str, type_args: &[ResolvedType]) -> String {
    if type_args.is_empty() {
        return base_name.to_string();
    }

    let mut result = base_name.to_string();
    for ty in type_args {
        result.push('$');
        result.push_str(&mangle_type(ty));
    }
    result
}

pub fn mangle_type(ty: &ResolvedType) -> String {
    match ty {
        ResolvedType::I64 => "i64".to_string(),
        ResolvedType::String => "String".to_string(),
        ResolvedType::Struct(name, args) => {
            let mut s = name.clone();
            for arg in args {
                s.push('$');
                s.push_str(&mangle_type(arg));
            }
            s
        }
        // ... other cases
    }
}
```

### Lazy Instantiation

Vais uses **lazy monomorphization**: only instantiate generic functions that are actually called.

**Algorithm**:
1. Type checker records all instantiations in `generic_instantiations`
2. Code generator processes only recorded instantiations
3. Unreferenced generic functions are never generated

**Example**:
```vais
F identity<T>(x:T)->T = x

F main()->i64 {
  x = identity(42)      # Only identity<i64> is instantiated
  x
}
# identity<f64> is never generated, even though it's valid
```

### Struct Monomorphization

Generic structs are also specialized:

**Example**:
```vais
S Vec<T> { data:Ptr<T>, len:i64, cap:i64 }

# Usage:
v1:Vec<i64> = Vec.new()     # Instantiate Vec<i64>
v2:Vec<String> = Vec.new()  # Instantiate Vec<String>
```

**Generated LLVM IR**:
```llvm
; Vec<i64>
%Vec$i64 = type { i64*, i64, i64 }

; Vec<String>
%Vec$String = type { %String*, i64, i64 }

define %Vec$i64* @Vec$i64.new() { ... }
define %Vec$String* @Vec$String.new() { ... }
```

**Implementation**:
```rust
fn generate_specialized_struct_type(
    &mut self,
    instantiation: &GenericInstantiation,
) -> CodegenResult<()> {
    let struct_def = self.structs.get(&instantiation.base_name)?;
    let mangled = &instantiation.mangled_name;

    // Substitute type parameters in field types
    let mut fields = Vec::new();
    for field in &struct_def.fields {
        let specialized_ty = substitute_type(
            &field.ty,
            &struct_def.generics,
            &instantiation.type_args,
        );
        fields.push(type_to_llvm(&specialized_ty));
    }

    // Emit struct type definition
    self.emit(&format!("%{} = type {{ {} }}",
        mangled,
        fields.join(", ")
    ));

    Ok(())
}
```

---

## Code Generation Strategy

### SSA Form and Basic Blocks

LLVM requires code in **Static Single Assignment (SSA)** form:
- Each variable is assigned exactly once
- Variables are immutable after assignment
- Control flow uses phi nodes for merging values

**Example transformation**:
```vais
# Vais code:
F fib(n:i64)->i64 {
  a = 0
  b = 1
  L {
    I n == 0 { B a }
    temp = a + b
    a = b
    b = temp
    n = n - 1
  }
}
```

**Generated SSA form**:
```llvm
define i64 @fib(i64 %n) {
entry:
  br label %loop

loop:
  %a = phi i64 [ 0, %entry ], [ %b, %loop ]
  %b = phi i64 [ 1, %entry ], [ %temp, %loop ]
  %n_phi = phi i64 [ %n, %entry ], [ %n_next, %loop ]

  %cond = icmp eq i64 %n_phi, 0
  br i1 %cond, label %exit, label %continue

continue:
  %temp = add i64 %a, %b
  %n_next = sub i64 %n_phi, 1
  br label %loop

exit:
  ret i64 %a
}
```

### Expression Code Generation

**Strategy**: Generate temporary variables for each subexpression.

**Example**:
```vais
x = (a + b) * (c - d)
```

**Generated IR**:
```llvm
%0 = add i64 %a, %b      ; a + b
%1 = sub i64 %c, %d      ; c - d
%x = mul i64 %0, %1      ; (%0) * (%1)
```

**Implementation** (vais-codegen/expr.rs):
```rust
fn generate_expr(&mut self, expr: &Expr) -> CodegenResult<(String, String)> {
    match expr {
        Expr::IntLit(n) => Ok((format!("{}", n), String::new())),

        Expr::Variable(name) => {
            let var = self.locals.get(name)?;
            let tmp = self.fresh_temp();
            let load = format!("{} = load {}, {}* {}",
                tmp, var.llvm_ty, var.llvm_ty, var.llvm_name);
            Ok((tmp, load))
        }

        Expr::BinaryOp { op, left, right } => {
            let (l_val, l_ir) = self.generate_expr(left)?;
            let (r_val, r_ir) = self.generate_expr(right)?;
            let tmp = self.fresh_temp();

            let op_str = match op {
                BinOp::Add => "add",
                BinOp::Sub => "sub",
                BinOp::Mul => "mul",
                BinOp::Div => "sdiv",
                // ...
            };

            let ir = format!("{}{}{} = {} i64 {}, {}",
                l_ir, r_ir, tmp, op_str, l_val, r_val);
            Ok((tmp, ir))
        }

        // ... other cases
    }
}
```

### Control Flow: If-Else

**Vais code**:
```vais
result = I x > 0 { 1 } E { -1 }
```

**Generated IR**:
```llvm
%cond = icmp sgt i64 %x, 0
br i1 %cond, label %then, label %else

then:
  br label %merge

else:
  br label %merge

merge:
  %result = phi i64 [ 1, %then ], [ -1, %else ]
```

**Key pattern**: Use **phi nodes** to merge values from different branches.

### Control Flow: Loops

**Vais code**:
```vais
sum = L {
  I i >= n { B sum }
  sum = sum + i
  i = i + 1
}
```

**Generated IR**:
```llvm
br label %loop

loop:
  %sum = phi i64 [ 0, %entry ], [ %sum_next, %loop ]
  %i = phi i64 [ 0, %entry ], [ %i_next, %loop ]

  %cond = icmp sge i64 %i, %n
  br i1 %cond, label %exit, label %continue

continue:
  %sum_next = add i64 %sum, %i
  %i_next = add i64 %i, 1
  br label %loop

exit:
  ret i64 %sum
```

### Memory Management

Vais uses **explicit memory management** with no garbage collection.

**Stack allocation** (local variables):
```llvm
define void @example() {
entry:
  %x = alloca i64     ; allocate 8 bytes on stack
  store i64 42, i64* %x
  ; ...
}  ; %x automatically freed when function returns
```

**Heap allocation** (Box<T>):
```vais
b:Box<Point> = Box.new(Point{x:1,y:2})
```

```llvm
%size = ... ; sizeof(Point)
%raw_ptr = call i8* @malloc(i64 %size)
%typed_ptr = bitcast i8* %raw_ptr to %Point*
; ... initialize fields
```

**Deallocation**:
```vais
Box.free(b)
```

```llvm
%raw_ptr = bitcast %Point* %b to i8*
call void @free(i8* %raw_ptr)
```

**FFI declarations**:
```llvm
declare i8* @malloc(i64)
declare void @free(i8*)
declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)
```

---

## Optimization Pipeline

Vais includes custom LLVM IR optimization passes before handing off to clang.

### Optimization Levels

| Level | Passes Enabled | Use Case |
|-------|----------------|----------|
| **O0** | None | Fast compilation, debugging |
| **O1** | Basic (constant folding, DSE, branch) | Development builds |
| **O2** | O1 + CSE + strength reduction + DCE | Production builds |
| **O3** | O2 + inlining + loop opts | Maximum performance |

### Pass Descriptions

#### 1. Constant Folding

**What**: Evaluate constant expressions at compile time.

**Example**:
```llvm
; Before:
%0 = add i64 2, 3
%1 = mul i64 %0, 4

; After:
%0 = 5        ; (folded)
%1 = 20       ; (folded)
```

**Implementation**:
```rust
fn constant_folding(ir: &str) -> String {
    for line in ir.lines() {
        if let Some((var, op, a, b)) = parse_binop(line) {
            if let (Ok(a_val), Ok(b_val)) = (parse_const(a), parse_const(b)) {
                let result = match op {
                    "add" => a_val + b_val,
                    "sub" => a_val - b_val,
                    "mul" => a_val * b_val,
                    "sdiv" => if b_val != 0 { a_val / b_val } else { continue },
                    _ => continue,
                };
                output.push_str(&format!("{} = {}", var, result));
                continue;
            }
        }
        output.push_str(line);
    }
    output
}
```

#### 2. Dead Store Elimination (DSE)

**What**: Remove stores to variables that are never read.

**Example**:
```llvm
; Before:
%x = alloca i64
store i64 1, i64* %x    ; Dead (overwritten)
store i64 2, i64* %x
%val = load i64, i64* %x

; After:
%x = alloca i64
store i64 2, i64* %x    ; Only store kept
%val = load i64, i64* %x
```

#### 3. Common Subexpression Elimination (CSE)

**What**: Reuse results of identical computations.

**Example**:
```llvm
; Before:
%0 = add i64 %a, %b
%1 = mul i64 %0, 2
%2 = add i64 %a, %b     ; Duplicate!
%3 = mul i64 %2, 3

; After:
%0 = add i64 %a, %b
%1 = mul i64 %0, 2
%3 = mul i64 %0, 3      ; Reuse %0
```

#### 4. Strength Reduction

**What**: Replace expensive operations with cheaper equivalents.

**Example**:
```llvm
; Before:
%0 = mul i64 %x, 2      ; Multiplication
%1 = mul i64 %x, 8
%2 = sdiv i64 %x, 4     ; Division

; After:
%0 = shl i64 %x, 1      ; Left shift (faster)
%1 = shl i64 %x, 3
%2 = ashr i64 %x, 2     ; Arithmetic right shift
```

**Patterns**:
- `x * 2^n` → `x << n`
- `x / 2^n` → `x >> n`
- `x * 0` → `0`
- `x * 1` → `x`

#### 5. Dead Code Elimination (DCE)

**What**: Remove unreachable basic blocks and unused instructions.

**Example**:
```llvm
; Before:
define i64 @example() {
entry:
  ret i64 42

unreachable_block:      ; Dead!
  %x = add i64 1, 2
  ret i64 %x
}

; After:
define i64 @example() {
entry:
  ret i64 42
}
```

#### 6. Loop Invariant Code Motion (LICM)

**What**: Move loop-invariant computations outside the loop.

**Example**:
```vais
# Before:
L {
  y = x * 2     # x doesn't change in loop
  sum = sum + y
  i = i + 1
  I i >= n { B sum }
}

# After optimization:
y = x * 2       # Hoisted!
L {
  sum = sum + y
  i = i + 1
  I i >= n { B sum }
}
```

**Generated IR**:
```llvm
; Before:
loop:
  %y = mul i64 %x, 2     ; Invariant!
  %sum_next = add i64 %sum, %y
  ; ...

; After:
preheader:
  %y = mul i64 %x, 2     ; Hoisted to preheader
  br label %loop

loop:
  %sum_next = add i64 %sum, %y
  ; ...
```

#### 7. Function Inlining (O3)

**What**: Replace function calls with the function body (aggressive at O3).

**Heuristic**: Inline if function body < 20 instructions.

**Example**:
```vais
F add(a,b) = a + b
F main() = add(1, 2)
```

**Before inlining**:
```llvm
define i64 @add(i64 %a, i64 %b) {
  %result = add i64 %a, %b
  ret i64 %result
}

define i64 @main() {
  %call = call i64 @add(i64 1, i64 2)
  ret i64 %call
}
```

**After inlining**:
```llvm
define i64 @main() {
  %result = add i64 1, 2    ; Inlined
  ret i64 %result           ; (constant folding can further optimize to "ret i64 3")
}
```

**Implementation**:
```rust
fn aggressive_inline(ir: &str) -> String {
    let functions = parse_functions(ir);

    for func in &functions {
        if should_inline(func) {  // Check size < 20 instructions
            ir = inline_function(ir, func);
        }
    }

    ir
}
```

---

## Developer Tools

### LSP Architecture

The Vais Language Server provides IDE integration via tower-lsp.

**Components**:
```
┌───────────────────────────────────────────────────────────────┐
│                     IDE (VSCode/Neovim/etc.)                  │
└────────────────────────────┬──────────────────────────────────┘
                             │
                             │ JSON-RPC over stdio/socket
                             ▼
┌───────────────────────────────────────────────────────────────┐
│ LSP Server (vais-lsp)                                          │
│ ┌───────────────────────────────────────────────────────────┐ │
│ │ VaisLanguageServer                                        │ │
│ │ ├─ Document Management (open/change/close)               │ │
│ │ ├─ Symbol Cache (per-document AST + types)               │ │
│ │ ├─ Request Handlers:                                      │ │
│ │ │  ├─ textDocument/didChange → Update cache, send diags  │ │
│ │ │  ├─ textDocument/completion → Suggest keywords/types   │ │
│ │ │  ├─ textDocument/definition → Find declaration         │ │
│ │ │  ├─ textDocument/hover → Show type info                │ │
│ │ │  ├─ textDocument/references → Find all uses            │ │
│ │ │  └─ textDocument/rename → Safe refactoring             │ │
│ │ └─ Background Compiler (lex/parse/typecheck)             │ │
│ └───────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
```

**Key features**:

1. **Real-time diagnostics**: Syntax and type errors on every keystroke
2. **Smart completion**: Context-aware suggestions (keywords, types, methods)
3. **Symbol navigation**: Go-to-definition, find-references
4. **Hover info**: Type signatures and documentation
5. **Refactoring**: Rename symbols across files

**Performance**:
- **Incremental parsing**: Only re-parse changed regions (future)
- **Symbol caching**: Avoid redundant type checking
- **Async processing**: Non-blocking request handling

**Example request flow**:
```
User types: "F add(a,b)=a+"
  ↓
IDE → textDocument/didChange
  ↓
LSP Server:
  1. Update document map
  2. Lex + Parse → AST
  3. Type check → Errors
  4. Send diagnostics: "Expected expression after '+'"
  ↓
IDE ← publishDiagnostics
  ↓
Red squiggle under '+'
```

### REPL (Read-Eval-Print Loop)

Interactive environment for rapid prototyping.

**Features**:
- **Multi-line input**: Bracket/brace balance detection
- **History**: Arrow keys, persistent across sessions (~/.vais_history)
- **Tab completion**: Keywords + built-in functions
- **Commands**: `:help`, `:clear`, `:load <file>`, `:quit`

**Example session**:
```vais
vais> F add(a,b)=a+b
Defined function: add (i64, i64) -> i64

vais> add(40, 2)
42 : i64

vais> S Point{x:i64, y:i64}
Defined struct: Point

vais> p = Point{x:3, y:4}
p = Point { x: 3, y: 4 } : Point

vais> :load examples/option.vais
Loaded 12 items from examples/option.vais

vais> :quit
Goodbye!
```

**Implementation** (vaisc/src/repl.rs):
```rust
pub fn run_repl() {
    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(".vais_history");

    let mut checker = TypeChecker::new();
    let mut codegen = CodeGenerator::new("repl");

    loop {
        let readline = rl.readline("vais> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                if line.starts_with(":") {
                    handle_command(&line, &mut checker);
                    continue;
                }

                // Compile and execute
                let tokens = tokenize(&line)?;
                let module = parse(tokens)?;
                checker.check_module(&module)?;
                let ir = codegen.generate_module(&module)?;

                // Execute via JIT (simplified)
                let result = execute_ir(&ir)?;
                println!("{}", result);
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    rl.save_history(".vais_history").ok();
}
```

### Formatter (vaisc fmt)

AST-based code formatter ensures consistent style.

**Options**:
- `--check`: Exit with code 1 if formatting needed (CI use)
- `--indent N`: Set indentation size (default: 4)

**Example**:
```bash
# Format file in-place:
vaisc fmt src/main.vais

# Check formatting (CI):
vaisc fmt --check src/

# Custom indentation:
vaisc fmt --indent 2 src/
```

**Formatting rules**:
- Indent: 4 spaces (configurable)
- Braces: Same line for functions/structs, new line for blocks
- Trailing commas: Always for multi-line lists
- Blank lines: One between top-level items

**Implementation** (vais-codegen/formatter.rs):
```rust
pub struct Formatter {
    indent_size: usize,
    current_indent: usize,
}

impl Formatter {
    pub fn format_module(&mut self, module: &Module) -> String {
        let mut output = String::new();

        for item in &module.items {
            output.push_str(&self.format_item(&item.node));
            output.push_str("\n\n");
        }

        output
    }

    fn format_function(&mut self, func: &Function) -> String {
        let mut s = format!("F {}", func.name);

        // Generics
        if !func.generics.is_empty() {
            s.push('<');
            s.push_str(&func.generics.join(", "));
            s.push('>');
        }

        // Parameters
        s.push('(');
        let params: Vec<_> = func.params.iter()
            .map(|p| format!("{}:{}", p.name, self.format_type(&p.ty)))
            .collect();
        s.push_str(&params.join(", "));
        s.push(')');

        // Return type
        if func.ret_type != Type::Infer {
            s.push_str(" -> ");
            s.push_str(&self.format_type(&func.ret_type));
        }

        // Body
        s.push_str(" = ");
        s.push_str(&self.format_expr(&func.body));

        s
    }
}
```

### Debugger Support

Vais generates DWARF debug information for source-level debugging with lldb/gdb.

**CLI option**:
```bash
vaisc build file.vais -g       # Enable debug info
vaisc build file.vais -g -O0   # Debug build (no optimization)
```

**Generated metadata**:
```llvm
; Debug info metadata
!0 = !DIFile(filename: "file.vais", directory: "/path/to/project")
!1 = !DICompileUnit(language: DW_LANG_C99, file: !0, producer: "vaisc 0.1.0")

; Function debug info
define i64 @add(i64 %a, i64 %b) !dbg !3 {
entry:
  ; Source location for each instruction
  %0 = add i64 %a, %b, !dbg !4
  ret i64 %0, !dbg !5
}

!3 = !DISubprogram(name: "add", file: !0, line: 1, ...)
!4 = !DILocation(line: 1, column: 15, scope: !3)
!5 = !DILocation(line: 1, column: 10, scope: !3)
```

**Debugging session**:
```bash
$ vaisc build file.vais -g -o program
$ lldb program
(lldb) b add
Breakpoint 1: where = program`add, address = 0x100000f40

(lldb) run
Process 1234 stopped
* frame #0: 0x100000f40 program`add(a=10, b=20) at file.vais:1:15

(lldb) p a
(i64) $0 = 10

(lldb) s
Process 1234 stopped
* frame #0: 0x100000f45 program`add(a=10, b=20) at file.vais:1:10
-> 1    F add(a,b)=a+b
```

**Implementation** (vais-codegen/debug.rs):
```rust
pub struct DebugInfoBuilder {
    config: DebugConfig,
    metadata_counter: usize,
    source_code: Option<String>,
    line_starts: Vec<usize>,  // Byte offset of each line

    di_file_id: Option<usize>,
    di_compile_unit_id: Option<usize>,
    function_di: HashMap<String, usize>,
}

impl DebugInfoBuilder {
    pub fn create_di_file(&mut self) -> usize {
        let id = self.next_metadata_id();
        let node = format!(
            "!{} = !DIFile(filename: \"{}\", directory: \"{}\")",
            id, self.config.source_file, self.config.source_dir
        );
        self.metadata_nodes.push(node);
        self.di_file_id = Some(id);
        id
    }

    pub fn create_di_subprogram(&mut self, func_name: &str, line: usize) -> usize {
        let id = self.next_metadata_id();
        let file_id = self.di_file_id.unwrap();
        let node = format!(
            "!{} = !DISubprogram(name: \"{}\", file: !{}, line: {}, ...)",
            id, func_name, file_id, line
        );
        self.metadata_nodes.push(node);
        self.function_di.insert(func_name.to_string(), id);
        id
    }

    pub fn create_di_location(&mut self, span: Span, scope_id: usize) -> String {
        let (line, col) = self.get_line_col(span.start);
        format!("!DILocation(line: {}, column: {}, scope: !{})", line, col, scope_id)
    }
}
```

---

## Plugin System

Vais supports four types of plugins for extending the compiler.

### Plugin Types

```
┌────────────────────────────────────────────────────────────┐
│ Plugin Type │ Execution Point │ Input → Output            │
├─────────────┼─────────────────┼───────────────────────────┤
│ Lint        │ After parsing   │ AST → Diagnostics         │
│ Transform   │ Before typeck   │ AST → Modified AST        │
│ Optimize    │ After codegen   │ LLVM IR → Modified IR     │
│ Codegen     │ After optimize  │ Module → Additional files │
└────────────────────────────────────────────────────────────┘
```

### Lint Plugins

**Purpose**: Check code for style violations, anti-patterns, or project-specific rules.

**Example**: Naming convention checker
```rust
pub struct NamingConventionPlugin;

impl LintPlugin for NamingConventionPlugin {
    fn check(&self, module: &Module) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &module.items {
            if let Item::Function(func) = &item.node {
                if !is_snake_case(&func.name) {
                    diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Warning,
                        message: format!("Function '{}' should use snake_case", func.name),
                        span: Some(item.span),
                        help: Some(format!("Try renaming to '{}'", to_snake_case(&func.name))),
                    });
                }
            }
        }

        diagnostics
    }
}
```

**Output**:
```
Warning: Function 'AddNumbers' should use snake_case
  --> file.vais:3:3
   |
 3 | F AddNumbers(a,b)=a+b
   |   ^^^^^^^^^^
   |
Help: Try renaming to 'add_numbers'
```

### Transform Plugins

**Purpose**: Modify AST before type checking (macro expansion, desugaring, etc.).

**Example**: Auto-derive Debug trait
```rust
pub struct AutoDebugPlugin;

impl TransformPlugin for AutoDebugPlugin {
    fn transform(&self, mut module: Module) -> Result<Module, String> {
        for item in &mut module.items {
            if let Item::Struct(s) = &mut item.node {
                if s.attributes.iter().any(|a| a.name == "derive" && a.args.contains(&"Debug")) {
                    // Generate Debug impl
                    let debug_impl = generate_debug_impl(s);
                    module.items.push(debug_impl);
                }
            }
        }
        Ok(module)
    }
}
```

### Optimize Plugins

**Purpose**: Apply custom LLVM IR optimizations.

**Example**: Loop vectorization hint
```rust
pub struct VectorizationPlugin;

impl OptimizePlugin for VectorizationPlugin {
    fn optimize(&self, ir: &str, level: OptLevel) -> Result<String, String> {
        if level < OptLevel::O2 {
            return Ok(ir.to_string());
        }

        // Add LLVM vectorization metadata to loops
        let mut output = String::new();
        for line in ir.lines() {
            output.push_str(line);
            if line.trim() == "br label %loop" {
                output.push_str(", !llvm.loop !{metadata !\"llvm.loop.vectorize.enable\"}");
            }
            output.push('\n');
        }

        Ok(output)
    }
}
```

### Codegen Plugins

**Purpose**: Generate additional output files (bindings, documentation, etc.).

**Example**: C header generator
```rust
pub struct CHeaderPlugin;

impl CodegenPlugin for CHeaderPlugin {
    fn generate(&self, module: &Module) -> Result<Vec<GeneratedFile>, String> {
        let mut header = String::new();
        header.push_str("#pragma once\n\n");

        for item in &module.items {
            if let Item::Function(func) = &item.node {
                if func.visibility == Visibility::Public {
                    header.push_str(&format!(
                        "extern {} {}({});\n",
                        c_type(&func.ret_type),
                        func.name,
                        func.params.iter()
                            .map(|p| format!("{} {}", c_type(&p.ty), p.name))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }

        Ok(vec![GeneratedFile {
            path: "output.h".to_string(),
            contents: header,
        }])
    }
}
```

### Plugin Loading

**Configuration** (vais-plugins.toml):
```toml
[plugins]
# Paths to plugin shared libraries
path = [
    "./plugins/naming-convention.dylib",
    "/usr/local/lib/vais-plugins/auto-debug.so",
]

# Plugin-specific configuration
[plugins.config]
naming-convention = { enforce_snake_case = true, max_name_length = 40 }
auto-debug = { include_private_fields = false }
```

**CLI integration**:
```bash
# Load plugins from vais-plugins.toml
vaisc build file.vais

# Disable all plugins
vaisc build file.vais --no-plugins

# Load additional plugin
vaisc build file.vais --plugin ./my-plugin.dylib
```

**Dynamic loading** (vais-plugin/loader.rs):
```rust
pub fn load_plugin(path: &Path) -> Result<LoadedPlugin, String> {
    unsafe {
        let lib = Library::new(path)?;

        // Get plugin type
        let get_type: Symbol<extern "C" fn() -> PluginType> =
            lib.get(b"get_plugin_type")?;
        let plugin_type = get_type();

        // Create plugin instance
        let create: Symbol<extern "C" fn() -> *mut dyn Plugin> =
            lib.get(b"create_plugin")?;
        let plugin = Box::from_raw(create());

        Ok(LoadedPlugin {
            lib,
            plugin,
            plugin_type,
        })
    }
}
```

---

## Internationalization

Vais supports localized error messages in multiple languages.

### Locale Detection

**Priority order**:
1. CLI flag: `--locale ko`
2. Environment: `VAIS_LANG=ja`
3. System locale: `LANG=ko_KR.UTF-8` → `ko`
4. Default: `en`

**Supported locales**:
- `en` - English (default)
- `ko` - Korean (한국어)
- `ja` - Japanese (日本語)

### Message Format

**File structure** (vais-i18n/locales/en.json):
```json
{
  "type.E001.title": "Type Mismatch",
  "type.E001.message": "Expected type {expected}, found {found}",
  "type.E001.help": "Try converting {found} to {expected}",

  "type.E002.title": "Undefined Variable",
  "type.E002.message": "Variable '{name}' is not defined",
  "type.E002.help": "Did you mean '{suggestion}'?",

  "parse.P001.title": "Unexpected Token",
  "parse.P001.message": "Expected {expected}, found {found}",
  "parse.P001.help": "Check for missing punctuation"
}
```

**Korean** (ko.json):
```json
{
  "type.E001.title": "타입 불일치",
  "type.E001.message": "예상 타입: {expected}, 실제 타입: {found}",
  "type.E001.help": "{found}를 {expected}로 변환해 보세요"
}
```

**Japanese** (ja.json):
```json
{
  "type.E001.title": "型の不一致",
  "type.E001.message": "期待される型: {expected}、実際の型: {found}",
  "type.E001.help": "{found}を{expected}に変換してみてください"
}
```

### Usage

**Code integration**:
```rust
use vais_i18n::{get, get_simple, set_locale};

// Simple message:
let msg = get_simple("type.E001.title");
// → "Type Mismatch"

// Message with variables:
let msg = get("type.E001.message", &[
    ("expected", "i64"),
    ("found", "String"),
]);
// → "Expected type i64, found String"

// Change locale:
set_locale(Locale::Korean);
let msg = get_simple("type.E001.title");
// → "타입 불일치"
```

**CLI usage**:
```bash
# English (default):
$ vaisc check file.vais
Error: Type Mismatch
Expected type i64, found String
Help: Try converting String to i64

# Korean:
$ vaisc --locale ko check file.vais
에러: 타입 불일치
예상 타입: i64, 실제 타입: String
도움말: String를 i64로 변환해 보세요

# Japanese:
$ vaisc --locale ja check file.vais
エラー: 型の不一致
期待される型: i64、実際の型: String
ヘルプ: Stringをi64に変換してみてください
```

---

## Testing Strategy

Vais has a comprehensive testing suite across multiple levels.

### Test Pyramid

```
         ┌─────────────┐
         │   E2E (47)  │  Integration tests (full pipeline)
         ├─────────────┤
         │  Unit (198) │  Component tests (per crate)
         ├─────────────┤
         │ Edge (100+) │  Boundary/error cases
         └─────────────┘
```

**Total**: 402 tests (100% passing)

### Unit Tests

Each crate has isolated unit tests:

**vais-lexer** (34 tests):
```rust
#[test]
fn test_single_letter_keywords() {
    assert_eq!(tokenize("F S E").unwrap()[0].token, Token::Function);
    assert_eq!(tokenize("F S E").unwrap()[1].token, Token::Struct);
}

#[test]
fn test_keyword_vs_identifier() {
    // "F" should be keyword, "Foo" should be identifier
    let tokens = tokenize("F Foo").unwrap();
    assert_eq!(tokens[0].token, Token::Function);
    assert!(matches!(tokens[1].token, Token::Ident(_)));
}

#[test]
fn test_empty_input() {
    assert!(tokenize("").unwrap().is_empty());
}
```

**vais-parser** (57 tests):
```rust
#[test]
fn test_parse_function_expr_form() {
    let src = "F add(a,b)=a+b";
    let module = parse(src).unwrap();
    assert_eq!(module.items.len(), 1);
    // ... assertions on AST structure
}

#[test]
fn test_parse_error_missing_paren() {
    let src = "F add(a,b=a+b";  // Missing ')'
    assert!(parse(src).is_err());
}
```

**vais-types** (48 tests):
```rust
#[test]
fn test_infer_binary_op() {
    let src = "F add(a,b)=a+b";
    let module = parse(src).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();

    let sig = checker.get_function("add").unwrap();
    assert_eq!(sig.params, vec![ResolvedType::I64, ResolvedType::I64]);
    assert_eq!(sig.ret, ResolvedType::I64);
}

#[test]
fn test_type_error_mismatch() {
    let src = "F bad()->i64=\"hello\"";
    let module = parse(src).unwrap();
    let mut checker = TypeChecker::new();

    let result = checker.check_module(&module);
    assert!(matches!(result, Err(TypeError::Mismatch { .. })));
}
```

**vais-codegen** (58 tests):
```rust
#[test]
fn test_generate_function() {
    let src = "F add(a:i64,b:i64)->i64=a+b";
    let module = parse(src).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();

    let mut codegen = CodeGenerator::new("test");
    let ir = codegen.generate_module(&module).unwrap();

    assert!(ir.contains("define i64 @add(i64 %a, i64 %b)"));
    assert!(ir.contains("add i64 %a, %b"));
}

#[test]
fn test_optimization_constant_folding() {
    let ir = "%0 = add i64 2, 3\n%1 = mul i64 %0, 4";
    let optimized = optimize_ir(ir, OptLevel::O1);
    assert!(optimized.contains("20"));  // Should fold to constant
}
```

### Integration Tests

**E2E tests** (47 tests in vaisc/tests/integration_tests.rs):

```rust
#[test]
fn test_compile_hello_world() {
    let src = r#"
        F main() {
            print("Hello, world!\n")
        }
    "#;

    // Full pipeline: lex → parse → typecheck → codegen
    let tokens = tokenize(src).unwrap();
    let module = parse(tokens).unwrap();
    let mut checker = TypeChecker::new();
    checker.check_module(&module).unwrap();

    let mut codegen = CodeGenerator::new("hello");
    let ir = codegen.generate_module(&module).unwrap();

    // Verify IR structure
    assert!(ir.contains("define void @main()"));
    assert!(ir.contains("call i32 @puts"));
}

#[test]
fn test_generic_instantiation() {
    let src = r#"
        F identity<T>(x:T)->T=x
        F main()->i64=identity(42)
    "#;

    compile_and_check(src, |ir| {
        assert!(ir.contains("define i64 @identity$i64(i64 %x)"));
        assert!(ir.contains("call i64 @identity$i64(i64 42)"));
    });
}

#[test]
fn test_error_undefined_variable() {
    let src = "F main()=x+1";  // 'x' not defined

    let result = compile(src);
    assert!(matches!(result, Err(CompileError::Type(TypeError::UndefinedVariable { .. }))));
}
```

### Edge Case Tests

**Boundary conditions** (100+ tests):
- Integer overflow: `i64::MAX + 1`
- Empty collections: `Vec.new().get(0)`
- Nested generics: `Vec<Option<HashMap<i64, String>>>`
- Mutually recursive functions
- Deep pattern matching nesting
- Long identifier names (1000+ chars)
- Unicode identifiers
- Invalid UTF-8 input

### Benchmark Suite

**Performance tests** (benches/ using Criterion):

```rust
fn bench_compile_stages(c: &mut Criterion) {
    let src = fs::read_to_string("fixtures/fibonacci.vais").unwrap();

    c.bench_function("lex", |b| {
        b.iter(|| tokenize(black_box(&src)))
    });

    c.bench_function("parse", |b| {
        let tokens = tokenize(&src).unwrap();
        b.iter(|| parse(black_box(tokens.clone())))
    });

    c.bench_function("typecheck", |b| {
        let module = parse_module(&src);
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_module(black_box(&module))
        })
    });

    c.bench_function("codegen", |b| {
        let (module, checker) = parse_and_check(&src);
        b.iter(|| {
            let mut codegen = CodeGenerator::new("bench");
            codegen.generate_module(black_box(&module))
        })
    });
}
```

**Results** (M2 MacBook Pro):
```
lex          time:   [12.5 µs 12.7 µs 12.9 µs]
parse        time:   [45.2 µs 46.1 µs 47.0 µs]
typecheck    time:   [78.3 µs 79.8 µs 81.5 µs]
codegen      time:   [105 µs 108 µs 111 µs]
full_compile time:   [241 µs 247 µs 253 µs]
```

---

## Performance Considerations

### Compilation Speed

**Target**: Compile 1000 lines of code in < 100ms (excluding LLVM/clang).

**Bottlenecks**:
1. **Lexing**: Fast (logos state machine, ~5µs per 100 tokens)
2. **Parsing**: Medium (recursive descent, ~50µs per 100 LOC)
3. **Type checking**: Slow (H-M inference, ~100µs per 100 LOC)
4. **Code generation**: Medium (text concatenation, ~50µs per 100 LOC)
5. **LLVM/clang**: Very slow (seconds for large programs)

**Optimizations**:
- **Parallel crate compilation**: Future work (requires module system)
- **Incremental compilation**: Cache type-checked modules
- **Lazy monomorphization**: Only instantiate called generics
- **Symbol table caching**: Avoid redundant lookups

### Runtime Performance

**Zero-cost abstractions**:
- Generics: Monomorphized (no boxing/vtables)
- Traits: Static dispatch (future: dynamic dispatch opt-in)
- Closures: Stack-allocated when possible
- Pattern matching: Compiled to jump tables

**Memory model**:
- Stack allocation by default (fast)
- Explicit heap via `Box<T>`, `Rc<T>`
- No garbage collection (predictable latency)

**LLVM optimizations**:
- **O2 default**: Balance compile time and runtime
- **O3 optional**: Aggressive inlining + loop opts
- **LTO**: Link-time optimization (future)

**Benchmark**: Fibonacci (n=40)
| Language | Time (ms) | Memory (KB) |
|----------|-----------|-------------|
| Vais O2  | 1,250     | 120         |
| Vais O3  | 850       | 120         |
| Rust O2  | 1,100     | 110         |
| C O2     | 980       | 100         |

*(Vais is within 10-20% of Rust/C performance)*

---

## Conclusion

The Vais compiler demonstrates a clean, modular architecture with clear separation of concerns:

1. **Lexer**: Fast tokenization using logos state machines
2. **Parser**: Predictive recursive descent for AI-friendly syntax
3. **Type Checker**: H-M inference with monomorphization tracking
4. **Code Generator**: LLVM IR text generation with debug support
5. **Optimizer**: Custom IR passes + LLVM pipeline
6. **LSP**: IDE integration for developer productivity
7. **Plugin System**: Extensibility for custom tooling
8. **i18n**: Localized errors for global developers

**Key strengths**:
- Token efficiency (40-60% reduction via single-letter keywords)
- Type safety (static typing with inference)
- Performance (native code via LLVM, zero-cost abstractions)
- Developer experience (LSP, REPL, formatter, debugger)
- Extensibility (plugins for custom lints/transforms)

**Future work** (see ROADMAP.md Phase 6):
- Incremental compilation
- WebAssembly target (wasm32)
- IntelliJ plugin
- Python/Node.js bindings for library use

**Metrics** (as of 2026-01-21):
- **Lines of code**: ~24,000 (Rust compiler + tools)
- **Test coverage**: 402 tests, 100% passing
- **Example programs**: 40+ demonstrating all features
- **Documentation**: 1,500+ lines across 4 documents
- **Supported platforms**: macOS (x86_64, arm64), Linux (x86_64)

For more information, see:
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) - Language specification
- [TUTORIAL.md](./TUTORIAL.md) - Getting started guide
- [STDLIB.md](./STDLIB.md) - Standard library reference
- [ROADMAP.md](../ROADMAP.md) - Project roadmap and status

---

**End of Architecture Document**
