//! Unit tests for configuration entities and validation

use crate::{
    config::{
        RepositoryConfiguration, AuthType, AuthConfig, RepositoryStatus,
        RootNodeConfig, RootNodeType, CziConfig, AppConfig, AnalysisConfig,
    },
    CziError, Result,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_repository_configuration_validation() -> Result<()> {
    // Valid configuration
    let valid_config = RepositoryConfiguration {
        id: "test_repo".to_string(),
        name: "Test Repository".to_string(),
        url: "https://github.com/example/repo.git".to_string(),
        local_path: Some(PathBuf::from("/tmp/test_repo")),
        branch: "main".to_string(),
        auth_type: AuthType::None,
        auth_config: AuthConfig::None,
        last_sync: None,
        status: RepositoryStatus::Active,
    };

    assert!(valid_config.validate().is_ok(), "Valid configuration should pass validation");

    // Test empty ID
    let mut invalid_config = valid_config.clone();
    invalid_config.id = "".to_string();
    assert!(invalid_config.validate().is_err(), "Empty ID should fail validation");

    // Test empty name
    let mut invalid_config = valid_config.clone();
    invalid_config.name = "".to_string();
    assert!(invalid_config.validate().is_err(), "Empty name should fail validation");

    // Test invalid URL
    let mut invalid_config = valid_config.clone();
    invalid_config.url = "not-a-url".to_string();
    assert!(invalid_config.validate().is_err(), "Invalid URL should fail validation");

    // Test empty branch
    let mut invalid_config = valid_config.clone();
    invalid_config.branch = "".to_string();
    assert!(invalid_config.validate().is_err(), "Empty branch should fail validation");

    // Test SSH key validation
    let mut ssh_config = valid_config.clone();
    ssh_config.auth_type = AuthType::SshKey;
    ssh_config.auth_config = AuthConfig::SshKey {
        key_path: PathBuf::from("/nonexistent/ssh/key"),
        passphrase: None,
    };
    assert!(ssh_config.validate().is_err(), "Non-existent SSH key should fail validation");

    // Test token validation
    let mut token_config = valid_config.clone();
    token_config.auth_type = AuthType::Token;
    token_config.auth_config = AuthConfig::Token {
        token: "".to_string(),
        username: None,
    };
    assert!(token_config.validate().is_err(), "Empty token should fail validation");

    // Test basic auth validation
    let mut basic_config = valid_config.clone();
    basic_config.auth_type = AuthType::Basic;
    basic_config.auth_config = AuthConfig::Basic {
        username: "".to_string(),
        password: "".to_string(),
    };
    assert!(basic_config.validate().is_err(), "Empty credentials should fail validation");

    Ok(())
}

#[test]
fn test_repository_configuration_serialization() -> Result<()> {
    let config = RepositoryConfiguration {
        id: "serialize_test".to_string(),
        name: "Serialization Test".to_string(),
        url: "https://github.com/example/serialize.git".to_string(),
        local_path: Some(PathBuf::from("/tmp/serialize_repo")),
        branch: "main".to_string(),
        auth_type: AuthType::Token,
        auth_config: AuthConfig::Token {
            token: "test_token_12345".to_string(),
            username: Some("testuser".to_string()),
        },
        last_sync: None,
        status: RepositoryStatus::Active,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&config)?;
    let deserialized: RepositoryConfiguration = serde_json::from_str(&json)?;

    assert_eq!(deserialized.id, config.id);
    assert_eq!(deserialized.name, config.name);
    assert_eq!(deserialized.url, config.url);
    assert_eq!(deserialized.branch, config.branch);
    assert_eq!(deserialized.auth_type, config.auth_type);

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config)?;
    let deserialized: RepositoryConfiguration = serde_yaml::from_str(&yaml)?;

    assert_eq!(deserialized.id, config.id);
    assert_eq!(deserialized.name, config.name);
    assert_eq!(deserialized.url, config.url);

    Ok(())
}

#[test]
fn test_root_node_configuration_validation() -> Result<()> {
    // Valid root node configuration
    let valid_node = RootNodeConfig {
        id: "root_node_1".to_string(),
        repository_id: "test_repo".to_string(),
        node_type: RootNodeType::Controller,
        symbol_path: "com.example.Controller.handleRequest".to_string(),
        file_path: "src/main/java/com/example/Controller.java".to_string(),
        line_number: 42,
        metadata: std::collections::HashMap::new(),
    };

    assert!(valid_node.validate().is_ok(), "Valid root node should pass validation");

    // Test empty ID
    let mut invalid_node = valid_node.clone();
    invalid_node.id = "".to_string();
    assert!(invalid_node.validate().is_err(), "Empty ID should fail validation");

    // Test empty repository ID
    let mut invalid_node = valid_node.clone();
    invalid_node.repository_id = "".to_string();
    assert!(invalid_node.validate().is_err(), "Empty repository ID should fail validation");

    // Test empty symbol path
    let mut invalid_node = valid_node.clone();
    invalid_node.symbol_path = "".to_string();
    assert!(invalid_node.validate().is_err(), "Empty symbol path should fail validation");

    // Test empty file path
    let mut invalid_node = valid_node.clone();
    invalid_node.file_path = "".to_string();
    assert!(invalid_node.validate().is_err(), "Empty file path should fail validation");

    // Test invalid line number
    let mut invalid_node = valid_node.clone();
    invalid_node.line_number = 0;
    assert!(invalid_node.validate().is_err(), "Line number 0 should fail validation");

    Ok(())
}

