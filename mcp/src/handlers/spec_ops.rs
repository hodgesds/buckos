//! Specification validation and query handlers

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use crate::spec_registry::SpecRegistry;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::info;

/// Get the specs path from environment or use default
/// Checks in this order:
/// 1. BUCKOS_SPECS_PATH environment variable
/// 2. /usr/share/buckos/specs (system installation)
/// 3. BUCKOS_BUILD_PATH/specs (development)
/// 4. ../buckos-build/specs (relative fallback)
fn get_specs_path() -> PathBuf {
    // Check for explicit specs path
    if let Ok(specs_path) = std::env::var("BUCKOS_SPECS_PATH") {
        let path = PathBuf::from(specs_path);
        if path.exists() {
            return path;
        }
    }

    // Check system installation location
    let system_path = PathBuf::from("/usr/share/buckos/specs");
    if system_path.exists() {
        return system_path;
    }

    // Check buckos-build repo location
    if let Ok(build_path) = std::env::var("BUCKOS_BUILD_PATH") {
        let path = PathBuf::from(build_path).join("specs");
        if path.exists() {
            return path;
        }
    }

    // Fallback to relative path (for development)
    PathBuf::from("../buckos-build/specs")
}

/// Handle spec_list tool
pub async fn handle_spec_list(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let category = args["category"].as_str();
    let status = args["status"].as_str();

    info!(
        category = category,
        status = status,
        "Listing specifications"
    );

    // Load specification registry
    let specs_path = get_specs_path();
    let registry = SpecRegistry::load(&specs_path)
        .map_err(|e| McpError::Internal(format!("Failed to load spec registry: {}", e)))?;

    // Filter specs
    let specs = registry.list_specs(category, status);

    // Convert to JSON
    let spec_list: Vec<Value> = specs
        .iter()
        .map(|spec| {
            json!({
                "id": spec.id,
                "title": spec.title,
                "status": spec.status,
                "version": spec.version,
                "category": spec.category,
                "description": spec.description,
                "implementation": {
                    "status": spec.implementation.status,
                    "completeness": spec.implementation.completeness
                }
            })
        })
        .collect();

    Ok(json!({
        "specs": spec_list,
        "count": spec_list.len(),
        "total": registry.total_specs
    }))
}

