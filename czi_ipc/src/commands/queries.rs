//! Query commands

use crate::{Result, IpcCommand, IpcResponse, CommandHandler, commands::BaseCommandHandler};
use serde_json::Value;

/// Handler for query commands
pub struct QueryCommandHandler {
    base: BaseCommandHandler,
}

impl QueryCommandHandler {
    pub fn new() -> Self {
        Self {
            base: BaseCommandHandler,
        }
    }

    pub fn query_dependencies(&self, command: IpcCommand) -> Result<IpcResponse> {
        self.base.validate_params(&command, &["symbol_id"])?;

        let _symbol_id: String = self.base.get_param(&command, "symbol_id")?;

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

        Ok(self.base.success_response(command.id, Some(response_data)))
    }

    pub fn get_symbol_info(&self, command: IpcCommand) -> Result<IpcResponse> {
        self.base.validate_params(&command, &["symbol_id"])?;

        let _symbol_id: String = self.base.get_param(&command, "symbol_id")?;

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

        Ok(self.base.success_response(command.id, Some(response_data)))
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
            _ => Ok(self.base.error_response(
                command.id,
                format!("Unknown query command: {}", command.name)
            )),
        }
    }
}