//! Stack Frame Management
//!
//! Handles stack frame ID allocation and tracking.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::protocol::types::{StackFrame, Source};

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
    pub fn cache_frames(
        &mut self,
        thread_id: i64,
        raw_frames: Vec<RawFrame>,
    ) -> Vec<StackFrame> {
        // Clear existing cache for this thread
        self.cached_frames.remove(&thread_id);

        // Create new cached frames
        let mut cached = Vec::with_capacity(raw_frames.len());
        let mut dap_frames = Vec::with_capacity(raw_frames.len());

        for (index, raw) in raw_frames.into_iter().enumerate() {
            let id = self.next_id();

            // Store mapping
            self.frame_map.insert(id, FrameInfo {
                thread_id,
                frame_index: index,
            });

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

        let raw_frames = vec![
            RawFrame {
                function_name: "main".to_string(),
                source_path: None,
                line: 1,
                column: 1,
                instruction_pointer: 0x1000,
                module_name: None,
            },
        ];

        let dap_frames = manager.cache_frames(1, raw_frames);
        let id = dap_frames[0].id;

        assert!(manager.get_frame_info(id).is_some());

        manager.invalidate();

        assert!(manager.get_frame_info(id).is_none());
    }
}
