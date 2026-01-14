//! Async Runtime for Vais VM
//!
//! Manages async tasks and channels for concurrent operations.

// Allow Arc for future multi-thread support
#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use vais_ir::{Value, TaskId, ChannelId, FutureState, ChannelState};

// Type alias for faster HashMap
type FastMap<K, V> = HashMap<K, V>;

/// Async runtime state (shared between threads)
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
            let mut state = state.lock().expect("task lock poisoned");
            *state = FutureState::Completed(Box::new(value));
        }
    }

    /// Mark a task as failed
    #[allow(dead_code)]
    pub fn fail_task(&mut self, id: TaskId, error: String) {
        if let Some(state) = self.tasks.get(&id) {
            let mut state = state.lock().expect("task lock poisoned");
            *state = FutureState::Failed(error);
        }
    }

    /// Get the current state of a task
    pub fn get_task_state(&self, id: TaskId) -> Option<FutureState> {
        self.tasks.get(&id).map(|s| s.lock().expect("task lock poisoned").clone())
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
        if let Some(chan) = self.channels.get(&id) {
            let mut chan = chan.lock().expect("channel lock poisoned");
            if chan.closed {
                return Err("channel closed".to_string());
            }
            // Simple blocking send (busy wait for now)
            while chan.buffer.len() >= chan.capacity {
                drop(chan);
                thread::yield_now();
                chan = self.channels.get(&id).expect("channel gone").lock().expect("channel lock poisoned");
                if chan.closed {
                    return Err("channel closed".to_string());
                }
            }
            chan.buffer.push(value);
            Ok(())
        } else {
            Err("channel not found".to_string())
        }
    }

    /// Receive a value from a channel (blocking)
    pub fn recv_from_channel(&self, id: ChannelId) -> Result<Value, String> {
        if let Some(chan) = self.channels.get(&id) {
            // Simple blocking receive (busy wait for now)
            loop {
                let mut chan_guard = chan.lock().expect("channel lock poisoned");
                if !chan_guard.buffer.is_empty() {
                    return Ok(chan_guard.buffer.remove(0));
                }
                if chan_guard.closed {
                    return Err("channel closed".to_string());
                }
                drop(chan_guard);
                thread::yield_now();
            }
        } else {
            Err("channel not found".to_string())
        }
    }

    /// Close a channel
    #[allow(dead_code)]
    pub fn close_channel(&self, id: ChannelId) {
        if let Some(chan) = self.channels.get(&id) {
            let mut chan = chan.lock().expect("channel lock poisoned");
            chan.closed = true;
        }
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
