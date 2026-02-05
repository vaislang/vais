//! Debug Session Management
//!
//! Manages the state of a debugging session, including process control,
//! breakpoints, stack frames, and variables.

use base64::Engine;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::debugger::Debugger;
use crate::error::{DapError, DapResult};
use crate::protocol::requests::*;
use crate::protocol::types::*;
use crate::source_map::SourceMap;

/// Check if a type name represents a compound type that has children.
///
/// Compound types include:
/// - Structs (any PascalCase name or containing `Struct`)
/// - Arrays (`Vec<...>`, `[T; N]`, etc.)
/// - Pointers to structs (`*T` where T is a struct)
/// - Enums with fields
fn is_compound_type(type_name: Option<&str>) -> bool {
    let Some(t) = type_name else { return false };
    let t = t.trim();

    // Empty or primitive types
    if t.is_empty() {
        return false;
    }

    // Known primitive types
    let primitives = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "()", "register",
    ];
    if primitives.contains(&t) {
        return false;
    }

    // String types that don't expand well
    if t == "str" || t == "&str" || t == "String" {
        return false;
    }

    // Arrays and vectors
    if t.starts_with("Vec<") || t.starts_with('[') || t.contains("Array") {
        return true;
    }

    // Pointers (might point to structs)
    if t.starts_with('*') || t.starts_with('&') {
        // Check if it's a pointer to a primitive
        let inner = t.trim_start_matches(['*', '&', ' ']);
        if primitives.contains(&inner) || inner == "str" {
            return false;
        }
        return true;
    }

    // Generic types like Option<T>, Result<T, E>, Box<T>
    if t.contains('<') {
        return true;
    }

    // Struct-like types (PascalCase or contains "struct")
    if t.contains("struct") || t.contains("Struct") {
        return true;
    }

    // If it starts with an uppercase letter, assume it's a struct/enum
    if t.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return true;
    }

    false
}

/// A debug session managing the debuggee process
pub struct DebugSession {
    /// The underlying debugger
    debugger: Arc<RwLock<Debugger>>,
    /// Source file mapping
    source_map: Arc<RwLock<SourceMap>>,
    /// Breakpoint ID counter
    breakpoint_id_counter: AtomicI64,
    /// Frame ID counter
    frame_id_counter: AtomicI64,
    /// Variables reference counter
    var_ref_counter: AtomicI64,
    /// Mapping from frame ID to (thread ID, frame index)
    frame_mapping: Arc<RwLock<HashMap<i64, (i64, usize)>>>,
    /// Mapping from variables reference to scope/variable info
    var_ref_mapping: Arc<RwLock<HashMap<i64, VariableRef>>>,
    /// Launch configuration
    launch_config: Arc<RwLock<Option<LaunchRequestArguments>>>,
    /// Attach configuration
    attach_config: Arc<RwLock<Option<AttachRequestArguments>>>,
}

#[derive(Debug, Clone)]
enum VariableRef {
    Scope {
        frame_id: i64,
        scope_type: ScopeType,
    },
    Variable {
        frame_id: i64,
        _parent_ref: i64,
        _name: String,
        /// Full evaluation path for lldb (e.g., "myStruct.field.subfield")
        eval_path: String,
    },
}

#[derive(Debug, Clone, Copy)]
enum ScopeType {
    Locals,
    Arguments,
    Registers,
}

impl DebugSession {
    pub fn new() -> Self {
        Self {
            debugger: Arc::new(RwLock::new(Debugger::new())),
            source_map: Arc::new(RwLock::new(SourceMap::new())),
            breakpoint_id_counter: AtomicI64::new(1),
            frame_id_counter: AtomicI64::new(1),
            var_ref_counter: AtomicI64::new(1),
            frame_mapping: Arc::new(RwLock::new(HashMap::new())),
            var_ref_mapping: Arc::new(RwLock::new(HashMap::new())),
            launch_config: Arc::new(RwLock::new(None)),
            attach_config: Arc::new(RwLock::new(None)),
        }
    }

