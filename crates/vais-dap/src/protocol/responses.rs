//! DAP Response Types
//!
//! Response body types for all DAP commands.

use serde::{Deserialize, Serialize};
use super::types::*;

/// Response to 'initialize' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponseBody {
    #[serde(flatten)]
    pub capabilities: Capabilities,
}

/// Response to 'setBreakpoints' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsResponseBody {
    /// Information about the breakpoints
    pub breakpoints: Vec<Breakpoint>,
}

/// Response to 'setFunctionBreakpoints' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFunctionBreakpointsResponseBody {
    /// Information about the breakpoints
    pub breakpoints: Vec<Breakpoint>,
}

/// Response to 'setExceptionBreakpoints' request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExceptionBreakpointsResponseBody {
    /// Information about the exception breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakpoints: Option<Vec<Breakpoint>>,
}

/// Response to 'continue' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueResponseBody {
    /// If true, the 'continue' request has ignored the specified thread
    /// and continued all threads instead
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_continued: Option<bool>,
}

/// Response to 'threads' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadsResponseBody {
    /// All threads
    pub threads: Vec<Thread>,
}

/// Response to 'stackTrace' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceResponseBody {
    /// The frames of the stackframe
    pub stack_frames: Vec<StackFrame>,
    /// The total number of frames available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_frames: Option<i64>,
}

/// Response to 'scopes' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesResponseBody {
    /// The scopes of the stackframe
    pub scopes: Vec<Scope>,
}

/// Response to 'variables' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesResponseBody {
    /// All child variables
    pub variables: Vec<Variable>,
}

/// Response to 'setVariable' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetVariableResponseBody {
    /// The new value of the variable
    pub value: String,
    /// The type of the new value
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub var_type: Option<String>,
    /// If variablesReference > 0, the new value is structured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i64>,
    /// The number of named child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// The number of indexed child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
}

/// Response to 'source' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceResponseBody {
    /// Content of the source reference
    pub content: String,
    /// Optional content type (mime type) of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Response to 'evaluate' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponseBody {
    /// The result of the evaluate request
    pub result: String,
    /// The optional type of the evaluate result
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub result_type: Option<String>,
    /// Properties of a evaluate result that can be used to determine how to render the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<VariablePresentationHint>,
    /// If variablesReference > 0, the evaluate result is structured
    pub variables_reference: i64,
    /// The number of named child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i64>,
    /// The number of indexed child variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i64>,
    /// Optional memory reference to the evaluate result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reference: Option<String>,
}

/// Response to 'readMemory' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadMemoryResponseBody {
    /// The address of the first byte of data returned
    pub address: String,
    /// The number of unreadable bytes encountered after the last successfully read byte
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unreadable_bytes: Option<i64>,
    /// The bytes read from memory, encoded using base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Response to 'writeMemory' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteMemoryResponseBody {
    /// Optional property that should be returned when 'allowPartial' is true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Optional property that should be returned when 'allowPartial' is true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_written: Option<i64>,
}

/// Response to 'disassemble' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisassembleResponseBody {
    /// The list of disassembled instructions
    pub instructions: Vec<DisassembledInstruction>,
}

/// Response to 'modules' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulesResponseBody {
    /// All modules
    pub modules: Vec<Module>,
    /// The total number of modules available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_modules: Option<i64>,
}

/// Response to 'exceptionInfo' request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionInfoResponseBody {
    /// ID of the exception that was thrown
    pub exception_id: String,
    /// Descriptive text for the exception
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Mode that caused the exception notification to be raised
    pub break_mode: super::requests::ExceptionBreakMode,
    /// Detailed information about the exception
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ExceptionDetails>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionDetails {
    /// Message contained in the exception
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Short type name of the exception object
    #[serde(rename = "typeName", skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    /// Fully-qualified type name of the exception object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_type_name: Option<String>,
    /// Optional expression that can be evaluated in the current scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_name: Option<String>,
    /// Stack trace at the time the exception was thrown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Details of the exception contained by this exception, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_exception: Option<Vec<ExceptionDetails>>,
}
