//! IR instructions and opcodes

use serde::{Deserialize, Serialize};
use crate::value::Value;

/// IR opcode for VM execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    // === Stack Operations ===
    /// Push a constant value onto the stack
    Const(Value),
    /// Pop the top value from the stack
    Pop,
    /// Duplicate the top value
    Dup,

    // === Variable Operations ===
    /// Load a variable by name (for globals/closures)
    Load(String),
    /// Store top of stack to variable (for globals/closures)
    Store(String),
    /// Load local variable by index (fast path)
    LoadLocal(u16),
    /// Store to local variable by index (fast path)
    StoreLocal(u16),
    /// Load input field
    LoadInput(String),
    /// Store to output field
    StoreOutput(String),

    // === Arithmetic Operations ===
    /// Add top two values
    Add,
    /// Subtract top two values
    Sub,
    /// Multiply top two values
    Mul,
    /// Divide top two values
    Div,
    /// Modulo (remainder)
    Mod,
    /// Negate top value
    Neg,

    // === Comparison Operations ===
    /// Equal comparison
    Eq,
    /// Not equal comparison
    Neq,
    /// Less than
    Lt,
    /// Greater than
    Gt,
    /// Less than or equal
    Lte,
    /// Greater than or equal
    Gte,

    // === Logical Operations ===
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,

    // === Collection Operations ===
    /// Get array/map length
    Len,
    /// Get element at index
    Index,
    /// Get field from struct
    GetField(String),
    /// Create array from top N elements
    MakeArray(usize),
    /// Create set from top N elements
    MakeSet(usize),
    /// Convert array on stack to set
    ArrayToSet,
    /// Create struct from field names
    MakeStruct(Vec<String>),
    /// Slice array: arr[start:end]
    Slice,
    /// Create range: start..end
    Range,
    /// Check containment: x @ arr
    Contains,
    /// Concatenate arrays or strings
    Concat,

    // === Array Operations (for FLOW) ===
    /// Map operation: apply function to each element
    Map(Box<Vec<Instruction>>),
    /// Filter operation: keep elements matching predicate
    Filter(Box<Vec<Instruction>>),
    /// Reduce operation with initial value
    Reduce(ReduceOp, Value),

    // === Optimized Array Operations (Native) ===
    /// Native map: multiply each element by constant
    MapMulConst(i64),
    /// Native map: add constant to each element
    MapAddConst(i64),
    /// Native map: subtract constant from each element
    MapSubConst(i64),
    /// Native map: divide each element by constant
    MapDivConst(i64),
    /// Native filter: keep elements greater than constant
    FilterGtConst(i64),
    /// Native filter: keep elements less than constant
    FilterLtConst(i64),
    /// Native filter: keep elements greater than or equal to constant
    FilterGteConst(i64),
    /// Native filter: keep elements less than or equal to constant
    FilterLteConst(i64),
    /// Native filter: keep elements equal to constant
    FilterEqConst(i64),
    /// Native filter: keep elements not equal to constant
    FilterNeqConst(i64),
    /// Native filter: keep even numbers
    FilterEven,
    /// Native filter: keep odd numbers
    FilterOdd,
    /// Native filter: keep positive numbers
    FilterPositive,
    /// Native filter: keep negative numbers
    FilterNegative,

    // === Control Flow ===
    /// Jump to instruction offset
    Jump(i32),
    /// Jump if top of stack is truthy
    JumpIf(i32),
    /// Jump if top of stack is falsy
    JumpIfNot(i32),
    /// Call a node by ID
    CallNode(String),
    /// Call a function by name with arg count
    Call(String, usize),
    /// Self-recursive call ($)
    SelfCall(usize),
    /// Tail-recursive self-call (optimized, no stack growth)
    TailSelfCall(usize),
    /// Create a closure (params, body instructions)
    MakeClosure(Vec<String>, Box<Vec<Instruction>>),
    /// Call a closure on stack with arg count
    CallClosure(usize),
    /// Return from current function/node
    Return,

    // === Optional/Error Handling ===
    /// Optional unwrap (?)
    Try,
    /// Coalesce: value ?? default
    Coalesce,
    /// Set catch handler: relative offset to handler on error
    SetCatch(usize),
    /// Clear catch handler
    ClearCatch,

    // === Built-in Functions ===
    /// Call built-in function by name
    CallBuiltin(String, usize), // (name, arg_count)

    // === Module ===
    /// Call function from a module: (module_path, fn_name, arg_count)
    CallModule(Vec<String>, String, usize),

    // === FFI (Foreign Function Interface) ===
    /// Call FFI function: (lib_name, fn_name, arg_count)
    CallFfi(String, String, usize),

    // === Async Operations ===
    /// Await a future/task
    Await,
    /// Spawn a new task
    Spawn,

    // === Channel Operations ===
    /// Create a new channel with optional capacity
    MakeChannel(usize),
    /// Send value to channel
    Send,
    /// Receive value from channel
    Recv,

    // === Parallel Collection Operations ===
    /// Parallel map
    ParallelMap(Box<Vec<Instruction>>),
    /// Parallel filter
    ParallelFilter(Box<Vec<Instruction>>),
    /// Parallel reduce
    ParallelReduce(ReduceOp, Value),

    // === Fused Operations (single-pass, no intermediate arrays) ===
    /// Map then Reduce in single pass
    MapReduce(Box<Vec<Instruction>>, ReduceOp, Value),
    /// Filter then Reduce in single pass
    FilterReduce(Box<Vec<Instruction>>, ReduceOp, Value),
    /// Map then Filter in single pass
    MapFilter(Box<Vec<Instruction>>, Box<Vec<Instruction>>),
    /// Map, Filter, then Reduce in single pass
    MapFilterReduce(Box<Vec<Instruction>>, Box<Vec<Instruction>>, ReduceOp, Value),

    // === Algebraic Effects ===
    /// Perform an effect operation: (effect_name, operation_name, arg_count)
    Perform(String, String, usize),
    /// Install effect handlers: (handler_count)
    InstallHandlers(usize),
    /// Define a handler: (effect, op, param_count, has_resume)
    DefineHandler(String, String, usize, bool),
    /// End of handler definition
    EndHandler,
    /// Uninstall effect handlers
    UninstallHandlers,
    /// Resume a continuation
    Resume,

    // === Special ===
    /// No operation
    Nop,
    /// Halt execution
    Halt,
    /// Raise an error
    Error(String),
}

