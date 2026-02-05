//! DAP Protocol Types
//!
//! Core types used throughout the Debug Adapter Protocol.

use serde::{Deserialize, Serialize};

/// Base protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Sequence number (unique id for messages)
    pub seq: i64,
    /// Message type: "request", "response", "event"
    #[serde(rename = "type")]
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Request,
    Response,
    Event,
}

/// A client or debug adapter initiated request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    #[serde(flatten)]
    pub base: ProtocolMessage,
    /// The command to execute
    pub command: String,
    /// Object containing arguments for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// Response to a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(flatten)]
    pub base: ProtocolMessage,
    /// Sequence number of the corresponding request
    pub request_seq: i64,
    /// Outcome of the request
    pub success: bool,
    /// The command requested
    pub command: String,
    /// Contains the raw error in short form if success is false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Contains request result if success is true and error details if success is false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// A debug adapter initiated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub base: ProtocolMessage,
    /// Type of event
    pub event: String,
    /// Event-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

// ============================================================================
// Capabilities
// ============================================================================

/// Information about the capabilities of a debug adapter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    /// The debug adapter supports the configurationDone request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_configuration_done_request: Option<bool>,

    /// The debug adapter supports function breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_function_breakpoints: Option<bool>,

    /// The debug adapter supports conditional breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_conditional_breakpoints: Option<bool>,

    /// The debug adapter supports hit conditional breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_hit_conditional_breakpoints: Option<bool>,

    /// The debug adapter supports evaluate for hovers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_evaluate_for_hovers: Option<bool>,

    /// Available exception filter options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_breakpoint_filters: Option<Vec<ExceptionBreakpointsFilter>>,

    /// The debug adapter supports stepping back via stepBack and reverseContinue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_back: Option<bool>,

    /// The debug adapter supports setting a variable to a value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_set_variable: Option<bool>,

    /// The debug adapter supports restarting a frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_restart_frame: Option<bool>,

    /// The debug adapter supports the gotoTargets request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_goto_targets_request: Option<bool>,

    /// The debug adapter supports the stepInTargets request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_step_in_targets_request: Option<bool>,

    /// The debug adapter supports the completions request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_completions_request: Option<bool>,

    /// The debug adapter supports the modules request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_modules_request: Option<bool>,

    /// The debug adapter supports the restart request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_restart_request: Option<bool>,

    /// The debug adapter supports the exceptionInfo request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_exception_info_request: Option<bool>,

    /// The debug adapter supports value formatting options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_value_formatting_options: Option<bool>,

    /// The debug adapter supports the exceptionOptions for setExceptionBreakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_exception_options: Option<bool>,

    /// The debug adapter supports the terminate request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_terminate_request: Option<bool>,

    /// The debug adapter supports the disconnect request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_terminate_debuggee: Option<bool>,

    /// The debug adapter supports data breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_data_breakpoints: Option<bool>,

    /// The debug adapter supports the readMemory request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_read_memory_request: Option<bool>,

    /// The debug adapter supports the writeMemory request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_write_memory_request: Option<bool>,

    /// The debug adapter supports the disassemble request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_disassemble_request: Option<bool>,

    /// The debug adapter supports the cancel request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_cancel_request: Option<bool>,

    /// The debug adapter supports breakpoint locations request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_breakpoint_locations_request: Option<bool>,

    /// The debug adapter supports clipboard context for evaluate requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_clipboard_context: Option<bool>,

    /// The debug adapter supports stepping granularities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_stepping_granularity: Option<bool>,

    /// The debug adapter supports instruction breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_instruction_breakpoints: Option<bool>,

    /// The debug adapter supports the setExpression request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_set_expression: Option<bool>,
}

