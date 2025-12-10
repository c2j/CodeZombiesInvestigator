//! Contract tests for analysis execution endpoint
//!
//! These tests validate the contract between frontend and backend for analysis operations.

use czi_ipc::{IpcCommand, CommandHandler};
use serde_json::{json, Value};

#[tokio::test]
async fn test_run_analysis_success() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_1".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": ["repo_1", "repo_2"],
            "options": {
                "max_depth": 10,
                "include_test_files": false,
                "languages": ["java", "python", "javascript"]
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);

    let data = response.data.expect("Response should contain data");

    // Check analysis session was created
    assert!(data["analysis_id"].is_string(), "Should contain analysis_id");

    // Check status
    let status = data["status"].as_str().expect("Should have status field");
    assert!(status == "started" || status == "queued", "Should have started or queued status");
}

#[tokio::test]
async fn test_run_analysis_missing_repositories() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_missing_repos".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "options": {
                "max_depth": 5
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail when repository_ids is missing");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("repository_ids") || error.contains("repositories"), "Error should mention missing repositories");
}

#[tokio::test]
async fn test_run_analysis_invalid_repository() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_invalid_repo".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": ["nonexistent_repo"],
            "options": {
                "max_depth": 10
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail when repository doesn't exist");

    let error = response.error.expect("Should have error message");
    assert!(!error.is_empty(), "Error message should not be empty");
}

#[tokio::test]
async fn test_run_analysis_invalid_options() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_invalid_options".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": ["repo_1"],
            "options": {
                "max_depth": -1  // Invalid depth
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail with invalid options");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("max_depth") || error.contains("invalid"), "Error should mention the invalid parameter");
}

#[tokio::test]
async fn test_get_analysis_status() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_status".to_string(),
        command: "get_analysis_status".to_string(),
        params: Some(json!({
            "analysis_id": "test_analysis_12345"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();

    // Should handle gracefully whether analysis exists or not
    if response.success {
        let data = response.data.expect("Should contain data when successful");

        // Check status fields
        assert!(data["status"].is_string(), "Should have status field");
        assert!(data["progress"].is_number(), "Should have progress field");
        assert!(data["total_files"].is_number(), "Should have total_files field");
        assert!(data["processed_files"].is_number(), "Should have processed_files field");
    } else {
        // Should handle non-existent analysis gracefully
        let error = response.error.expect("Should have error message");
        assert!(error.contains("not found") || error.contains("Analysis"), "Error should mention analysis not found");
    }
}

#[tokio::test]
async fn test_get_analysis_results() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_analysis_results".to_string(),
        command: "get_analysis_results".to_string(),
        params: Some(json!({
            "analysis_id": "test_analysis_67890",
            "filters": {
                "repository_id": "repo_1",
                "language": "java"
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();

    // Should handle gracefully whether analysis exists or not
    if response.success {
        let data = response.data.expect("Should contain data when successful");

        // Check result structure
        assert!(data["zombie_code_items"].is_array(), "Should have zombie_code_items array");
        assert!(data["summary"].is_object(), "Should have summary object");

        // Check summary fields
        let summary = &data["summary"];
        assert!(summary["total_symbols"].is_number(), "Should have total_symbols");
        assert!(summary["zombie_symbols"].is_number(), "Should have zombie_symbols");
        assert!(summary["zombie_percentage"].is_string(), "Should have zombie_percentage");
    } else {
        // Should handle non-existent analysis gracefully
        let error = response.error.expect("Should have error message");
        assert!(error.contains("not found") || error.contains("Analysis"), "Error should mention analysis not found");
    }
}

#[tokio::test]
async fn test_get_analysis_results_missing_analysis_id() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_results_missing_id".to_string(),
        command: "get_analysis_results".to_string(),
        params: Some(json!({
            "filters": {
                "language": "java"
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail when analysis_id is missing");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("analysis_id") || error.contains("missing"), "Error should mention missing analysis_id");
}

#[tokio::test]
async fn test_stop_analysis() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_stop_analysis".to_string(),
        command: "stop_analysis".to_string(),
        params: Some(json!({
            "analysis_id": "test_analysis_stop_98765"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();

    // Should handle gracefully whether analysis exists or not
    if response.success {
        let data = response.data.expect("Should contain data when successful");
        assert!(data["stopped"].is_boolean(), "Should have stopped field");
        assert!(data["message"].is_string(), "Should have message field");
    } else {
        // Should handle non-existent analysis gracefully
        let error = response.error.expect("Should have error message");
        assert!(error.contains("not found") || error.contains("Analysis"), "Error should mention analysis not found");
    }
}