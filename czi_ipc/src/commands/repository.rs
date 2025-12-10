//! Repository management commands

use crate::{Result, IpcCommand, IpcResponse, CommandHandler, commands::BaseCommandHandler};
use serde_json::Value;
use std::sync::Arc;

/// Handler for repository management commands
pub struct RepositoryCommandHandler {
    handler: Option<crate::handlers::repository::RepositoryHandler>,
}

impl RepositoryCommandHandler {
    /// Create a new repository command handler
    pub fn new() -> Self {
        Self { handler: None }
    }

    /// Create a repository command handler with dependencies
    pub fn with_handler(handler: crate::handlers::repository::RepositoryHandler) -> Self {
        Self { handler: Some(handler) }
    }

    /// Handle list_repositories command
    pub async fn list_repositories(&self, command: IpcCommand) -> Result<IpcResponse> {
        match &self.handler {
            Some(handler) => handler.handle_list_repositories(command).await,
            None => self.mock_list_repositories(command),
        }
    }

    /// Handle add_repository command
    pub async fn add_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        match &self.handler {
            Some(handler) => handler.handle_add_repository(command).await,
            None => self.mock_add_repository(command),
        }
    }

    /// Handle remove_repository command
    pub async fn remove_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        match &self.handler {
            Some(handler) => handler.handle_remove_repository(command).await,
            None => self.mock_remove_repository(command),
        }
    }

    /// Handle sync_repository command
    pub async fn sync_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        match &self.handler {
            Some(handler) => handler.handle_sync_repository(command).await,
            None => self.mock_sync_repository(command),
        }
    }

    /// Handle validate_repository command
    pub async fn validate_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        match &self.handler {
            Some(handler) => handler.handle_validate_repository(command).await,
            None => self.mock_validate_repository(command),
        }
    }

    /// Load repositories from storage
    pub async fn load_repositories(&self) -> Result<()> {
        if let Some(handler) = &self.handler {
            handler.load_repositories().await
        } else {
            Ok(())
        }
    }

    // Mock implementations for backward compatibility

    fn mock_list_repositories(&self, command: IpcCommand) -> Result<IpcResponse> {
        let repositories = Value::Array(vec![
            Value::Object(serde_json::json!({
                "id": "example_repo_1",
                "name": "Example Repository 1",
                "url": "https://github.com/example/repo1.git",
                "status": "active"
            })),
            Value::Object(serde_json::json!({
                "id": "example_repo_2",
                "name": "Example Repository 2",
                "url": "https://github.com/example/repo2.git",
                "status": "syncing"
            })),
        ]);

        Ok(BaseCommandHandler.success_response(command.id, Some(repositories)))
    }

    fn mock_add_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        // Validate required parameters
        BaseCommandHandler.validate_params(&command, &["name", "url"])?;

        let name: String = BaseCommandHandler::get_param(&command, "name")?;
        let url: String = BaseCommandHandler::get_param(&command, "url")?;

        let repo_id = format!("repo_{}", uuid::Uuid::new_v4());

        let response_data = serde_json::json!({
            "id": repo_id,
            "name": name,
            "url": url,
            "status": "added",
            "created_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(BaseCommandHandler.success_response(command.id, Some(response_data)))
    }

    fn mock_remove_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        // Validate required parameters
        BaseCommandHandler.validate_params(&command, &["id"])?;

        let response_data = serde_json::json!({
            "success": true,
            "message": "Repository removed successfully"
        });

        Ok(BaseCommandHandler.success_response(command.id, Some(response_data)))
    }

    fn mock_sync_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        // Validate required parameters
        BaseCommandHandler.validate_params(&command, &["id"])?;

        let response_data = serde_json::json!({
            "status": "sync_started",
            "message": "Repository synchronization started"
        });

        Ok(BaseCommandHandler.success_response(command.id, Some(response_data)))
    }

    fn mock_validate_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        // Validate required parameters
        BaseCommandHandler.validate_params(&command, &["url"])?;

        let _url: String = BaseCommandHandler::get_param(&command, "url")?;

        // In a real implementation, this would validate repository access
        let response_data = serde_json::json!({
            "accessible": true,
            "repository_type": "git",
            "default_branch": "main",
            "available_branches": ["main", "develop"],
            "message": "Repository is valid and accessible"
        });

        Ok(BaseCommandHandler.success_response(command.id, Some(response_data)))
    }
}

impl Default for RepositoryCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for RepositoryCommandHandler {
    fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
        match command.name.as_str() {
            "list_repositories" => self.list_repositories(command),
            "add_repository" => self.add_repository(command),
            "remove_repository" => self.remove_repository(command),
            "sync_repository" => self.sync_repository(command),
            "validate_repository" => self.validate_repository(command),
            _ => Ok(self.base.error_response(
                command.id,
                format!("Unknown repository command: {}", command.name)
            )),
        }
    }
}

// Tauri command exports (to be used when Tauri is re-enabled)
#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn list_repositories() -> Result<Value, String> {
    let handler = RepositoryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "list_repositories".to_string(),
        parameters: None,
        timestamp: chrono::Utc::now(),
    };

    handler.list_repositories(command)
        .map(|response| response.data.unwrap_or(Value::Null))
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn add_repository(config: Value) -> Result<String, String> {
    let handler = RepositoryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "add_repository".to_string(),
        parameters: Some(config),
        timestamp: chrono::Utc::now(),
    };

    handler.add_repository(command)
        .map(|response| response.data
            .and_then(|data| data.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string())
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn remove_repository(id: String) -> Result<(), String> {
    let handler = RepositoryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "remove_repository".to_string(),
        parameters: Some(serde_json::json!({"id": id})),
        timestamp: chrono::Utc::now(),
    };

    handler.remove_repository(command)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn sync_repository(id: String) -> Result<Value, String> {
    let handler = RepositoryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "sync_repository".to_string(),
        parameters: Some(serde_json::json!({"id": id})),
        timestamp: chrono::Utc::now(),
    };

    handler.sync_repository(command)
        .map(|response| response.data.unwrap_or(Value::Null))
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn validate_repository(url: String) -> Result<Value, String> {
    let handler = RepositoryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "validate_repository".to_string(),
        parameters: Some(serde_json::json!({"url": url})),
        timestamp: chrono::Utc::now(),
    };

    handler.validate_repository(command)
        .map(|response| response.data.unwrap_or(Value::Null))
        .map_err(|e| e.to_string())
}