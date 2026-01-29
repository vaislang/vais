//! DAP Protocol Codec
//!
//! Implements the DAP wire format: Content-Length headers followed by JSON payload.

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::error::{DapError, DapResult};
use super::types::{Request, Response, Event, MessageType, ProtocolMessage};

const CONTENT_LENGTH: &str = "Content-Length: ";
const HEADER_DELIMITER: &[u8] = b"\r\n\r\n";

/// Codec for DAP protocol messages
#[derive(Debug, Default)]
pub struct DapCodec {
    /// Content length parsed from header
    content_length: Option<usize>,
}

impl DapCodec {
    pub fn new() -> Self {
        Self {
            content_length: None,
        }
    }
}

/// A decoded DAP message
#[derive(Debug, Clone)]
pub enum DapMessage {
    Request(Request),
    Response(Response),
    Event(Event),
}

impl DapMessage {
    /// Get the sequence number of this message
    pub fn seq(&self) -> i64 {
        match self {
            DapMessage::Request(r) => r.base.seq,
            DapMessage::Response(r) => r.base.seq,
            DapMessage::Event(e) => e.base.seq,
        }
    }

    /// Create a request message
    pub fn request(seq: i64, command: impl Into<String>, arguments: Option<serde_json::Value>) -> Self {
        DapMessage::Request(Request {
            base: ProtocolMessage {
                seq,
                message_type: MessageType::Request,
            },
            command: command.into(),
            arguments,
        })
    }

    /// Create a success response
    pub fn response_success(
        seq: i64,
        request_seq: i64,
        command: impl Into<String>,
        body: Option<serde_json::Value>,
    ) -> Self {
        DapMessage::Response(Response {
            base: ProtocolMessage {
                seq,
                message_type: MessageType::Response,
            },
            request_seq,
            success: true,
            command: command.into(),
            message: None,
            body,
        })
    }

    /// Create an error response
    pub fn response_error(
        seq: i64,
        request_seq: i64,
        command: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        DapMessage::Response(Response {
            base: ProtocolMessage {
                seq,
                message_type: MessageType::Response,
            },
            request_seq,
            success: false,
            command: command.into(),
            message: Some(message.into()),
            body: None,
        })
    }

    /// Create an event
    pub fn event(seq: i64, event: impl Into<String>, body: Option<serde_json::Value>) -> Self {
        DapMessage::Event(Event {
            base: ProtocolMessage {
                seq,
                message_type: MessageType::Event,
            },
            event: event.into(),
            body,
        })
    }
}

impl Decoder for DapCodec {
    type Item = DapMessage;
    type Error = DapError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // If we don't have a content length yet, try to parse the header
        if self.content_length.is_none() {
            // Find the header delimiter
            if let Some(pos) = find_subsequence(src, HEADER_DELIMITER) {
                let header_bytes = src.split_to(pos);
                let header = std::str::from_utf8(&header_bytes)
                    .map_err(|e| DapError::Protocol(format!("Invalid header encoding: {}", e)))?;

                // Parse Content-Length
                let content_length = parse_content_length(header)?;
                self.content_length = Some(content_length);

                // Skip the delimiter
                src.advance(HEADER_DELIMITER.len());
            } else {
                // Need more data for header
                return Ok(None);
            }
        }

        // Check if we have enough data for the body
        if let Some(content_length) = self.content_length {
            if src.len() >= content_length {
                let body_bytes = src.split_to(content_length);
                self.content_length = None;

                // Parse JSON
                let value: serde_json::Value = serde_json::from_slice(&body_bytes)?;

                // Determine message type
                let message_type = value
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DapError::Protocol("Missing 'type' field".to_string()))?;

                let message = match message_type {
                    "request" => {
                        let request: Request = serde_json::from_value(value)?;
                        DapMessage::Request(request)
                    }
                    "response" => {
                        let response: Response = serde_json::from_value(value)?;
                        DapMessage::Response(response)
                    }
                    "event" => {
                        let event: Event = serde_json::from_value(value)?;
                        DapMessage::Event(event)
                    }
                    _ => {
                        return Err(DapError::Protocol(format!(
                            "Unknown message type: {}",
                            message_type
                        )));
                    }
                };

                return Ok(Some(message));
            }
        }

        // Need more data
        Ok(None)
    }
}

impl Encoder<DapMessage> for DapCodec {
    type Error = DapError;

    fn encode(&mut self, item: DapMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the message to JSON
        let json = match item {
            DapMessage::Request(r) => serde_json::to_string(&r)?,
            DapMessage::Response(r) => serde_json::to_string(&r)?,
            DapMessage::Event(e) => serde_json::to_string(&e)?,
        };

        // Write header
        let header = format!("{}{}\r\n\r\n", CONTENT_LENGTH, json.len());
        dst.put_slice(header.as_bytes());

        // Write body
        dst.put_slice(json.as_bytes());

        Ok(())
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_content_length(header: &str) -> DapResult<usize> {
    for line in header.lines() {
        if let Some(value) = line.strip_prefix(CONTENT_LENGTH) {
            return value
                .trim()
                .parse()
                .map_err(|e| DapError::Protocol(format!("Invalid Content-Length: {}", e)));
        }
    }
    Err(DapError::Protocol("Missing Content-Length header".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_request() {
        let mut codec = DapCodec::new();
        let mut buf = BytesMut::new();

        let json = r#"{"seq":1,"type":"request","command":"initialize","arguments":{"clientID":"test"}}"#;
        let header = format!("Content-Length: {}\r\n\r\n", json.len());

        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(json.as_bytes());

        let result = codec.decode(&mut buf).unwrap();
        assert!(result.is_some());

        if let Some(DapMessage::Request(req)) = result {
            assert_eq!(req.base.seq, 1);
            assert_eq!(req.command, "initialize");
        } else {
            panic!("Expected Request");
        }
    }

    #[test]
    fn test_encode_response() {
        let mut codec = DapCodec::new();
        let mut buf = BytesMut::new();

        let msg = DapMessage::response_success(
            1,
            1,
            "initialize",
            Some(serde_json::json!({"supportsConfigurationDoneRequest": true})),
        );

        codec.encode(msg, &mut buf).unwrap();

        let output = std::str::from_utf8(&buf).unwrap();
        assert!(output.starts_with("Content-Length: "));
        assert!(output.contains("\"success\":true"));
    }
}
