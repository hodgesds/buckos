//! MCP tool definitions and registry

use crate::permissions::ExecutionContext;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,

    /// Whether the tool is available in the current context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    /// Reason if not available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Get all available tools
pub fn get_all_tools(exec_context: &ExecutionContext) -> Vec<ToolDefinition> {
    vec![
        tool_package_search(exec_context),
        tool_package_info(exec_context),
        tool_package_list(exec_context),
        tool_package_deps(exec_context),
        tool_package_install(exec_context),
        tool_config_show(exec_context),
    ]
}

fn check_availability(
    tool_name: &str,
    exec_context: &ExecutionContext,
) -> (Option<bool>, Option<String>) {
    let (available, reason) = exec_context.tool_available(tool_name);
    if available {
        (None, None) // Don't include if available (implicit true)
    } else {
        (
            Some(false),
            Some(reason.unwrap_or_else(|| "Not available".to_string())),
        )
    }
}

fn tool_package_search(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_search", exec_context);

    ToolDefinition {
        name: "package_search".to_string(),
        description: "Search for packages matching a query string. Searches package names, descriptions, and maintainers.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (package name, description keyword, maintainer)",
                    "minLength": 1
                }
            },
            "required": ["query"]
        }),
        available,
        reason,
    }
}

fn tool_package_info(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_info", exec_context);

    ToolDefinition {
        name: "package_info".to_string(),
        description: "Get detailed information about a specific package including version, description, USE flags, and dependencies.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package": {
                    "type": "string",
                    "description": "Package name (e.g., 'bash', 'sys-apps/systemd')"
                }
            },
            "required": ["package"]
        }),
        available,
        reason,
    }
}

fn tool_package_list(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_list", exec_context);

    ToolDefinition {
        name: "package_list".to_string(),
        description: "List installed packages.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "string",
                    "description": "Filter packages (default: installed)",
                    "enum": ["installed", "available"]
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_package_deps(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_deps", exec_context);

    ToolDefinition {
        name: "package_deps".to_string(),
        description: "Show dependencies for a package.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package": {
                    "type": "string",
                    "description": "Package name"
                },
                "depth": {
                    "type": "integer",
                    "description": "Maximum depth to traverse (default: unlimited)",
                    "minimum": 1
                }
            },
            "required": ["package"]
        }),
        available,
        reason,
    }
}

fn tool_package_install(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_install", exec_context);

    ToolDefinition {
        name: "package_install".to_string(),
        description: "Install packages. Two-phase: call with dry_run=true first to preview, then with confirmation_token to execute.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "packages": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Package names to install",
                    "minItems": 1
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "If true, only show what would be installed and return a confirmation token",
                    "default": true
                },
                "confirmation_token": {
                    "type": "string",
                    "description": "Token from dry-run call, required to execute installation"
                }
            },
            "required": ["packages"]
        }),
        available,
        reason,
    }
}

fn tool_config_show(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("config_show", exec_context);

    ToolDefinition {
        name: "config_show".to_string(),
        description: "Show current package manager configuration.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
        available,
        reason,
    }
}
