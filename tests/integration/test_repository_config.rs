//! Integration tests for repository configuration workflow
//!
//! These tests validate the end-to-end repository configuration process.

use czi_core::{config::ConfigManager, CziConfig};
use czi_ipc::{IpcCommand, CommandHandler};
use serde_json::{json, Value};
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_repository_configuration_workflow() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.json");
    let handler = CommandHandler::new();

    // Step 1: Add first repository
    let add_command = IpcCommand {
        id: "add_repo_1".to_string(),
        command: "add_repository".to_string(),
        params: Some(json!({
            "name": "Test Repository 1",
            "url": "https://github.com/example/test-repo-1.git",
            "branch": "main",
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Act
    let result = handler.handle_command(add_command).await;

    // Assert
    assert!(result.is_ok(), "Should successfully add repository");
    let response = result.unwrap();
    assert!(response.success, "Add repository should succeed");

    let repo_id = response.data.expect("Should return repository ID")["id"]
        .as_str()
        .expect("ID should be a string");
    assert!(!repo_id.is_empty(), "Repository ID should not be empty");

    // Step 2: List repositories
    let list_command = IpcCommand {
        id: "list_repos".to_string(),
        command: "list_repositories".to_string(),
        params: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(list_command).await;
    assert!(result.is_ok(), "Should successfully list repositories");

    let response = result.unwrap();
    assert!(response.success, "List repositories should succeed");

    let repositories = response.data.expect("Should return repositories list")
        .as_array()
        .expect("Repositories should be an array");

    assert_eq!(repositories.len(), 1, "Should have exactly one repository");

    let repo = &repositories[0];
    assert_eq!(repo["name"], "Test Repository 1");
    assert_eq!(repo["url"], "https://github.com/example/test-repo-1.git");
    assert_eq!(repo["id"], repo_id);
    assert_eq!(repo["status"], "active"); // Initial status should be active

    // Step 3: Add second repository with authentication
    let add_command_2 = IpcCommand {
        id: "add_repo_2".to_string(),
        command: "add_repository".to_string(),
        params: Some(json!({
            "name": "Test Repository 2",
            "url": "https://github.com/example/test-repo-2.git",
            "branch": "develop",
            "auth_type": "token",
            "auth_config": {
                "token": "test_token_67890"
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(add_command_2).await;
    assert!(result.is_ok(), "Should successfully add second repository");

    // Step 4: Verify both repositories are listed
    let result = handler.handle_command(IpcCommand {
        id: "list_repos_2".to_string(),
        command: "list_repositories".to_string(),
        params: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);

    let repositories = response.data.unwrap().as_array().unwrap();
    assert_eq!(repositories.len(), 2, "Should have two repositories");

    // Step 5: Remove first repository
    let remove_command = IpcCommand {
        id: "remove_repo_1".to_string(),
        command: "remove_repository".to_string(),
        params: Some(json!({
            "id": repo_id
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(remove_command).await;
    assert!(result.is_ok(), "Should successfully remove repository");

    let response = result.unwrap();
    assert!(response.success, "Remove repository should succeed");

    // Step 6: Verify only second repository remains
    let result = handler.handle_command(IpcCommand {
        id: "list_repos_final".to_string(),
        command: "list_repositories".to_string(),
        params: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);

    let repositories = response.data.unwrap().as_array().unwrap();
    assert_eq!(repositories.len(), 1, "Should have only one repository");
    assert_eq!(repositories[0]["name"], "Test Repository 2");
}

#[tokio::test]
async fn test_repository_sync_workflow() {
    // Arrange
    let handler = CommandHandler::new();

    // Add a repository first
    let add_command = IpcCommand {
        id: "add_sync_repo".to_string(),
        command: "add_repository".to_string(),
        params: Some(json!({
            "name": "Sync Test Repository",
            "url": "https://github.com/example/sync-test-repo.git",
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(add_command).await;
    assert!(result.is_ok(), "Should add repository for sync test");

    let response = result.unwrap();
    let repo_id = response.data.unwrap()["id"].as_str().unwrap();

    // Act: Sync the repository
    let sync_command = IpcCommand {
        id: "sync_repo".to_string(),
        command: "sync_repository".to_string(),
        params: Some(json!({
            "id": repo_id
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(sync_command).await;

    // Assert
    assert!(result.is_ok(), "Should successfully trigger sync");
    let response = result.unwrap();

    // Note: The sync might fail with actual network errors, but the command should be handled
    if response.success {
        // If sync succeeds, verify repository status updates
        assert!(response.data.is_some(), "Should return sync status");
    } else {
        // If sync fails, it should have a proper error message
        assert!(response.error.is_some(), "Should have error message for sync failure");
        let error = response.error.unwrap();
        assert!(!error.is_empty(), "Error message should not be empty");
    }
}

#[tokio::test]
async fn test_repository_validation_before_add() {
    // Arrange
    let handler = CommandHandler::new();

    // Act: Validate repository before adding
    let validate_command = IpcCommand {
        id: "validate_before_add".to_string(),
        command: "validate_repository".to_string(),
        params: Some(json!({
            "url": "https://github.com/example/valid-repo.git",
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(validate_command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success, "Validation should succeed");

    let data = response.data.expect("Should return validation data");
    assert!(data["accessible"].is_bool(), "Should have accessibility status");
    assert!(data.get("branches").is_some(), "Should have branch information");

    // If validation succeeded, we should be able to add the repository
    let add_command = IpcCommand {
        id: "add_validated_repo".to_string(),
        command: "add_repository".to_string(),
        params: Some(json!({
            "name": "Validated Repository",
            "url": "https://github.com/example/valid-repo.git",
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(add_command).await;
    assert!(result.is_ok(), "Should be able to add validated repository");
    let response = result.unwrap();
    assert!(response.success, "Adding validated repository should succeed");
}

#[tokio::test]
async fn test_config_persistence() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.json");

    // Create initial configuration
    let initial_config = CziConfig {
        app: czi_core::config::AppConfig {
            log_level: "info".to_string(),
            data_dir: temp_dir.path().join("data"),
            cache_dir: temp_dir.path().join("cache"),
            max_concurrent_operations: 10,
            debug: false,
        },
        repositories: vec![],
        active_root_nodes: vec![],
        analysis: czi_core::config::AnalysisConfig {
            max_depth: 10,
            enabled_languages: vec!["java".to_string(), "python".to_string()],
        },
        performance: czi_core::config::PerformanceConfig {
            max_memory_gb: 8,
            cache_enabled: true,
        },
    };

    // Save initial configuration
    let manager = ConfigManager::new(&config_path);
    let save_result = manager.save_config(&initial_config);
    assert!(save_result.is_ok(), "Should save initial configuration");

    // Load configuration
    let loaded_config = manager.load_config();
    assert!(loaded_config.is_ok(), "Should load saved configuration");

    let loaded = loaded_config.unwrap();
    assert_eq!(loaded.repositories.len(), 0, "Should start with no repositories");
    assert_eq!(loaded.app.log_level, "info", "Should preserve app settings");

    // Simulate adding a repository through the system
    let updated_config = CziConfig {
        repositories: vec![czi_core::config::RepositoryConfig {
            id: "test_repo_123".to_string(),
            name: "Test Repository".to_string(),
            url: "https://github.com/example/test.git".to_string(),
            local_path: None,
            branch: "main".to_string(),
            auth: None,
            enabled: true,
            last_sync: None,
        }],
        ..loaded
    };

    let save_result = manager.save_config(&updated_config);
    assert!(save_result.is_ok(), "Should save updated configuration");

    // Verify persistence
    let reloaded_config = manager.load_config();
    assert!(reloaded_config.is_ok(), "Should reload updated configuration");

    let reloaded = reloaded_config.unwrap();
    assert_eq!(reloaded.repositories.len(), 1, "Should have one repository");
    assert_eq!(reloaded.repositories[0].name, "Test Repository", "Should preserve repository name");
    assert_eq!(reloaded.repositories[0].url, "https://github.com/example/test.git", "Should preserve repository URL");
}