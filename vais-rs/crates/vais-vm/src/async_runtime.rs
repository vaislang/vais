//! Async Runtime for Vais VM
//!
//! Manages async tasks and channels for concurrent operations.

// Allow Arc for future multi-thread support
#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use vais_ir::{Value, TaskId, ChannelId, FutureState, ChannelState};

// Type alias for faster HashMap
#[allow(dead_code)]
type FastMap<K, V> = HashMap<K, V>;

/// Async runtime state (shared between threads)
///
/// Note: Currently unused as async/await is planned but not fully implemented.
/// This module provides the infrastructure for future async support.
#[derive(Debug)]
#[allow(dead_code)]
pub struct AsyncRuntime {
    /// Task states (TaskId -> FutureState)
    tasks: FastMap<TaskId, Arc<Mutex<FutureState>>>,
    /// Channels (ChannelId -> ChannelState)
    channels: FastMap<ChannelId, Arc<Mutex<ChannelState>>>,
    /// Next task ID
    next_task_id: TaskId,
    /// Next channel ID
    next_channel_id: ChannelId,
}

#[allow(dead_code)]
impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> Self {
        Self {
            tasks: FastMap::new(),
            channels: FastMap::new(),
            next_task_id: 1,
            next_channel_id: 1,
        }
    }

    /// Create a new task and return its ID
    pub fn create_task(&mut self) -> TaskId {
        let id = self.next_task_id;
        self.next_task_id += 1;
        self.tasks.insert(id, Arc::new(Mutex::new(FutureState::Pending)));
        id
    }

    /// Complete a task with a value
    pub fn complete_task(&mut self, id: TaskId, value: Value) {
        if let Some(state) = self.tasks.get(&id) {
            if let Ok(mut state) = state.lock() {
                *state = FutureState::Completed(Box::new(value));
            }
        }
    }

    /// Mark a task as failed (reserved for error propagation in async operations)
    pub fn fail_task(&mut self, id: TaskId, error: String) {
        if let Some(state) = self.tasks.get(&id) {
            if let Ok(mut state) = state.lock() {
                *state = FutureState::Failed(error);
            }
        }
    }

    /// Get the current state of a task
    pub fn get_task_state(&self, id: TaskId) -> Option<FutureState> {
        self.tasks.get(&id).and_then(|s| s.lock().ok().map(|g| g.clone()))
    }

    /// Create a new channel with the given capacity
    pub fn create_channel(&mut self, capacity: usize) -> ChannelId {
        let id = self.next_channel_id;
        self.next_channel_id += 1;
        self.channels.insert(id, Arc::new(Mutex::new(ChannelState::new(capacity))));
        id
    }

    /// Send a value to a channel (blocking)
    pub fn send_to_channel(&self, id: ChannelId, value: Value) -> Result<(), String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        let mut guard = chan.lock().map_err(|_| "channel lock poisoned")?;
        if guard.closed {
            return Err("channel closed".to_string());
        }
        // Blocking send with backoff sleep to reduce CPU usage
        while guard.buffer.len() >= guard.capacity {
            drop(guard);
            thread::sleep(Duration::from_micros(100)); // Backoff instead of busy spin
            let chan = self.channels.get(&id).ok_or("channel gone")?;
            guard = chan.lock().map_err(|_| "channel lock poisoned")?;
            if guard.closed {
                return Err("channel closed".to_string());
            }
        }
        guard.buffer.push_back(value);
        Ok(())
    }

    /// Receive a value from a channel (blocking)
    pub fn recv_from_channel(&self, id: ChannelId) -> Result<Value, String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        // Blocking receive with backoff sleep to reduce CPU usage
        loop {
            let mut guard = chan.lock().map_err(|_| "channel lock poisoned")?;
            if let Some(value) = guard.buffer.pop_front() {
                return Ok(value);
            }
            if guard.closed {
                return Err("channel closed".to_string());
            }
            drop(guard);
            thread::sleep(Duration::from_micros(100)); // Backoff instead of busy spin
        }
    }

    /// Close a channel (reserved for cleanup operations)
    pub fn close_channel(&self, id: ChannelId) {
        if let Some(chan) = self.channels.get(&id) {
            if let Ok(mut guard) = chan.lock() {
                guard.closed = true;
            }
        }
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
