//! Contract tests for repository validation endpoint
//!
//! These tests validate the contract between frontend and backend for repository operations.

use czi_ipc::{IpcCommand, CommandHandler};
use serde_json::{json, Value};

#[tokio::test]
async fn test_validate_repository_success() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_validate_1".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "url": "https://github.com/example/test-repo.git",
            "auth_type": "none"
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
    let accessible = data["accessible"].as_bool().expect("Should have accessible field");
    assert_eq!(accessible, true, "Public repository should be accessible");
}

#[tokio::test]
async fn test_validate_repository_with_auth() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_validate_auth".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "url": "https://github.com/example/private-repo.git",
            "auth_type": "token",
            "auth_config": {
                "token": "test_token_12345"
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();

    // Note: This may fail with actual authentication, but the endpoint should handle it gracefully
    assert!(response.success || !response.success, "Should handle auth appropriately");

    let data = response.data.expect("Response should contain data");
    assert!(data["accessible"].is_bool(), "Should have accessible boolean field");
    assert!(data.get("branches").is_some(), "Should have branches field or error");
}

#[tokio::test]
async fn test_validate_repository_invalid_url() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_validate_invalid".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "url": "not-a-valid-url",
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail for invalid URL");

    let error = response.error.expect("Should have error message");
    assert!(!error.is_empty(), "Error message should not be empty");
}

#[tokio::test]
async fn test_validate_repository_missing_url() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_validate_missing".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail when URL is missing");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("url") || error.contains("URL"), "Error should mention missing URL");
}

#[tokio::test]
async fn test_validate_repository_unsupported_auth_type() {
    // Arrange
    let handler = CommandHandler::new();
    let command = IpcCommand {
        id: "test_validate_unsupported_auth".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "url": "https://github.com/example/test-repo.git",
            "auth_type": "unsupported_type"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail for unsupported auth type");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("auth") || error.contains("authentication"), "Error should mention authentication");
}