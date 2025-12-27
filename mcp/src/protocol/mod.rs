//! MCP protocol implementation
//!
//! This module provides the core protocol types and transport layer for
//! Model Context Protocol (MCP) communication using JSON-RPC 2.0.

pub mod jsonrpc;
pub mod transport;

pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId};
pub use transport::StdioTransport;
