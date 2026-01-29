//! Debugger Backend
//!
//! Abstract debugger interface with LLDB implementation.
//! Uses LLDB's command-line interface (lldb -batch) for cross-platform support.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, trace, warn};

use crate::error::{DapError, DapResult};
use crate::protocol::types::*;

/// Raw stack frame from debugger
#[derive(Debug, Clone)]
pub struct RawStackFrame {
    pub function_name: String,
    pub instruction_pointer: u64,
    pub module_name: Option<String>,
}

/// Raw variable from debugger
#[derive(Debug, Clone)]
pub struct RawVariable {
    pub name: String,
    pub value: String,
    pub type_name: Option<String>,
    pub evaluate_name: Option<String>,
    pub memory_reference: Option<String>,
}

/// Debugger backend that wraps LLDB
pub struct Debugger {
    /// LLDB process
    lldb: Option<LldbProcess>,
    /// Source breakpoints: (file, line) -> breakpoint_id
    source_breakpoints: HashMap<(String, i64), u32>,
    /// Function breakpoints: name -> breakpoint_id
    function_breakpoints: HashMap<String, u32>,
    /// Exception filters enabled
    exception_filters: Vec<String>,
    /// Whether stopped on entry
    stopped_on_entry: AtomicBool,
}

struct LldbProcess {
    child: Child,
    stdin_tx: std::sync::mpsc::Sender<String>,
    stdout_rx: std::sync::mpsc::Receiver<String>,
}

// SAFETY: Debugger is safe to Send/Sync because:
// - Access to the Child process is controlled by RwLock in DebugSession
// - All operations on the process are synchronized
unsafe impl Send for Debugger {}
unsafe impl Sync for Debugger {}

impl Debugger {
    pub fn new() -> Self {
        Self {
            lldb: None,
            source_breakpoints: HashMap::new(),
            function_breakpoints: HashMap::new(),
            exception_filters: Vec::new(),
            stopped_on_entry: AtomicBool::new(false),
        }
    }

    // ========================================================================
    // Process Control
    // ========================================================================

    pub async fn launch(
        &mut self,
        binary: &str,
        args: &[String],
        cwd: Option<&str>,
        env: Option<&HashMap<String, String>>,
        stop_on_entry: bool,
    ) -> DapResult<()> {
        info!("Launching LLDB for: {}", binary);

        // Start LLDB process
        let mut cmd = Command::new("lldb");
        cmd.arg("--no-use-colors")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

        let mut child = cmd.spawn()
            .map_err(|e| DapError::Debugger(format!("Failed to start LLDB: {}", e)))?;

        // Set up communication channels
        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        let (stdin_tx, stdin_rx) = std::sync::mpsc::channel::<String>();
        let (stdout_tx, stdout_rx) = std::sync::mpsc::channel::<String>();

        // Spawn stdin writer thread
        std::thread::spawn(move || {
            let mut stdin = stdin;
            while let Ok(cmd) = stdin_rx.recv() {
                if writeln!(stdin, "{}", cmd).is_err() {
                    break;
                }
                if stdin.flush().is_err() {
                    break;
                }
            }
        });

        // Spawn stdout reader thread
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if stdout_tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.lldb = Some(LldbProcess {
            child,
            stdin_tx,
            stdout_rx,
        });

        // Load the binary
        self.send_command(&format!("file {}", binary))?;
        self.wait_for_prompt()?;

        // Set args if any
        if !args.is_empty() {
            let args_str = args.join(" ");
            self.send_command(&format!("settings set target.run-args {}", args_str))?;
            self.wait_for_prompt()?;
        }

        // Set environment variables
        if let Some(env) = env {
            for (key, value) in env {
                self.send_command(&format!("settings set target.env-vars {}={}", key, value))?;
                self.wait_for_prompt()?;
            }
        }

        // Set stop-on-entry breakpoint if requested
        if stop_on_entry {
            self.send_command("breakpoint set --name main")?;
            self.wait_for_prompt()?;
            self.stopped_on_entry.store(true, Ordering::SeqCst);
        }

        // Start the process
        self.send_command("process launch --stop-at-entry")?;
        self.wait_for_prompt()?;

        Ok(())
    }

    pub async fn attach(&mut self, pid: u32, stop_on_attach: bool) -> DapResult<()> {
        info!("Attaching LLDB to PID: {}", pid);

        // Start LLDB process
        let mut cmd = Command::new("lldb");
        cmd.arg("--no-use-colors")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| DapError::Debugger(format!("Failed to start LLDB: {}", e)))?;

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        let (stdin_tx, stdin_rx) = std::sync::mpsc::channel::<String>();
        let (stdout_tx, stdout_rx) = std::sync::mpsc::channel::<String>();

