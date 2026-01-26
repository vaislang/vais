//! DAP Request Types
//!
//! Request argument types for all DAP commands.

use serde::{Deserialize, Serialize};
use super::types::*;

/// Arguments for 'initialize' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestArguments {
    /// The ID of the client using this adapter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// The human readable name of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// The ID of the debug adapter
    pub adapter_id: String,
    /// The ISO-639 locale of the client using this adapter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// If true, lines start at 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_start_at1: Option<bool>,
    /// If true, columns start at 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_start_at1: Option<bool>,
    /// Determines in what format paths are specified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_format: Option<PathFormat>,
    /// Client supports the optional type attribute for variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_type: Option<bool>,
    /// Client supports the paging of variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_variable_paging: Option<bool>,
    /// Client supports the runInTerminal request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_run_in_terminal_request: Option<bool>,
    /// Client supports memory references
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_references: Option<bool>,
    /// Client supports progress reporting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_progress_reporting: Option<bool>,
    /// Client supports the invalidated event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_invalidated_event: Option<bool>,
    /// Client supports the memory event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_memory_event: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathFormat {
    Path,
    Uri,
}

/// Arguments for 'launch' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    /// If true, stop the debuggee after launch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_debug: Option<bool>,
    /// If true, stop at the entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_on_entry: Option<bool>,
    /// Vais source file to debug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// Compiled binary path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    /// Command line arguments passed to the debuggee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Working directory for the debuggee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Automatically compile before debugging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_compile: Option<bool>,
    /// Optimization level for compilation (0-3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_level: Option<i32>,
}

/// Arguments for 'attach' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRequestArguments {
    /// Process ID to attach to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i64>,
    /// Program path (for symbol loading)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// Stop all threads on attach
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_on_attach: Option<bool>,
}

/// Arguments for 'disconnect' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisconnectRequestArguments {
    /// If true, the debuggee should be terminated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminate_debuggee: Option<bool>,
    /// If true, restart the debug session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<bool>,
    /// If true, the debuggee should stay suspended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_debuggee: Option<bool>,
}

/// Arguments for 'terminate' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminateRequestArguments {
    /// If true, restart the debuggee after termination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<bool>,
}

/// Arguments for 'setBreakpoints' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsRequestArguments {
    /// The source location of the breakpoints
    pub source: Source,
    /// The code locations of the breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
    /// Deprecated: Use 'breakpoints' instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<i64>>,
    /// A hint for how breakpoint verification is done
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_modified: Option<bool>,
}

/// Arguments for 'setFunctionBreakpoints' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFunctionBreakpointsRequestArguments {
    /// The function names of the breakpoints
    pub breakpoints: Vec<FunctionBreakpoint>,
}

/// Arguments for 'setExceptionBreakpoints' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExceptionBreakpointsRequestArguments {
    /// Set of exception filters specified by their ID
    pub filters: Vec<String>,
    /// Configuration options for selected exceptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_options: Option<Vec<ExceptionFilterOptions>>,
    /// Configuration options for exceptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_options: Option<Vec<ExceptionOptions>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionFilterOptions {
    /// ID of an exception filter
    pub filter_id: String,
    /// An optional expression for conditional exceptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionOptions {
    /// A path that selects a single or multiple exceptions in a tree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<ExceptionPathSegment>>,
    /// Condition when a thrown exception should result in a break
    pub break_mode: ExceptionBreakMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionPathSegment {
    /// If true, all names are negated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negate: Option<bool>,
    /// Depending on the value of 'negate', these are exception names or exception ids
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExceptionBreakMode {
    Never,
    Always,
    Unhandled,
    UserUnhandled,
}

/// Arguments for 'continue' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueRequestArguments {
    /// Continue execution for the specified thread
    pub thread_id: i64,
    /// If this optional flag is true, execution is resumed only for the specified thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
}

/// Arguments for 'next' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextRequestArguments {
    /// Execute next for this thread
    pub thread_id: i64,
    /// If this optional flag is true, all other suspended threads are not resumed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// Stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

