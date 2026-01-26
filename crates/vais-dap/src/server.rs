//! DAP Server Implementation
//!
//! Main server that handles DAP protocol communication and request dispatching.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader, BufWriter};
use tracing::{debug, error, info, warn};

use crate::error::{DapError, DapResult};
use crate::protocol::codec::DapMessage;
use crate::protocol::events::*;
use crate::protocol::requests::*;
use crate::protocol::responses::*;
use crate::protocol::types::*;
use crate::session::DebugSession;

/// DAP Server state
pub struct DapServer {
    /// Current session
    session: Option<DebugSession>,
    /// Sequence number counter
    seq: AtomicI64,
    /// Client capabilities
    client_caps: Option<InitializeRequestArguments>,
    /// Server capabilities
    capabilities: Capabilities,
    /// Whether the client has sent configurationDone
    configured: bool,
    /// Whether to lines start at 1 (true) or 0 (false)
    lines_start_at1: bool,
    /// Whether columns start at 1 (true) or 0 (false)
    columns_start_at1: bool,
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            session: None,
            seq: AtomicI64::new(1),
            client_caps: None,
            capabilities: Capabilities::vais_defaults(),
            configured: false,
            lines_start_at1: true,
            columns_start_at1: true,
        }
    }

    fn next_seq(&self) -> i64 {
        self.seq.fetch_add(1, Ordering::SeqCst)
    }

    /// Run the DAP server over the given reader/writer streams
    pub async fn run<R, W>(&mut self, reader: R, writer: W) -> std::io::Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(reader);
        let mut writer = BufWriter::new(writer);

        loop {
            // Read header
            let mut header = String::new();
            let mut content_length: Option<usize> = None;

            loop {
                header.clear();
                let n = reader.read_line(&mut header).await?;
                if n == 0 {
                    info!("Client disconnected");
                    return Ok(());
                }

                let line = header.trim();
                if line.is_empty() {
                    break;
                }

                if line.starts_with("Content-Length: ") {
                    content_length = line[16..].parse().ok();
                }
            }

            let content_length = match content_length {
                Some(len) => len,
                None => {
                    error!("Missing Content-Length header");
                    continue;
                }
            };

            // Read body
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).await?;

            // Parse JSON
            let request: Request = match serde_json::from_slice(&body) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    continue;
                }
            };

            debug!("Received request: {} (seq={})", request.command, request.base.seq);

            // Handle request
            let response = self.handle_request(&request).await;

            // Send response
            let response_json = serde_json::to_string(&response)?;
            let response_header = format!("Content-Length: {}\r\n\r\n", response_json.len());

            writer.write_all(response_header.as_bytes()).await?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.flush().await?;

            debug!("Sent response for {} (success={})", request.command, response.success);

            // Check for disconnect
            if request.command == "disconnect" {
                info!("Disconnect requested, shutting down");
                break;
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, request: &Request) -> Response {
        let result = match request.command.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "launch" => self.handle_launch(request).await,
            "attach" => self.handle_attach(request).await,
            "disconnect" => self.handle_disconnect(request).await,
            "terminate" => self.handle_terminate(request).await,
            "restart" => self.handle_restart(request).await,
            "configurationDone" => self.handle_configuration_done(request).await,
            "setBreakpoints" => self.handle_set_breakpoints(request).await,
            "setFunctionBreakpoints" => self.handle_set_function_breakpoints(request).await,
            "setExceptionBreakpoints" => self.handle_set_exception_breakpoints(request).await,
            "continue" => self.handle_continue(request).await,
            "next" => self.handle_next(request).await,
            "stepIn" => self.handle_step_in(request).await,
            "stepOut" => self.handle_step_out(request).await,
            "pause" => self.handle_pause(request).await,
            "threads" => self.handle_threads(request).await,
            "stackTrace" => self.handle_stack_trace(request).await,
            "scopes" => self.handle_scopes(request).await,
            "variables" => self.handle_variables(request).await,
            "setVariable" => self.handle_set_variable(request).await,
            "source" => self.handle_source(request).await,
            "evaluate" => self.handle_evaluate(request).await,
            "readMemory" => self.handle_read_memory(request).await,
            "writeMemory" => self.handle_write_memory(request).await,
            "disassemble" => self.handle_disassemble(request).await,
            _ => Err(DapError::Unsupported(format!(
                "Command '{}' is not supported",
                request.command
            ))),
        };

        match result {
            Ok(body) => Response {
                base: ProtocolMessage {
                    seq: self.next_seq(),
                    message_type: MessageType::Response,
                },
                request_seq: request.base.seq,
                success: true,
                command: request.command.clone(),
                message: None,
                body,
            },
            Err(e) => {
                error!("Request '{}' failed: {}", request.command, e);
                Response {
                    base: ProtocolMessage {
                        seq: self.next_seq(),
                        message_type: MessageType::Response,
                    },
                    request_seq: request.base.seq,
                    success: false,
                    command: request.command.clone(),
                    message: Some(e.to_string()),
                    body: None,
                }
            }
        }
    }

    fn parse_args<T: serde::de::DeserializeOwned>(&self, request: &Request) -> DapResult<T> {
        let args = request
            .arguments
            .as_ref()
            .ok_or_else(|| DapError::InvalidRequest("Missing arguments".to_string()))?;
        serde_json::from_value(args.clone()).map_err(|e| {
            DapError::InvalidRequest(format!("Invalid arguments: {}", e))
        })
    }

    // ========================================================================
    // Request Handlers
    // ========================================================================

    async fn handle_initialize(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: InitializeRequestArguments = self.parse_args(request)?;

        self.lines_start_at1 = args.lines_start_at1.unwrap_or(true);
        self.columns_start_at1 = args.columns_start_at1.unwrap_or(true);
        self.client_caps = Some(args);

        info!("DAP initialized (lines_start_at1={}, columns_start_at1={})",
              self.lines_start_at1, self.columns_start_at1);

        let body = InitializeResponseBody {
            capabilities: self.capabilities.clone(),
        };

        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_launch(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: LaunchRequestArguments = self.parse_args(request)?;

        info!("Launching debug session: {:?}", args.program);

        // Create debug session
        let session = DebugSession::new();
        session.launch(args).await?;
        self.session = Some(session);

        Ok(None)
    }

    async fn handle_attach(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: AttachRequestArguments = self.parse_args(request)?;

        info!("Attaching to process: {:?}", args.pid);

        let session = DebugSession::new();
        session.attach(args).await?;
        self.session = Some(session);

        Ok(None)
    }

    async fn handle_disconnect(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: DisconnectRequestArguments = match self.parse_args(request) {
            Ok(a) => a,
            Err(_) => DisconnectRequestArguments::default(),
        };

        info!("Disconnecting (terminate_debuggee={:?})", args.terminate_debuggee);

        if let Some(session) = self.session.take() {
            session.disconnect(args.terminate_debuggee.unwrap_or(false)).await?;
        }

        Ok(None)
    }

    async fn handle_terminate(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: TerminateRequestArguments = match self.parse_args(request) {
            Ok(a) => a,
            Err(_) => TerminateRequestArguments::default(),
        };

        info!("Terminating debug session (restart={:?})", args.restart);

        if let Some(ref session) = self.session {
            session.terminate().await?;
        }

        Ok(None)
    }

    async fn handle_restart(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: RestartRequestArguments = match self.parse_args(request) {
            Ok(a) => a,
            Err(_) => RestartRequestArguments::default(),
        };

        info!("Restarting debug session");

        if let Some(ref session) = self.session {
            session.restart().await?;
        }

        Ok(None)
    }

    async fn handle_configuration_done(&mut self, _request: &Request) -> DapResult<Option<serde_json::Value>> {
        info!("Configuration done");
        self.configured = true;

        if let Some(ref session) = self.session {
            session.configuration_done().await?;
        }

        Ok(None)
    }

    async fn handle_set_breakpoints(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: SetBreakpointsRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let breakpoints = session.set_breakpoints(args).await?;

        let body = SetBreakpointsResponseBody { breakpoints };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_set_function_breakpoints(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: SetFunctionBreakpointsRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let breakpoints = session.set_function_breakpoints(args).await?;

        let body = SetFunctionBreakpointsResponseBody { breakpoints };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_set_exception_breakpoints(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: SetExceptionBreakpointsRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.set_exception_breakpoints(args).await?;

        let body = SetExceptionBreakpointsResponseBody::default();
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_continue(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: ContinueRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.continue_execution(args.thread_id, args.single_thread.unwrap_or(false)).await?;

        let body = ContinueResponseBody {
            all_threads_continued: Some(true),
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_next(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: NextRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.step_over(args.thread_id, args.granularity).await?;

        Ok(None)
    }

    async fn handle_step_in(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: StepInRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.step_in(args.thread_id, args.granularity).await?;

        Ok(None)
    }

    async fn handle_step_out(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: StepOutRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.step_out(args.thread_id).await?;

        Ok(None)
    }

    async fn handle_pause(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: PauseRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        session.pause(args.thread_id).await?;

        Ok(None)
    }

    async fn handle_threads(&mut self, _request: &Request) -> DapResult<Option<serde_json::Value>> {
        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let threads = session.get_threads().await?;

        let body = ThreadsResponseBody { threads };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_stack_trace(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: StackTraceRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let (stack_frames, total_frames) = session.get_stack_trace(
            args.thread_id,
            args.start_frame.unwrap_or(0) as usize,
            args.levels.map(|l| l as usize),
        ).await?;

        let body = StackTraceResponseBody {
            stack_frames,
            total_frames: Some(total_frames as i64),
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_scopes(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: ScopesRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let scopes = session.get_scopes(args.frame_id).await?;

        let body = ScopesResponseBody { scopes };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_variables(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: VariablesRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let variables = session.get_variables(
            args.variables_reference,
            args.start.map(|s| s as usize),
            args.count.map(|c| c as usize),
        ).await?;

        let body = VariablesResponseBody { variables };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_set_variable(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: SetVariableRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let (value, var_type, variables_reference) = session.set_variable(
            args.variables_reference,
            &args.name,
            &args.value,
        ).await?;

        let body = SetVariableResponseBody {
            value,
            var_type,
            variables_reference,
            named_variables: None,
            indexed_variables: None,
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_source(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: SourceRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let content = session.get_source(args.source_reference).await?;

        let body = SourceResponseBody {
            content,
            mime_type: Some("text/x-vais".to_string()),
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_evaluate(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: EvaluateRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let (result, result_type, variables_reference) = session.evaluate(
            &args.expression,
            args.frame_id,
            args.context,
        ).await?;

        let body = EvaluateResponseBody {
            result,
            result_type,
            presentation_hint: None,
            variables_reference,
            named_variables: None,
            indexed_variables: None,
            memory_reference: None,
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_read_memory(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: ReadMemoryRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let (address, data) = session.read_memory(
            &args.memory_reference,
            args.offset.unwrap_or(0),
            args.count as usize,
        ).await?;

        let body = ReadMemoryResponseBody {
            address,
            unreadable_bytes: None,
            data,
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_write_memory(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: WriteMemoryRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let bytes_written = session.write_memory(
            &args.memory_reference,
            args.offset.unwrap_or(0),
            &args.data,
        ).await?;

        let body = WriteMemoryResponseBody {
            offset: None,
            bytes_written: Some(bytes_written as i64),
        };
        Ok(Some(serde_json::to_value(body)?))
    }

    async fn handle_disassemble(&mut self, request: &Request) -> DapResult<Option<serde_json::Value>> {
        let args: DisassembleRequestArguments = self.parse_args(request)?;

        let session = self.session.as_ref().ok_or(DapError::NoActiveSession)?;
        let instructions = session.disassemble(
            &args.memory_reference,
            args.offset.unwrap_or(0),
            args.instruction_offset.unwrap_or(0),
            args.instruction_count as usize,
            args.resolve_symbols.unwrap_or(true),
        ).await?;

        let body = DisassembleResponseBody { instructions };
        Ok(Some(serde_json::to_value(body)?))
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}
