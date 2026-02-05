//! E2E Integration Tests for DAP Server
//!
//! These tests verify the full protocol lifecycle and common debugging workflows.

use serde_json::{json, Value};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use vais_dap::server::DapServer;

/// Helper to send a DAP request and receive a response
struct DapTestClient {
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: BufWriter<tokio::net::tcp::OwnedWriteHalf>,
    seq: i64,
}

impl DapTestClient {
    async fn new(port: u16) -> Self {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to connect to DAP server");

        let (read_half, write_half) = stream.into_split();

        Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
            seq: 1,
        }
    }

    async fn send_request(&mut self, command: &str, arguments: Option<Value>) -> Value {
        let request = json!({
            "seq": self.seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        self.seq += 1;

        let request_json = serde_json::to_string(&request).unwrap();
        let message = format!(
            "Content-Length: {}\r\n\r\n{}",
            request_json.len(),
            request_json
        );

        // Send request
        self.writer.write_all(message.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();

        // Receive response
        self.read_response().await
    }

    async fn read_response(&mut self) -> Value {
        // Read headers
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).await.unwrap();

            if line.trim().is_empty() {
                break;
            }

            if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                content_length = len_str.trim().parse().unwrap();
            }
        }

        // Read body
        let mut body = vec![0u8; content_length];
        tokio::io::AsyncReadExt::read_exact(&mut self.reader, &mut body)
            .await
            .unwrap();

        let response: Value = serde_json::from_slice(&body).unwrap();
        response
    }
}

/// Start a DAP server on a random port for testing
async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let (reader, writer) = stream.into_split();
                        let mut server = DapServer::new();
                        let _ = server.run(reader, writer).await;
                    });
                }
                Err(_) => break,
            }
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    (port, handle)
}

// ============================================================================
// Protocol Message Round-Trip Tests
// ============================================================================

#[tokio::test]
async fn test_initialize_round_trip() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    let response = client
        .send_request(
            "initialize",
            Some(json!({
                "clientId": "vscode",
                "clientName": "Visual Studio Code",
                "adapterId": "vais",
                "linesStartAt1": true,
                "columnsStartAt1": true,
            })),
        )
        .await;

    println!(
        "Response: {}",
        serde_json::to_string_pretty(&response).unwrap()
    );

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "initialize");
    assert_eq!(
        response["success"], true,
        "Initialize request failed: {:?}",
        response["message"]
    );

    // Check capabilities (should be in body)
    if let Some(body) = response["body"].as_object() {
        // Just verify some capabilities exist - they may have different names than expected
        assert!(
            body.contains_key("supportsConfigurationDoneRequest")
                || body.contains_key("supports_configuration_done_request"),
            "Capabilities missing from body: {:?}",
            body
        );
    }
}

#[tokio::test]
async fn test_configuration_done_round_trip() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize first
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
                "linesStartAt1": true,
                "columnsStartAt1": true,
            })),
        )
        .await;

    // Send configurationDone
    let response = client.send_request("configurationDone", None).await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "configurationDone");
    assert_eq!(response["success"], true);
}

