//! IPC command implementations

pub mod repository;
pub mod analysis;
pub mod queries;

// Re-export all command handlers
pub use repository::*;
pub use analysis::*;
pub use queries::*;

use crate::{Result, IpcCommand, IpcResponse};
use serde_json::Value;

/// Base command handler with common functionality
pub struct BaseCommandHandler;

impl BaseCommandHandler {
    /// Create a success response
    pub fn success_response(command_id: String, data: Option<Value>) -> IpcResponse {
        IpcResponse {
            command_id,
            success: true,
            data,
            error: None,
            execution_time_ms: 0,
        }
    }

    /// Create an error response
    pub fn error_response(command_id: String, error: String) -> IpcResponse {
        IpcResponse {
            command_id,
            success: false,
            data: None,
            error: Some(error),
            execution_time_ms: 0,
        }
    }

    /// Validate command parameters
    pub fn validate_params(command: &IpcCommand, required_params: &[&str]) -> Result<()> {
        if let Some(params) = &command.parameters {
            for param in required_params {
                if !params.get(*param).is_some() {
                    return Err(crate::CziError::validation(
                        format!("Missing required parameter: {}", param)
                    ));
                }
            }
        } else if !required_params.is_empty() {
            return Err(crate::CziError::validation("No parameters provided".to_string()));
        }
        Ok(())
    }

    /// Extract parameter from command
    pub fn get_param<T: serde::de::DeserializeOwned>(
        command: &IpcCommand,
        param_name: &str
    ) -> Result<T> {
        let param_value = command.parameters
            .as_ref()
            .and_then(|params| params.get(param_name))
            .ok_or_else(|| crate::CziError::validation(
                format!("Missing parameter: {}", param_name)
            ))?
            .clone();
        serde_json::from_value(param_value)
            .map_err(|e| crate::CziError::validation(
                format!("Invalid parameter {}: {}", param_name, e)
            ))
    }

    /// Extract optional parameter from command
    pub fn get_optional_param<T: serde::de::DeserializeOwned>(
        command: &IpcCommand,
        param_name: &str
    ) -> Result<Option<T>> {
        if let Some(params) = &command.parameters {
            if let Some(value) = params.get(param_name) {
                let typed_value: T = serde_json::from_value(value.clone())
                    .map_err(|e| crate::CziError::validation(
                        format!("Invalid parameter {}: {}", param_name, e)
                    ))?;
                Ok(Some(typed_value))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}