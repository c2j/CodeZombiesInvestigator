//! IPC transport layer implementations

use crate::{Result, IpcCommand, IpcResponse, IpcTransport};
use std::sync::mpsc::{self, Receiver, Sender};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// In-memory transport for testing and local communication
pub struct InMemoryTransport {
    /// Channel for sending commands
    command_sender: Sender<IpcCommand>,
    /// Channel for receiving commands
    command_receiver: Arc<Mutex<Receiver<IpcCommand>>>,
    /// Channel for sending responses
    response_sender: Arc<Mutex<VecDeque<IpcResponse>>>,
    /// Channel for receiving responses
    response_receiver: Arc<Mutex<VecDeque<IpcResponse>>>,
}

impl InMemoryTransport {
    /// Create a new in-memory transport
    pub fn new() -> (Self, Self) {
        // Create channels for commands
        let (command_tx, command_rx) = mpsc::channel();
        let command_receiver = Arc::new(Mutex::new(command_rx));

        // Create channels for responses
        let response_storage = Arc::new(Mutex::new(VecDeque::new()));

        let transport1 = Self {
            command_sender: command_tx,
            command_receiver: Arc::clone(&command_receiver),
            response_sender: Arc::clone(&response_storage),
            response_receiver: Arc::clone(&response_storage),
        };

        let transport2 = Self {
            command_sender: mpsc::channel().0, // Dummy sender
            command_receiver: Arc::clone(&command_receiver),
            response_sender: Arc::clone(&response_storage),
            response_receiver: Arc::clone(&response_storage),
        };

        (transport1, transport2)
    }

    /// Try to receive a command without blocking
    pub fn try_receive_command(&self) -> Option<IpcCommand> {
        self.command_receiver.lock().unwrap().try_recv().ok()
    }

    /// Try to receive a response without blocking
    pub fn try_receive_response(&self) -> Option<IpcResponse> {
        self.response_receiver.lock().unwrap().pop_front()
    }

    /// Check if there are pending commands
    pub fn has_pending_commands(&self) -> bool {
        match self.command_receiver.lock().unwrap().try_recv() {
            Ok(command) => {
                // Put it back since we just wanted to check
                drop(self.command_receiver.lock().unwrap().send(command));
                true
            }
            Err(_) => false,
        }
    }

    /// Check if there are pending responses
    pub fn has_pending_responses(&self) -> bool {
        !self.response_receiver.lock().unwrap().is_empty()
    }

    /// Get the number of pending commands
    pub fn pending_command_count(&self) -> usize {
        let receiver = self.command_receiver.lock().unwrap();
        match receiver.try_recv() {
            Ok(command) => {
                // Put it back
                drop(receiver.send(command));
                1
            }
            Err(_) => 0,
        }
    }
}

impl IpcTransport for InMemoryTransport {
    fn send_response(&self, response: IpcResponse) -> Result<()> {
        self.response_sender.lock().unwrap().push_back(response);
        Ok(())
    }

    fn receive_command(&self) -> Result<IpcCommand> {
        match self.command_receiver.lock().unwrap().recv() {
            Ok(command) => Ok(command),
            Err(_) => Err(crate::CziError::ipc("No command available".to_string())),
        }
    }
}

/// Channel-based transport for async communication
pub struct ChannelTransport {
    /// Command sender
    command_sender: Arc<Mutex<Option<mpsc::UnboundedSender<IpcCommand>>>>,
    /// Command receiver
    command_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<IpcCommand>>>>,
    /// Response sender
    response_sender: Arc<Mutex<Option<mpsc::UnboundedSender<IpcResponse>>>>,
    /// Response receiver
    response_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<IpcResponse>>>>,
}

impl ChannelTransport {
    /// Create a new channel transport pair
    pub fn new() -> (Self, Self) {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        let transport1 = Self {
            command_sender: Arc::new(Mutex::new(Some(command_tx))),
            command_receiver: Arc::new(Mutex::new(Some(command_rx))),
            response_sender: Arc::new(Mutex::new(Some(response_tx))),
            response_receiver: Arc::new(Mutex::new(Some(response_rx))),
        };

        let transport2 = Self {
            command_sender: Arc::new(Mutex::new(None)),
            command_receiver: Arc::new(Mutex::new(None)),
            response_sender: Arc::new(Mutex::new(None)),
            response_receiver: Arc::new(Mutex::new(None)),
        };

        (transport1, transport2)
    }

    /// Send a command
    pub fn send_command(&self, command: IpcCommand) -> Result<()> {
        if let Some(sender) = self.command_sender.lock().unwrap().as_ref() {
            sender.send(command)
                .map_err(|e| crate::CziError::ipc(format!("Failed to send command: {}", e)))
        } else {
            Err(crate::CziError::ipc("Command sender not available".to_string()))
        }
    }

    /// Receive a command asynchronously
    pub async fn receive_command_async(&self) -> Result<IpcCommand> {
        // This is a simplified implementation
        // In a real async transport, you'd use proper async channels
        self.receive_command()
    }

    /// Send a response
    pub async fn send_response_async(&self, response: IpcResponse) -> Result<()> {
        self.send_response(response)
    }

    /// Receive a response asynchronously
    pub async fn receive_response_async(&self) -> Result<IpcResponse> {
        // This is a simplified implementation
        // In a real async transport, you'd use proper async channels
        if let Some(receiver) = self.response_receiver.lock().unwrap().as_ref() {
            receiver.recv()
                .map_err(|e| crate::CziError::ipc(format!("Failed to receive response: {}", e)))
        } else {
            Err(crate::CziError::ipc("Response receiver not available".to_string()))
        }
    }
}

