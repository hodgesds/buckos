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
        // Package management tools
        tool_package_search(exec_context),
        tool_package_info(exec_context),
        tool_package_list(exec_context),
        tool_package_deps(exec_context),
        tool_package_install(exec_context),
        tool_config_show(exec_context),
        // Spec validation tools
        tool_spec_list(exec_context),
        tool_spec_info(exec_context),
        tool_spec_validate_system(exec_context),
        tool_spec_validate_use_flags(exec_context),
        tool_spec_validate_package_set(exec_context),
        // Package creation tools
        tool_package_create_template(exec_context),
        tool_package_validate_definition(exec_context),
        tool_package_suggest_dependencies(exec_context),
        tool_package_suggest_use_flags(exec_context),
        tool_package_convert_ebuild(exec_context),
        tool_package_get_examples(exec_context),
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

fn tool_spec_list(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("spec_list", exec_context);

    ToolDefinition {
        name: "spec_list".to_string(),
        description: "List all BuckOS specifications with their status and version information."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "Filter by category (core, bootstrap, integration, features, tooling)",
                    "enum": ["core", "bootstrap", "integration", "features", "tooling"]
                },
                "status": {
                    "type": "string",
                    "description": "Filter by status (approved, rfc, draft, deprecated, rejected)",
                    "enum": ["approved", "rfc", "draft", "deprecated", "rejected"]
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_spec_info(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("spec_info", exec_context);

    ToolDefinition {
        name: "spec_info".to_string(),
        description: "Get detailed information about a specific BuckOS specification.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "spec_id": {
                    "type": "string",
                    "description": "Specification ID (e.g., 'SPEC-001', 'SPEC-002')",
                    "pattern": "^SPEC-\\d{3}$"
                }
            },
            "required": ["spec_id"]
        }),
        available,
        reason,
    }
}

fn tool_spec_validate_system(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("spec_validate_system", exec_context);

    ToolDefinition {
        name: "spec_validate_system".to_string(),
        description: "Validate system configuration against BuckOS specifications. Checks package versions, USE flags, dependencies, and conformance to specified profile.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "profile": {
                    "type": "string",
                    "description": "System profile to validate against (minimal, server, desktop, developer, hardened, embedded, container)",
                    "enum": ["minimal", "server", "desktop", "developer", "hardened", "embedded", "container"]
                },
                "check_dependencies": {
                    "type": "boolean",
                    "description": "Also validate dependency resolution and circular dependencies",
                    "default": true
                },
                "check_use_flags": {
                    "type": "boolean",
                    "description": "Validate USE flag configuration",
                    "default": true
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_spec_validate_use_flags(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("spec_validate_use_flags", exec_context);

    ToolDefinition {
        name: "spec_validate_use_flags".to_string(),
        description: "Validate USE flag configuration for a package or globally. Checks for unknown flags, conflicts, and profile consistency.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package": {
                    "type": "string",
                    "description": "Package to validate (optional, validates global USE flags if not specified)"
                },
                "use_flags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "USE flags to validate (optional, uses current configuration if not specified)"
                },
                "profile": {
                    "type": "string",
                    "description": "Profile to validate against (minimal, server, desktop, etc.)",
                    "enum": ["minimal", "server", "desktop", "developer", "hardened", "embedded", "container"]
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_spec_validate_package_set(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("spec_validate_package_set", exec_context);

    ToolDefinition {
        name: "spec_validate_package_set".to_string(),
        description: "Validate a package set definition. Checks inheritance chains, package availability, and set consistency.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "set_name": {
                    "type": "string",
                    "description": "Package set name (e.g., '@system', '@world', 'web-server', 'gnome-desktop')"
                },
                "check_inheritance": {
                    "type": "boolean",
                    "description": "Check for circular inheritance and validate inheritance chain",
                    "default": true
                },
                "check_packages": {
                    "type": "boolean",
                    "description": "Validate that all packages in the set exist and are available",
                    "default": true
                }
            },
            "required": ["set_name"]
        }),
        available,
        reason,
    }
}

