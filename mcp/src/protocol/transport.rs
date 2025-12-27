//! Stdio transport for JSON-RPC messages
//!
//! Implements line-delimited JSON communication over stdin/stdout for MCP.
//! Each message is a single line of JSON terminated by a newline.

use super::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error};

/// Stdio transport for JSON-RPC messages
///
/// Reads JSON-RPC requests from stdin and writes responses to stdout.
/// Each message is a single line of JSON.
pub struct StdioTransport {
    stdin: BufReader<io::Stdin>,
    stdout: io::Stdout,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(io::stdin()),
            stdout: io::stdout(),
        }
    }

    /// Read a JSON-RPC request from stdin
    ///
    /// Reads one line from stdin and parses it as a JSON-RPC request.
    /// Returns None on EOF.
    pub async fn read_request(&mut self) -> io::Result<Option<JsonRpcRequest>> {
        loop {
            let mut line = String::new();

            let n = self.stdin.read_line(&mut line).await?;
            if n == 0 {
                // EOF
                return Ok(None);
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                // Empty line, try again
                continue;
            }

            match serde_json::from_str(trimmed) {
                Ok(request) => {
                    debug!(request = ?request, "Received JSON-RPC request");
                    return Ok(Some(request));
                }
                Err(e) => {
                    error!(error = %e, line = %trimmed, "Failed to parse JSON-RPC request");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid JSON: {}", e),
                    ));
                }
            }
        }
    }

    /// Write a JSON-RPC response to stdout
    ///
    /// Serializes the response as JSON and writes it as a single line to stdout,
    /// followed by a newline and flush.
    pub async fn write_response(&mut self, response: &JsonRpcResponse) -> io::Result<()> {
        let json = serde_json::to_string(response).map_err(|e| {
            error!(error = %e, "Failed to serialize JSON-RPC response");
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

        debug!(response = ?response, "Sending JSON-RPC response");

        self.stdout.write_all(json.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await?;

        Ok(())
    }

    /// Close the transport
    ///
    /// Flushes stdout and shuts down.
    pub async fn close(&mut self) -> io::Result<()> {
        self.stdout.flush().await?;
        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcError, RequestId};

    #[test]
    fn test_serialize_request() {
        let req = JsonRpcRequest::new(
            RequestId::Number(1),
            "test_method",
            Some(serde_json::json!({"arg": "value"})),
        );

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_serialize_response() {
        let resp =
            JsonRpcResponse::success(RequestId::Number(1), serde_json::json!({"result": "ok"}));

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\":{\"result\":\"ok\"}"));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_serialize_error_response() {
        let error = JsonRpcError::method_not_found("unknown");
        let resp = JsonRpcResponse::error(Some(RequestId::Number(1)), error);

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
        assert!(!json.contains("\"result\""));
    }
}