#[test]
fn test_authentication_configuration_validation() -> Result<()> {
    // Test None authentication
    let none_auth = AuthConfig::None;
    assert!(none_auth.validate().is_ok(), "None authentication should be valid");

    // Test valid SSH key authentication
    let temp_dir = TempDir::new()?;
    let ssh_key_path = temp_dir.path().join("test_ssh_key");
    std::fs::write(&ssh_key_path, "dummy ssh key content")?;

    let ssh_auth = AuthConfig::SshKey {
        key_path: ssh_key_path.clone(),
        passphrase: None,
    };
    assert!(ssh_auth.validate().is_ok(), "Valid SSH key should pass validation");

    // Test non-existent SSH key
    let invalid_ssh_auth = AuthConfig::SshKey {
        key_path: PathBuf::from("/nonexistent/path/to/key"),
        passphrase: None,
    };
    assert!(invalid_ssh_auth.validate().is_err(), "Non-existent SSH key should fail validation");

    // Test valid token authentication
    let token_auth = AuthConfig::Token {
        token: "ghp_valid_token_12345".to_string(),
        username: Some("testuser".to_string()),
    };
    assert!(token_auth.validate().is_ok(), "Valid token should pass validation");

    // Test empty token
    let invalid_token_auth = AuthConfig::Token {
        token: "".to_string(),
        username: None,
    };
    assert!(invalid_token_auth.validate().is_err(), "Empty token should fail validation");

    // Test valid basic authentication
    let basic_auth = AuthConfig::Basic {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };
    assert!(basic_auth.validate().is_ok(), "Valid basic auth should pass validation");

    // Test empty credentials
    let invalid_basic_auth = AuthConfig::Basic {
        username: "".to_string(),
        password: "".to_string(),
    };
    assert!(invalid_basic_auth.validate().is_err(), "Empty credentials should fail validation");

    Ok(())
}

#[test]
fn test_configuration_cross_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a complete configuration
    let mut config = CziConfig {
        app: AppConfig {
            log_level: "info".to_string(),
            data_dir: temp_dir.path().join("data"),
            cache_dir: temp_dir.path().join("cache"),
            max_concurrent_operations: 10,
            debug: false,
        },
        repositories: vec![],
        active_root_nodes: vec![],
        analysis: AnalysisConfig {
            max_depth: 10,
            enabled_languages: vec!["java".to_string(), "python".to_string()],
        },
        performance: czi_core::config::PerformanceConfig {
            max_memory_gb: 8,
            cache_enabled: true,
        },
    };

    // Add a repository
    let repo_config = RepositoryConfiguration {
        id: "test_repo".to_string(),
        name: "Test Repository".to_string(),
        url: "https://github.com/example/repo.git".to_string(),
        local_path: Some(temp_dir.path().join("test_repo")),
        branch: "main".to_string(),
        auth_type: AuthType::None,
        auth_config: AuthConfig::None,
        last_sync: None,
        status: RepositoryStatus::Active,
    };
    config.repositories.push(repo_config);

    // Add a root node that references the repository
    let root_node = RootNodeConfig {
        id: "root_node_1".to_string(),
        repository_id: "test_repo".to_string(),
        node_type: RootNodeType::Controller,
        symbol_path: "com.example.Controller.handleRequest".to_string(),
        file_path: "src/main/java/com/example/Controller.java".to_string(),
        line_number: 42,
        metadata: std::collections::HashMap::new(),
    };
    config.active_root_nodes.push(root_node);

    // Cross-validation should pass
    assert!(config.validate().is_ok(), "Configuration with valid references should pass cross-validation");

    // Test with non-existent repository reference
    let mut invalid_config = config.clone();
    invalid_config.active_root_nodes[0].repository_id = "nonexistent_repo".to_string();
    assert!(invalid_config.validate().is_err(), "Configuration with invalid repository reference should fail cross-validation");

    Ok(())
}