        std::thread::spawn(move || {
            let mut stdin = stdin;
            while let Ok(cmd) = stdin_rx.recv() {
                if writeln!(stdin, "{}", cmd).is_err() {
                    break;
                }
                if stdin.flush().is_err() {
                    break;
                }
            }
        });

        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if stdout_tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.lldb = Some(LldbProcess {
            child,
            stdin_tx,
            stdout_rx,
        });

        // Attach to process
        self.send_command(&format!("process attach --pid {}", pid))?;
        self.wait_for_prompt()?;

        if stop_on_attach {
            self.stopped_on_entry.store(true, Ordering::SeqCst);
        }

        Ok(())
    }

    pub async fn detach(&mut self) -> DapResult<()> {
        if self.lldb.is_some() {
            self.send_command("process detach")?;
            self.wait_for_prompt()?;
            self.send_command("quit")?;
            self.lldb = None;
        }
        Ok(())
    }

    pub async fn terminate(&mut self) -> DapResult<()> {
        if self.lldb.is_some() {
            self.send_command("process kill")?;
            self.wait_for_prompt()?;
            self.send_command("quit")?;
            if let Some(ref mut lldb) = self.lldb {
                let _ = lldb.child.wait();
            }
            self.lldb = None;
        }
        Ok(())
    }

    pub async fn is_stopped_on_entry(&self) -> bool {
        self.stopped_on_entry.load(Ordering::SeqCst)
    }

    // ========================================================================
    // Breakpoints
    // ========================================================================

    pub async fn set_breakpoint(
        &mut self,
        file: &str,
        line: i64,
        condition: Option<&str>,
    ) -> DapResult<i64> {
        let mut cmd = format!("breakpoint set --file {} --line {}", file, line);
        if let Some(cond) = condition {
            cmd.push_str(&format!(" --condition '{}'", cond));
        }

        self.send_command(&cmd)?;
        let output = self.wait_for_prompt()?;

        // Parse breakpoint ID from output like "Breakpoint 1: where = ..."
        let bp_id = parse_breakpoint_id(&output)?;
        self.source_breakpoints.insert((file.to_string(), line), bp_id);

        Ok(line) // Return the actual line (LLDB might adjust)
    }

    pub async fn set_function_breakpoint(
        &mut self,
        name: &str,
        condition: Option<&str>,
    ) -> DapResult<()> {
        let mut cmd = format!("breakpoint set --name {}", name);
        if let Some(cond) = condition {
            cmd.push_str(&format!(" --condition '{}'", cond));
        }

        self.send_command(&cmd)?;
        let output = self.wait_for_prompt()?;

        let bp_id = parse_breakpoint_id(&output)?;
        self.function_breakpoints.insert(name.to_string(), bp_id);

        Ok(())
    }

    pub async fn clear_breakpoints_for_source(&mut self, file: &str) -> DapResult<()> {
        let to_remove: Vec<_> = self.source_breakpoints.iter()
            .filter(|((f, _), _)| f == file)
            .map(|((f, l), id)| (f.clone(), *l, *id))
            .collect();

        for (f, l, id) in to_remove {
            self.send_command(&format!("breakpoint delete {}", id))?;
            self.wait_for_prompt()?;
            self.source_breakpoints.remove(&(f, l));
        }

        Ok(())
    }

    pub async fn clear_function_breakpoints(&mut self) -> DapResult<()> {
        let ids: Vec<u32> = self.function_breakpoints.drain().map(|(_, id)| id).collect();
        for id in ids {
            self.send_command(&format!("breakpoint delete {}", id))?;
            self.wait_for_prompt()?;
        }
        Ok(())
    }

    pub async fn set_exception_breakpoints(&mut self, filters: &[String]) -> DapResult<()> {
        // Clear existing exception breakpoints
        self.exception_filters.clear();

        for filter in filters {
            match filter.as_str() {
                "panic" => {
                    // Set breakpoint on Vais panic function
                    self.send_command("breakpoint set --name vais_panic")?;
                    self.wait_for_prompt()?;
                }
                _ => {
                    warn!("Unknown exception filter: {}", filter);
                }
            }
            self.exception_filters.push(filter.clone());
        }

        Ok(())
    }

    // ========================================================================
    // Execution Control
    // ========================================================================

    pub async fn continue_all(&mut self) -> DapResult<()> {
        self.stopped_on_entry.store(false, Ordering::SeqCst);
        self.send_command("continue")?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn continue_thread(&mut self, thread_id: i64) -> DapResult<()> {
        self.stopped_on_entry.store(false, Ordering::SeqCst);
        self.send_command(&format!("thread continue {}", thread_id))?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn step_over(&mut self, thread_id: i64) -> DapResult<()> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command("thread step-over")?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn step_in(&mut self, thread_id: i64) -> DapResult<()> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command("thread step-in")?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn step_out(&mut self, thread_id: i64) -> DapResult<()> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command("thread step-out")?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn step_instruction(&mut self, thread_id: i64) -> DapResult<()> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command("thread step-inst")?;
        self.wait_for_stop()?;
        Ok(())
    }

    pub async fn pause(&mut self, _thread_id: i64) -> DapResult<()> {
        self.send_command("process interrupt")?;
        self.wait_for_stop()?;
        Ok(())
    }

    // ========================================================================
    // Threads
    // ========================================================================

    pub async fn get_threads(&self) -> DapResult<Vec<Thread>> {
        let output = self.send_command_get_output("thread list")?;

        // Parse thread list output
        let mut threads = Vec::new();
        for line in output.lines() {
            if let Some(thread) = parse_thread_line(line) {
                threads.push(thread);
            }
        }

        if threads.is_empty() {
            // Return at least main thread
            threads.push(Thread {
                id: 1,
                name: "main".to_string(),
            });
        }

        Ok(threads)
    }

    // ========================================================================
    // Stack
    // ========================================================================

    pub async fn get_stack_frames(&self, thread_id: i64) -> DapResult<Vec<RawStackFrame>> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;

        let output = self.send_command_get_output("thread backtrace")?;

        // Parse backtrace output
        let mut frames = Vec::new();
        for line in output.lines() {
            if let Some(frame) = parse_frame_line(line) {
                frames.push(frame);
            }
        }

        Ok(frames)
    }

    // ========================================================================
    // Variables
    // ========================================================================

    pub async fn get_local_variables(&self, thread_id: i64, frame_idx: usize) -> DapResult<Vec<RawVariable>> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command(&format!("frame select {}", frame_idx))?;
        self.wait_for_prompt()?;

        let output = self.send_command_get_output("frame variable")?;

        // Parse variables
        let vars = parse_variables(&output);
        Ok(vars)
    }

    pub async fn get_arguments(&self, thread_id: i64, frame_idx: usize) -> DapResult<Vec<RawVariable>> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command(&format!("frame select {}", frame_idx))?;
        self.wait_for_prompt()?;

        let output = self.send_command_get_output("frame variable -a")?;

        // Parse variables (arguments only)
        let vars = parse_variables(&output);
        Ok(vars)
    }

    pub async fn get_registers(&self, thread_id: i64) -> DapResult<Vec<RawVariable>> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;

        let output = self.send_command_get_output("register read")?;

        // Parse registers
        let regs = parse_registers(&output);
        Ok(regs)
    }

    /// Get child variables of a compound type (struct, array, pointer)
    ///
    /// Uses lldb's expression evaluation to access nested members.
    /// For example, `eval_path="myStruct.field"` will evaluate `myStruct.field`
    /// and return its child members.
    pub async fn get_children(
        &self,
        thread_id: i64,
        frame_idx: usize,
        eval_path: &str,
    ) -> DapResult<Vec<RawVariable>> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command(&format!("frame select {}", frame_idx))?;
        self.wait_for_prompt()?;

        // Use "frame variable" with the path to get child members
        // The -T flag shows types, and we specify the variable path
        let cmd = format!("frame variable -T {}", eval_path);
        let output = self.send_command_get_output(&cmd)?;

        // Parse the output - lldb returns nested children with indentation
        let vars = parse_nested_variables(&output, eval_path);
        Ok(vars)
    }

    pub async fn set_variable(
        &mut self,
        thread_id: i64,
        frame_idx: usize,
        name: &str,
        value: &str,
    ) -> DapResult<(String, Option<String>, Option<i64>)> {
        self.send_command(&format!("thread select {}", thread_id))?;
        self.wait_for_prompt()?;
        self.send_command(&format!("frame select {}", frame_idx))?;
        self.wait_for_prompt()?;

        self.send_command(&format!("expression {} = {}", name, value))?;
        self.wait_for_prompt()?;

        // Get the new value
        let output = self.send_command_get_output(&format!("frame variable {}", name))?;
        let vars = parse_variables(&output);

        if let Some(var) = vars.first() {
            Ok((var.value.clone(), var.type_name.clone(), None))
        } else {
            Ok((value.to_string(), None, None))
        }
    }

    // ========================================================================
    // Evaluate
    // ========================================================================

    pub async fn evaluate(
        &self,
        expression: &str,
        thread_id: i64,
        frame_idx: usize,
    ) -> DapResult<(String, Option<String>, i64)> {
        if thread_id > 0 {
            self.send_command(&format!("thread select {}", thread_id))?;
            self.wait_for_prompt()?;
            self.send_command(&format!("frame select {}", frame_idx))?;
            self.wait_for_prompt()?;
        }

        let output = self.send_command_get_output(&format!("expression {}", expression))?;

        // Parse result like "(int) $0 = 42"
        if let Some((type_name, value)) = parse_expression_result(&output) {
            Ok((value, Some(type_name), 0))
        } else {
            Ok((output.trim().to_string(), None, 0))
        }
    }

    // ========================================================================
    // Memory
    // ========================================================================

    pub async fn read_memory(&self, address: u64, count: usize) -> DapResult<Vec<u8>> {
        let output = self.send_command_get_output(
            &format!("memory read --size 1 --count {} 0x{:x}", count, address)
        )?;

        // Parse memory output
        let data = parse_memory_output(&output);
        Ok(data)
    }

    pub async fn write_memory(&mut self, address: u64, data: &[u8]) -> DapResult<()> {
        for (i, byte) in data.iter().enumerate() {
            let addr = address + i as u64;
            self.send_command(&format!("memory write 0x{:x} 0x{:02x}", addr, byte))?;
            self.wait_for_prompt()?;
        }
        Ok(())
    }

    // ========================================================================
    // Disassembly
    // ========================================================================

    pub async fn disassemble(
        &self,
        address: u64,
        instruction_offset: i64,
        instruction_count: usize,
        resolve_symbols: bool,
    ) -> DapResult<Vec<DisassembledInstruction>> {
        let start_addr = (address as i64 + instruction_offset) as u64;

        let output = self.send_command_get_output(
            &format!("disassemble --start-address 0x{:x} --count {}", start_addr, instruction_count)
        )?;

        // Parse disassembly output
        let instructions = parse_disassembly(&output, resolve_symbols);
        Ok(instructions)
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    fn send_command(&self, cmd: &str) -> DapResult<()> {
        trace!("LLDB command: {}", cmd);
        let lldb = self.lldb.as_ref().ok_or(DapError::ProcessNotRunning)?;
        lldb.stdin_tx.send(cmd.to_string())
            .map_err(|e| DapError::Debugger(format!("Failed to send command: {}", e)))
    }

    fn send_command_get_output(&self, cmd: &str) -> DapResult<String> {
        self.send_command(cmd)?;
        self.wait_for_prompt()
    }

    fn wait_for_prompt(&self) -> DapResult<String> {
        let lldb = self.lldb.as_ref().ok_or(DapError::ProcessNotRunning)?;

        let mut output = String::new();
        let timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();

        loop {
            match lldb.stdout_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(line) => {
                    trace!("LLDB output: {}", line);
                    output.push_str(&line);
                    output.push('\n');

                    // Check for prompt
                    if line.contains("(lldb)") || line.starts_with("Process ") {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if start.elapsed() > timeout {
                        return Err(DapError::Timeout("LLDB response".to_string()));
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(DapError::ProcessNotRunning);
                }
            }
        }

        Ok(output)
    }

    fn wait_for_stop(&self) -> DapResult<String> {
        // Wait for process to stop (with longer timeout)
        let lldb = self.lldb.as_ref().ok_or(DapError::ProcessNotRunning)?;

        let mut output = String::new();
        let timeout = std::time::Duration::from_secs(300); // 5 minutes
        let start = std::time::Instant::now();

        loop {
            match lldb.stdout_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(line) => {
                    trace!("LLDB output: {}", line);
                    output.push_str(&line);
                    output.push('\n');

                    // Check for stop events
                    if line.contains("Process ") && line.contains("stopped") {
                        break;
                    }
                    if line.contains("Process ") && line.contains("exited") {
                        break;
                    }
                    if line.contains("(lldb)") && output.contains("stop reason") {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if start.elapsed() > timeout {
                        return Err(DapError::Timeout("Process stop".to_string()));
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(DapError::ProcessNotRunning);
                }
            }
        }

        Ok(output)
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        if let Some(ref mut lldb) = self.lldb {
            // Send quit command directly to avoid borrow issues
            let _ = lldb.stdin_tx.send("quit".to_string());
            let _ = lldb.child.kill();
        }
    }
}

// ============================================================================
// Parsing Helpers
// ============================================================================

fn parse_breakpoint_id(output: &str) -> DapResult<u32> {
    // Parse "Breakpoint 1: where = ..."
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("Breakpoint ") {
            if let Some(colon_pos) = rest.find(':') {
                if let Ok(id) = rest[..colon_pos].trim().parse() {
                    return Ok(id);
                }
            }
        }
    }
    Err(DapError::Debugger("Failed to parse breakpoint ID".to_string()))
}

fn parse_thread_line(line: &str) -> Option<Thread> {
    // Parse "* thread #1: tid = 0x1234, name = 'main'"
    // or "  thread #2: tid = 0x5678, name = 'worker'"
    let line = line.trim();
    if !line.contains("thread #") {
        return None;
    }

    let thread_start = line.find("thread #")?;
    let rest = &line[thread_start + 8..];
    let colon_pos = rest.find(':')?;
    let id: i64 = rest[..colon_pos].parse().ok()?;

    let name = if let Some(name_start) = line.find("name = '") {
        let rest = &line[name_start + 8..];
        if let Some(end) = rest.find('\'') {
            rest[..end].to_string()
        } else {
            format!("thread-{}", id)
        }
    } else {
        format!("thread-{}", id)
    };

    Some(Thread { id, name })
}

fn parse_frame_line(line: &str) -> Option<RawStackFrame> {
    // Parse "frame #0: 0x00001234 binary`function_name at file.c:123"
    let line = line.trim();
    if !line.starts_with("frame #") {
        return None;
    }

    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }

    // Parse address
    let addr_str = parts[1].trim_start_matches("0x");
    let instruction_pointer = u64::from_str_radix(addr_str, 16).unwrap_or(0);

    // Parse function name
    let rest = parts[2..].join(" ");
    let function_name = if let Some(bt) = rest.find('`') {
        let after_bt = &rest[bt + 1..];
        if let Some(space) = after_bt.find(' ') {
            after_bt[..space].to_string()
        } else {
            after_bt.to_string()
        }
    } else {
        "unknown".to_string()
    };

    // Parse module name
    let module_name = rest.split('`').next().map(|s| s.trim().to_string());

    Some(RawStackFrame {
        function_name,
        instruction_pointer,
        module_name,
    })
}