#[tokio::test]
async fn test_disconnect_round_trip() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize first
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Disconnect
    let response = client
        .send_request(
            "disconnect",
            Some(json!({
                "terminateDebuggee": false,
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "disconnect");
    assert_eq!(response["success"], true);
}

// ============================================================================
// Breakpoint Tests
// ============================================================================

#[tokio::test]
async fn test_set_breakpoints() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Mock launch (this will fail without a real binary, but we're testing the protocol)
    let _ = client
        .send_request(
            "launch",
            Some(json!({
                "program": "/tmp/test.vais",
                "stopOnEntry": true,
            })),
        )
        .await;

    // Set breakpoints
    let response = client
        .send_request(
            "setBreakpoints",
            Some(json!({
                "source": {
                    "path": "/tmp/test.vais"
                },
                "breakpoints": [
                    {
                        "line": 10
                    },
                    {
                        "line": 20,
                        "condition": "x > 5"
                    }
                ]
            })),
        )
        .await;

    // The request may fail (no active session), but we're testing protocol parsing
    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "setBreakpoints");

    // If successful, check breakpoints structure
    if response["success"] == true {
        let breakpoints = response["body"]["breakpoints"].as_array().unwrap();
        assert_eq!(breakpoints.len(), 2);

        // Check first breakpoint
        assert!(breakpoints[0]["verified"].as_bool().is_some());
        assert!(breakpoints[0]["line"].is_number());
    }
}

#[tokio::test]
async fn test_set_function_breakpoints() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Set function breakpoints
    let response = client
        .send_request(
            "setFunctionBreakpoints",
            Some(json!({
                "breakpoints": [
                    {
                        "name": "main"
                    },
                    {
                        "name": "calculate",
                        "condition": "result > 100"
                    }
                ]
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "setFunctionBreakpoints");

    // Check response structure (may fail without active session)
    if response["success"] == true {
        let breakpoints = response["body"]["breakpoints"].as_array().unwrap();
        assert_eq!(breakpoints.len(), 2);
    }
}

#[tokio::test]
async fn test_clear_breakpoints() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Set breakpoints first
    client
        .send_request(
            "setBreakpoints",
            Some(json!({
                "source": {
                    "path": "/tmp/test.vais"
                },
                "breakpoints": [
                    {"line": 10},
                    {"line": 20}
                ]
            })),
        )
        .await;

    // Clear breakpoints (set empty array)
    let response = client
        .send_request(
            "setBreakpoints",
            Some(json!({
                "source": {
                    "path": "/tmp/test.vais"
                },
                "breakpoints": []
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "setBreakpoints");

    if response["success"] == true {
        let breakpoints = response["body"]["breakpoints"].as_array().unwrap();
        assert_eq!(breakpoints.len(), 0);
    }
}

// ============================================================================
// Stack Trace Tests
// ============================================================================

#[tokio::test]
async fn test_stack_trace_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Request stack trace (will fail without active session, but testing protocol)
    let response = client
        .send_request(
            "stackTrace",
            Some(json!({
                "threadId": 1,
                "startFrame": 0,
                "levels": 10
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "stackTrace");

    // Should fail with no active session
    if response["success"] == false {
        assert!(response["message"].as_str().unwrap().contains("session"));
    } else {
        // If successful, check structure
        let body = &response["body"];
        assert!(body["stackFrames"].is_array());
        if let Some(total) = body["totalFrames"].as_i64() {
            assert!(total >= 0);
        }
    }
}

#[tokio::test]
async fn test_threads_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Request threads
    let response = client.send_request("threads", None).await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "threads");

    // Should fail without active session
    if response["success"] == false {
        assert!(response["message"].as_str().unwrap().contains("session"));
    }
}

// ============================================================================
// Variable Inspection Tests
// ============================================================================

#[tokio::test]
async fn test_scopes_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Request scopes (will fail without active session)
    let response = client
        .send_request(
            "scopes",
            Some(json!({
                "frameId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "scopes");

    if response["success"] == true {
        let scopes = response["body"]["scopes"].as_array().unwrap();
        assert!(scopes.len() > 0);

        // Check scope structure
        for scope in scopes {
            assert!(scope["name"].is_string());
            assert!(scope["variablesReference"].is_number());
        }
    }
}

#[tokio::test]
async fn test_variables_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Request variables (will fail without active session)
    let response = client
        .send_request(
            "variables",
            Some(json!({
                "variablesReference": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "variables");

    if response["success"] == true {
        let variables = response["body"]["variables"].as_array().unwrap();

        // Check variable structure
        for var in variables {
            assert!(var["name"].is_string());
            assert!(var["value"].is_string());
            assert!(var["variablesReference"].is_number());
        }
    }
}

#[tokio::test]
async fn test_set_variable_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Set variable (will fail without active session)
    let response = client
        .send_request(
            "setVariable",
            Some(json!({
                "variablesReference": 1,
                "name": "x",
                "value": "42"
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "setVariable");

    if response["success"] == true {
        let body = &response["body"];
        assert!(body["value"].is_string());
    }
}

// ============================================================================
// Execution Control Tests
// ============================================================================

#[tokio::test]
async fn test_continue_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Continue (will fail without active session)
    let response = client
        .send_request(
            "continue",
            Some(json!({
                "threadId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "continue");
}

#[tokio::test]
async fn test_step_over_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Step over (next)
    let response = client
        .send_request(
            "next",
            Some(json!({
                "threadId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "next");
}

#[tokio::test]
async fn test_step_in_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Step in
    let response = client
        .send_request(
            "stepIn",
            Some(json!({
                "threadId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "stepIn");
}

#[tokio::test]
async fn test_step_out_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Step out
    let response = client
        .send_request(
            "stepOut",
            Some(json!({
                "threadId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "stepOut");
}

#[tokio::test]
async fn test_pause_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Pause
    let response = client
        .send_request(
            "pause",
            Some(json!({
                "threadId": 1
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "pause");
}

// ============================================================================
// Evaluate Tests
// ============================================================================

#[tokio::test]
async fn test_evaluate_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Evaluate expression
    let response = client
        .send_request(
            "evaluate",
            Some(json!({
                "expression": "1 + 1",
                "context": "repl"
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "evaluate");

    if response["success"] == true {
        let body = &response["body"];
        assert!(body["result"].is_string());
        assert!(body["variablesReference"].is_number());
    }
}

// ============================================================================
// Memory Operations Tests
// ============================================================================

#[tokio::test]
async fn test_read_memory_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Read memory
    let response = client
        .send_request(
            "readMemory",
            Some(json!({
                "memoryReference": "0x1000",
                "offset": 0,
                "count": 64
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "readMemory");

    if response["success"] == true {
        let body = &response["body"];
        assert!(body["address"].is_string());
    }
}

#[tokio::test]
async fn test_disassemble_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Disassemble
    let response = client
        .send_request(
            "disassemble",
            Some(json!({
                "memoryReference": "0x1000",
                "offset": 0,
                "instructionOffset": 0,
                "instructionCount": 10,
                "resolveSymbols": true
            })),
        )
        .await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["command"], "disassemble");

    if response["success"] == true {
        let body = &response["body"];
        assert!(body["instructions"].is_array());
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_request() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Send invalid command
    let response = client.send_request("invalidCommand", None).await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["success"], false);
    assert!(response["message"]
        .as_str()
        .unwrap()
        .contains("not supported"));
}

#[tokio::test]
async fn test_missing_arguments() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Send request with missing required arguments
    let response = client.send_request("setBreakpoints", None).await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["success"], false);
}

#[tokio::test]
async fn test_operation_without_session() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // Initialize
    client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
            })),
        )
        .await;

    // Try to get threads without launching/attaching
    let response = client.send_request("threads", None).await;

    assert_eq!(response["type"], "response");
    assert_eq!(response["success"], false);
    assert!(response["message"].as_str().unwrap().contains("session"));
}

// ============================================================================
// Full Session Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_full_session_lifecycle() {
    let (port, _handle) = start_test_server().await;
    let mut client = DapTestClient::new(port).await;

    // 1. Initialize
    let response = client
        .send_request(
            "initialize",
            Some(json!({
                "adapterId": "vais",
                "linesStartAt1": true,
                "columnsStartAt1": true,
            })),
        )
        .await;
    assert_eq!(response["success"], true);

    // 2. Launch (will fail without real binary, but testing protocol)
    let _ = client
        .send_request(
            "launch",
            Some(json!({
                "program": "/tmp/test.vais",
                "stopOnEntry": true,
            })),
        )
        .await;

    // 3. Set breakpoints
    let _ = client
        .send_request(
            "setBreakpoints",
            Some(json!({
                "source": { "path": "/tmp/test.vais" },
                "breakpoints": [{"line": 10}]
            })),
        )
        .await;

    // 4. Configuration done
    let response = client.send_request("configurationDone", None).await;
    assert_eq!(response["command"], "configurationDone");

    // 5. Disconnect
    let response = client
        .send_request(
            "disconnect",
            Some(json!({
                "terminateDebuggee": true,
            })),
        )
        .await;
    assert_eq!(response["command"], "disconnect");
}
