//! Error types for the MCP server

use crate::protocol::JsonRpcError;
use thiserror::Error;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, McpError>;

/// MCP server errors
#[derive(Debug, Error)]
pub enum McpError {
    /// Protocol error (invalid JSON-RPC)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Method not found
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Package manager error
    #[error("Package manager error: {0}")]
    PackageManager(#[from] buckos_package::Error),

    /// Permission error
    #[error("Permission error: {0}")]
    Permission(String),

    /// Invalid confirmation token
    #[error("Invalid confirmation token: {0}")]
    InvalidToken(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl McpError {
    /// Convert to JSON-RPC error
    pub fn to_jsonrpc(&self) -> JsonRpcError {
        match self {
            McpError::Protocol(msg) => JsonRpcError::invalid_request(msg),
            McpError::MethodNotFound(method) => JsonRpcError::method_not_found(method),
            McpError::InvalidParams(msg) => JsonRpcError::invalid_params(msg),
            McpError::PackageManager(e) => {
                // Map package manager errors to appropriate JSON-RPC errors
                match e {
                    buckos_package::Error::PackageNotFound(name) => {
                        JsonRpcError::package_not_found(name)
                    }
                    buckos_package::Error::PackageNotInstalled(name) => {
                        JsonRpcError::package_not_found(name)
                    }
                    buckos_package::Error::ResolutionFailed(msg) => {
                        JsonRpcError::dependency_failed(msg)
                    }
                    buckos_package::Error::BuildFailed { message, .. } => {
                        JsonRpcError::build_failed(message)
                    }
                    _ => JsonRpcError::internal_error(e.to_string()),
                }
            }
            McpError::Permission(msg) => JsonRpcError::insufficient_permissions(
                "Package operation",
                format!("{}\n\nSuggestions:\n- Restart with: sudo buckos mcp\n- Or use read-only tools for exploration", msg),
            ),
            McpError::InvalidToken(reason) => JsonRpcError::invalid_token(reason),
            McpError::Io(e) => JsonRpcError::internal_error(e.to_string()),
            McpError::Json(e) => JsonRpcError::invalid_params(e.to_string()),
            McpError::Internal(msg) => JsonRpcError::internal_error(msg),
        }
    }

    /// Create a permission error for operations requiring root
    pub fn requires_root(operation: &str) -> Self {
        McpError::Permission(format!(
            "Operation '{}' requires root privileges",
            operation
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_jsonrpc() {
        let err = McpError::MethodNotFound("test".to_string());
        let jsonrpc_err = err.to_jsonrpc();
        assert_eq!(jsonrpc_err.code, -32601);
    }

    #[test]
    fn test_permission_error() {
        let err = McpError::requires_root("install");
        assert!(err.to_string().contains("root privileges"));
    }
}