fn tool_package_create_template(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_create_template", exec_context);

    ToolDefinition {
        name: "package_create_template".to_string(),
        description: "Generate a BUCK file template for creating a new BuckOS package. Supports different package types (simple, autotools, cmake, meson, cargo, go, python) and generates appropriate structure based on build system.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package_name": {
                    "type": "string",
                    "description": "Name of the package (e.g., 'vim', 'nginx')",
                    "minLength": 1
                },
                "category": {
                    "type": "string",
                    "description": "Package category (e.g., 'editors', 'www-servers', 'dev-lang')",
                    "minLength": 1
                },
                "package_type": {
                    "type": "string",
                    "description": "Type of package to create based on build system",
                    "enum": ["simple", "autotools", "cmake", "meson", "cargo", "go", "python"],
                    "default": "simple"
                },
                "version": {
                    "type": "string",
                    "description": "Package version (e.g., '9.0.1', '1.2.3')"
                },
                "description": {
                    "type": "string",
                    "description": "Brief description of the package"
                },
                "homepage": {
                    "type": "string",
                    "description": "Package homepage URL"
                },
                "license": {
                    "type": "string",
                    "description": "Package license (e.g., 'MIT', 'GPL-2', 'Apache-2.0')"
                }
            },
            "required": ["package_name", "category"]
        }),
        available,
        reason,
    }
}

fn tool_package_validate_definition(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_validate_definition", exec_context);

    ToolDefinition {
        name: "package_validate_definition".to_string(),
        description: "Validate a package BUCK file definition against BuckOS specifications. Checks syntax, required fields, USE flags, dependencies, and conformance to SPEC-001.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "buck_content": {
                    "type": "string",
                    "description": "Content of the BUCK file to validate"
                },
                "package_path": {
                    "type": "string",
                    "description": "Optional path to BUCK file to validate (alternative to buck_content)"
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_package_suggest_dependencies(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_suggest_dependencies", exec_context);

    ToolDefinition {
        name: "package_suggest_dependencies".to_string(),
        description: "Suggest likely dependencies for a package based on package type, build system, and common patterns. Uses package category and build tools to recommend dependencies.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package_name": {
                    "type": "string",
                    "description": "Name of the package"
                },
                "category": {
                    "type": "string",
                    "description": "Package category (e.g., 'dev-lang', 'www-servers')"
                },
                "package_type": {
                    "type": "string",
                    "description": "Package type based on build system",
                    "enum": ["simple", "autotools", "cmake", "meson", "cargo", "go", "python"],
                    "default": "simple"
                },
                "build_system": {
                    "type": "string",
                    "description": "Build system used (for informational purposes)"
                },
                "description": {
                    "type": "string",
                    "description": "Package description (helps identify likely dependencies)"
                }
            },
            "required": ["package_name"]
        }),
        available,
        reason,
    }
}

fn tool_package_suggest_use_flags(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_suggest_use_flags", exec_context);

    ToolDefinition {
        name: "package_suggest_use_flags".to_string(),
        description: "Suggest appropriate USE flags for a package based on its type and common optional features. Follows SPEC-002 USE flag conventions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package_name": {
                    "type": "string",
                    "description": "Name of the package"
                },
                "category": {
                    "type": "string",
                    "description": "Package category"
                },
                "package_type": {
                    "type": "string",
                    "description": "Package type based on build system",
                    "enum": ["simple", "autotools", "cmake", "meson", "cargo", "go", "python"],
                    "default": "simple"
                },
                "features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Known optional features of the package"
                },
                "description": {
                    "type": "string",
                    "description": "Package description"
                }
            },
            "required": ["package_name"]
        }),
        available,
        reason,
    }
}

fn tool_package_convert_ebuild(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_convert_ebuild", exec_context);

    ToolDefinition {
        name: "package_convert_ebuild".to_string(),
        description: "Help convert a Gentoo ebuild to BuckOS package format. Analyzes ebuild structure and generates equivalent BUCK file with USE flags, dependencies, and build instructions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "ebuild_content": {
                    "type": "string",
                    "description": "Content of the Gentoo ebuild file"
                },
                "ebuild_path": {
                    "type": "string",
                    "description": "Optional path to ebuild file (alternative to ebuild_content)"
                },
                "analyze_only": {
                    "type": "boolean",
                    "description": "If true, only analyze the ebuild without generating BUCK file",
                    "default": false
                }
            }
        }),
        available,
        reason,
    }
}

fn tool_package_get_examples(exec_context: &ExecutionContext) -> ToolDefinition {
    let (available, reason) = check_availability("package_get_examples", exec_context);

    ToolDefinition {
        name: "package_get_examples".to_string(),
        description: "Get example package definitions and templates from buckos-build. Shows real-world examples of different package types (simple_package, autotools_package, cmake_package, meson_package, cargo_package, go_package, python_package) and macro definitions.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "package_type": {
                    "type": "string",
                    "description": "Package type to get template/example for",
                    "enum": ["simple", "autotools", "cmake", "meson", "cargo", "go", "python"],
                    "default": "simple"
                }
            }
        }),
        available,
        reason,
    }
}
