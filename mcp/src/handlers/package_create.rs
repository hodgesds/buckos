// Package creation handlers for MCP server

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use serde_json::{json, Value};

/// Handle package_create_template tool call
pub async fn handle_create_template(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package_type = args.get("package_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("package_type is required".to_string()))?;

    let name = args.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("PACKAGE_NAME");

    let version = args.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("VERSION");

    // Read template file from specs/templates/
    let template_path = format!(
        "/home/daniel/git/buckos-build/specs/templates/{}-package-template.bzl",
        package_type
    );

    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| McpError::Internal(format!("Failed to read template: {}", e)))?;

    // Basic substitutions
    let result = template
        .replace("PACKAGE_NAME", name)
        .replace("VERSION", version);

    Ok(json!({
        "template": result,
        "template_type": package_type,
        "path": template_path,
        "description": format!("Template for {} package: {}-{}", package_type, name, version)
    }))
}

/// Handle package_validate_definition tool call
pub async fn handle_validate_definition(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let definition = args.get("definition")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("definition is required".to_string()))?;

    // TODO: Implement actual validation against specs
    // For now, just check basic syntax

    let has_name = definition.contains("name =");
    let has_version = definition.contains("version =");
    let has_src_uri = definition.contains("src_uri =");
    let has_sha256 = definition.contains("sha256 =");

    let required_fields_present = has_name && has_version && has_src_uri && has_sha256;

    Ok(json!({
        "valid": required_fields_present,
        "errors": if !required_fields_present {
            let mut errors = Vec::new();
            if !has_name { errors.push("Missing required field: name".to_string()); }
            if !has_version { errors.push("Missing required field: version".to_string()); }
            if !has_src_uri { errors.push("Missing required field: src_uri".to_string()); }
            if !has_sha256 { errors.push("Missing required field: sha256".to_string()); }
            errors
        } else {
            Vec::<String>::new()
        },
        "warnings": Vec::<String>::new(),
    }))
}

/// Handle package_suggest_dependencies tool call
pub async fn handle_suggest_dependencies(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package_type = args.get("package_type")
        .and_then(|v| v.as_str())
        .unwrap_or("simple");

    let _build_system = args.get("build_system")
        .and_then(|v| v.as_str());

    // Provide common dependency suggestions based on package type
    let suggestions = match package_type {
        "autotools" | "simple" => vec![
            "//packages/linux/dev-util:pkg-config",
            "//packages/linux/sys-devel:autoconf",
            "//packages/linux/sys-devel:automake",
        ],
        "cmake" => vec![
            "//packages/linux/dev-util:cmake",
            "//packages/linux/dev-util:pkg-config",
        ],
        "meson" => vec![
            "//packages/linux/dev-util:meson",
            "//packages/linux/dev-util:ninja",
            "//packages/linux/dev-util:pkg-config",
        ],
        "cargo" => vec![
            "//packages/linux/dev-lang:rust",
        ],
        "go" => vec![
            "//packages/linux/dev-lang:go",
        ],
        "python" => vec![
            "//packages/linux/dev-lang:python",
            "//packages/linux/dev-python:setuptools",
        ],
        _ => vec![],
    };

    Ok(json!({
        "package_type": package_type,
        "suggested_build_deps": suggestions,
        "common_runtime_deps": [
            "//packages/linux/core:glibc",
        ],
    }))
}

/// Handle package_suggest_use_flags tool call
pub async fn handle_suggest_use_flags(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package_type = args.get("package_type")
        .and_then(|v| v.as_str())
        .unwrap_or("simple");

    let _package_name = args.get("package_name")
        .and_then(|v| v.as_str());

    // Provide common USE flag suggestions
    let suggestions = match package_type {
        "autotools" | "simple" => vec![
            ("ssl", "Enable SSL/TLS support"),
            ("doc", "Build and install documentation"),
            ("nls", "Native Language Support (i18n/l10n)"),
            ("ipv6", "Enable IPv6 support"),
            ("test", "Run test suite"),
        ],
        "cmake" | "meson" => vec![
            ("doc", "Build and install documentation"),
            ("test", "Run test suite"),
            ("examples", "Build example programs"),
        ],
        "cargo" => vec![
            ("default", "Enable default features"),
        ],
        "go" => vec![
            ("netgo", "Pure Go networking (no CGO)"),
        ],
        "python" => vec![
            ("test", "Run test suite"),
        ],
        _ => vec![],
    };

    Ok(json!({
        "package_type": package_type,
        "suggested_use_flags": suggestions.iter().map(|(flag, desc)| {
            json!({
                "name": flag,
                "description": desc,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// Handle package_convert_ebuild tool call
pub async fn handle_convert_ebuild(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let _ebuild_content = args.get("ebuild_content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams("ebuild_content is required".to_string()))?;

    // TODO: Implement actual ebuild parsing and conversion
    // This is a placeholder

    Ok(json!({
        "success": false,
        "message": "Ebuild conversion not yet implemented - use package templates instead",
        "recommendation": "Use package_create_template to generate a BuckOS package definition",
    }))
}

/// Handle package_get_examples tool call
pub async fn handle_get_examples(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package_type = args.get("package_type")
        .and_then(|v| v.as_str())
        .unwrap_or("simple");

    // Map package types to example files
    let examples = match package_type {
        "simple" => vec![
            "/home/daniel/git/buckos-build/specs/templates/simple-package-template.bzl",
        ],
        "autotools" => vec![
            "/home/daniel/git/buckos-build/specs/templates/autotools-package-template.bzl",
        ],
        "cmake" => vec![
            "/home/daniel/git/buckos-build/specs/templates/cmake-package-template.bzl",
        ],
        "meson" => vec![
            "/home/daniel/git/buckos-build/specs/templates/meson-package-template.bzl",
        ],
        "cargo" => vec![
            "/home/daniel/git/buckos-build/specs/templates/cargo-package-template.bzl",
        ],
        "go" => vec![
            "/home/daniel/git/buckos-build/specs/templates/go-package-template.bzl",
        ],
        "python" => vec![
            "/home/daniel/git/buckos-build/specs/templates/python-package-template.bzl",
        ],
        _ => vec![],
    };

    // Read example files
    let mut example_contents = Vec::new();
    for path in examples {
        if let Ok(content) = std::fs::read_to_string(path) {
            example_contents.push(json!({
                "path": path,
                "content": content,
            }));
        }
    }

    Ok(json!({
        "package_type": package_type,
        "examples": example_contents,
    }))
}
