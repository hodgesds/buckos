// Package creation handlers for MCP server

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Get the templates path from environment or use default
/// Checks in this order:
/// 1. BUCKOS_TEMPLATES_PATH environment variable
/// 2. BUCKOS_SPECS_PATH/templates (if specs path is set)
/// 3. /usr/share/buckos/specs/templates (system installation)
/// 4. BUCKOS_BUILD_PATH/specs/templates (development)
/// 5. ../buckos-build/specs/templates (relative fallback)
fn get_templates_path() -> PathBuf {
    // Check for explicit templates path
    if let Ok(templates_path) = std::env::var("BUCKOS_TEMPLATES_PATH") {
        let path = PathBuf::from(templates_path);
        if path.exists() {
            return path;
        }
    }

    // Check if specs path is set and has templates subdir
    if let Ok(specs_path) = std::env::var("BUCKOS_SPECS_PATH") {
        let path = PathBuf::from(specs_path).join("templates");
        if path.exists() {
            return path;
        }
    }

    // Check system installation location
    let system_path = PathBuf::from("/usr/share/buckos/specs/templates");
    if system_path.exists() {
        return system_path;
    }

    // Check buckos-build repo location
    if let Ok(build_path) = std::env::var("BUCKOS_BUILD_PATH") {
        let path = PathBuf::from(build_path).join("specs/templates");
        if path.exists() {
            return path;
        }
    }

    // Fallback to relative path (for development)
    PathBuf::from("../buckos-build/specs/templates")
}

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

    // Read template file from templates directory
    let templates_path = get_templates_path();
    let template_path = templates_path.join(format!("{}-package-template.bzl", package_type));

    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| McpError::Internal(format!(
            "Failed to read template '{}' from {}: {}. Set BUCKOS_TEMPLATES_PATH or BUCKOS_BUILD_PATH environment variable.",
            package_type,
            template_path.display(),
            e
        )))?;

    // Basic substitutions
    let result = template
        .replace("PACKAGE_NAME", name)
        .replace("VERSION", version);

    Ok(json!({
        "template": result,
        "template_type": package_type,
        "path": template_path.to_string_lossy(),
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

    let templates_path = get_templates_path();

    // Map package types to template filenames
    let template_files: Vec<&str> = match package_type {
        "simple" => vec!["simple-package-template.bzl"],
        "autotools" => vec!["autotools-package-template.bzl"],
        "cmake" => vec!["cmake-package-template.bzl"],
        "meson" => vec!["meson-package-template.bzl"],
        "cargo" => vec!["cargo-package-template.bzl"],
        "go" => vec!["go-package-template.bzl"],
        "python" => vec!["python-package-template.bzl"],
        "all" => vec![
            "simple-package-template.bzl",
            "autotools-package-template.bzl",
            "cmake-package-template.bzl",
            "meson-package-template.bzl",
            "cargo-package-template.bzl",
            "go-package-template.bzl",
            "python-package-template.bzl",
        ],
        _ => vec![],
    };

    // Read example files
    let mut example_contents = Vec::new();
    for filename in template_files {
        let path = templates_path.join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            example_contents.push(json!({
                "path": path.to_string_lossy(),
                "filename": filename,
                "content": content,
            }));
        }
    }

    Ok(json!({
        "package_type": package_type,
        "templates_path": templates_path.to_string_lossy(),
        "examples": example_contents,
        "count": example_contents.len(),
    }))
}
