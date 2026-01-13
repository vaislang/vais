//! Debug Adapter Protocol (DAP) Server
//!
//! VS Code 및 기타 IDE와 디버깅 통합을 위한 DAP 서버

use crate::debugger::Debugger;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use vais_lowering::CompiledFunction;

/// DAP 메시지 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DapMessage {
    #[serde(rename = "request")]
    Request(DapRequest),
    #[serde(rename = "response")]
    Response(DapResponse),
    #[serde(rename = "event")]
    Event(DapEvent),
}

/// DAP 요청
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq: i64,
    pub command: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

/// DAP 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq: i64,
    pub request_seq: i64,
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

/// DAP 이벤트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapEvent {
    pub seq: i64,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

/// DAP 서버
pub struct DapServer {
    debugger: Debugger,
    seq: i64,
    initialized: bool,
    /// 소스 파일 경로 -> 브레이크포인트 라인 번호 매핑
    #[allow(dead_code)]
    source_breakpoints: HashMap<String, Vec<i64>>,
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
            seq: 1,
            initialized: false,
            source_breakpoints: HashMap::new(),
        }
    }

    /// 함수 로드
    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        self.debugger.load_functions(functions);
    }

    /// TCP 서버 시작 (기본 포트: 4711)
    pub fn start(&mut self, port: u16) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        println!("DAP server listening on port {}", port);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_connection(stream)?;
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }
        Ok(())
    }

    /// 연결 처리
    fn handle_connection(&mut self, mut stream: TcpStream) -> std::io::Result<()> {
        let reader = BufReader::new(stream.try_clone()?);

        // DAP 메시지 읽기 루프
        let mut lines = reader.lines();

        loop {
            // Content-Length 헤더 읽기
            let mut content_length: Option<usize> = None;

            while let Some(Ok(line)) = lines.next() {
                if line.is_empty() {
                    break;
                }
                if line.starts_with("Content-Length:") {
                    let len_str = line.trim_start_matches("Content-Length:").trim();
                    content_length = len_str.parse().ok();
                }
            }

            let content_length = match content_length {
                Some(len) => len,
                None => break, // 연결 종료
            };

            // JSON 본문 읽기
            let mut body = vec![0u8; content_length];
            if let Some(Ok(line)) = lines.next() {
                if line.len() <= content_length {
                    body[..line.len()].copy_from_slice(line.as_bytes());
                }
            }

            let body_str = String::from_utf8_lossy(&body);

            // 요청 파싱
            if let Ok(request) = serde_json::from_str::<DapRequest>(&body_str) {
                let response = self.handle_request(request);
                self.send_message(&mut stream, &response)?;
            }
        }

        Ok(())
    }

    /// 요청 처리
    fn handle_request(&mut self, request: DapRequest) -> DapResponse {
        let seq = self.next_seq();

        match request.command.as_str() {
            "initialize" => self.handle_initialize(seq, request),
            "launch" => self.handle_launch(seq, request),
            "setBreakpoints" => self.handle_set_breakpoints(seq, request),
            "configurationDone" => self.handle_configuration_done(seq, request),
            "threads" => self.handle_threads(seq, request),
            "stackTrace" => self.handle_stack_trace(seq, request),
            "scopes" => self.handle_scopes(seq, request),
            "variables" => self.handle_variables(seq, request),
            "continue" => self.handle_continue(seq, request),
            "next" => self.handle_next(seq, request),
            "stepIn" => self.handle_step_in(seq, request),
            "stepOut" => self.handle_step_out(seq, request),
            "pause" => self.handle_pause(seq, request),
            "disconnect" => self.handle_disconnect(seq, request),
            "evaluate" => self.handle_evaluate(seq, request),
            _ => DapResponse {
                seq,
                request_seq: request.seq,
                success: false,
                command: request.command,
                message: Some("Unknown command".to_string()),
                body: None,
            },
        }
    }

    fn next_seq(&mut self) -> i64 {
        let seq = self.seq;
        self.seq += 1;
        seq
    }

    fn handle_initialize(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.initialized = true;

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "initialize".to_string(),
            message: None,
            body: Some(json!({
                "supportsConfigurationDoneRequest": true,
                "supportsFunctionBreakpoints": true,
                "supportsConditionalBreakpoints": true,
                "supportsEvaluateForHovers": true,
                "supportsStepBack": false,
                "supportsSetVariable": false,
                "supportsRestartFrame": false,
                "supportsGotoTargetsRequest": false,
                "supportsStepInTargetsRequest": false,
                "supportsCompletionsRequest": false,
                "supportsModulesRequest": false,
                "supportsExceptionOptions": false,
                "supportsValueFormattingOptions": false,
                "supportsExceptionInfoRequest": false,
                "supportTerminateDebuggee": true,
                "supportsDelayedStackTraceLoading": false,
                "supportsLoadedSourcesRequest": false,
                "supportsLogPoints": false,
                "supportsTerminateThreadsRequest": false,
                "supportsSetExpression": false,
                "supportsTerminateRequest": true,
                "supportsDataBreakpoints": false,
                "supportsReadMemoryRequest": false,
                "supportsDisassembleRequest": false,
                "supportsCancelRequest": false,
                "supportsBreakpointLocationsRequest": false
            })),
        }
    }

    fn handle_launch(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        // launch 요청에서 프로그램 경로 등을 받을 수 있음
        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "launch".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_set_breakpoints(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        let mut breakpoints = Vec::new();

        if let Some(args) = &request.arguments {
            if let Some(source) = args.get("source") {
                let _path = source.get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");

                if let Some(bps) = args.get("breakpoints").and_then(|b| b.as_array()) {
                    for bp in bps {
                        let line = bp.get("line").and_then(|l| l.as_i64()).unwrap_or(0);

                        // 실제로는 소스 라인을 명령어 인덱스로 변환해야 함
                        // 여기서는 간단히 라인 번호를 명령어 인덱스로 사용
                        let bp_id = self.debugger.set_breakpoint("__main__", line as usize);

                        breakpoints.push(json!({
                            "id": bp_id,
                            "verified": true,
                            "line": line
                        }));
                    }
                }
            }
        }

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "setBreakpoints".to_string(),
            message: None,
            body: Some(json!({
                "breakpoints": breakpoints
            })),
        }
    }

    fn handle_configuration_done(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "configurationDone".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_threads(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        // 단일 스레드 모델
        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "threads".to_string(),
            message: None,
            body: Some(json!({
                "threads": [{
                    "id": 1,
                    "name": "main"
                }]
            })),
        }
    }

    fn handle_stack_trace(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        let frames = self.debugger.get_call_stack();
        let stack_frames: Vec<Value> = frames.iter().enumerate().map(|(i, frame)| {
            json!({
                "id": i,
                "name": frame.function,
                "line": frame.instruction_pointer,
                "column": 0,
                "source": {
                    "name": format!("{}.vais", frame.function),
                    "path": format!("{}.vais", frame.function)
                }
            })
        }).collect();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "stackTrace".to_string(),
            message: None,
            body: Some(json!({
                "stackFrames": stack_frames,
                "totalFrames": stack_frames.len()
            })),
        }
    }

    fn handle_scopes(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        let frame_id = request.arguments
            .as_ref()
            .and_then(|a| a.get("frameId"))
            .and_then(|f| f.as_i64())
            .unwrap_or(0);

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "scopes".to_string(),
            message: None,
            body: Some(json!({
                "scopes": [{
                    "name": "Locals",
                    "variablesReference": frame_id + 1,
                    "expensive": false
                }]
            })),
        }
    }

    fn handle_variables(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        let variables_ref = request.arguments
            .as_ref()
            .and_then(|a| a.get("variablesReference"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let frame_id = (variables_ref - 1) as usize;
        let frames = self.debugger.get_call_stack();

        let variables: Vec<Value> = if frame_id < frames.len() {
            frames[frame_id].locals.iter().map(|(name, value)| {
                json!({
                    "name": name,
                    "value": format!("{}", value),
                    "type": format!("{:?}", value.value_type()),
                    "variablesReference": 0
                })
            }).collect()
        } else {
            Vec::new()
        };

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "variables".to_string(),
            message: None,
            body: Some(json!({
                "variables": variables
            })),
        }
    }

    fn handle_continue(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.debugger.continue_execution();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "continue".to_string(),
            message: None,
            body: Some(json!({
                "allThreadsContinued": true
            })),
        }
    }

    fn handle_next(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.debugger.step_over();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "next".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_step_in(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.debugger.step_into();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "stepIn".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_step_out(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.debugger.step_out();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "stepOut".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_pause(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        self.debugger.pause();

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "pause".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_disconnect(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "disconnect".to_string(),
            message: None,
            body: None,
        }
    }

    fn handle_evaluate(&mut self, seq: i64, request: DapRequest) -> DapResponse {
        let expression = request.arguments
            .as_ref()
            .and_then(|a| a.get("expression"))
            .and_then(|e| e.as_str())
            .unwrap_or("");

        // 간단한 평가: 변수 이름으로 값 조회
        let frames = self.debugger.get_call_stack();
        let result = frames.first()
            .and_then(|f| f.locals.get(expression))
            .map(|v| format!("{}", v))
            .unwrap_or_else(|| "<undefined>".to_string());

        DapResponse {
            seq,
            request_seq: request.seq,
            success: true,
            command: "evaluate".to_string(),
            message: None,
            body: Some(json!({
                "result": result,
                "variablesReference": 0
            })),
        }
    }

    /// DAP 메시지 전송
    fn send_message(&self, stream: &mut TcpStream, response: &DapResponse) -> std::io::Result<()> {
        let json = serde_json::to_string(response)?;
        let header = format!("Content-Length: {}\r\n\r\n", json.len());
        stream.write_all(header.as_bytes())?;
        stream.write_all(json.as_bytes())?;
        stream.flush()
    }

    /// 이벤트 전송
    pub fn send_event(&self, stream: &mut TcpStream, event: DapEvent) -> std::io::Result<()> {
        let json = serde_json::to_string(&event)?;
        let header = format!("Content-Length: {}\r\n\r\n", json.len());
        stream.write_all(header.as_bytes())?;
        stream.write_all(json.as_bytes())?;
        stream.flush()
    }
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_server_creation() {
        let server = DapServer::new();
        assert!(!server.initialized);
        assert_eq!(server.seq, 1);
    }

    #[test]
    fn test_initialize_request() {
        let mut server = DapServer::new();
        let request = DapRequest {
            seq: 1,
            command: "initialize".to_string(),
            arguments: None,
        };

        let response = server.handle_request(request);
        assert!(response.success);
        assert!(server.initialized);
    }

    #[test]
    fn test_threads_request() {
        let mut server = DapServer::new();
        let request = DapRequest {
            seq: 1,
            command: "threads".to_string(),
            arguments: None,
        };

        let response = server.handle_request(request);
        assert!(response.success);
        assert!(response.body.is_some());
    }
}