fn parse_variables(output: &str) -> Vec<RawVariable> {
    let mut vars = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("(lldb)") {
            continue;
        }

        // Parse "(type) name = value"
        if let Some(paren_end) = line.find(')') {
            let type_name = line[1..paren_end].to_string();
            let rest = line[paren_end + 1..].trim();

            if let Some(eq_pos) = rest.find('=') {
                let name = rest[..eq_pos].trim().to_string();
                let value = rest[eq_pos + 1..].trim().to_string();

                vars.push(RawVariable {
                    name: name.clone(),
                    value,
                    type_name: Some(type_name),
                    evaluate_name: Some(name),
                    memory_reference: None,
                });
            }
        }
    }

    vars
}

fn parse_registers(output: &str) -> Vec<RawVariable> {
    let mut regs = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("(lldb)") {
            continue;
        }

        // Parse "rax = 0x0000000000000001"
        if let Some(eq_pos) = line.find('=') {
            let name = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();

            regs.push(RawVariable {
                name: name.clone(),
                value,
                type_name: Some("register".to_string()),
                evaluate_name: Some(format!("${}", name)),
                memory_reference: None,
            });
        }
    }

    regs
}

/// Parse nested variable output from `frame variable -T <path>`.
///
/// LLDB outputs nested structures with indentation:
/// ```text
/// (MyStruct) myVar = {
///   (i64) field1 = 42
///   (String) field2 = "hello"
///   (Inner) inner = {
///     (i64) x = 10
///   }
/// }
/// ```
///
/// For arrays:
/// ```text
/// (Vec<i64>) arr = size=3 {
///   [0] = 1
///   [1] = 2
///   [2] = 3
/// }
/// ```
fn parse_nested_variables(output: &str, parent_path: &str) -> Vec<RawVariable> {
    let mut vars = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    // Skip the first line (parent variable itself) and trailing prompt
    let mut depth = 0;
    let mut in_parent = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("(lldb)") {
            continue;
        }

        // Track braces for nesting level
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();

        if !in_parent {
            // Look for the opening brace of the parent
            if line.contains('{') {
                in_parent = true;
                depth = 1;
            }
            continue;
        }

        // Update depth
        depth = depth + open_braces - close_braces;

        // Only process direct children (depth == 1)
        if depth == 1 && !trimmed.is_empty() && !trimmed.starts_with('}') {
            // Parse array index pattern: "[0] = value" or "(type) [0] = value"
            if trimmed.contains('[') && trimmed.contains(']') {
                if let Some(bracket_start) = trimmed.find('[') {
                    if let Some(bracket_end) = trimmed.find(']') {
                        let index = &trimmed[bracket_start + 1..bracket_end];
                        let rest = trimmed[bracket_end + 1..].trim();

                        if let Some(eq_pos) = rest.find('=') {
                            let value = rest[eq_pos + 1..].trim().to_string();
                            let name = format!("[{}]", index);
                            let eval_name = format!("{}[{}]", parent_path, index);

                            vars.push(RawVariable {
                                name,
                                value,
                                type_name: None, // Array elements usually share parent type
                                evaluate_name: Some(eval_name),
                                memory_reference: None,
                            });
                        }
                    }
                }
            }
            // Parse struct field pattern: "(type) name = value"
            else if trimmed.starts_with('(') {
                if let Some(paren_end) = trimmed.find(')') {
                    let type_name = trimmed[1..paren_end].to_string();
                    let rest = trimmed[paren_end + 1..].trim();

                    if let Some(eq_pos) = rest.find('=') {
                        let name = rest[..eq_pos].trim().to_string();
                        let value = rest[eq_pos + 1..].trim().to_string();
                        let eval_name = format!("{}.{}", parent_path, name);

                        vars.push(RawVariable {
                            name: name.clone(),
                            value,
                            type_name: Some(type_name),
                            evaluate_name: Some(eval_name),
                            memory_reference: None,
                        });
                    }
                }
            }
        }

        // Stop if we've closed all braces
        if depth == 0 {
            break;
        }
    }

    vars
}