#[test]
fn test_configuration_serialization_roundtrip() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let original_config = CziConfig {
        app: AppConfig {
            log_level: "debug".to_string(),
            data_dir: temp_dir.path().join("data"),
            cache_dir: temp_dir.path().join("cache"),
            max_concurrent_operations: 20,
            debug: true,
        },
        repositories: vec![],
        active_root_nodes: vec![],
        analysis: AnalysisConfig {
            max_depth: 15,
            enabled_languages: vec!["java".to_string(), "javascript".to_string(), "python".to_string()],
        },
        performance: czi_core::config::PerformanceConfig {
            max_memory_gb: 16,
            cache_enabled: true,
        },
    };

    // Test JSON serialization roundtrip
    let json = serde_json::to_string_pretty(&original_config)?;
    let deserialized_config: CziConfig = serde_json::from_str(&json)?;

    assert_eq!(deserialized_config.app.log_level, original_config.app.log_level);
    assert_eq!(deserialized_config.repositories.len(), original_config.repositories.len());
    assert_eq!(deserialized_config.active_root_nodes.len(), original_config.active_root_nodes.len());

    // Test YAML serialization roundtrip
    let yaml = serde_yaml::to_string(&original_config)?;
    let deserialized_config: CziConfig = serde_yaml::from_str(&yaml)?;

    assert_eq!(deserialized_config.app.log_level, original_config.app.log_level);
    assert_eq!(deserialized_config.repositories.len(), original_config.repositories.len());

    Ok(())
}

#[test]
fn test_configuration_default_values() {
    let config = CziConfig::default();

    assert_eq!(config.app.log_level, "info");
    assert_eq!(config.repositories.len(), 0);
    assert_eq!(config.active_root_nodes.len(), 0);
    assert!(config.app.debug == false);
    assert!(config.performance.cache_enabled == true);
}

#[test]
fn test_error_handling_and_validation_messages() -> Result<()> {
    // Test repository with invalid URL
    let invalid_repo = RepositoryConfiguration {
        id: "test".to_string(),
        name: "Test".to_string(),
        url: "not-a-url".to_string(),
        local_path: None,
        branch: "main".to_string(),
        auth_type: AuthType::None,
        auth_config: AuthConfig::None,
        last_sync: None,
        status: RepositoryStatus::Active,
    };

    let result = invalid_repo.validate();
    assert!(result.is_err());

    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(error_msg.contains("URL") || error_msg.contains("url"), "Error message should mention URL");
    }

    // Test with non-existent SSH key
    let invalid_ssh_repo = RepositoryConfiguration {
        id: "ssh_test".to_string(),
        name: "SSH Test".to_string(),
        url: "git@github.com:example/repo.git".to_string(),
        local_path: None,
        branch: "main".to_string(),
        auth_type: AuthType::SshKey,
        auth_config: AuthConfig::SshKey {
            key_path: PathBuf::from("/nonexistent/ssh/key"),
            passphrase: None,
        },
        last_sync: None,
        status: RepositoryStatus::Active,
    };

    let result = invalid_ssh_repo.validate();
    assert!(result.is_err());

    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(error_msg.contains("SSH key") || error_msg.contains("key"), "Error message should mention SSH key");
    }

    Ok(())
}

#[test]
fn test_configuration_performance_characteristics() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create configuration with many repositories
    let mut config = CziConfig {
        app: AppConfig {
            log_level: "info".to_string(),
            data_dir: temp_dir.path().join("data"),
            cache_dir: temp_dir.path().join("cache"),
            max_concurrent_operations: 100,
            debug: false,
        },
        repositories: vec![],
        active_root_nodes: vec![],
        analysis: AnalysisConfig {
            max_depth: 50,
            enabled_languages: vec!["java".to_string(), "javascript".to_string(), "python".to_string(), "shell".to_string()],
        },
        performance: czi_core::config::PerformanceConfig {
            max_memory_gb: 32,
            cache_enabled: true,
        },
    };

    // Add many repositories for performance testing
    for i in 0..100 {
        let repo_config = RepositoryConfiguration {
            id: format!("perf_test_repo_{}", i),
            name: format!("Performance Test Repository {}", i),
            url: format!("https://github.com/example/repo{}.git", i),
            local_path: None,
            branch: "main".to_string(),
            auth_type: AuthType::None,
            auth_config: AuthConfig::None,
            last_sync: None,
            status: RepositoryStatus::Active,
        };
        config.repositories.push(repo_config);
    }

    // Test serialization performance
    let start_time = std::time::Instant::now();
    let json = serde_json::to_string(&config)?;
    let serialization_time = start_time.elapsed();

    let start_time = std::time::Instant::now();
    let _: CziConfig = serde_json::from_str(&json)?;
    let deserialization_time = start_time.elapsed();

    // Performance assertions
    assert!(serialization_time.as_millis() < 1000, "Serialization should complete within 1 second");
    assert!(deserialization_time.as_millis() < 1000, "Deserialization should complete within 1 second");

    println!("Performance test results:");
    println!("  - Serialization time: {}ms", serialization_time.as_millis());
    println!("  - Deserialization time: {}ms", deserialization_time.as_millis());
    println!("  - JSON size: {} bytes", json.len());

    Ok(())
}