/// Reduce operation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReduceOp {
    /// ./+ Sum
    Sum,
    /// Count elements
    Count,
    /// ./min Minimum
    Min,
    /// ./max Maximum
    Max,
    /// ./* Product
    Product,
    /// Average
    Avg,
    /// First element
    First,
    /// Last element
    Last,
    /// ./and Logical AND of all elements
    All,
    /// ./or Logical OR of all elements
    Any,
    /// Custom reducer with initial value and function
    Custom(Box<Vec<Instruction>>),
}

/// A single instruction with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: OpCode,
    pub comment: Option<String>,
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            comment: None,
        }
    }

    pub fn with_comment(opcode: OpCode, comment: impl Into<String>) -> Self {
        Self {
            opcode,
            comment: Some(comment.into()),
        }
    }
}

impl From<OpCode> for Instruction {
    fn from(opcode: OpCode) -> Self {
        Instruction::new(opcode)
    }
}

/// IR representation of a FLOW node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIR {
    /// Node identifier
    pub id: String,
    /// Node operation type
    pub op_type: NodeOpType,
    /// Instructions to execute
    pub instructions: Vec<Instruction>,
    /// Input port names
    pub inputs: Vec<String>,
    /// Output port names
    pub outputs: Vec<String>,
}

/// Node operation type in IR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeOpType {
    /// Simple transform (execute instructions)
    Transform,
    /// Map over array
    Map,
    /// Filter array
    Filter,
    /// Reduce array
    Reduce(ReduceOp),
    /// Branch based on condition
    Branch,
    /// Merge multiple inputs
    Merge,
    /// External fetch (placeholder)
    Fetch,
    /// External store (placeholder)
    Store,
    /// Validate data
    Validate,
}

/// IR representation of a FLOW edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeIR {
    /// Source node ID
    pub source_node: String,
    /// Source port (empty for default)
    pub source_port: String,
    /// Target node ID
    pub target_node: String,
    /// Target port (empty for default)
    pub target_port: String,
    /// Optional condition instructions
    pub condition: Option<Vec<Instruction>>,
}

impl NodeIR {
    pub fn new(id: impl Into<String>, op_type: NodeOpType) -> Self {
        Self {
            id: id.into(),
            op_type,
            instructions: Vec::new(),
            inputs: vec!["default".to_string()],
            outputs: vec!["default".to_string()],
        }
    }

    pub fn with_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.instructions = instructions;
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<String>) -> Self {
        self.outputs = outputs;
        self
    }
}

impl EdgeIR {
    pub fn new(
        source_node: impl Into<String>,
        source_port: impl Into<String>,
        target_node: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Self {
        Self {
            source_node: source_node.into(),
            source_port: source_port.into(),
            target_node: target_node.into(),
            target_port: target_port.into(),
            condition: None,
        }
    }

    pub fn simple(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, "default", target, "default")
    }

    pub fn with_condition(mut self, condition: Vec<Instruction>) -> Self {
        self.condition = Some(condition);
        self
    }
}
