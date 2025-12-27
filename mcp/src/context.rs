//! MCP server context
//!
//! Provides the execution context for MCP tool handlers, including access to
//! the PackageManager and permission checking.

use crate::error::{McpError, Result};
use crate::permissions::ExecutionContext;
use crate::server::confirmation::{ConfirmationToken, PendingOperation};
use buckos_package::PackageManager;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// MCP server context
///
/// Holds the PackageManager instance and manages server state including
/// confirmation tokens.
pub struct McpServerContext {
    /// Package manager instance
    pub pm: Arc<PackageManager>,

    /// Execution context (permissions, user mode, etc.)
    pub exec_context: ExecutionContext,

    /// Pending confirmation tokens
    confirmations: Arc<RwLock<HashMap<String, ConfirmationToken>>>,

    /// Confirmation token TTL (default: 5 minutes)
    confirmation_ttl: Duration,
}

impl McpServerContext {
    /// Create a new server context
    pub fn new(pm: PackageManager, exec_context: ExecutionContext) -> Self {
        Self {
            pm: Arc::new(pm),
            exec_context,
            confirmations: Arc::new(RwLock::new(HashMap::new())),
            confirmation_ttl: Duration::minutes(5),
        }
    }

    /// Create a confirmation token for a pending operation
    pub async fn create_confirmation(
        &self,
        operation: PendingOperation,
    ) -> Result<ConfirmationToken> {
        let token = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let expires_at = created_at + self.confirmation_ttl;

        let confirmation = ConfirmationToken {
            token: token.clone(),
            operation,
            created_at,
            expires_at,
        };

        let mut confirmations = self.confirmations.write().await;
        confirmations.insert(token.clone(), confirmation.clone());

        // Clean up expired tokens
        self.cleanup_expired_tokens(&mut confirmations).await;

        Ok(confirmation)
    }

    /// Consume a confirmation token and return the pending operation
    pub async fn consume_confirmation(&self, token: &str) -> Result<PendingOperation> {
        let mut confirmations = self.confirmations.write().await;

        let confirmation = confirmations
            .remove(token)
            .ok_or_else(|| McpError::InvalidToken("Token not found".to_string()))?;

        // Check if expired
        if Utc::now() > confirmation.expires_at {
            return Err(McpError::InvalidToken("Token expired".to_string()));
        }

        Ok(confirmation.operation)
    }

    /// Clean up expired confirmation tokens
    async fn cleanup_expired_tokens(&self, confirmations: &mut HashMap<String, ConfirmationToken>) {
        let now = Utc::now();
        confirmations.retain(|_, token| token.expires_at > now);
    }

    /// Check if current user can perform an operation
    pub fn check_permission(&self, operation: &str) -> Result<()> {
        let (available, reason) = self.exec_context.tool_available(operation);

        if !available {
            return Err(McpError::Permission(reason.unwrap_or_else(|| {
                format!("Operation '{}' not available", operation)
            })));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buckos_package::Config;

    async fn create_test_context() -> McpServerContext {
        let config = Config::default();
        let pm = PackageManager::new(config).await.unwrap();
        let exec_context = ExecutionContext::detect();
        McpServerContext::new(pm, exec_context)
    }

    #[tokio::test]
    async fn test_confirmation_token_creation() {
        let ctx = create_test_context().await;

        let op = PendingOperation::Install {
            packages: vec!["bash".to_string()],
        };

        let token = ctx.create_confirmation(op).await.unwrap();
        assert!(!token.token.is_empty());
        assert!(token.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_confirmation_token_consumption() {
        let ctx = create_test_context().await;

        let op = PendingOperation::Install {
            packages: vec!["bash".to_string()],
        };

        let token = ctx.create_confirmation(op).await.unwrap();
        let token_str = token.token.clone();

        // Should be able to consume once
        let consumed = ctx.consume_confirmation(&token_str).await;
        assert!(consumed.is_ok());

        // Should not be able to consume again
        let consumed_again = ctx.consume_confirmation(&token_str).await;
        assert!(consumed_again.is_err());
    }

    #[tokio::test]
    async fn test_check_permission() {
        let ctx = create_test_context().await;

        // Read-only operations should always be allowed
        assert!(ctx.check_permission("package_search").is_ok());

        // Install may or may not be allowed depending on if running as root
        let install_result = ctx.check_permission("package_install");
        if ctx.exec_context.is_root {
            assert!(install_result.is_ok());
        }
    }
}
