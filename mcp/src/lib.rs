//! # Buckos MCP Server
//!
//! Model Context Protocol (MCP) server implementation for the Buckos package manager.
//! Enables AI assistants to interact with package management operations through a
//! standardized JSON-RPC 2.0 interface over stdio.
//!
//! ## Architecture
//!
//! - **Protocol Layer**: JSON-RPC 2.0 types and stdio transport
//! - **Permission Layer**: ExecutionContext for detecting root vs non-root
//! - **Server Layer**: Request routing and tool registry
//! - **Handler Layer**: Package manager operation handlers
//!
//! ## Usage
//!
//! ```rust,no_run
//! use buckos_mcp::{McpServer, ServerConfig, ExecutionContext};
//! use buckos_package::{PackageManager, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pm_config = Config::default();
//!     let pm = PackageManager::new(pm_config).await?;
//!
//!     let server_config = ServerConfig::default();
//!     let context = ExecutionContext::detect();
//!
//!     let server = McpServer::new(pm, server_config, context);
//!     server.serve_stdio().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod context;
pub mod error;
pub mod handlers;
pub mod permissions;
pub mod protocol;
pub mod server;

// Re-export main types
pub use context::McpServerContext;
pub use error::{McpError, Result};
pub use permissions::ExecutionContext;
pub use protocol::{JsonRpcRequest, JsonRpcResponse, StdioTransport};
pub use server::{McpServer, ServerConfig};
