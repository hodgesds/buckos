//! MCP server implementation
//!
//! Core server that routes JSON-RPC requests to appropriate handlers.

pub mod confirmation;
pub mod tools;

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use crate::handlers::{package_create, package_ops, spec_ops};
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
    pub fn load_from(path: &str) -> Result<Self> {
        let path = std::path::Path::new(path);

        if !path.exists() {
            // If config file doesn't exist, return defaults
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| McpError::Internal(format!("Failed to read config file: {}", e)))?;

        let config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| McpError::Internal(format!("Failed to parse config file: {}", e)))?;

        Ok(Self {
            name: config["name"].as_str().unwrap_or("buckos-mcp").to_string(),
            version: config["version"]
                .as_str()
                .unwrap_or(env!("CARGO_PKG_VERSION"))
                .to_string(),
        })
    }
}

/// MCP server
pub struct McpServer {
    context: Arc<McpServerContext>,
    config: ServerConfig,
}

impl McpServer {
    /// Create a new MCP server
    #[allow(clippy::arc_with_non_send_sync)]
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
            "resources/read" => self.handle_resources_read(request.params).await,
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
            // Package operations
            "package_search" => package_ops::handle_search(&self.context, arguments).await,
            "package_info" => package_ops::handle_info(&self.context, arguments).await,
            "package_list" => package_ops::handle_list(&self.context, arguments).await,
            "package_deps" => package_ops::handle_deps(&self.context, arguments).await,
            "package_install" => package_ops::handle_install(&self.context, arguments).await,
            "config_show" => package_ops::handle_config_show(&self.context, arguments).await,
            // Spec operations
            "spec_list" => spec_ops::handle_spec_list(&self.context, arguments).await,
            "spec_info" => spec_ops::handle_spec_info(&self.context, arguments).await,
            "spec_validate_system" => {
                spec_ops::handle_spec_validate_system(&self.context, arguments).await
            }
            "spec_validate_use_flags" => {
                spec_ops::handle_spec_validate_use_flags(&self.context, arguments).await
            }
            "spec_validate_package_set" => {
                spec_ops::handle_spec_validate_package_set(&self.context, arguments).await
            }
            // Package creation operations
            "package_create_template" => {
                package_create::handle_create_template(&self.context, arguments).await
            }
            "package_validate_definition" => {
                package_create::handle_validate_definition(&self.context, arguments).await
            }
            "package_suggest_dependencies" => {
                package_create::handle_suggest_dependencies(&self.context, arguments).await
            }
            "package_suggest_use_flags" => {
                package_create::handle_suggest_use_flags(&self.context, arguments).await
            }
            "package_convert_ebuild" => {
                package_create::handle_convert_ebuild(&self.context, arguments).await
            }
            "package_get_examples" => {
                package_create::handle_get_examples(&self.context, arguments).await
            }
            _ => Err(McpError::MethodNotFound(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self) -> Result<Value> {
        // Define available resources that clients can access
        let resources = vec![
            json!({
                "uri": "buckos://config/make.conf",
                "name": "System Configuration",
                "description": "Main system configuration (make.conf)",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://config/use",
                "name": "USE Flags",
                "description": "Global and per-package USE flag configuration",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://config/repos",
                "name": "Repositories",
                "description": "Configured package repositories",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://packages/installed",
                "name": "Installed Packages",
                "description": "List of currently installed packages",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://packages/world",
                "name": "World Set",
                "description": "User-selected packages (@world set)",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://specs/registry",
                "name": "Specification Registry",
                "description": "Available BuckOS specifications",
                "mimeType": "application/json"
            }),
            json!({
                "uri": "buckos://templates/list",
                "name": "Package Templates",
                "description": "Available package definition templates",
                "mimeType": "application/json"
            }),
        ];

        Ok(json!({
            "resources": resources
        }))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value> {
        let params =
            params.ok_or_else(|| McpError::InvalidParams("Missing parameters".to_string()))?;

        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParams("Missing 'uri' parameter".to_string()))?;

        info!(uri = uri, "Reading resource");

        // Parse the URI
        let content = match uri {
            "buckos://config/make.conf" => {
                let config = buckos_config::load_system_config()
                    .map_err(|e| McpError::Internal(format!("Failed to load config: {}", e)))?;
                json!({
                    "cflags": config.make_conf.cflags,
                    "cxxflags": config.make_conf.cxxflags,
                    "chost": config.make_conf.chost,
                    "use_flags": config.make_conf.use_config.global,
                    "features": config.make_conf.features.enabled,
                    "makeopts": config.make_conf.makeopts,
                })
            }
            "buckos://config/use" => {
                let config = buckos_config::load_system_config()
                    .map_err(|e| McpError::Internal(format!("Failed to load config: {}", e)))?;
                json!({
                    "global": config.make_conf.use_config.global,
                    "expand": config.make_conf.use_config.expand,
                })
            }
            "buckos://config/repos" => {
                let config = buckos_config::load_system_config()
                    .map_err(|e| McpError::Internal(format!("Failed to load config: {}", e)))?;
                let repos: Vec<_> = config
                    .repos
                    .repos
                    .iter()
                    .map(|(name, repo)| {
                        json!({
                            "name": name,
                            "location": repo.location.to_string_lossy(),
                            "sync_type": format!("{:?}", repo.sync_type),
                            "priority": repo.priority,
                        })
                    })
                    .collect();
                json!({ "repos": repos })
            }
            "buckos://packages/installed" => {
                let packages = self.context.pm.list_installed().await?;
                let result: Vec<Value> = packages
                    .iter()
                    .map(|pkg| {
                        json!({
                            "name": pkg.id.to_string(),
                            "version": pkg.version,
                            "category": pkg.id.category,
                            "package": pkg.id.name
                        })
                    })
                    .collect();
                json!({ "packages": result, "count": result.len() })
            }
            "buckos://packages/world" => {
                let config = buckos_config::load_system_config()
                    .map_err(|e| McpError::Internal(format!("Failed to load config: {}", e)))?;
                let world_set = config.sets.get("world");
                let packages: Vec<String> = world_set
                    .map(|s| s.atoms.iter().map(|a| a.to_string()).collect())
                    .unwrap_or_default();
                json!({
                    "packages": packages,
                    "count": packages.len(),
                })
            }
            "buckos://specs/registry" => {
                use crate::spec_registry::SpecRegistry;
                let specs_path = std::env::var("BUCKOS_SPECS_PATH")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("/usr/share/buckos/specs"));
                let registry = SpecRegistry::load(&specs_path)
                    .map_err(|e| McpError::Internal(format!("Failed to load specs: {}", e)))?;
                let specs = registry.list_specs(None, None);
                json!({
                    "specs": specs.iter().map(|s| json!({
                        "id": s.id,
                        "title": s.title,
                        "status": s.status,
                        "category": s.category,
                    })).collect::<Vec<_>>(),
                    "count": specs.len(),
                })
            }
            "buckos://templates/list" => {
                let templates = vec![
                    "simple",
                    "autotools",
                    "cmake",
                    "meson",
                    "cargo",
                    "go",
                    "python",
                ];
                json!({
                    "templates": templates,
                    "count": templates.len(),
                })
            }
            _ => {
                return Err(McpError::InvalidParams(format!(
                    "Unknown resource URI: {}",
                    uri
                )));
            }
        };

        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string_pretty(&content)
                    .map_err(|e| McpError::Internal(format!("Failed to serialize: {}", e)))?
            }]
        }))
    }
}
