//! Statement and block definitions

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};
use crate::{AstNode, Ident, QualifiedName, ExternalRef, Type, Expr};

// =============================================================================
// Meta Block
// =============================================================================

/// Meta entry key types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetaKey {
    Domain,
    Determinism,
    Idempotent,
    Pure,
    Timeout,
    Retry,
}

/// A single META entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaEntry {
    pub key: MetaKey,
    pub value: MetaValue,
    pub span: Span,
}

/// Meta entry value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaValue {
    Bool(bool),
    String(String),
    Integer(i64),
    Duration(String),
}

/// META block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaBlock {
    pub entries: Vec<MetaEntry>,
    pub span: Span,
}

// =============================================================================
// Input/Output Blocks
// =============================================================================

/// Input field constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConstraint {
    pub expr: Expr,
    pub span: Span,
}

/// Input field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputField {
    pub name: Ident,
    pub ty: Type,
    pub constraints: Vec<FieldConstraint>,
    pub span: Span,
}

/// INPUT block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBlock {
    pub fields: Vec<InputField>,
    pub span: Span,
}

/// Output field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputField {
    pub name: Ident,
    pub ty: Type,
    pub constraints: Vec<FieldConstraint>,
    pub span: Span,
}

/// OUTPUT block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputBlock {
    pub fields: Vec<OutputField>,
    pub span: Span,
}

// =============================================================================
// Intent Block
// =============================================================================

/// Goal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalType {
    Transform,
    Validate,
    Aggregate,
    Filter,
    Route,
    Compose,
    Fetch,
}

/// Priority types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriorityKind {
    Correctness,
    Performance,
    Memory,
    Latency,
    Throughput,
}

/// Failure strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureStrategy {
    Abort,
    Retry,
    Fallback(ExternalRef),
    Default(Expr),
}

/// Goal specification (inputs -> outputs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalSpec {
    pub inputs: Vec<Expr>,
    pub outputs: Vec<Expr>,
    pub span: Span,
}

/// INTENT block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentBlock {
    pub goal_type: GoalType,
    pub goal_spec: GoalSpec,
    pub priorities: Vec<PriorityKind>,
    pub on_failure: Option<FailureStrategy>,
    pub span: Span,
}

// =============================================================================
// Constraint Block
// =============================================================================

/// Constraint types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintKind {
    Require,
    Forbid,
    Prefer,
    Invariant,
}

/// A single constraint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub expr: Expr,
    pub span: Span,
}

/// CONSTRAINT block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintBlock {
    pub constraints: Vec<Constraint>,
    pub span: Span,
}

// =============================================================================
// Flow Block
// =============================================================================

/// Flow operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    // Data transformation
    Map,
    Filter,
    Reduce,
    Transform,
    Flatten,
    Group,
    Sort,
    Distinct,

    // Flow control
    Branch,
    Merge,
    Split,
    Join,
    Race,

    // External
    Fetch,
    Store,
    Call,
    Emit,
    Subscribe,

    // Validation
    Validate,
    Sanitize,
    Authorize,
}

/// Node parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeParam {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

/// Flow node definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: Ident,
    pub op_type: OpType,
    pub params: Vec<NodeParam>,
    pub custom_op: Option<ExternalRef>,
    pub span: Span,
}

/// Flow edge definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowEdge {
    pub source: Expr,
    pub target: Expr,
    pub params: Vec<NodeParam>,
    pub condition: Option<Expr>,
    pub span: Span,
}

/// FLOW block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowBlock {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub span: Span,
}

// =============================================================================
// Execution Block
// =============================================================================

/// Target platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    Any,
    Cpu,
    Gpu,
    Wasm,
    Native,
}

/// Memory modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryMode {
    Bounded(String),  // Size string like "256MB"
    Unbounded,
    StackOnly,
}

/// Isolation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationKind {
    None,
    Thread,
    Process,
    Container,
}

/// Cache modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CacheMode {
    None,
    Lru(Option<u64>),
    Ttl(Option<String>),
}

/// EXECUTION block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBlock {
    pub parallel: bool,
    pub target: TargetKind,
    pub memory: MemoryMode,
    pub isolation: IsolationKind,
    pub cache: CacheMode,
    pub span: Span,
}

// =============================================================================
// Verify Block
// =============================================================================

/// Verify entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifyKind {
    Assert,
    Property,
    Invariant,
    Postcondition,
    Test,
}

/// A single verify entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyEntry {
    pub kind: VerifyKind,
    pub expr: Option<Expr>,
    pub test_ref: Option<ExternalRef>,
    pub span: Span,
}

/// VERIFY block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyBlock {
    pub entries: Vec<VerifyEntry>,
    pub span: Span,
}
