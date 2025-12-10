//! Query commands

use crate::{Result, IpcCommand, IpcResponse, CommandHandler, commands::BaseCommandHandler};

/// Handler for query commands
pub struct QueryCommandHandler;

impl QueryCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn query_dependencies(&self, command: IpcCommand) -> Result<IpcResponse> {
        BaseCommandHandler::validate_params(&command, &["symbol_id"])?;

        let _symbol_id: String = BaseCommandHandler::get_param(&command, "symbol_id")?;

        let response_data = serde_json::json!({
            "symbol_id": command.id,
            "dependencies": [
                {
                    "id": "dep_1",
                    "name": "HelperFunction",
                    "type": "Function",
                    "relationship": "Calls"
                },
                {
                    "id": "dep_2",
                    "name": "UtilityClass",
                    "type": "Class",
                    "relationship": "Instantiates"
                }
            ],
            "total_count": 2
        });

        Ok(BaseCommandHandler::success_response(command.id, Some(response_data)))
    }

    pub fn get_symbol_info(&self, command: IpcCommand) -> Result<IpcResponse> {
        BaseCommandHandler::validate_params(&command, &["symbol_id"])?;

        let _symbol_id: String = BaseCommandHandler::get_param(&command, "symbol_id")?;

        let response_data = serde_json::json!({
            "id": command.id,
            "name": "ExampleSymbol",
            "type": "Function",
            "file_path": "src/example.rs",
            "line_number": 42,
            "language": "Rust",
            "metadata": {
                "visibility": "Public",
                "signature": "fn example_symbol() -> Result<()>",
                "documentation": "Example function for demonstration"
            }
        });

        Ok(BaseCommandHandler::success_response(command.id, Some(response_data)))
    }
}

impl Default for QueryCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for QueryCommandHandler {
    fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
        match command.name.as_str() {
            "query_dependencies" => self.query_dependencies(command),
            "get_symbol_info" => self.get_symbol_info(command),
            _ => Ok(BaseCommandHandler::error_response(
                command.id,
                format!("Unknown query command: {}", command.name)
            )),
        }
    }
}

// Tauri command exports
#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn query_dependencies(symbol_id: String) -> std::result::Result<serde_json::Value, String> {
    let handler = QueryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "query_dependencies".to_string(),
        parameters: Some(serde_json::json!({ "symbol_id": symbol_id })),
        timestamp: chrono::Utc::now(),
    };

    handler.query_dependencies(command)
        .map(|response| response.data.unwrap_or(serde_json::Value::Null))
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn get_symbol_info(symbol_id: String) -> std::result::Result<serde_json::Value, String> {
    let handler = QueryCommandHandler::new();
    let command = IpcCommand {
        id: uuid::Uuid::new_v4().to_string(),
        name: "get_symbol_info".to_string(),
        parameters: Some(serde_json::json!({ "symbol_id": symbol_id })),
        timestamp: chrono::Utc::now(),
    };

    handler.get_symbol_info(command)
        .map(|response| response.data.unwrap_or(serde_json::Value::Null))
        .map_err(|e| e.to_string())
}

// Mock query command
#[cfg(feature = "tauri")]
#[tauri::command]
pub async fn query_dependents() -> std::result::Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}