impl IpcTransport for ChannelTransport {
    fn send_response(&self, response: IpcResponse) -> Result<()> {
        if let Some(sender) = self.response_sender.lock().unwrap().as_ref() {
            sender.send(response)
                .map_err(|e| crate::CziError::ipc(format!("Failed to send response: {}", e)))
        } else {
            Err(crate::CziError::ipc("Response sender not available".to_string()))
        }
    }

    fn receive_command(&self) -> Result<IpcCommand> {
        if let Some(receiver) = self.command_receiver.lock().unwrap().as_ref() {
            receiver.recv()
                .map_err(|e| crate::CziError::ipc(format!("Failed to receive command: {}", e)))
        } else {
            Err(crate::CziError::ipc("Command receiver not available".to_string()))
        }
    }
}

/// Transport factory for creating different transport types
pub struct TransportFactory;

impl TransportFactory {
    /// Create an in-memory transport for testing
    pub fn create_in_memory() -> (Box<dyn IpcTransport>, Box<dyn IpcTransport>) {
        let (transport1, transport2) = InMemoryTransport::new();
        (Box::new(transport1), Box::new(transport2))
    }

    /// Create a channel-based transport
    pub fn create_channel() -> (Box<dyn IpcTransport>, Box<dyn IpcTransport>) {
        let (transport1, transport2) = ChannelTransport::new();
        (Box::new(transport1), Box::new(transport2))
    }

    /// Create a transport based on configuration
    pub fn create_from_config(config: &TransportConfig) -> Result<Box<dyn IpcTransport>> {
        match config.transport_type {
            TransportType::InMemory => {
                let (transport, _) = Self::create_in_memory();
                Ok(transport)
            }
            TransportType::Channel => {
                let (transport, _) = Self::create_channel();
                Ok(transport)
            }
            TransportType::WebSocket => {
                Err(crate::CziError::ipc("WebSocket transport not implemented".to_string()))
            }
            TransportType::Tauri => {
                Err(crate::CziError::ipc("Tauri transport not implemented".to_string()))
            }
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub timeout_ms: Option<u64>,
    pub buffer_size: Option<usize>,
}

/// Transport types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    /// In-memory transport for testing
    InMemory,
    /// Channel-based transport
    Channel,
    /// WebSocket transport (future implementation)
    WebSocket,
    /// Tauri transport (future implementation)
    Tauri,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            transport_type: TransportType::InMemory,
            timeout_ms: Some(30000), // 30 seconds
            buffer_size: Some(1000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_transport() {
        let (transport1, transport2) = TransportFactory::create_in_memory();

        // Send command from transport2
        let command = crate::IpcCommand {
            id: "test-123".to_string(),
            name: "test".to_string(),
            parameters: None,
            timestamp: chrono::Utc::now(),
        };

        transport2.command_sender.send(command).unwrap();

        // Receive command on transport1
        let received_command = transport1.receive_command().unwrap();
        assert_eq!(received_command.id, "test-123");
        assert_eq!(received_command.name, "test");

        // Send response
        let response = crate::IpcResponse::success(
            received_command.id.clone(),
            Some(serde_json::json!({"result": "ok"})),
        );

        transport1.send_response(response).unwrap();

        // Receive response
        let received_response = transport2.try_receive_response().unwrap();
        assert_eq!(received_response.command_id, "test-123");
        assert!(received_response.success);
    }

    #[test]
    fn test_channel_transport() {
        let (transport1, transport2) = TransportFactory::create_channel();

        // Send command from transport2
        let command = crate::IpcCommand {
            id: "test-456".to_string(),
            name: "test".to_string(),
            parameters: None,
            timestamp: chrono::Utc::now(),
        };

        assert!(transport2.send_command(command).is_ok());

        // Receive command on transport1
        let received_command = transport1.receive_command().unwrap();
        assert_eq!(received_command.id, "test-456");

        // Send response
        let response = crate::IpcResponse::success(
            received_command.id.clone(),
            Some(serde_json::json!({"result": "ok"})),
        );

        assert!(transport1.send_response(response).is_ok());
    }

    #[test]
    fn test_transport_factory() {
        let config = TransportConfig::default();
        let transport = TransportFactory::create_from_config(&config);
        assert!(transport.is_ok());

        let config = TransportConfig {
            transport_type: TransportType::Channel,
            timeout_ms: Some(5000),
            buffer_size: Some(100),
        };
        let transport = TransportFactory::create_from_config(&config);
        assert!(transport.is_ok());

        let config = TransportConfig {
            transport_type: TransportType::WebSocket,
            timeout_ms: None,
            buffer_size: None,
        };
        let transport = TransportFactory::create_from_config(&config);
        assert!(transport.is_err());
    }

    #[test]
    fn test_in_memory_transport_checkers() {
        let (transport1, transport2) = TransportFactory::create_in_memory();

        // Initially no pending commands
        assert!(!transport1.has_pending_commands());
        assert!(!transport1.has_pending_responses());

        // Send a command
        let command = crate::IpcCommand {
            id: "test".to_string(),
            name: "test".to_string(),
            parameters: None,
            timestamp: chrono::Utc::now(),
        };

        transport2.command_sender.send(command).unwrap();

        // Should have pending command now
        assert!(transport1.has_pending_commands());
        assert_eq!(transport1.pending_command_count(), 1);

        // Receive the command
        let _ = transport1.try_receive_command();
        assert!(!transport1.has_pending_commands());
    }
}