/// Extension trait for validation
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

impl Validate for RepositoryConfiguration {
    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(CziError::validation("Repository ID cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(CziError::validation("Repository name cannot be empty".to_string()));
        }

        if self.url.is_empty() {
            return Err(CziError::validation("Repository URL cannot be empty".to_string()));
        }

        // Basic URL format validation
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") && !self.url.starts_with("git@") {
            return Err(CziError::validation("Repository URL must be HTTP(S) or SSH format".to_string()));
        }

        if self.branch.is_empty() {
            return Err(CziError::validation("Repository branch cannot be empty".to_string()));
        }

        // Authentication validation
        self.auth_config.validate()?;

        Ok(())
    }
}

impl Validate for RootNodeConfig {
    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(CziError::validation("Root node ID cannot be empty".to_string()));
        }

        if self.repository_id.is_empty() {
            return Err(CziError::validation("Repository ID cannot be empty".to_string()));
        }

        if self.symbol_path.is_empty() {
            return Err(CziError::validation("Symbol path cannot be empty".to_string()));
        }

        if self.file_path.is_empty() {
            return Err(CziError::validation("File path cannot be empty".to_string()));
        }

        if self.line_number == 0 {
            return Err(CziError::validation("Line number must be greater than 0".to_string()));
        }

        Ok(())
    }
}

impl Validate for AuthConfig {
    fn validate(&self) -> Result<()> {
        match self {
            AuthConfig::None => Ok(()),
            AuthConfig::SshKey { key_path, .. } => {
                if !key_path.exists() {
                    return Err(CziError::validation(format!(
                        "SSH key not found at path: {}",
                        key_path.display()
                    )));
                }
                Ok(())
            }
            AuthConfig::Token { token, .. } => {
                if token.is_empty() {
                    return Err(CziError::validation("Token cannot be empty".to_string()));
                }
                Ok(())
            }
            AuthConfig::Basic { username, password } => {
                if username.is_empty() || password.is_empty() {
                    return Err(CziError::validation("Username and password cannot be empty".to_string()));
                }
                Ok(())
            }
        }
    }
}

impl Validate for CziConfig {
    fn validate(&self) -> Result<()> {
        // Validate all repositories
        for repo in &self.repositories {
            repo.validate()?;
        }

        // Validate all root nodes
        for node in &self.active_root_nodes {
            node.validate()?;
        }

        // Cross-validation: ensure root nodes reference existing repositories
        let repo_ids: std::collections::HashSet<&str> = self.repositories
            .iter()
            .map(|r| r.id.as_str())
            .collect();

        for node in &self.active_root_nodes {
            if !repo_ids.contains(node.repository_id.as_str()) {
                return Err(CziError::validation(format!(
                    "Root node references non-existent repository: {}",
                    node.repository_id
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Create a test repository configuration
    pub fn create_test_repository() -> RepositoryConfiguration {
        RepositoryConfiguration {
            id: "test_repo".to_string(),
            name: "Test Repository".to_string(),
            url: "https://github.com/example/test.git".to_string(),
            local_path: None,
            branch: "main".to_string(),
            auth_type: AuthType::None,
            auth_config: AuthConfig::None,
            last_sync: None,
            status: RepositoryStatus::Active,
        }
    }

    /// Create a test root node configuration
    pub fn create_test_root_node() -> RootNodeConfig {
        RootNodeConfig {
            id: "test_root_node".to_string(),
            repository_id: "test_repo".to_string(),
            node_type: RootNodeType::Controller,
            symbol_path: "com.example.Controller.handleRequest".to_string(),
            file_path: "src/main/java/com/example/Controller.java".to_string(),
            line_number: 42,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a test configuration
    pub fn create_test_config() -> CziConfig {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        CziConfig {
            app: AppConfig {
                log_level: "info".to_string(),
                data_dir: temp_dir.path().join("data"),
                cache_dir: temp_dir.path().join("cache"),
                max_concurrent_operations: 10,
                debug: false,
            },
            repositories: vec![create_test_repository()],
            active_root_nodes: vec![create_test_root_node()],
            analysis: AnalysisConfig {
                max_depth: 10,
                enabled_languages: vec!["java".to_string(), "python".to_string()],
            },
            performance: czi_core::config::PerformanceConfig {
                max_memory_gb: 8,
                cache_enabled: true,
            },
        }
    }
}