    fn next_breakpoint_id(&self) -> i64 {
        self.breakpoint_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    fn next_frame_id(&self) -> i64 {
        self.frame_id_counter.fetch_add(1, Ordering::SeqCst)
    }

    fn next_var_ref(&self) -> i64 {
        self.var_ref_counter.fetch_add(1, Ordering::SeqCst)
    }

    // ========================================================================
    // Lifecycle
    // ========================================================================

    pub async fn launch(&self, args: LaunchRequestArguments) -> DapResult<()> {
        info!(
            "Launching: program={:?}, binary={:?}",
            args.program, args.binary
        );

        let program = args
            .program
            .clone()
            .or_else(|| args.binary.clone())
            .ok_or_else(|| DapError::InvalidRequest("No program specified".to_string()))?;

        // If auto_compile is enabled and we have a .vais file, compile it
        let binary = if args.auto_compile.unwrap_or(true) && program.ends_with(".vais") {
            self.compile_vais(&program, args.opt_level.unwrap_or(0))
                .await?
        } else {
            args.binary.clone().unwrap_or(program.clone())
        };

        // Load source mapping
        {
            let mut source_map = self.source_map.write().await;
            source_map.load_from_binary(&binary).await?;
        }

        // Launch the process
        {
            let mut debugger = self.debugger.write().await;
            debugger
                .launch(
                    &binary,
                    args.args.as_deref().unwrap_or(&[]),
                    args.cwd.as_deref(),
                    args.env.as_ref(),
                    args.stop_on_entry.unwrap_or(false),
                )
                .await?;
        }

        // Store config for restart
        *self.launch_config.write().await = Some(args);

        Ok(())
    }

    async fn compile_vais(&self, source_file: &str, opt_level: i32) -> DapResult<String> {
        use std::process::Command;

        let output_file = source_file.replace(".vais", "");

        info!(
            "Compiling {} -> {} (opt_level={})",
            source_file, output_file, opt_level
        );

        let status = Command::new("vaisc")
            .args([
                source_file,
                "-o",
                &output_file,
                "-g", // Enable debug info
                &format!("-O{}", opt_level),
            ])
            .status()
            .map_err(|e| DapError::Debugger(format!("Failed to run vaisc: {}", e)))?;

        if !status.success() {
            return Err(DapError::Debugger(format!(
                "Compilation failed with exit code: {:?}",
                status.code()
            )));
        }

        Ok(output_file)
    }

    pub async fn attach(&self, args: AttachRequestArguments) -> DapResult<()> {
        let pid = args
            .pid
            .ok_or_else(|| DapError::InvalidRequest("No PID specified for attach".to_string()))?;

        info!("Attaching to PID: {}", pid);

        // Load source mapping if program path provided
        if let Some(ref program) = args.program {
            let mut source_map = self.source_map.write().await;
            source_map.load_from_binary(program).await?;
        }

        // Attach to process
        {
            let mut debugger = self.debugger.write().await;
            debugger
                .attach(pid as u32, args.stop_on_attach.unwrap_or(true))
                .await?;
        }

        // Store config for restart
        *self.attach_config.write().await = Some(args);

        Ok(())
    }

    pub async fn disconnect(&self, terminate: bool) -> DapResult<()> {
        info!("Disconnecting (terminate={})", terminate);

        let mut debugger = self.debugger.write().await;
        if terminate {
            debugger.terminate().await?;
        } else {
            debugger.detach().await?;
        }

        Ok(())
    }

    pub async fn terminate(&self) -> DapResult<()> {
        info!("Terminating debuggee");

        let mut debugger = self.debugger.write().await;
        debugger.terminate().await
    }

    pub async fn restart(&self) -> DapResult<()> {
        info!("Restarting debug session");

        // Terminate current session
        self.terminate().await?;

        // Clear state
        self.frame_mapping.write().await.clear();
        self.var_ref_mapping.write().await.clear();

        // Relaunch or reattach
        if let Some(args) = self.launch_config.read().await.clone() {
            self.launch(args).await?;
        } else if let Some(args) = self.attach_config.read().await.clone() {
            self.attach(args).await?;
        }

        Ok(())
    }

    pub async fn configuration_done(&self) -> DapResult<()> {
        debug!("Configuration done, resuming execution");

        let debugger = self.debugger.read().await;
        if debugger.is_stopped_on_entry().await {
            // Resume if stopped on entry
            drop(debugger);
            let mut debugger = self.debugger.write().await;
            debugger.continue_all().await?;
        }

        Ok(())
    }

    // ========================================================================
    // Breakpoints
    // ========================================================================

    pub async fn set_breakpoints(
        &self,
        args: SetBreakpointsRequestArguments,
    ) -> DapResult<Vec<Breakpoint>> {
        let source_path = args.source.path.as_ref().ok_or_else(|| {
            DapError::InvalidRequest("No source path in setBreakpoints".to_string())
        })?;

        info!("Setting breakpoints in {}", source_path);

        let mut results = Vec::new();
        let mut debugger = self.debugger.write().await;

        // Clear existing breakpoints for this source
        debugger.clear_breakpoints_for_source(source_path).await?;

        // Set new breakpoints
        if let Some(breakpoints) = args.breakpoints {
            for bp in breakpoints {
                let id = self.next_breakpoint_id();
                let line = bp.line;

                match debugger
                    .set_breakpoint(source_path, line, bp.condition.as_deref())
                    .await
                {
                    Ok(verified_line) => {
                        results.push(Breakpoint {
                            id: Some(id),
                            verified: true,
                            line: Some(verified_line),
                            source: Some(args.source.clone()),
                            message: None,
                            column: bp.column,
                            end_line: None,
                            end_column: None,
                            instruction_reference: None,
                            offset: None,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to set breakpoint at line {}: {}", line, e);
                        results.push(Breakpoint {
                            id: Some(id),
                            verified: false,
                            line: Some(line),
                            source: Some(args.source.clone()),
                            message: Some(e.to_string()),
                            column: bp.column,
                            end_line: None,
                            end_column: None,
                            instruction_reference: None,
                            offset: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn set_function_breakpoints(
        &self,
        args: SetFunctionBreakpointsRequestArguments,
    ) -> DapResult<Vec<Breakpoint>> {
        info!("Setting function breakpoints");

        let mut results = Vec::new();
        let mut debugger = self.debugger.write().await;

        // Clear existing function breakpoints
        debugger.clear_function_breakpoints().await?;

        for bp in args.breakpoints {
            let id = self.next_breakpoint_id();

            match debugger
                .set_function_breakpoint(&bp.name, bp.condition.as_deref())
                .await
            {
                Ok(()) => {
                    results.push(Breakpoint {
                        id: Some(id),
                        verified: true,
                        line: None,
                        source: None,
                        message: None,
                        column: None,
                        end_line: None,
                        end_column: None,
                        instruction_reference: None,
                        offset: None,
                    });
                }
                Err(e) => {
                    warn!("Failed to set function breakpoint '{}': {}", bp.name, e);
                    results.push(Breakpoint {
                        id: Some(id),
                        verified: false,
                        line: None,
                        source: None,
                        message: Some(e.to_string()),
                        column: None,
                        end_line: None,
                        end_column: None,
                        instruction_reference: None,
                        offset: None,
                    });
                }
            }
        }

        Ok(results)
    }

    pub async fn set_exception_breakpoints(
        &self,
        args: SetExceptionBreakpointsRequestArguments,
    ) -> DapResult<()> {
        info!("Setting exception breakpoints: {:?}", args.filters);

        let mut debugger = self.debugger.write().await;
        debugger.set_exception_breakpoints(&args.filters).await
    }

    // ========================================================================
    // Execution Control
    // ========================================================================

    pub async fn continue_execution(&self, thread_id: i64, single_thread: bool) -> DapResult<()> {
        debug!(
            "Continue: thread_id={}, single_thread={}",
            thread_id, single_thread
        );

        let mut debugger = self.debugger.write().await;
        if single_thread {
            debugger.continue_thread(thread_id).await
        } else {
            debugger.continue_all().await
        }
    }

    pub async fn step_over(
        &self,
        thread_id: i64,
        granularity: Option<SteppingGranularity>,
    ) -> DapResult<()> {
        debug!(
            "Step over: thread_id={}, granularity={:?}",
            thread_id, granularity
        );

        let mut debugger = self.debugger.write().await;
        match granularity {
            Some(SteppingGranularity::Instruction) => debugger.step_instruction(thread_id).await,
            _ => debugger.step_over(thread_id).await,
        }
    }

    pub async fn step_in(
        &self,
        thread_id: i64,
        granularity: Option<SteppingGranularity>,
    ) -> DapResult<()> {
        debug!(
            "Step in: thread_id={}, granularity={:?}",
            thread_id, granularity
        );

        let mut debugger = self.debugger.write().await;
        match granularity {
            Some(SteppingGranularity::Instruction) => debugger.step_instruction(thread_id).await,
            _ => debugger.step_in(thread_id).await,
        }
    }

    pub async fn step_out(&self, thread_id: i64) -> DapResult<()> {
        debug!("Step out: thread_id={}", thread_id);

        let mut debugger = self.debugger.write().await;
        debugger.step_out(thread_id).await
    }

    pub async fn pause(&self, thread_id: i64) -> DapResult<()> {
        debug!("Pause: thread_id={}", thread_id);

        let mut debugger = self.debugger.write().await;
        debugger.pause(thread_id).await
    }

    // ========================================================================
    // Threads
    // ========================================================================

    pub async fn get_threads(&self) -> DapResult<Vec<Thread>> {
        let debugger = self.debugger.read().await;
        debugger.get_threads().await
    }

    // ========================================================================
    // Stack Trace
    // ========================================================================

    pub async fn get_stack_trace(
        &self,
        thread_id: i64,
        start_frame: usize,
        levels: Option<usize>,
    ) -> DapResult<(Vec<StackFrame>, usize)> {
        let debugger = self.debugger.read().await;
        let raw_frames = debugger.get_stack_frames(thread_id).await?;
        let total = raw_frames.len();

        let end_frame = levels
            .map(|l| (start_frame + l).min(total))
            .unwrap_or(total);
        let frames_slice = &raw_frames[start_frame..end_frame];

        let source_map = self.source_map.read().await;
        let mut frame_mapping = self.frame_mapping.write().await;

        let mut frames = Vec::with_capacity(frames_slice.len());
        for (idx, raw) in frames_slice.iter().enumerate() {
            let frame_id = self.next_frame_id();
            let frame_idx = start_frame + idx;

            frame_mapping.insert(frame_id, (thread_id, frame_idx));

            let source = source_map
                .get_source_for_address(raw.instruction_pointer)
                .map(Source::from_path);

            let (line, column) = source_map
                .get_line_column(raw.instruction_pointer)
                .unwrap_or((1, 1));

            frames.push(StackFrame {
                id: frame_id,
                name: raw.function_name.clone(),
                source,
                line: line as i64,
                column: column as i64,
                end_line: None,
                end_column: None,
                can_restart: None,
                instruction_pointer_reference: Some(format!("0x{:x}", raw.instruction_pointer)),
                module_id: None,
                presentation_hint: None,
            });
        }

        Ok((frames, total))
    }

    // ========================================================================
    // Scopes and Variables
    // ========================================================================

    pub async fn get_scopes(&self, frame_id: i64) -> DapResult<Vec<Scope>> {
        let frame_mapping = self.frame_mapping.read().await;
        let (_thread_id, _frame_idx) = frame_mapping
            .get(&frame_id)
            .copied()
            .ok_or(DapError::FrameNotFound(frame_id))?;

        let mut var_ref_mapping = self.var_ref_mapping.write().await;

        // Create scopes
        let locals_ref = self.next_var_ref();
        var_ref_mapping.insert(
            locals_ref,
            VariableRef::Scope {
                frame_id,
                scope_type: ScopeType::Locals,
            },
        );

        let args_ref = self.next_var_ref();
        var_ref_mapping.insert(
            args_ref,
            VariableRef::Scope {
                frame_id,
                scope_type: ScopeType::Arguments,
            },
        );

        let registers_ref = self.next_var_ref();
        var_ref_mapping.insert(
            registers_ref,
            VariableRef::Scope {
                frame_id,
                scope_type: ScopeType::Registers,
            },
        );

        Ok(vec![
            Scope {
                name: "Locals".to_string(),
                presentation_hint: Some(ScopePresentationHint::Locals),
                variables_reference: locals_ref,
                named_variables: None,
                indexed_variables: None,
                expensive: Some(false),
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
            Scope {
                name: "Arguments".to_string(),
                presentation_hint: Some(ScopePresentationHint::Arguments),
                variables_reference: args_ref,
                named_variables: None,
                indexed_variables: None,
                expensive: Some(false),
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
            Scope {
                name: "Registers".to_string(),
                presentation_hint: Some(ScopePresentationHint::Registers),
                variables_reference: registers_ref,
                named_variables: None,
                indexed_variables: None,
                expensive: Some(true),
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
        ])
    }

    pub async fn get_variables(
        &self,
        variables_reference: i64,
        start: Option<usize>,
        count: Option<usize>,
    ) -> DapResult<Vec<Variable>> {
        let var_ref_mapping = self.var_ref_mapping.read().await;
        let var_ref = var_ref_mapping
            .get(&variables_reference)
            .cloned()
            .ok_or(DapError::VariableNotFound(variables_reference))?;

        let frame_mapping = self.frame_mapping.read().await;

        match var_ref {
            VariableRef::Scope {
                frame_id,
                scope_type,
            } => {
                let (thread_id, frame_idx) = frame_mapping
                    .get(&frame_id)
                    .copied()
                    .ok_or(DapError::FrameNotFound(frame_id))?;

                let debugger = self.debugger.read().await;
                let raw_vars = match scope_type {
                    ScopeType::Locals => debugger.get_local_variables(thread_id, frame_idx).await?,
                    ScopeType::Arguments => debugger.get_arguments(thread_id, frame_idx).await?,
                    ScopeType::Registers => debugger.get_registers(thread_id).await?,
                };
                drop(debugger);

                let mut var_ref_mapping = self.var_ref_mapping.write().await;

                let result = raw_vars
                    .into_iter()
                    .skip(start.unwrap_or(0))
                    .take(count.unwrap_or(usize::MAX))
                    .map(|v| {
                        // Check if this variable has children (struct, array, pointer to struct)
                        let has_children = is_compound_type(v.type_name.as_deref());
                        let variables_reference = if has_children {
                            let new_ref = self.next_var_ref();
                            let eval_path =
                                v.evaluate_name.clone().unwrap_or_else(|| v.name.clone());
                            var_ref_mapping.insert(
                                new_ref,
                                VariableRef::Variable {
                                    frame_id,
                                    _parent_ref: variables_reference,
                                    _name: v.name.clone(),
                                    eval_path,
                                },
                            );
                            new_ref
                        } else {
                            0
                        };

                        Variable {
                            name: v.name,
                            value: v.value,
                            var_type: v.type_name,
                            presentation_hint: None,
                            evaluate_name: v.evaluate_name,
                            variables_reference,
                            named_variables: None,
                            indexed_variables: None,
                            memory_reference: v.memory_reference,
                        }
                    })
                    .collect();

                Ok(result)
            }
            VariableRef::Variable {
                frame_id,
                _parent_ref: _,
                _name: _,
                eval_path,
            } => {
                // Get child variables using the evaluation path
                let (thread_id, frame_idx) = frame_mapping
                    .get(&frame_id)
                    .copied()
                    .ok_or(DapError::FrameNotFound(frame_id))?;

                let debugger = self.debugger.read().await;
                let raw_vars = debugger
                    .get_children(thread_id, frame_idx, &eval_path)
                    .await?;
                drop(debugger);

                let mut var_ref_mapping = self.var_ref_mapping.write().await;

                let result = raw_vars
                    .into_iter()
                    .skip(start.unwrap_or(0))
                    .take(count.unwrap_or(usize::MAX))
                    .map(|v| {
                        let has_children = is_compound_type(v.type_name.as_deref());
                        let child_ref = if has_children {
                            let new_ref = self.next_var_ref();
                            let child_eval_path = v
                                .evaluate_name
                                .clone()
                                .unwrap_or_else(|| format!("{}.{}", eval_path, v.name));
                            var_ref_mapping.insert(
                                new_ref,
                                VariableRef::Variable {
                                    frame_id,
                                    _parent_ref: variables_reference,
                                    _name: v.name.clone(),
                                    eval_path: child_eval_path,
                                },
                            );
                            new_ref
                        } else {
                            0
                        };

                        Variable {
                            name: v.name,
                            value: v.value,
                            var_type: v.type_name,
                            presentation_hint: None,
                            evaluate_name: v.evaluate_name,
                            variables_reference: child_ref,
                            named_variables: None,
                            indexed_variables: None,
                            memory_reference: v.memory_reference,
                        }
                    })
                    .collect();

                Ok(result)
            }
        }
    }

    pub async fn set_variable(
        &self,
        variables_reference: i64,
        name: &str,
        value: &str,
    ) -> DapResult<(String, Option<String>, Option<i64>)> {
        let var_ref_mapping = self.var_ref_mapping.read().await;
        let var_ref = var_ref_mapping
            .get(&variables_reference)
            .cloned()
            .ok_or(DapError::VariableNotFound(variables_reference))?;

        let frame_mapping = self.frame_mapping.read().await;

        match var_ref {
            VariableRef::Scope { frame_id, .. } => {
                let (thread_id, frame_idx) = frame_mapping
                    .get(&frame_id)
                    .copied()
                    .ok_or(DapError::FrameNotFound(frame_id))?;

                let mut debugger = self.debugger.write().await;
                debugger
                    .set_variable(thread_id, frame_idx, name, value)
                    .await
            }
            VariableRef::Variable { .. } => Err(DapError::Unsupported(
                "Setting nested variables not yet supported".to_string(),
            )),
        }
    }

    // ========================================================================
    // Source
    // ========================================================================

    pub async fn get_source(&self, source_reference: i64) -> DapResult<String> {
        let source_map = self.source_map.read().await;
        source_map
            .get_source_content(source_reference)
            .ok_or_else(|| {
                DapError::SourceMapping(format!("Source reference {} not found", source_reference))
            })
    }

    // ========================================================================
    // Evaluate
    // ========================================================================

    pub async fn evaluate(
        &self,
        expression: &str,
        frame_id: Option<i64>,
        _context: Option<EvaluateContext>,
    ) -> DapResult<(String, Option<String>, i64)> {
        let (thread_id, frame_idx) = if let Some(fid) = frame_id {
            let frame_mapping = self.frame_mapping.read().await;
            frame_mapping
                .get(&fid)
                .copied()
                .ok_or(DapError::FrameNotFound(fid))?
        } else {
            (0, 0) // Use current thread/frame
        };

        let debugger = self.debugger.read().await;
        debugger.evaluate(expression, thread_id, frame_idx).await
    }

    // ========================================================================
    // Memory
    // ========================================================================

    pub async fn read_memory(
        &self,
        memory_reference: &str,
        offset: i64,
        count: usize,
    ) -> DapResult<(String, Option<String>)> {
        let address = parse_memory_reference(memory_reference)?;
        let adjusted_addr = (address as i64 + offset) as u64;

        let debugger = self.debugger.read().await;
        let data = debugger.read_memory(adjusted_addr, count).await?;

        // Encode as base64
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);

        Ok((format!("0x{:x}", adjusted_addr), Some(encoded)))
    }

    pub async fn write_memory(
        &self,
        memory_reference: &str,
        offset: i64,
        data: &str,
    ) -> DapResult<usize> {
        let address = parse_memory_reference(memory_reference)?;
        let adjusted_addr = (address as i64 + offset) as u64;

        // Decode base64
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| DapError::InvalidRequest(format!("Invalid base64 data: {}", e)))?;

        let mut debugger = self.debugger.write().await;
        debugger.write_memory(adjusted_addr, &bytes).await?;

        Ok(bytes.len())
    }

    // ========================================================================
    // Disassembly
    // ========================================================================

    pub async fn disassemble(
        &self,
        memory_reference: &str,
        offset: i64,
        instruction_offset: i64,
        instruction_count: usize,
        resolve_symbols: bool,
    ) -> DapResult<Vec<DisassembledInstruction>> {
        let address = parse_memory_reference(memory_reference)?;
        let adjusted_addr = (address as i64 + offset) as u64;

        let debugger = self.debugger.read().await;
        debugger
            .disassemble(
                adjusted_addr,
                instruction_offset,
                instruction_count,
                resolve_symbols,
            )
            .await
    }
}

fn parse_memory_reference(reference: &str) -> DapResult<u64> {
    let reference = reference.trim();
    if reference.starts_with("0x") || reference.starts_with("0X") {
        u64::from_str_radix(&reference[2..], 16)
            .map_err(|e| DapError::InvalidRequest(format!("Invalid memory reference: {}", e)))
    } else {
        reference
            .parse()
            .map_err(|e| DapError::InvalidRequest(format!("Invalid memory reference: {}", e)))
    }
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new()
    }
}
