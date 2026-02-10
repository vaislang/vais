//! Stack Frame Management
//!
//! Handles stack frame ID allocation and tracking.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{Source, StackFrame};

/// Manages stack frame IDs and their mapping to thread/frame indices
#[derive(Debug, Default)]
pub struct StackManager {
    /// Next frame ID
    next_id: AtomicI64,
    /// Mapping from frame ID to (thread_id, frame_index)
    frame_map: HashMap<i64, FrameInfo>,
    /// Cached stack frames per thread
    cached_frames: HashMap<i64, Vec<CachedFrame>>,
}

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub thread_id: i64,
    pub frame_index: usize,
}

#[derive(Debug, Clone)]
pub struct CachedFrame {
    pub id: i64,
    pub name: String,
    pub source_path: Option<String>,
    pub line: i64,
    pub column: i64,
    pub instruction_pointer: u64,
}

impl StackManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicI64::new(1),
            frame_map: HashMap::new(),
            cached_frames: HashMap::new(),
        }
    }

    fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Clear cached frames (call when execution resumes)
    pub fn invalidate(&mut self) {
        self.frame_map.clear();
        self.cached_frames.clear();
    }

    /// Cache stack frames for a thread and return DAP stack frames
    pub fn cache_frames(&mut self, thread_id: i64, raw_frames: Vec<RawFrame>) -> Vec<StackFrame> {
        // Clear existing cache for this thread
        self.cached_frames.remove(&thread_id);

        // Create new cached frames
        let mut cached = Vec::with_capacity(raw_frames.len());
        let mut dap_frames = Vec::with_capacity(raw_frames.len());

        for (index, raw) in raw_frames.into_iter().enumerate() {
            let id = self.next_id();

            // Store mapping
            self.frame_map.insert(
                id,
                FrameInfo {
                    thread_id,
                    frame_index: index,
                },
            );

            // Create cached frame
            let cache_entry = CachedFrame {
                id,
                name: raw.function_name.clone(),
                source_path: raw.source_path.clone(),
                line: raw.line,
                column: raw.column,
                instruction_pointer: raw.instruction_pointer,
            };
            cached.push(cache_entry.clone());

            // Create DAP frame
            let source = raw.source_path.map(Source::from_path);
            dap_frames.push(StackFrame {
                id,
                name: raw.function_name,
                source,
                line: raw.line,
                column: raw.column,
                end_line: None,
                end_column: None,
                can_restart: Some(false),
                instruction_pointer_reference: Some(format!("0x{:x}", raw.instruction_pointer)),
                module_id: raw.module_name.map(|n| serde_json::json!(n)),
                presentation_hint: None,
            });
        }

        self.cached_frames.insert(thread_id, cached);
        dap_frames
    }

    /// Get frame info by ID
    pub fn get_frame_info(&self, frame_id: i64) -> Option<&FrameInfo> {
        self.frame_map.get(&frame_id)
    }

    /// Get cached frame by ID
    pub fn get_cached_frame(&self, frame_id: i64) -> Option<&CachedFrame> {
        let info = self.frame_map.get(&frame_id)?;
        let frames = self.cached_frames.get(&info.thread_id)?;
        frames.get(info.frame_index)
    }

    /// Get all cached frames for a thread
    pub fn get_thread_frames(&self, thread_id: i64) -> Option<&Vec<CachedFrame>> {
        self.cached_frames.get(&thread_id)
    }

    /// Get the topmost frame for a thread
    pub fn get_top_frame(&self, thread_id: i64) -> Option<&CachedFrame> {
        self.cached_frames.get(&thread_id).and_then(|f| f.first())
    }
}

/// Raw frame data from debugger
#[derive(Debug, Clone)]
pub struct RawFrame {
    pub function_name: String,
    pub source_path: Option<String>,
    pub line: i64,
    pub column: i64,
    pub instruction_pointer: u64,
    pub module_name: Option<String>,
}

/// Granularity of stepping operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepGranularity {
    #[default]
    Statement,   // Statement-level (default)
    Line,        // Line-level
    Instruction, // Instruction-level
}

/// Stepping mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepMode {
    /// Step over: next statement at same depth
    Over,
    /// Step in: enter function calls
    In { target_function: Option<String> },
    /// Step out: return from current function
    Out,
}

/// Controls precise stepping behavior
pub struct StepController {
    /// Current step mode (None = running freely)
    active_step: Option<ActiveStep>,
}

#[derive(Debug, Clone)]
struct ActiveStep {
    mode: StepMode,
    granularity: StepGranularity,
    /// Stack depth when step was initiated
    origin_depth: usize,
    /// Line number when step was initiated
    origin_line: i64,
    /// Thread ID that initiated the step
    thread_id: i64,
}

