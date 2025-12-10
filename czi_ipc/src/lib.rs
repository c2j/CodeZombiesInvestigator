//! CodeZombiesInvestigator IPC Layer
//!
//! This crate provides the IPC interface for communicating between the
//! frontend and backend of the CZI desktop application.

pub mod commands;
pub mod types;
pub mod handlers;
pub mod transport;

// Re-export common types from core
pub use czi_core::CziError;
pub use czi_core::Result as CoreResult;

// Re-export IPC types
pub use types::{IpcCommand, IpcResponse, IpcEvent, CommandStatus};

use crate::CoreResult as Result;
use std::collections::HashMap;

/// Main IPC manager for command routing and handling
pub struct IpcManager {
    /// Command handlers registry
    handlers: HashMap<String, Box<dyn CommandHandler>>,
    /// Transport layer
    transport: Box<dyn IpcTransport>,
}

/// IPC command trait
pub trait CommandHandler: Send + Sync {
    /// Execute the command
    fn execute(&self, command: IpcCommand) -> Result<IpcResponse>;
}

/// IPC transport trait for different communication mechanisms
pub trait IpcTransport: Send + Sync {
    /// Send a response
    fn send_response(&self, response: IpcResponse) -> Result<()>;

    /// Receive a command
    fn receive_command(&self) -> Result<IpcCommand>;
}

impl IpcManager {
    /// Create a new IPC manager
    pub fn new(transport: Box<dyn IpcTransport>) -> Self {
        Self {
            handlers: HashMap::new(),
            transport,
        }
    }

    /// Register a command handler
    pub fn register_handler(&mut self, command_name: &str, handler: Box<dyn CommandHandler>) {
        self.handlers.insert(command_name.to_string(), handler);
    }

    /// Process an incoming command
    pub fn process_command(&self, command: IpcCommand) -> IpcResponse {
        let start_time = std::time::Instant::now();
        let command_id = command.id.clone();

        let response = match self.handlers.get(&command.name) {
            Some(handler) => {
                match handler.execute(command) {
                    Ok(result) => result,
                    Err(e) => IpcResponse {
                        command_id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                    },
                }
            }
            None => IpcResponse {
                command_id,
                success: false,
                data: None,
                error: Some(format!("Unknown command: {}", command.name)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            },
        };

        // Send response through transport
        if let Err(e) = self.transport.send_response(response.clone()) {
            tracing::error!("Failed to send IPC response: {}", e);
        }

        response
    }

    /// Get registered command names
    pub fn get_command_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Check if a command is registered
    pub fn has_command(&self, command_name: &str) -> bool {
        self.handlers.contains_key(command_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_manager_creation() {
        let transport = Box::new(MockTransport::new());
        let manager = IpcManager::new(transport);
        assert_eq!(manager.get_command_names().len(), 0);
    }

    #[test]
    fn test_command_registration() {
        let transport = Box::new(MockTransport::new());
        let mut manager = IpcManager::new(transport);

        manager.register_handler("test", Box::new(MockHandler::new()));

        assert!(manager.has_command("test"));
        assert_eq!(manager.get_command_names().len(), 1);
    }

    // Mock implementations for testing
    struct MockTransport {
        responses: Vec<IpcResponse>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                responses: Vec::new(),
            }
        }
    }

    impl IpcTransport for MockTransport {
        fn send_response(&self, _response: IpcResponse) -> Result<()> {
            // Mock implementation
            Ok(())
        }

        fn receive_command(&self) -> Result<IpcCommand> {
            // Mock implementation
            Err(crate::CziError::ipc("Not implemented".to_string()))
        }
    }

    struct MockHandler;

    impl MockHandler {
        fn new() -> Self {
            Self
        }
    }

    impl CommandHandler for MockHandler {
        fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
            Ok(IpcResponse {
                command_id: command.id,
                success: true,
                data: Some(serde_json::json!({"result": "ok"})),
                error: None,
                execution_time_ms: 0,
            })
        }
    }
}