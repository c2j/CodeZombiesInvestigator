//! Analysis commands

use crate::{Result, IpcCommand, IpcResponse, CommandHandler, commands::BaseCommandHandler};
use serde_json::Value;

/// Handler for analysis commands
pub struct AnalysisCommandHandler {
    base: BaseCommandHandler,
}

impl AnalysisCommandHandler {
    pub fn new() -> Self {
        Self {
            base: BaseCommandHandler,
        }
    }

    pub fn run_analysis(&self, command: IpcCommand) -> Result<IpcResponse> {
        // Validate required parameters
        self.base.validate_params(&command, &["repository_ids"])?;

        let _repository_ids: Vec<String> = self.base.get_param(&command, "repository_ids")?;

        // Generate analysis ID
        let analysis_id = format!("analysis_{}", uuid::Uuid::new_v4());

        let response_data = serde_json::json!({
            "id": analysis_id,
            "status": "Started",
            "message": "Analysis started successfully"
        });

        Ok(self.base.success_response(command.id, Some(response_data)))
    }

    pub fn get_analysis_status(&self, command: IpcCommand) -> Result<IpcResponse> {
        self.base.validate_params(&command, &["id"])?;

        let _analysis_id: String = self.base.get_param(&command, "id")?;

        let response_data = serde_json::json!({
            "id": command.id,
            "status": "Running",
            "progress": 45,
            "total_symbols": 1000,
            "processed_symbols": 450,
            "estimated_completion": "2023-12-06T12:00:00Z"
        });

        Ok(self.base.success_response(command.id, Some(response_data)))
    }

    pub fn get_analysis_results(&self, command: IpcCommand) -> Result<IpcResponse> {
        self.base.validate_params(&command, &["id"])?;

        let _analysis_id: String = self.base.get_param(&command, "id")?;

        let response_data = serde_json::json!({
            "id": command.id,
            "status": "Completed",
            "summary": {
                "total_symbols": 1000,
                "zombie_symbols": 45,
                "reachable_symbols": 955,
                "zombie_percentage": 4.5,
                "analysis_duration_ms": 120000
            },
            "zombie_code_items": [
                {
                    "id": "zombie_1",
                    "symbol_id": "unused_function_123",
                    "symbol_name": "oldUtilityFunction",
                    "file_path": "src/utils/legacy.rs",
                    "zombie_type": "DeadCode",
                    "confidence": 0.95,
                    "isolation_distance": 8,
                    "line_number": 42
                },
                {
                    "id": "zombie_2",
                    "symbol_id": "deprecated_class_456",
                    "symbol_name": "OldAuthSystem",
                    "file_path": "src/auth/legacy.rs",
                    "zombie_type": "Unreachable",
                    "confidence": 0.88,
                    "isolation_distance": 3,
                    "line_number": 15
                }
            ],
            "generated_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(self.base.success_response(command.id, Some(response_data)))
    }
}

impl Default for AnalysisCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for AnalysisCommandHandler {
    fn execute(&self, command: IpcCommand) -> Result<IpcResponse> {
        match command.name.as_str() {
            "run_analysis" => self.run_analysis(command),
            "get_analysis_status" => self.get_analysis_status(command),
            "get_analysis_results" => self.get_analysis_results(command),
            _ => Ok(self.base.error_response(
                command.id,
                format!("Unknown analysis command: {}", command.name)
            )),
        }
    }
}