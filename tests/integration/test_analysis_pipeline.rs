//! Integration tests for full analysis pipeline
//!
//! These tests validate the end-to-end analysis workflow from repositories to results.

use czi_core::{
    config::{RepositoryConfiguration, CziConfig, AuthType, AuthConfig, RepositoryStatus},
    io::RepositorySyncService,
};
use czi_ipc::{IpcCommand, CommandHandler};
use serde_json::json;
use tempfile::TempDir;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// Test data structure for creating sample code files
struct TestCodebase {
    temp_dir: TempDir,
    repositories: Vec<RepositoryConfiguration>,
}

impl TestCodebase {
    /// Create a test codebase with sample Java files
    async fn create() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create repository directories
        let repo1_path = temp_dir.path().join("repo1");
        let repo2_path = temp_dir.path().join("repo2");

        fs::create_dir_all(&repo1_path).await?;
        fs::create_dir_all(&repo2_path).await?;

        // Initialize git repositories
        for path in [&repo1_path, &repo2_path] {
            tokio::process::Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
                .await?;

            tokio::process::Command::new("git")
                .args(["config", "user.email", "test@example.com"])
                .current_dir(path)
                .output()
                .await?;

            tokio::process::Command::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(path)
                .output()
                .await?;
        }

        // Create sample Java files in repo1
        let main_java = repo1_path.join("src/main/java/com/example/Main.java");
        fs::create_dir_all(main_java.parent().unwrap()).await?;
        fs::write(
            &main_java,
            r#"
package com.example;

import com.example.service.UserService;
import com.example.util.StringUtils;

public class Main {
    public static void main(String[] args) {
        UserService userService = new UserService();
        String result = userService.processUser("test");
        StringUtils.print(result);

        // Dead code - never called
        unusedMethod();
    }

    private static void unusedMethod() {
        System.out.println("This is dead code");
        StringUtils.debug("Dead method called");
    }
}
"#
        ).await?;

        // UserService.java (active)
        let service_java = repo1_path.join("src/main/java/com/example/service/UserService.java");
        fs::create_dir_all(service_java.parent().unwrap()).await?;
        fs::write(
            &service_java,
            r#"
package com.example.service;

import com.example.util.StringUtils;

public class UserService {
    public String processUser(String input) {
        return StringUtils.process(input);
    }

    // Another unused method
    private void internalHelper() {
        StringUtils.debug("Internal helper");
    }
}
"#
        ).await?;

        // StringUtils.java (active utility)
        let util_java = repo1_path.join("src/main/java/com/example/util/StringUtils.java");
        fs::create_dir_all(util_java.parent().unwrap()).await?;
        fs::write(
            &util_java,
            r#"
package com.example.util;

public class StringUtils {
    public static String process(String input) {
        return input.toUpperCase();
    }

    public static void print(String message) {
        System.out.println(message);
    }

    public static void debug(String message) {
        System.err.println("DEBUG: " + message);
    }
}
"#
        ).await?;

        // Create Python files in repo2
        let main_py = repo2_path.join("main.py");
        fs::write(
            &main_py,
            r#"
import utils.string_utils
import services.data_service

def main():
    result = data_service.get_data()
    processed = utils.string_utils.process(result)
    print(processed)

    # Dead function - never called
    dead_function()

def dead_function():
    print("This is dead code")
    utils.string_utils.debug("Dead function called")

# Another dead class
class UnusedClass:
    def __init__(self):
        self.value = utils.string_utils.process("unused")
"#
        ).await?;

        // Python utils
        let utils_dir = repo2_path.join("utils");
        fs::create_dir_all(&utils_dir).await?;
        fs::write(
            utils_dir.join("string_utils.py"),
            r#"
def process(text):
    return text.upper()

def debug(message):
    print(f"DEBUG: {message}")
"#
        ).await?;

        // Python services
        let services_dir = repo2_path.join("services");
        fs::create_dir_all(&services_dir).await?;
        fs::write(
            services_dir.join("data_service.py"),
            r#"
from utils.string_utils import process

def get_data():
    return "sample_data"

def unused_helper():
    return process("helper")
"#
        ).await?;

        // Create repository configurations
        let repositories = vec![
            RepositoryConfiguration {
                id: "test_repo_1".to_string(),
                name: "Test Repository 1".to_string(),
                url: repo1_path.to_string_lossy().to_string(),
                local_path: Some(repo1_path.clone()),
                branch: "main".to_string(),
                auth_type: AuthType::None,
                auth_config: AuthConfig::None,
                last_sync: None,
                status: RepositoryStatus::Active,
            },
            RepositoryConfiguration {
                id: "test_repo_2".to_string(),
                name: "Test Repository 2".to_string(),
                url: repo2_path.to_string_lossy().to_string(),
                local_path: Some(repo2_path.clone()),
                branch: "main".to_string(),
                auth_type: AuthType::None,
                auth_config: AuthConfig::None,
                last_sync: None,
                status: RepositoryStatus::Active,
            },
        ];

        Ok(Self {
            temp_dir,
            repositories,
        })
    }

    /// Get repository configurations
    fn get_repositories(&self) -> &Vec<RepositoryConfiguration> {
        &self.repositories
    }

    /// Get temporary directory path
    fn temp_path(&self) -> &PathBuf {
        self.temp_dir.path()
    }
}

