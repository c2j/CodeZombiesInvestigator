//! IPC data structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// IPC command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcCommand {
    /// Unique command identifier
    pub id: String,
    /// Command name (e.g., "list_repositories", "run_analysis")
    pub name: String,
    /// Command parameters
    pub parameters: Option<serde_json::Value>,
    /// Command timestamp
    pub timestamp: DateTime<Utc>,
}

/// IPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Command ID this response corresponds to
    pub command_id: String,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// IPC event for push notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event source (e.g., "analyzer", "repository_sync")
    pub source: String,
}

/// Command execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    /// Command is queued for execution
    Queued,
    /// Command is currently executing
    Running,
    /// Command completed successfully
    Completed,
    /// Command failed
    Failed,
    /// Command was cancelled
    Cancelled,
    /// Command timed out
    TimedOut,
}

/// Repository status for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatusIpc {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Analysis status for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatusIpc {
    pub id: String,
    pub repository_ids: Vec<String>,
    pub status: String,
    pub progress_percent: f64,
    pub current_stage: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Symbol information for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfoIpc {
    pub id: String,
    pub name: String,
    pub symbol_type: String,
    pub file_path: String,
    pub line_number: u32,
    pub repository_id: String,
    pub language: String,
    pub metadata: serde_json::Value,
}

/// Dependency information for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfoIpc {
    pub source_symbol_id: String,
    pub target_symbol_id: String,
    pub relationship_type: String,
    pub context: Option<String>,
    pub confidence: f64,
}

/// Zombie code item for IPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZombieCodeItemIpc {
    pub id: String,
    pub symbol_info: SymbolInfoIpc,
    pub zombie_type: String,
    pub confidence_level: String,
    pub isolation_distance: u32,
    pub last_modified: DateTime<Utc>,
    pub primary_contributor: Option<String>,
    pub context_path: Vec<String>,
    pub notes: Option<String>,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Task progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub task_name: String,
    pub status: CommandStatus,
    pub progress_percent: f64,
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub estimated_remaining: Option<u64>, // seconds
    pub error_message: Option<String>,
}

impl IpcCommand {
    /// Create a new command
    pub fn new(name: String, parameters: Option<serde_json::Value>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            parameters,
            timestamp: Utc::now(),
        }
    }

    /// Get a parameter value
    pub fn get_param<T: serde::de::DeserializeOwned>(&self, param_name: &str) -> Result<T> {
        self.parameters
            .as_ref()
            .and_then(|params| params.get(param_name))
            .ok_or_else(|| crate::CziError::validation(
                format!("Missing parameter: {}", param_name)
            ))?
            .clone()
            .try_into()
            .map_err(|e| crate::CziError::validation(
                format!("Invalid parameter {}: {}", param_name, e)
            ))
    }

    /// Get an optional parameter value
    pub fn get_optional_param<T: serde::de::DeserializeOwned>(&self, param_name: &str) -> Result<Option<T>> {
        if let Some(params) = &self.parameters {
            if let Some(value) = params.get(param_name) {
                let typed_value: T = value.clone()
                    .try_into()
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

    /// Validate that required parameters exist
    pub fn validate_params(&self, required_params: &[&str]) -> Result<()> {
        if let Some(params) = &self.parameters {
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
}

impl IpcResponse {
    /// Create a success response
    pub fn success(command_id: String, data: Option<serde_json::Value>) -> Self {
        Self {
            command_id,
            success: true,
            data,
            error: None,
            execution_time_ms: 0,
        }
    }

    /// Create an error response
    pub fn error(command_id: String, error: String) -> Self {
        Self {
            command_id,
            success: false,
            data: None,
            error: Some(error),
            execution_time_ms: 0,
        }
    }

    /// Set execution time
    pub fn with_execution_time(mut self, execution_time_ms: u64) -> Self {
        self.execution_time_ms = execution_time_ms;
        self
    }

    /// Get response data as specific type
    pub fn get_data<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        self.data
            .as_ref()
            .ok_or_else(|| crate::CziError::ipc("No data in response".to_string()))?
            .clone()
            .try_into()
            .map_err(|e| crate::CziError::ipc(
                format!("Failed to deserialize response data: {}", e)
            ))
    }

    /// Get error message
    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.success && self.error.is_none()
    }
}

impl IpcEvent {
    /// Create a new event
    pub fn new(event_type: String, data: serde_json::Value, source: String) -> Self {
        Self {
            event_type,
            data,
            timestamp: Utc::now(),
            source,
        }
    }
}