impl StepController {
    pub fn new() -> Self {
        Self { active_step: None }
    }

    /// Start a new step operation
    pub fn start_step(
        &mut self,
        mode: StepMode,
        granularity: StepGranularity,
        current_depth: usize,
        current_line: i64,
        thread_id: i64,
    ) {
        self.active_step = Some(ActiveStep {
            mode,
            granularity,
            origin_depth: current_depth,
            origin_line: current_line,
            thread_id,
        });
    }

    /// Check if execution should stop at the current location
    /// Returns true if a step is complete and we should break
    pub fn should_stop(
        &self,
        current_depth: usize,
        current_line: i64,
        current_function: Option<&str>,
        thread_id: i64,
    ) -> bool {
        let Some(step) = &self.active_step else {
            return false;
        };

        // Only check for the thread that initiated the step
        if step.thread_id != thread_id {
            return false;
        }

        match &step.mode {
            StepMode::Over => {
                // Stop if: same or lower depth AND different line (for Statement/Line)
                // OR same or lower depth (for Instruction)
                match step.granularity {
                    StepGranularity::Instruction => current_depth <= step.origin_depth,
                    _ => current_depth <= step.origin_depth && current_line != step.origin_line,
                }
            }
            StepMode::In { target_function } => {
                // If target specified, only stop when entering that function
                if let Some(target) = target_function {
                    if let Some(func) = current_function {
                        return func == target;
                    }
                    return false;
                }
                // Otherwise stop at next statement (deeper or same level)
                match step.granularity {
                    StepGranularity::Instruction => true,
                    _ => current_line != step.origin_line || current_depth > step.origin_depth,
                }
            }
            StepMode::Out => {
                // Stop when depth is less than origin (returned from function)
                current_depth < step.origin_depth
            }
        }
    }

    /// Complete the current step (called when execution stops)
    pub fn complete_step(&mut self) {
        self.active_step = None;
    }

    /// Cancel any active step
    pub fn cancel(&mut self) {
        self.active_step = None;
    }

    /// Check if a step operation is active
    pub fn is_stepping(&self) -> bool {
        self.active_step.is_some()
    }

    /// Get the current step mode
    pub fn current_mode(&self) -> Option<&StepMode> {
        self.active_step.as_ref().map(|s| &s.mode)
    }
}

impl Default for StepController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_frames() {
        let mut manager = StackManager::new();

        let raw_frames = vec![
            RawFrame {
                function_name: "main".to_string(),
                source_path: Some("/test/main.vais".to_string()),
                line: 10,
                column: 1,
                instruction_pointer: 0x1000,
                module_name: Some("test".to_string()),
            },
            RawFrame {
                function_name: "foo".to_string(),
                source_path: Some("/test/foo.vais".to_string()),
                line: 20,
                column: 5,
                instruction_pointer: 0x2000,
                module_name: None,
            },
        ];

        let dap_frames = manager.cache_frames(1, raw_frames);

        assert_eq!(dap_frames.len(), 2);
        assert_eq!(dap_frames[0].name, "main");
        assert_eq!(dap_frames[0].line, 10);
        assert_eq!(dap_frames[1].name, "foo");

        // Test lookup
        let frame_info = manager.get_frame_info(dap_frames[0].id).unwrap();
        assert_eq!(frame_info.thread_id, 1);
        assert_eq!(frame_info.frame_index, 0);

