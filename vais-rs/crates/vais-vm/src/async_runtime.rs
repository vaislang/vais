//! Async Runtime for Vais VM
//!
//! Manages async tasks and channels for concurrent operations.
//! Uses Condvar for efficient blocking on channel operations.

// Allow Arc for future multi-thread support
#![allow(clippy::arc_with_non_send_sync)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;

use vais_ir::{Value, TaskId, ChannelId, FutureState};

// Type alias for faster HashMap
#[allow(dead_code)]
type FastMap<K, V> = HashMap<K, V>;

/// Channel with Condvar for efficient blocking
/// Replaces busy-wait with proper condition variable signaling
#[derive(Debug)]
pub struct CondvarChannel {
    /// Mutex-protected channel state
    state: Mutex<CondvarChannelState>,
    /// Condition variable for send (notified when space available)
    not_full: Condvar,
    /// Condition variable for recv (notified when data available)
    not_empty: Condvar,
}

#[derive(Debug)]
struct CondvarChannelState {
    buffer: VecDeque<Value>,
    capacity: usize,
    closed: bool,
}

impl CondvarChannel {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            state: Mutex::new(CondvarChannelState {
                buffer: VecDeque::with_capacity(cap),
                capacity: cap,
                closed: false,
            }),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
        }
    }

    /// Send a value to the channel (blocking with Condvar)
    pub fn send(&self, value: Value) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| "channel lock poisoned")?;

        // Wait until there's space or channel is closed
        while state.buffer.len() >= state.capacity && !state.closed {
            state = self.not_full.wait(state).map_err(|_| "condvar wait failed")?;
        }

        if state.closed {
            return Err("channel closed".to_string());
        }

        state.buffer.push_back(value);

        // Notify one waiting receiver
        self.not_empty.notify_one();
        Ok(())
    }

    /// Send with timeout
    pub fn send_timeout(&self, value: Value, timeout: Duration) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|_| "channel lock poisoned")?;

        // Wait with timeout
        while state.buffer.len() >= state.capacity && !state.closed {
            let (new_state, timeout_result) = self.not_full
                .wait_timeout(state, timeout)
                .map_err(|_| "condvar wait failed")?;
            state = new_state;
            if timeout_result.timed_out() {
                return Err("send timeout".to_string());
            }
        }

        if state.closed {
            return Err("channel closed".to_string());
        }

        state.buffer.push_back(value);
        self.not_empty.notify_one();
        Ok(())
    }

    /// Receive a value from the channel (blocking with Condvar)
    pub fn recv(&self) -> Result<Value, String> {
        let mut state = self.state.lock().map_err(|_| "channel lock poisoned")?;

        // Wait until there's data or channel is closed
        while state.buffer.is_empty() && !state.closed {
            state = self.not_empty.wait(state).map_err(|_| "condvar wait failed")?;
        }

        if let Some(value) = state.buffer.pop_front() {
            // Notify one waiting sender
            self.not_full.notify_one();
            return Ok(value);
        }

        // Buffer empty and closed
        Err("channel closed".to_string())
    }

    /// Receive with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Value, String> {
        let mut state = self.state.lock().map_err(|_| "channel lock poisoned")?;

        while state.buffer.is_empty() && !state.closed {
            let (new_state, timeout_result) = self.not_empty
                .wait_timeout(state, timeout)
                .map_err(|_| "condvar wait failed")?;
            state = new_state;
            if timeout_result.timed_out() {
                return Err("recv timeout".to_string());
            }
        }

        if let Some(value) = state.buffer.pop_front() {
            self.not_full.notify_one();
            return Ok(value);
        }

        Err("channel closed".to_string())
    }

    /// Try to receive without blocking
    pub fn try_recv(&self) -> Result<Option<Value>, String> {
        let mut state = self.state.lock().map_err(|_| "channel lock poisoned")?;

        if let Some(value) = state.buffer.pop_front() {
            self.not_full.notify_one();
            return Ok(Some(value));
        }

        if state.closed {
            return Err("channel closed".to_string());
        }

        Ok(None)
    }

    /// Close the channel
    pub fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
        }
        // Wake up all waiters
        self.not_full.notify_all();
        self.not_empty.notify_all();
    }

    /// Check if channel is closed
    pub fn is_closed(&self) -> bool {
        self.state.lock().map(|s| s.closed).unwrap_or(true)
    }

    /// Get current buffer length
    pub fn len(&self) -> usize {
        self.state.lock().map(|s| s.buffer.len()).unwrap_or(0)
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Async runtime state (shared between threads)
///
/// Uses CondvarChannel for efficient blocking on channel operations.
/// This eliminates busy-wait loops and reduces CPU usage significantly.
#[derive(Debug)]
#[allow(dead_code)]
pub struct AsyncRuntime {
    /// Task states (TaskId -> FutureState)
    tasks: FastMap<TaskId, Arc<Mutex<FutureState>>>,
    /// Channels with Condvar for efficient blocking
    channels: FastMap<ChannelId, Arc<CondvarChannel>>,
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
        self.channels.insert(id, Arc::new(CondvarChannel::new(capacity)));
        id
    }

    /// Send a value to a channel (blocking with Condvar - no busy-wait)
    pub fn send_to_channel(&self, id: ChannelId, value: Value) -> Result<(), String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        chan.send(value)
    }

    /// Send with timeout
    pub fn send_to_channel_timeout(&self, id: ChannelId, value: Value, timeout: Duration) -> Result<(), String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        chan.send_timeout(value, timeout)
    }

    /// Receive a value from a channel (blocking with Condvar - no busy-wait)
    pub fn recv_from_channel(&self, id: ChannelId) -> Result<Value, String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        chan.recv()
    }

    /// Receive with timeout
    pub fn recv_from_channel_timeout(&self, id: ChannelId, timeout: Duration) -> Result<Value, String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        chan.recv_timeout(timeout)
    }

    /// Try to receive without blocking
    pub fn try_recv_from_channel(&self, id: ChannelId) -> Result<Option<Value>, String> {
        let chan = self.channels.get(&id).ok_or("channel not found")?;
        chan.try_recv()
    }

    /// Close a channel
    pub fn close_channel(&self, id: ChannelId) {
        if let Some(chan) = self.channels.get(&id) {
            chan.close();
        }
    }

    /// Check if a channel is closed
    pub fn is_channel_closed(&self, id: ChannelId) -> bool {
        self.channels.get(&id).map(|c| c.is_closed()).unwrap_or(true)
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
