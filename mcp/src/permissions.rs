//! Permission detection and context awareness
//!
//! Detects the execution context (root vs non-root) and provides
//! context-aware capabilities reporting.

use std::env;
use std::path::PathBuf;
use tracing::info;

/// Execution context for the MCP server
///
/// Detects whether the server is running as root and determines
/// what operations are available.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Running as root (UID 0)
    pub is_root: bool,

    /// Effective user ID
    pub effective_uid: u32,

    /// Can perform system-wide installations
    pub can_install: bool,

    /// Root directory for installations
    pub install_root: PathBuf,

    /// User mode enabled (install to ~/.local)
    pub user_mode: bool,
}

impl ExecutionContext {
    /// Detect execution context from the environment
    pub fn detect() -> Self {
        let effective_uid = unsafe { libc::geteuid() };
        let is_root = effective_uid == 0;

        let install_root = if is_root {
            PathBuf::from("/")
        } else {
            // Non-root: would install to ~/.local
            env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
                .join(".local")
        };

        let context = Self {
            is_root,
            effective_uid,
            can_install: is_root,
            install_root,
            user_mode: false,
        };

        info!(
            is_root = context.is_root,
            uid = context.effective_uid,
            can_install = context.can_install,
            install_root = ?context.install_root,
            "Detected execution context"
        );

        context
    }

    /// Enable user mode (install to ~/.local without root)
    pub fn enable_user_mode(&mut self) {
        if !self.is_root {
            self.user_mode = true;
            self.can_install = true;
            info!("User mode enabled: installations will go to ~/.local");
        }
    }

    /// Check if a tool is available in the current context
    pub fn tool_available(&self, tool: &str) -> (bool, Option<String>) {
        match tool {
            // Read-only tools: always available
            "package_search" | "package_info" | "package_list" | "package_deps" | "config_show" => {
                (true, None)
            }

            // Mutating tools: require root or user mode
            "package_install" => {
                if self.can_install {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "Requires root privileges. Current user: UID {}\n\
                            \nSolutions:\n\
                            - Restart with: sudo buckos mcp\n\
                            - Or use: buckos mcp --user-mode (installs to ~/.local)\n\
                            - Or use read-only tools for package exploration",
                            self.effective_uid
                        )),
                    )
                }
            }

            // Unknown tool
            _ => (false, Some(format!("Unknown tool: {}", tool))),
        }
    }

    /// Get a description of the current context for users
    pub fn description(&self) -> String {
        if self.is_root {
            "Running as root: all operations available".to_string()
        } else if self.user_mode {
            format!(
                "Running in user mode: installations to {} (UID {})",
                self.install_root.display(),
                self.effective_uid
            )
        } else {
            format!(
                "Running as non-root user (UID {}): only read-only operations available.\n\
                Restart with 'sudo buckos mcp' for full functionality.",
                self.effective_uid
            )
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_detect() {
        let ctx = ExecutionContext::detect();

        // Root status should match UID 0
        assert_eq!(ctx.is_root, ctx.effective_uid == 0);

        // can_install should match is_root (until user_mode is enabled)
        if !ctx.user_mode {
            assert_eq!(ctx.can_install, ctx.is_root);
        }
    }

    #[test]
    fn test_tool_availability() {
        let ctx = ExecutionContext::detect();

        // Read-only tools should always be available
        let (available, reason) = ctx.tool_available("package_search");
        assert!(available);
        assert!(reason.is_none());

        let (available, _) = ctx.tool_available("package_info");
        assert!(available);
    }

    #[test]
    fn test_user_mode() {
        let mut ctx = ExecutionContext::detect();

        if !ctx.is_root {
            assert!(!ctx.can_install);

            ctx.enable_user_mode();
            assert!(ctx.user_mode);
            assert!(ctx.can_install);
        }
    }

    #[test]
    fn test_description() {
        let ctx = ExecutionContext::detect();
        let desc = ctx.description();

        // Description should contain some useful information
        assert!(!desc.is_empty());

        if ctx.is_root {
            assert!(desc.contains("root"));
        } else {
            assert!(desc.contains("UID"));
        }
    }
}