        let cached = manager.get_cached_frame(dap_frames[1].id).unwrap();
        assert_eq!(cached.name, "foo");
        assert_eq!(cached.instruction_pointer, 0x2000);
    }

    #[test]
    fn test_invalidate() {
        let mut manager = StackManager::new();

        let raw_frames = vec![RawFrame {
            function_name: "main".to_string(),
            source_path: None,
            line: 1,
            column: 1,
            instruction_pointer: 0x1000,
            module_name: None,
        }];

        let dap_frames = manager.cache_frames(1, raw_frames);
        let id = dap_frames[0].id;

        assert!(manager.get_frame_info(id).is_some());

        manager.invalidate();

        assert!(manager.get_frame_info(id).is_none());
    }

    // StepController tests

    #[test]
    fn test_step_over() {
        let mut controller = StepController::new();

        // Start step over at depth 1, line 10, thread 1
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);

        assert!(controller.is_stepping());

        // Same line, same depth - should not stop
        assert!(!controller.should_stop(1, 10, Some("main"), 1));

        // Different line, same depth - should stop
        assert!(controller.should_stop(1, 11, Some("main"), 1));

        // Same line, deeper (function call) - should not stop
        assert!(!controller.should_stop(2, 10, Some("foo"), 1));

        // Different line, deeper - should not stop
        assert!(!controller.should_stop(2, 20, Some("foo"), 1));

        // Back to same depth, different line - should stop
        assert!(controller.should_stop(1, 11, Some("main"), 1));
    }

    #[test]
    fn test_step_over_skip_deeper() {
        let mut controller = StepController::new();

        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);

        // Go deeper (into function call)
        assert!(!controller.should_stop(2, 20, Some("foo"), 1));
        assert!(!controller.should_stop(3, 30, Some("bar"), 1));

        // Return to original depth at different line - should stop
        assert!(controller.should_stop(1, 11, Some("main"), 1));
    }

    #[test]
    fn test_step_in() {
        let mut controller = StepController::new();

        // Step in without target function
        controller.start_step(
            StepMode::In { target_function: None },
            StepGranularity::Statement,
            1,
            10,
            1,
        );

        // Same line - should not stop
        assert!(!controller.should_stop(1, 10, Some("main"), 1));

        // Different line at same depth - should stop
        assert!(controller.should_stop(1, 11, Some("main"), 1));

        // Deeper level (entered function) - should stop
        assert!(controller.should_stop(2, 20, Some("foo"), 1));
    }

    #[test]
    fn test_step_in_target() {
        let mut controller = StepController::new();

        // Step in with target function
        controller.start_step(
            StepMode::In {
                target_function: Some("target_func".to_string()),
            },
            StepGranularity::Statement,
            1,
            10,
            1,
        );

        // Enter different function - should not stop
        assert!(!controller.should_stop(2, 20, Some("other_func"), 1));

        // Enter target function - should stop
        assert!(controller.should_stop(2, 30, Some("target_func"), 1));

        // Without function name - should not stop
        assert!(!controller.should_stop(2, 40, None, 1));
    }

    #[test]
    fn test_step_out() {
        let mut controller = StepController::new();

        // Start at depth 2 (inside a function)
        controller.start_step(StepMode::Out, StepGranularity::Statement, 2, 20, 1);

        // Same depth - should not stop
        assert!(!controller.should_stop(2, 21, Some("foo"), 1));

        // Deeper - should not stop
        assert!(!controller.should_stop(3, 30, Some("bar"), 1));

        // Shallower (returned from function) - should stop
        assert!(controller.should_stop(1, 10, Some("main"), 1));
    }

    #[test]
    fn test_step_granularity_instruction() {
        let mut controller = StepController::new();

        // Step over with instruction granularity
        controller.start_step(StepMode::Over, StepGranularity::Instruction, 1, 10, 1);

        // Instruction granularity stops at same depth regardless of line
        assert!(controller.should_stop(1, 10, Some("main"), 1));
        assert!(controller.should_stop(1, 11, Some("main"), 1));

        // Deeper should not stop
        assert!(!controller.should_stop(2, 20, Some("foo"), 1));

        // Reset and test step in with instruction granularity
        controller.start_step(
            StepMode::In { target_function: None },
            StepGranularity::Instruction,
            1,
            10,
            1,
        );

        // Instruction granularity always stops on step in
        assert!(controller.should_stop(1, 10, Some("main"), 1));
        assert!(controller.should_stop(2, 20, Some("foo"), 1));
    }

    #[test]
    fn test_step_thread_isolation() {
        let mut controller = StepController::new();

        // Start step on thread 1
        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);

        // Thread 1 should stop at different line
        assert!(controller.should_stop(1, 11, Some("main"), 1));

        // Thread 2 should not be affected
        assert!(!controller.should_stop(1, 11, Some("main"), 2));
        assert!(!controller.should_stop(2, 20, Some("foo"), 2));
    }

    #[test]
    fn test_step_complete_cancel() {
        let mut controller = StepController::new();

        controller.start_step(StepMode::Over, StepGranularity::Statement, 1, 10, 1);

        assert!(controller.is_stepping());
        assert!(controller.current_mode().is_some());

        // Complete step
        controller.complete_step();

        assert!(!controller.is_stepping());
        assert!(controller.current_mode().is_none());

        // Start new step
        controller.start_step(StepMode::Out, StepGranularity::Line, 2, 20, 1);

        assert!(controller.is_stepping());
        assert_eq!(
            controller.current_mode(),
            Some(&StepMode::Out)
        );

        // Cancel step
        controller.cancel();

        assert!(!controller.is_stepping());
        assert!(controller.current_mode().is_none());
    }

    #[test]
    fn test_step_granularity_default() {
        let default = StepGranularity::default();
        assert_eq!(default, StepGranularity::Statement);
    }

    #[test]
    fn test_step_controller_default() {
        let controller = StepController::default();
        assert!(!controller.is_stepping());
        assert!(controller.current_mode().is_none());
    }
}
