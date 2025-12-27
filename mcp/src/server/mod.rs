//! MCP server implementation
//!
//! Core server that routes JSON-RPC requests to appropriate handlers.

pub mod confirmation;
pub mod tools;

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use crate::handlers::package_ops;
use crate::permissions::ExecutionContext;
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId, StdioTransport};
use buckos_package::PackageManager;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, warn};

pub use confirmation::ConfirmationToken;
pub use tools::ToolDefinition;

/// MCP server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "buckos-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl ServerConfig {
    /// Load configuration from a file
    pub fn load_from(_path: &str) -> Result<Self> {
        // TODO: Implement configuration file loading
        Ok(Self::default())
    }
}

/// MCP server
pub struct McpServer {
    context: Arc<McpServerContext>,
    config: ServerConfig,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(pm: PackageManager, config: ServerConfig, exec_context: ExecutionContext) -> Self {
        let context = Arc::new(McpServerContext::new(pm, exec_context));

        info!(
            server = config.name,
            version = config.version,
            execution_context = ?context.exec_context,
            "MCP server initialized"
        );

        Self { context, config }
    }

    /// Serve requests over stdio
    pub async fn serve_stdio(&self) -> Result<()> {
        let mut transport = StdioTransport::new();

        info!("MCP server listening on stdio");

        loop {
            // Read request
            let request = match transport.read_request().await {
                Ok(Some(req)) => req,
                Ok(None) => {
                    // EOF - client disconnected
                    info!("Client disconnected");
                    break;
                }
                Err(e) => {
                    error!(error = %e, "Failed to read request");
                    // Send parse error
                    let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                    transport.write_response(&response).await?;
                    continue;
                }
            };

            // Handle request
            let response = self.handle_request(request).await;

            // Write response (skip for notifications)
            if response.id.is_some() || response.error.is_some() {
                transport.write_response(&response).await?;
            }
        }

        transport.close().await?;
        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        // Handle the method
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tool_call(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            _ => Err(McpError::MethodNotFound(request.method.clone())),
        };

        match result {
            Ok(value) => JsonRpcResponse::success(id.unwrap_or(RequestId::Number(0)), value),
            Err(e) => {
                warn!(error = %e, "Request failed");
                JsonRpcResponse::error(id, e.to_jsonrpc())
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        info!(?params, "Received initialize request");

        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": self.config.name,
                "version": self.config.version
            },
            "executionContext": {
                "description": self.context.exec_context.description(),
                "isRoot": self.context.exec_context.is_root,
                "uid": self.context.exec_context.effective_uid,
                "userMode": self.context.exec_context.user_mode,
                "installRoot": self.context.exec_context.install_root
            }
        }))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value> {
        let tools = tools::get_all_tools(&self.context.exec_context);

        Ok(json!({
            "tools": tools
        }))
    }

    /// Handle tools/call request
    async fn handle_tool_call(&self, params: Option<Value>) -> Result<Value> {
        let params =
            params.ok_or_else(|| McpError::InvalidParams("Missing parameters".to_string()))?;

        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing tool name".to_string()))?;

        let arguments = params["arguments"].clone();

        info!(tool = tool_name, "Calling tool");

        // Route to appropriate handler
        match tool_name {
            "package_search" => package_ops::handle_search(&self.context, arguments).await,
            "package_info" => package_ops::handle_info(&self.context, arguments).await,
            "package_list" => package_ops::handle_list(&self.context, arguments).await,
            "package_deps" => package_ops::handle_deps(&self.context, arguments).await,
            "package_install" => package_ops::handle_install(&self.context, arguments).await,
            "config_show" => package_ops::handle_config_show(&self.context, arguments).await,
            _ => Err(McpError::MethodNotFound(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self) -> Result<Value> {
        // TODO: Implement resource listing
        Ok(json!({
            "resources": []
        }))
    }
}