/// Arguments for 'stepIn' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepInRequestArguments {
    /// Execute step in for this thread
    pub thread_id: i64,
    /// If this optional flag is true, all other suspended threads are not resumed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// Optional id of the target to step into
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i64>,
    /// Stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

/// Arguments for 'stepOut' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepOutRequestArguments {
    /// Execute step out for this thread
    pub thread_id: i64,
    /// If this optional flag is true, all other suspended threads are not resumed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_thread: Option<bool>,
    /// Stepping granularity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granularity: Option<SteppingGranularity>,
}

/// Arguments for 'pause' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseRequestArguments {
    /// Pause execution for this thread
    pub thread_id: i64,
}

/// Arguments for 'stackTrace' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceRequestArguments {
    /// Retrieve the stacktrace for this thread
    pub thread_id: i64,
    /// The index of the first frame to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i64>,
    /// The maximum number of frames to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<i64>,
    /// Specifies details on how to format the stack frames
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StackFrameFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameFormat {
    /// Displays parameters for the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<bool>,
    /// Displays the types of parameters for the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_types: Option<bool>,
    /// Displays the names of parameters for the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_names: Option<bool>,
    /// Displays the values of parameters for the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<bool>,
    /// Displays the line number of the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<bool>,
    /// Displays the module of the stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<bool>,
    /// Includes all stack frames, including those the debug adapter might otherwise hide
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_all: Option<bool>,
}

/// Arguments for 'scopes' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesRequestArguments {
    /// Retrieve the scopes for this stackframe
    pub frame_id: i64,
}

/// Arguments for 'variables' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesRequestArguments {
    /// The Variable reference
    pub variables_reference: i64,
    /// Optional filter to limit the child variables to either named or indexed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<VariablesFilter>,
    /// The index of the first variable to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// The number of variables to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    /// Specifies details on how to format the Variable values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariablesFilter {
    Indexed,
    Named,
}

/// Arguments for 'setVariable' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableRequestArguments {
    /// The reference of the variable container
    pub variables_reference: i64,
    /// The name of the variable in the container
    pub name: String,
    /// The value of the variable
    pub value: String,
    /// Specifies details on how to format the response value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

/// Arguments for 'source' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRequestArguments {
    /// Specifies the source content to load
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The reference to the source
    pub source_reference: i64,
}

/// Arguments for 'evaluate' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequestArguments {
    /// The expression to evaluate
    pub expression: String,
    /// Evaluate the expression in the scope of this stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<i64>,
    /// The context in which the evaluate request is run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<EvaluateContext>,
    /// Specifies details on how to format the Evaluate result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

/// Arguments for 'readMemory' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMemoryRequestArguments {
    /// Memory reference to the base location from which data should be read
    pub memory_reference: String,
    /// Optional offset (can be negative) to be applied to the reference location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Number of bytes to read at the specified location
    pub count: i64,
}

/// Arguments for 'writeMemory' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteMemoryRequestArguments {
    /// Memory reference to the base location to which data should be written
    pub memory_reference: String,
    /// Optional offset (can be negative) to be applied to the reference location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Indicates that the client allows the data to be written with 'best effort'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_partial: Option<bool>,
    /// Bytes to write, encoded using base64
    pub data: String,
}

/// Arguments for 'disassemble' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisassembleRequestArguments {
    /// Memory reference to the base location containing the instructions to disassemble
    pub memory_reference: String,
    /// Optional offset to be applied to the reference location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Optional offset to be applied to instruction pointer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_offset: Option<i64>,
    /// Number of instructions to disassemble starting at the specified location
    pub instruction_count: i64,
    /// If true, the adapter should attempt to resolve memory addresses and other values to symbolic names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_symbols: Option<bool>,
}

/// Arguments for 'restart' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestartRequestArguments {
    /// The latest version of the 'launch' or 'attach' configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}
