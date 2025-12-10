//! IPC command handlers and coordination

use crate::{Result, IpcCommand, IpcResponse, IpcManager, CommandHandler};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Command handler registry and coordinator
pub struct HandlerRegistry {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn CommandHandler>>>>,
}

impl HandlerRegistry {
    /// Create a new handler registry
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a command handler
    pub async fn register_handler(&self, command_name: &str, handler: Arc<dyn CommandHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(command_name.to_string(), handler);
    }

    /// Get a command handler
    pub async fn get_handler(&self, command_name: &str) -> Option<Arc<dyn CommandHandler>> {
        let handlers = self.handlers.read().await;
        handlers.get(command_name).cloned()
    }

    /// List all registered commands
    pub async fn list_commands(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    /// Execute a command
    pub async fn execute_command(&self, command: IpcCommand) -> Result<IpcResponse> {
        let start_time = std::time::Instant::now();

        let handlers = self.handlers.read().await;

        match handlers.get(&command.name) {
            Some(handler) => {
                let mut response = handler.execute(command)?;
                response.execution_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(response)
            }
            None => Ok(IpcResponse::error(
                command.id,
                format!("Unknown command: {}", command.name)
            )),
        }
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Async command handler wrapper
pub struct AsyncCommandHandler<F, Fut>
where
    F: Fn(IpcCommand) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<IpcResponse>> + Send,
{
    handler_fn: F,
}

impl<F, Fut> AsyncCommandHandler<F, Fut>
where
    F: Fn(IpcCommand) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<IpcResponse>> + Send,
{
    /// Create a new async command handler
    pub fn new(handler_fn: F) -> Self {
        Self { handler_fn }
    }
}

impl<F, Fut> CommandHandler for AsyncCommandHandler<F, Fut>
where
    F: Fn(IpcCommand) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<IpcResponse>> + Send,
{
    fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
        // For now, return a synchronous placeholder
        // In a real implementation, you'd need to handle async execution differently
        Err(crate::CziError::ipc("Async handler execution not implemented in sync context".to_string()))
    }
}

/// Helper for creating async handlers
pub fn create_async_handler<F, Fut>(handler_fn: F) -> AsyncCommandHandler<F, Fut>
where
    F: Fn(IpcCommand) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<IpcResponse>> + Send,
{
    AsyncCommandHandler::new(handler_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::BaseCommandHandler;
    use serde_json::Value;

    #[tokio::test]
    async fn test_handler_registry() {
        let registry = HandlerRegistry::new();

        // Initially no commands
        let commands = registry.list_commands().await;
        assert_eq!(commands.len(), 0);

        // Register a mock handler
        let mock_handler = Arc::new(MockHandler::new());
        registry.register_handler("test_command", mock_handler).await;

        // Should have one command
        let commands = registry.list_commands().await;
        assert_eq!(commands.len(), 1);
        assert!(commands.contains(&"test_command".to_string()));

        // Should be able to get the handler
        let handler = registry.get_handler("test_command").await;
        assert!(handler.is_some());

        // Non-existent command should return None
        let handler = registry.get_handler("non_existent").await;
        assert!(handler.is_none());
    }

    #[tokio::test]
    async fn test_command_execution() {
        let registry = HandlerRegistry::new();

        let mock_handler = Arc::new(MockHandler::new());
        registry.register_handler("test", mock_handler).await;

        let command = crate::IpcCommand {
            id: "test-123".to_string(),
            name: "test".to_string(),
            parameters: None,
            timestamp: chrono::Utc::now(),
        };

        let response = registry.execute_command(command).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.command_id, "test-123");
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_unknown_command() {
        let registry = HandlerRegistry::new();

        let command = crate::IpcCommand {
            id: "test-123".to_string(),
            name: "unknown_command".to_string(),
            parameters: None,
            timestamp: chrono::Utc::now(),
        };

        let response = registry.execute_command(command).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(!response.success);
        assert!(response.error.unwrap().contains("Unknown command"));
    }

    // Mock handler for testing
    struct MockHandler {
        base: BaseCommandHandler,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                base: BaseCommandHandler,
            }
        }
    }

    impl CommandHandler for MockHandler {
        fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
            Ok(self.base.success_response(command.id, Some(Value::Object(serde_json::json!({
                "result": "test_success"
            })))))
        }
    }
}