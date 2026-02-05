//! DAP Event Types
//!
//! Event body types sent from debug adapter to client.

use super::types::*;
use serde::{Deserialize, Serialize};

/// Body of 'initialized' event
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitializedEventBody {}

/// Body of 'stopped' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoppedEventBody {
    /// The reason for the event
    pub reason: StoppedReason,
    /// Additional information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The thread which was stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    /// If 'allThreadsStopped' is true, a debug adapter can announce that all threads have stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_focus_hint: Option<bool>,
    /// A value of true hints to the frontend that this event should not change the focus
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// If 'allThreadsStopped' is true, all threads have stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_stopped: Option<bool>,
    /// Ids of the breakpoints that triggered the event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_breakpoint_ids: Option<Vec<i64>>,
}

impl StoppedEventBody {
    pub fn breakpoint(thread_id: i64, breakpoint_ids: Vec<i64>) -> Self {
        Self {
            reason: StoppedReason::Breakpoint,
            description: Some("Breakpoint hit".to_string()),
            thread_id: Some(thread_id),
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: Some(breakpoint_ids),
            preserve_focus_hint: None,
            text: None,
        }
    }

    pub fn step(thread_id: i64) -> Self {
        Self {
            reason: StoppedReason::Step,
            description: Some("Step completed".to_string()),
            thread_id: Some(thread_id),
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: None,
            preserve_focus_hint: None,
            text: None,
        }
    }

    pub fn pause(thread_id: i64) -> Self {
        Self {
            reason: StoppedReason::Pause,
            description: Some("Paused".to_string()),
            thread_id: Some(thread_id),
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: None,
            preserve_focus_hint: None,
            text: None,
        }
    }

    pub fn exception(thread_id: i64, description: String) -> Self {
        Self {
            reason: StoppedReason::Exception,
            description: Some(description),
            thread_id: Some(thread_id),
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: None,
            preserve_focus_hint: None,
            text: None,
        }
    }

    pub fn entry(thread_id: i64) -> Self {
        Self {
            reason: StoppedReason::Entry,
            description: Some("Entry point reached".to_string()),
            thread_id: Some(thread_id),
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: None,
            preserve_focus_hint: None,
            text: None,
        }
    }
}

/// Body of 'continued' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuedEventBody {
    /// The thread which was continued
    pub thread_id: i64,
    /// If 'allThreadsContinued' is true, all threads have continued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_threads_continued: Option<bool>,
}

/// Body of 'exited' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitedEventBody {
    /// The exit code returned from the debuggee
    pub exit_code: i64,
}

/// Body of 'terminated' event
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatedEventBody {
    /// If true, the debug session should be restarted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<serde_json::Value>,
}

/// Body of 'thread' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadEventBody {
    /// The reason for the event
    pub reason: ThreadEventReason,
    /// The identifier of the thread
    pub thread_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThreadEventReason {
    Started,
    Exited,
}

/// Body of 'output' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEventBody {
    /// The output category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<OutputCategory>,
    /// The output to report
    pub output: String,
    /// Support for keeping an output log organized by grouping related messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<OutputGroup>,
    /// If an attribute 'variablesReference' exists, the output contains objects which can be retrieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables_reference: Option<i64>,
    /// An optional source location where the output was produced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// An optional source location line where the output was produced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    /// An optional source location column where the output was produced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    /// Optional data to report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputCategory {
    Console,
    Important,
    Stdout,
    Stderr,
    Telemetry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputGroup {
    Start,
    StartCollapsed,
    End,
}

impl OutputEventBody {
    pub fn stdout(output: impl Into<String>) -> Self {
        Self {
            category: Some(OutputCategory::Stdout),
            output: output.into(),
            group: None,
            variables_reference: None,
            source: None,
            line: None,
            column: None,
            data: None,
        }
    }

    pub fn stderr(output: impl Into<String>) -> Self {
        Self {
            category: Some(OutputCategory::Stderr),
            output: output.into(),
            group: None,
            variables_reference: None,
            source: None,
            line: None,
            column: None,
            data: None,
        }
    }

    pub fn console(output: impl Into<String>) -> Self {
        Self {
            category: Some(OutputCategory::Console),
            output: output.into(),
            group: None,
            variables_reference: None,
            source: None,
            line: None,
            column: None,
            data: None,
        }
    }
}

/// Body of 'breakpoint' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointEventBody {
    /// The reason for the event
    pub reason: BreakpointEventReason,
    /// The 'id' attribute is used to find the target breakpoint
    pub breakpoint: Breakpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakpointEventReason {
    Changed,
    New,
    Removed,
}

/// Body of 'module' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleEventBody {
    /// The reason for the event
    pub reason: ModuleEventReason,
    /// The new, changed, or removed module
    pub module: Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModuleEventReason {
    New,
    Changed,
    Removed,
}

/// Body of 'loadedSource' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedSourceEventBody {
    /// The reason for the event
    pub reason: LoadedSourceEventReason,
    /// The new, changed, or removed source
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoadedSourceEventReason {
    New,
    Changed,
    Removed,
}

/// Body of 'process' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEventBody {
    /// The logical name of the process
    pub name: String,
    /// The system process id of the debugged process
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_process_id: Option<i64>,
    /// If true, the process is running on the same computer as the debug adapter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_local_process: Option<bool>,
    /// Describes how the debug engine started debugging this process
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_method: Option<ProcessStartMethod>,
    /// The size of a pointer or address for this process
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProcessStartMethod {
    Launch,
    Attach,
    AttachForSuspendedLaunch,
}

/// Body of 'capabilities' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesEventBody {
    /// The set of updated capabilities
    pub capabilities: Capabilities,
}

/// Body of 'progressStart' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressStartEventBody {
    /// An ID that must be used in subsequent 'progressUpdate' and 'progressEnd' events
    pub progress_id: String,
    /// Mandatory (short) title of the progress reporting
    pub title: String,
    /// The request ID that this progress report is related to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<i64>,
    /// If true, the request that reports progress may be canceled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellable: Option<bool>,
    /// Optional more detailed progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional progress percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

/// Body of 'progressUpdate' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressUpdateEventBody {
    /// The ID that was introduced in the initial 'progressStart' event
    pub progress_id: String,
    /// Optional more detailed progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Optional progress percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

/// Body of 'progressEnd' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEndEventBody {
    /// The ID that was introduced in the initial 'progressStart' event
    pub progress_id: String,
    /// Optional final message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Body of 'invalidated' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidatedEventBody {
    /// Optional set of logical areas that got invalidated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub areas: Option<Vec<InvalidatedArea>>,
    /// If specified, the client only needs to refetch data related to this thread
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    /// If specified, the client only needs to refetch data related to this stack frame
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_frame_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InvalidatedArea {
    All,
    Stacks,
    Threads,
    Variables,
}

/// Body of 'memory' event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEventBody {
    /// Memory reference of a memory range that has been updated
    pub memory_reference: String,
    /// Starting offset in bytes where memory has been updated
    pub offset: i64,
    /// Number of bytes updated
    pub count: i64,
}