impl Capabilities {
    /// Create default Vais DAP capabilities
    pub fn vais_defaults() -> Self {
        Self {
            supports_configuration_done_request: Some(true),
            supports_function_breakpoints: Some(true),
            supports_conditional_breakpoints: Some(true),
            supports_hit_conditional_breakpoints: Some(true),
            supports_evaluate_for_hovers: Some(true),
            supports_set_variable: Some(true),
            supports_terminate_request: Some(true),
            supports_terminate_debuggee: Some(true),
            supports_restart_request: Some(true),
            supports_read_memory_request: Some(true),
            supports_disassemble_request: Some(true),
            exception_breakpoint_filters: Some(vec![ExceptionBreakpointsFilter {
                filter: "panic".to_string(),
                label: "Panic".to_string(),
                description: Some("Break on Vais panic".to_string()),
                default: Some(true),
                supports_condition: Some(false),
                condition_description: None,
            }]),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionBreakpointsFilter {
    pub filter: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_condition: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_description: Option<String>,
}

// ============================================================================
// Source
// ============================================================================

/// A Source is a descriptor for source code
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The short name of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The path of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The identifier of the source (used if no path is available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i64>,
    /// The checksums associated with this source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<Checksum>>,
    /// Origin of this source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Hint for how to present this source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<SourcePresentationHint>,
}

impl Source {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path: String = path.into();
        let name = std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        Self {
            name,
            path: Some(path),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourcePresentationHint {
    Normal,
    Emphasize,
    Deemphasize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    MD5,
    SHA1,
    SHA256,
    #[serde(rename = "timestamp")]
    Timestamp,
}

// ============================================================================
// Breakpoint
// ============================================================================

/// Information about a Breakpoint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    /// Unique identifier for the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// If true, breakpoint could be set
    pub verified: bool,
    /// An optional message about the state of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The source where the breakpoint is located
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The start line of the actual range covered by the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Start position of the source range covered by the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// End line of the actual range covered by the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// End position of the source range covered by the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
    /// Memory reference to where the breakpoint is set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_reference: Option<String>,
    /// Offset from instruction reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

/// Properties of a breakpoint or logpoint passed to setBreakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    /// The source line of the breakpoint
    pub line: i64,
    /// Start position within source line of the breakpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// An optional expression for conditional breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// An optional expression for hit count based breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    /// If this attribute exists and is non-empty, this is a logpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

/// Properties of a breakpoint passed to setFunctionBreakpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionBreakpoint {
    /// The name of the function
    pub name: String,
    /// An optional expression for conditional breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// An optional expression for hit count based breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
}

// ============================================================================
// Stack Frame
// ============================================================================

/// A Stackframe contains information about one frame in the call stack
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    /// An identifier for the stack frame
    pub id: i64,
    /// The name of the stack frame (typically function or method name)
    pub name: String,
    /// The source of the frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The line within the file of the frame
    pub line: i64,
    /// The column within the line
    pub column: i64,
    /// End line of the range covered by the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// End column of the range covered by the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
    /// Indicates whether this frame can be restarted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_restart: Option<bool>,
    /// Memory reference for the current instruction pointer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_pointer_reference: Option<String>,
    /// Module associated with this frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_id: Option<serde_json::Value>,
    /// Presentation hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<StackFramePresentationHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StackFramePresentationHint {
    Normal,
    Label,
    Subtle,
}

// ============================================================================
// Scope and Variable
// ============================================================================

/// A Scope is a named container for variables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    /// Name of the scope
    pub name: String,
    /// Hint for how to present this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<ScopePresentationHint>,
    /// The variables of this scope can be retrieved by passing this to variables request
    pub variables_reference: i64,
    /// Number of named variables in this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// Number of indexed variables in this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
    /// If true, the number of variables is large or expensive to retrieve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expensive: Option<bool>,
    /// The source for this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Start line of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Start column of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// End line of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// End column of the range covered by this scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScopePresentationHint {
    Arguments,
    Locals,
    Registers,
}

/// A Variable is a name/value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    /// The variable's name
    pub name: String,
    /// The variable's value
    pub value: String,
    /// The type of the variable's value
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub var_type: Option<String>,
    /// Properties of a variable that can be used to determine how to render the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<VariablePresentationHint>,
    /// A hint to the client on how to present the value for formatting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_name: Option<String>,
    /// If variablesReference > 0, the variable is structured and can be retrieved
    pub variables_reference: i64,
    /// Number of named child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// Number of indexed child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
    /// Memory reference for the variable if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reference: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablePresentationHint {
    /// The kind of variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Set of attributes represented as an array of strings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    /// Visibility of variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// If true, clients can use the value of evaluateName to evaluate the variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lazy: Option<bool>,
}

// ============================================================================
// Thread
// ============================================================================

/// A Thread in a debuggee
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    /// Unique identifier for the thread
    pub id: i64,
    /// A name of the thread
    pub name: String,
}

// ============================================================================
// Module
// ============================================================================

/// A Module object represents a row in the modules view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Unique identifier for the module
    pub id: serde_json::Value,
    /// A name of the module
    pub name: String,
    /// Path of the module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// True if the module is optimized
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_optimized: Option<bool>,
    /// True if the module is a user module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user_code: Option<bool>,
    /// Version of module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// User understandable description of if symbols were found for the module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_status: Option<String>,
    /// Logical full path to the symbol file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_file_path: Option<String>,
    /// Module created or modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time_stamp: Option<String>,
    /// Address range covered by this module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_range: Option<String>,
}

// ============================================================================
// Stop Reason
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StoppedReason {
    Step,
    Breakpoint,
    Exception,
    Pause,
    Entry,
    Goto,
    #[serde(rename = "function breakpoint")]
    FunctionBreakpoint,
    #[serde(rename = "data breakpoint")]
    DataBreakpoint,
    #[serde(rename = "instruction breakpoint")]
    InstructionBreakpoint,
}

// ============================================================================
// Stepping Granularity
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SteppingGranularity {
    Statement,
    Line,
    Instruction,
}

// ============================================================================
// Value Format
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueFormat {
    /// Display the value in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hex: Option<bool>,
}

// ============================================================================
// Evaluate Context
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvaluateContext {
    Watch,
    Repl,
    Hover,
    Clipboard,
    Variables,
}

// ============================================================================
// Disassembled Instruction
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisassembledInstruction {
    /// The address of the instruction
    pub address: String,
    /// Raw bytes representing the instruction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_bytes: Option<String>,
    /// Text representing the instruction and its operands
    pub instruction: String,
    /// Name of the symbol that corresponds with the location of this instruction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Source location that corresponds to this instruction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Source>,
    /// Line number in the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// Column number in the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// End line number in the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<i64>,
    /// End column number in the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i64>,
}