/// Handle spec_info tool
pub async fn handle_spec_info(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let spec_id = args["spec_id"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing 'spec_id' parameter".to_string()))?;

    info!(spec_id = spec_id, "Getting specification info");

    // Load specification registry
    let specs_path = get_specs_path();
    let registry = SpecRegistry::load(&specs_path)
        .map_err(|e| McpError::Internal(format!("Failed to load spec registry: {}", e)))?;

    // Get spec info
    let spec = registry
        .get_spec(spec_id)
        .ok_or_else(|| McpError::Internal(format!("Specification '{}' not found", spec_id)))?;

    Ok(json!({
        "id": spec.id,
        "title": spec.title,
        "status": spec.status,
        "version": spec.version,
        "category": spec.category,
        "path": spec.path,
        "created": spec.created,
        "updated": spec.updated,
        "authors": spec.authors,
        "maintainers": spec.maintainers,
        "tags": spec.tags,
        "related": spec.related,
        "implementation": spec.implementation,
        "compatibility": spec.compatibility,
        "description": spec.description
    }))
}

/// Handle spec_validate_system tool
pub async fn handle_spec_validate_system(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let profile = args["profile"].as_str();
    let check_dependencies = args["check_dependencies"].as_bool().unwrap_or(true);
    let check_use_flags = args["check_use_flags"].as_bool().unwrap_or(true);

    info!(
        profile = profile,
        check_dependencies = check_dependencies,
        check_use_flags = check_use_flags,
        "Validating system against spec"
    );

    // This is a placeholder implementation
    // Full implementation would:
    // 1. Load the profile specification
    // 2. Get installed packages
    // 3. Validate against profile requirements
    // 4. Check dependencies if requested
    // 5. Check USE flags if requested

    let errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut info_msgs: Vec<String> = Vec::new();

    // Check if profile is specified
    if let Some(prof) = profile {
        info_msgs.push(format!("Validating against profile: {}", prof));

        // Placeholder: Get installed packages
        // In real implementation: let installed = ctx.pm.list_installed().await?;

        // Validate package requirements for profile
        // This would check against SPEC-004 package set definitions
        info_msgs.push(format!(
            "Profile '{}' requires validation against @system set",
            prof
        ));
    } else {
        warnings.push("No profile specified, performing basic validation only".to_string());
    }

    // Check dependencies
    if check_dependencies {
        info_msgs.push("Dependency validation enabled".to_string());
        // Placeholder: Would use ctx.pm.resolve_packages() to check for circular deps
    }

    // Check USE flags
    if check_use_flags {
        info_msgs.push("USE flag validation enabled".to_string());
        // Placeholder: Would validate against SPEC-002 USE flag requirements
    }

    // Generate validation report
    Ok(json!({
        "valid": errors.is_empty(),
        "profile": profile,
        "errors": errors,
        "warnings": warnings,
        "info": info_msgs,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle spec_validate_use_flags tool
pub async fn handle_spec_validate_use_flags(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package = args["package"].as_str();
    let use_flags = args["use_flags"].as_array();
    let profile = args["profile"].as_str();

    info!(package = package, profile = profile, "Validating USE flags");

    let mut errors: Vec<String> = Vec::new();
    let warnings: Vec<String> = Vec::new();
    let mut info_msgs: Vec<String> = Vec::new();

    // Placeholder implementation
    // Full implementation would:
    // 1. Load USE flag definitions from SPEC-002
    // 2. Validate flag names are valid (alphanumeric, dash-prefixed negation)
    // 3. Check for conflicts
    // 4. Validate against profile defaults
    // 5. Check package-specific USE flags (iuse)

    if let Some(pkg) = package {
        info_msgs.push(format!("Validating USE flags for package: {}", pkg));

        // Placeholder: Get package info to check IUSE
        // let pkg_info = ctx.pm.info(pkg).await?;
        // if let Some(info) = pkg_info {
        //     // Validate flags against iuse
        // }
    } else {
        info_msgs.push("Validating global USE flags".to_string());
    }

    if let Some(flags) = use_flags {
        info_msgs.push(format!("Checking {} USE flags", flags.len()));

        // Validate each flag format
        for flag in flags {
            if let Some(flag_str) = flag.as_str() {
                // Check format (alphanumeric or dash-prefixed)
                if flag_str.is_empty() {
                    errors.push("Empty USE flag detected".to_string());
                } else if flag_str.starts_with('-') {
                    info_msgs.push(format!("Negation flag: {}", flag_str));
                } else if !flag_str
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                {
                    errors.push(format!("Invalid USE flag format: {}", flag_str));
                }
            }
        }
    }

    if let Some(prof) = profile {
        info_msgs.push(format!("Validating against profile: {}", prof));
        // Placeholder: Would check against profile USE flag defaults
    }

    Ok(json!({
        "valid": errors.is_empty(),
        "package": package,
        "profile": profile,
        "errors": errors,
        "warnings": warnings,
        "info": info_msgs,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle spec_validate_package_set tool
pub async fn handle_spec_validate_package_set(
    _ctx: &McpServerContext,
    args: Value,
) -> Result<Value> {
    let set_name = args["set_name"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing 'set_name' parameter".to_string()))?;
    let check_inheritance = args["check_inheritance"].as_bool().unwrap_or(true);
    let check_packages = args["check_packages"].as_bool().unwrap_or(true);

    info!(
        set_name = set_name,
        check_inheritance = check_inheritance,
        check_packages = check_packages,
        "Validating package set"
    );

    let errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut info_msgs: Vec<String> = Vec::new();

    // Placeholder implementation
    // Full implementation would:
    // 1. Load set definition from SPEC-004
    // 2. Check for circular inheritance
    // 3. Validate all packages exist
    // 4. Check set type consistency

    // Strip @ prefix if present
    let set = set_name.trim_start_matches('@');

    // Check if it's a known set type
    let known_system_profiles = [
        "minimal",
        "server",
        "desktop",
        "developer",
        "hardened",
        "embedded",
        "container",
    ];
    let known_task_sets = [
        "web-server",
        "database-server",
        "container-host",
        "virtualization-host",
        "vpn-server",
        "monitoring",
        "benchmarking",
    ];
    let known_desktop_sets = [
        "gnome-desktop",
        "kde-desktop",
        "xfce-desktop",
        "sway-desktop",
        "hyprland-desktop",
        "i3-desktop",
    ];

    let is_known = known_system_profiles.contains(&set)
        || known_task_sets.contains(&set)
        || known_desktop_sets.contains(&set)
        || set == "system"
        || set == "world"
        || set == "selected";

    if !is_known {
        warnings.push(format!("Unknown package set '{}' - may be custom set", set));
    } else {
        info_msgs.push(format!("Known package set: {}", set));
    }

    if check_inheritance {
        info_msgs.push("Checking inheritance chain".to_string());
        // Placeholder: Would check for circular inheritance
    }

    if check_packages {
        info_msgs.push("Checking package availability".to_string());
        // Placeholder: Would validate all packages in set exist
        // This would use ctx.pm to check each package
    }

    Ok(json!({
        "valid": errors.is_empty(),
        "set_name": set_name,
        "known_set": is_known,
        "errors": errors,
        "warnings": warnings,
        "info": info_msgs,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