#[tokio::test]
async fn test_full_analysis_pipeline() {
    // Arrange
    let test_codebase = TestCodebase::create().await.expect("Failed to create test codebase");
    let handler = CommandHandler::new();

    // Add repositories to configuration
    for repo in test_codebase.get_repositories() {
        let add_command = IpcCommand {
            id: Uuid::new_v4().to_string(),
            command: "add_repository".to_string(),
            params: Some(json!({
                "id": repo.id.clone(),
                "name": repo.name.clone(),
                "url": repo.url.clone(),
                "local_path": repo.local_path.clone(),
                "branch": repo.branch.clone(),
                "auth_type": "none"
            })),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let result = handler.handle_command(add_command).await;
        assert!(result.is_ok(), "Should add repository successfully");
    }

    // Act: Run analysis
    let analysis_command = IpcCommand {
        id: "full_pipeline_test".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": ["test_repo_1", "test_repo_2"],
            "options": {
                "max_depth": 5,
                "include_test_files": false,
                "languages": ["java", "python"]
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(analysis_command).await;

    // Assert
    assert!(result.is_ok(), "Analysis should start successfully");
    let response = result.unwrap();
    assert!(response.success, "Analysis command should succeed");

    let data = response.data.expect("Should contain analysis data");
    let analysis_id = data["analysis_id"].as_str().expect("Should have analysis_id");

    // Wait a moment for analysis to process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check analysis status
    let status_command = IpcCommand {
        id: "check_status".to_string(),
        command: "get_analysis_status".to_string(),
        params: Some(json!({
            "analysis_id": analysis_id
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(status_command).await;
    assert!(result.is_ok(), "Status check should succeed");

    let response = result.unwrap();
    // Status might not be complete yet, but should be valid
    if response.success {
        let data = response.data.expect("Should contain status data");
        let status = data["status"].as_str().unwrap_or("unknown");
        assert!(!status.is_empty(), "Should have valid status");
    }

    // Test getting results (might be empty if analysis not complete)
    let results_command = IpcCommand {
        id: "get_results".to_string(),
        command: "get_analysis_results".to_string(),
        params: Some(json!({
            "analysis_id": analysis_id,
            "filters": {}
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(results_command).await;
    assert!(result.is_ok(), "Results query should succeed");
}

#[tokio::test]
async fn test_analysis_with_no_repositories() {
    // Arrange
    let handler = CommandHandler::new();

    // Act: Run analysis without any repositories
    let analysis_command = IpcCommand {
        id: "no_repos_test".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": [],
            "options": {
                "max_depth": 5
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(analysis_command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Should fail when no repositories provided");

    let error = response.error.expect("Should have error message");
    assert!(error.contains("repositories") || error.contains("No repositories"),
              "Error should mention no repositories");
}

#[tokio::test]
async fn test_analysis_progress_tracking() {
    // Arrange
    let test_codebase = TestCodebase::create().await.expect("Failed to create test codebase");
    let handler = CommandHandler::new();

    // Add one repository
    let repo = &test_codebase.get_repositories()[0];
    let add_command = IpcCommand {
        id: Uuid::new_v4().to_string(),
        command: "add_repository".to_string(),
        params: Some(json!({
            "id": repo.id.clone(),
            "name": repo.name.clone(),
            "url": repo.url.clone(),
            "auth_type": "none"
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(add_command).await;
    assert!(result.is_ok());

    // Act: Start analysis
    let analysis_command = IpcCommand {
        id: "progress_test".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": [repo.id.clone()],
            "options": {
                "max_depth": 3
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(analysis_command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success, "Analysis should start");

    let data = response.data.expect("Should contain analysis data");
    let analysis_id = data["analysis_id"].as_str().expect("Should have analysis_id");
    let status = data["status"].as_str().expect("Should have status");

    assert_eq!(status, "started", "Analysis should be started");

    // Test multiple status checks
    for _ in 0..3 {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let status_command = IpcCommand {
            id: format!("status_check_{}", uuid::Uuid::new_v4()),
            command: "get_analysis_status".to_string(),
            params: Some(json!({
                "analysis_id": analysis_id
            })),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let result = handler.handle_command(status_command).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_analysis_filters() {
    // Arrange
    let test_codebase = TestCodebase::create().await.expect("Failed to create test codebase");
    let handler = CommandHandler::new();

    // Add repositories
    for repo in test_codebase.get_repositories() {
        let add_command = IpcCommand {
            id: Uuid::new_v4().to_string(),
            command: "add_repository".to_string(),
            params: Some(json!({
                "id": repo.id.clone(),
                "name": repo.name.clone(),
                "url": repo.url.clone(),
                "auth_type": "none"
            })),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        handler.handle_command(add_command).await.unwrap();
    }

    // Act: Run analysis with language filter
    let analysis_command = IpcCommand {
        id: "filter_test".to_string(),
        command: "run_analysis".to_string(),
        params: Some(json!({
            "repository_ids": ["test_repo_1"],
            "options": {
                "max_depth": 5,
                "languages": ["java"],
                "exclude_patterns": ["*_test.java"]
            }
        })),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.handle_command(analysis_command).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success, "Filtered analysis should start");

    let data = response.data.expect("Should contain analysis data");
    assert!(data["analysis_id"].is_string(), "Should have analysis_id");
    assert!(data["status"].is_string(), "Should have status");
}