fn parse_expression_result(output: &str) -> Option<(String, String)> {
    // Parse "(type) $0 = value"
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with('(') {
            if let Some(paren_end) = line.find(')') {
                let type_name = line[1..paren_end].to_string();
                let rest = line[paren_end + 1..].trim();

                if let Some(eq_pos) = rest.find('=') {
                    let value = rest[eq_pos + 1..].trim().to_string();
                    return Some((type_name, value));
                }
            }
        }
    }
    None
}

fn parse_memory_output(output: &str) -> Vec<u8> {
    let mut data = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("(lldb)") {
            continue;
        }

        // Parse "0x00001234: 0x01 0x02 0x03 ..."
        if let Some(colon_pos) = line.find(':') {
            let bytes_part = &line[colon_pos + 1..];
            for byte_str in bytes_part.split_whitespace() {
                if let Some(hex) = byte_str.strip_prefix("0x") {
                    if let Ok(byte) = u8::from_str_radix(hex, 16) {
                        data.push(byte);
                    }
                }
            }
        }
    }

    data
}

fn parse_disassembly(output: &str, _resolve_symbols: bool) -> Vec<DisassembledInstruction> {
    let mut instructions = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("(lldb)") || line.starts_with("->") {
            continue;
        }

        // Parse "binary`function+offset: instruction"
        // or "0x00001234: instruction"
        if let Some(colon_pos) = line.find(':') {
            let addr_part = line[..colon_pos].trim();
            let instr_part = line[colon_pos + 1..].trim();

            // Extract address
            let address = if addr_part.contains("0x") {
                let hex_start = addr_part.find("0x").unwrap_or(0);
                let hex_end = addr_part[hex_start..].find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
                    .map(|p| hex_start + p)
                    .unwrap_or(addr_part.len());
                addr_part[hex_start..hex_end].to_string()
            } else {
                "0x0".to_string()
            };

            // Extract symbol if present
            let symbol = if addr_part.contains('`') {
                let parts: Vec<&str> = addr_part.split('`').collect();
                parts.get(1).map(|s| s.split_whitespace().next().unwrap_or("").to_string())
            } else {
                None
            };

            instructions.push(DisassembledInstruction {
                address,
                instruction_bytes: None,
                instruction: instr_part.to_string(),
                symbol,
                location: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            });
        }
    }

    instructions
}
