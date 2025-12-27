//! JSON-RPC 2.0 protocol types
//!
//! Implementation of JSON-RPC 2.0 specification for MCP communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID (may be null for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,

    /// Method name
    pub method: String,

    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: RequestId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: method.into(),
            params,
        }
    }

    /// Create a notification (request without ID)
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.into(),
            params,
        }
    }

    /// Check if this is a notification
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID (same as request, or null for error before ID was extracted)
    pub id: Option<RequestId>,

    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Option<RequestId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create an error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    // Standard JSON-RPC 2.0 errors

    /// Parse error (-32700): Invalid JSON
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    /// Invalid request (-32600): Not a valid request object
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::new(-32600, format!("Invalid request: {}", msg.into()))
    }

    /// Method not found (-32601): Method does not exist
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(-32601, format!("Method not found: {}", method.into()))
    }

    /// Invalid params (-32602): Invalid method parameters
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::new(-32602, format!("Invalid params: {}", msg.into()))
    }

    /// Internal error (-32603): Internal JSON-RPC error
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::new(-32603, format!("Internal error: {}", msg.into()))
    }

    // Custom server errors (-32000 to -32099)

    /// Package not found (-32001)
    pub fn package_not_found(package: impl Into<String>) -> Self {
        Self::with_data(
            -32001,
            "Package not found",
            serde_json::json!({"package": package.into()}),
        )
    }

    /// Dependency resolution failed (-32002)
    pub fn dependency_failed(details: impl Into<String>) -> Self {
        Self::with_data(
            -32002,
            "Dependency resolution failed",
            serde_json::json!({"details": details.into()}),
        )
    }

    /// Build failed (-32003)
    pub fn build_failed(details: impl Into<String>) -> Self {
        Self::with_data(
            -32003,
            "Build failed",
            serde_json::json!({"details": details.into()}),
        )
    }

    /// Invalid confirmation token (-32004)
    pub fn invalid_token(reason: impl Into<String>) -> Self {
        Self::with_data(
            -32004,
            "Invalid confirmation token",
            serde_json::json!({"reason": reason.into()}),
        )
    }

    /// Insufficient permissions (-32005)
    pub fn insufficient_permissions(
        operation: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::with_data(
            -32005,
            "Insufficient permissions",
            serde_json::json!({
                "operation": operation.into(),
                "suggestion": suggestion.into()
            }),
        )
    }
}

/// Request/Response ID (can be string or number)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String ID
    String(String),
    /// Numeric ID
    Number(i64),
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{}", s),
            RequestId::Number(n) => write!(f, "{}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(
            RequestId::Number(1),
            "test_method",
            Some(serde_json::json!({"key": "value"})),
        );

        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.method, "test_method");
        assert_eq!(parsed.id, Some(RequestId::Number(1)));
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success(
            RequestId::Number(1),
            serde_json::json!({"result": "success"}),
        );

        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let error = JsonRpcError::method_not_found("unknown_method");
        let resp = JsonRpcResponse::error(Some(RequestId::Number(1)), error);

        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_notification() {
        let notification = JsonRpcRequest::notification("notify", None);
        assert!(notification.is_notification());
        assert_eq!(notification.id, None);
    }
}
