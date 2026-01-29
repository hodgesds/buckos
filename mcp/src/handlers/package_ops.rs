//! Package operation handlers

use crate::context::McpServerContext;
use crate::error::{McpError, Result};
use crate::server::confirmation::PendingOperation;
use serde_json::{json, Value};
use tracing::info;

/// Handle package_search tool
pub async fn handle_search(ctx: &McpServerContext, args: Value) -> Result<Value> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing 'query' parameter".to_string()))?;

    info!(query = query, "Searching for packages");

    // Search packages
    let results = ctx.pm.search(query).await?;

    // Convert to JSON
    let packages: Vec<Value> = results
        .iter()
        .map(|pkg| {
            json!({
                "name": pkg.id.to_string(),
                "version": pkg.version,
                "description": pkg.description,
                "category": pkg.id.category,
                "package": pkg.id.name
            })
        })
        .collect();

    Ok(json!({
        "packages": packages,
        "count": packages.len()
    }))
}

/// Handle package_info tool
pub async fn handle_info(ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package = args["package"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing 'package' parameter".to_string()))?;

    info!(package = package, "Getting package info");

    // Get package info
    let info = ctx.pm.info(package).await?.ok_or_else(|| {
        McpError::PackageManager(buckos_package::Error::PackageNotFound(package.to_string()))
    })?;

    Ok(json!({
        "name": info.id.to_string(),
        "version": info.version,
        "description": info.description,
        "category": info.id.category,
        "package": info.id.name,
        "homepage": info.homepage,
        "license": info.license,
        "use_flags": info.use_flags,
        "slot": info.slot
    }))
}

/// Handle package_list tool
pub async fn handle_list(ctx: &McpServerContext, args: Value) -> Result<Value> {
    let filter = args["filter"].as_str().unwrap_or("installed");

    info!(filter = filter, "Listing packages");

    match filter {
        "installed" => {
            let packages = ctx.pm.list_installed().await?;

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

            Ok(json!({
                "packages": result,
                "count": result.len(),
                "filter": "installed"
            }))
        }
        _ => Err(McpError::InvalidParams(format!(
            "Unknown filter: {}",
            filter
        ))),
    }
}

/// Handle package_deps tool
pub async fn handle_deps(ctx: &McpServerContext, args: Value) -> Result<Value> {
    let package = args["package"]
        .as_str()
        .ok_or_else(|| McpError::InvalidParams("Missing 'package' parameter".to_string()))?;

    info!(package = package, "Getting dependencies");

    // Get dependencies by using resolve_packages
    use buckos_package::InstallOptions;
    let opts = InstallOptions::default();
    let resolution = ctx
        .pm
        .resolve_packages(&[package.to_string()], &opts)
        .await?;

    let dep_list: Vec<Value> = resolution
        .packages
        .iter()
        .filter(|p| p.id.to_string() != package) // Exclude the package itself
        .map(|dep| {
            json!({
                "package": dep.id.to_string(),
                "version": dep.version
            })
        })
        .collect();

    Ok(json!({
        "package": package,
        "dependencies": dep_list,
        "count": dep_list.len()
    }))
}

/// Handle package_install tool
pub async fn handle_install(ctx: &McpServerContext, args: Value) -> Result<Value> {
    let packages: Vec<String> = args["packages"]
        .as_array()
        .ok_or_else(|| McpError::InvalidParams("Missing 'packages' parameter".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    if packages.is_empty() {
        return Err(McpError::InvalidParams(
            "At least one package required".to_string(),
        ));
    }

    let dry_run = args["dry_run"].as_bool().unwrap_or(true);
    let confirmation_token = args["confirmation_token"].as_str();

    if dry_run {
        // Phase 1: Dry-run - show what would be installed
        info!(packages = ?packages, "Dry-run install");

        // Check permissions
        ctx.check_permission("package_install")?;

        // Resolve dependencies
        use buckos_package::InstallOptions;
        let opts = InstallOptions::default();
        let resolution = ctx.pm.resolve_packages(&packages, &opts).await?;

        // Create confirmation token
        let token = ctx
            .create_confirmation(PendingOperation::Install {
                packages: packages.clone(),
            })
            .await?;

        Ok(json!({
            "dry_run": true,
            "packages": packages,
            "to_install": resolution.packages.iter().map(|p| {
                json!({
                    "name": p.id.to_string(),
                    "version": p.version
                })
            }).collect::<Vec<_>>(),
            "confirmation_token": token.token,
            "expires_at": token.expires_at.to_rfc3339(),
            "message": format!(
                "This will install {} package(s). Use the confirmation_token to proceed.",
                resolution.packages.len()
            )
        }))
    } else if let Some(token) = confirmation_token {
        // Phase 2: Execute with confirmation token
        info!(token = token, "Executing confirmed install");

        // Check permissions
        ctx.check_permission("package_install")?;

        // Consume token and get operation
        let operation = ctx.consume_confirmation(token).await?;

        match operation {
            PendingOperation::Install { packages } => {
                // Execute installation
                use buckos_package::InstallOptions;
                let opts = InstallOptions::default();
                ctx.pm.install(&packages, opts).await?;

                Ok(json!({
                    "success": true,
                    "installed": packages,
                    "message": format!("Successfully installed {} package(s)", packages.len())
                }))
            }
        }
    } else {
        Err(McpError::InvalidParams(
            "Either dry_run=true or confirmation_token required".to_string(),
        ))
    }
}

/// Handle config_show tool
pub async fn handle_config_show(_ctx: &McpServerContext, args: Value) -> Result<Value> {
    info!("Showing configuration");

    let section = args["section"].as_str();

    // Load system configuration
    let config = buckos_config::load_system_config()
        .map_err(|e| McpError::Internal(format!("Failed to load configuration: {}", e)))?;

    match section {
        Some("make_conf") | Some("make.conf") => {
            Ok(json!({
                "section": "make.conf",
                "cflags": config.make_conf.cflags,
                "cxxflags": config.make_conf.cxxflags,
                "chost": config.make_conf.chost,
                "use_flags": config.make_conf.use_config.global,
                "features": config.make_conf.features.enabled,
                "makeopts": config.make_conf.makeopts,
            }))
        }
        Some("use") | Some("use_flags") => {
            Ok(json!({
                "section": "use_flags",
                "global": config.make_conf.use_config.global,
                "use_expand": config.make_conf.use_config.expand,
            }))
        }
        Some("features") => {
            Ok(json!({
                "section": "features",
                "enabled": config.make_conf.features.enabled,
                "disabled": config.make_conf.features.disabled,
            }))
        }
        Some("repos") | Some("repositories") => {
            let repos: Vec<_> = config.repos.repos.iter().map(|(name, repo)| {
                json!({
                    "name": name,
                    "location": repo.location.to_string_lossy(),
                    "sync_type": format!("{:?}", repo.sync_type),
                    "priority": repo.priority,
                })
            }).collect();
            Ok(json!({
                "section": "repositories",
                "repos": repos,
                "count": repos.len(),
            }))
        }
        Some("profile") => {
            Ok(json!({
                "section": "profile",
                "current": config.profile.current,
            }))
        }
        None | Some("all") => {
            // Return overview of all configuration sections
            Ok(json!({
                "section": "overview",
                "make_conf": {
                    "cflags": config.make_conf.cflags,
                    "chost": config.make_conf.chost,
                    "use_count": config.make_conf.use_config.global.len(),
                    "features_count": config.make_conf.features.enabled.len(),
                },
                "repos": {
                    "count": config.repos.repos.len(),
                    "names": config.repos.repos.keys().collect::<Vec<_>>(),
                },
                "profile": config.profile.current,
                "available_sections": ["make_conf", "use", "features", "repos", "profile"],
            }))
        }
        Some(other) => {
            Err(McpError::InvalidParams(format!(
                "Unknown configuration section: '{}'. Available: make_conf, use, features, repos, profile",
                other
            )))
        }
    }
}
