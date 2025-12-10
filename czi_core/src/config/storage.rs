//! Configuration persistence layer

use crate::{CziError, Result, config::{CziConfig, RepositoryConfiguration}};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn, error, instrument};
use std::sync::Arc;

/// Configuration storage service
pub struct ConfigStorage {
    config_path: PathBuf,
    backup_path: PathBuf,
}

impl ConfigStorage {
    /// Create a new configuration storage
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        let config_path = config_path.as_ref().to_path_buf();
        let backup_path = config_path.with_extension("bak");

        Self {
            config_path,
            backup_path,
        }
    }

    /// Load configuration from file
    #[instrument(skip(self))]
    pub async fn load_config(&self) -> Result<CziConfig> {
        info!("Loading configuration from: {:?}", self.config_path);

        if !self.config_path.exists() {
            warn!("Configuration file not found: {:?}, using defaults", self.config_path);
            return Ok(CziConfig::default());
        }

        // Read file content
        let content = fs::read_to_string(&self.config_path).await
            .map_err(|e| CziError::Repository(format!("Failed to read configuration file {}: {}", self.config_path.display(), e)))?;

        // Determine format by file extension
        let config = match self.config_path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&content)
                    .map_err(|e| CziError::serialization(
                        "Failed to parse YAML configuration",
                        e
                    ))?
            }
            Some("json") | _ => {
                serde_json::from_str(&content)
                    .map_err(|e| CziError::serialization(
                        "Failed to parse JSON configuration",
                        e
                    ))?
            }
        };

        debug!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to file
    #[instrument(skip(self))]
    pub async fn save_config(&self, config: &CziConfig) -> Result<()> {
        info!("Saving configuration to: {:?}", self.config_path);

        // Create backup of existing config if it exists
        if self.config_path.exists() {
            self.create_backup().await?;
        }

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| CziError::Repository(format!("Failed to create configuration directory {}: {}", parent.display(), e)))?;
        }

        // Serialize configuration based on file extension
        let content = match self.config_path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => {
                serde_yaml::to_string(config)
                    .map_err(|e| CziError::serialization(
                        "Failed to serialize YAML configuration",
                        e
                    ))?
            }
            Some("json") | _ => {
                serde_json::to_string_pretty(config)
                    .map_err(|e| CziError::serialization(
                        "Failed to serialize JSON configuration",
                        e
                    ))?
            }
        };

        // Write configuration to file
        fs::write(&self.config_path, content).await
            .map_err(|e| CziError::Repository(format!("Failed to write configuration file {}: {}", self.config_path.display(), e)))?;

        debug!("Configuration saved successfully");
        Ok(())
    }

    /// Create backup of existing configuration
    async fn create_backup(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path).await
                .map_err(|e| CziError::Repository(format!("Failed to create configuration backup {}: {}", self.config_path.display(), e)))?;
            debug!("Created configuration backup: {:?}", self.backup_path);
        }
        Ok(())
    }

    /// Load repositories from configuration
    pub async fn load_repositories(&self) -> Result<Vec<RepositoryConfiguration>> {
        let config = self.load_config().await?;
        Ok(config.repositories)
    }

    /// Save repositories to configuration
    pub async fn save_repositories(&self, repositories: Vec<RepositoryConfiguration>) -> Result<()> {
        let mut config = self.load_config().await?;
        config.repositories = repositories;
        self.save_config(&config).await
    }

    /// Add a repository to configuration
    pub async fn add_repository(&self, repository: RepositoryConfiguration) -> Result<()> {
        let mut repositories = self.load_repositories().await?;

        // Check if repository already exists
        if repositories.iter().any(|r| r.id == repository.id) {
            return Err(CziError::validation(
                "id",
                "Repository with this ID already exists"
            ));
        }

        repositories.push(repository);
        self.save_repositories(repositories).await
    }

    /// Remove a repository from configuration
    pub async fn remove_repository(&self, repository_id: &str) -> Result<bool> {
        let mut repositories = self.load_repositories().await?;
        let initial_len = repositories.len();

        repositories.retain(|r| r.id != repository_id);

        if repositories.len() < initial_len {
            self.save_repositories(repositories).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update a repository in configuration
    pub async fn update_repository(&self, repository: RepositoryConfiguration) -> Result<bool> {
        let mut repositories = self.load_repositories().await?;

        if let Some(existing_repo) = repositories.iter_mut().find(|r| r.id == repository.id) {
            *existing_repo = repository;
            self.save_repositories(repositories).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if configuration exists
    pub async fn exists(&self) -> bool {
        self.config_path.exists()
    }

    /// Delete configuration file
    pub async fn delete(&self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path).await
                .map_err(|e| CziError::Repository(format!("Failed to delete configuration file {}: {}", self.config_path.display(), e)))?;
        }

        if self.backup_path.exists() {
            fs::remove_file(&self.backup_path).await
                .map_err(|e| CziError::Repository(format!("Failed to delete configuration backup {}: {}", self.backup_path.display(), e)))?;
        }

        Ok(())
    }

    /// Export configuration to different format
    pub async fn export_config(&self, format: ConfigFormat, output_path: &Path) -> Result<()> {
        let config = self.load_config().await?;
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config)
                .map_err(|e| CziError::serialization("Failed to serialize JSON", e))?,
            ConfigFormat::Yaml => serde_yaml::to_string(&config)
                .map_err(|e| CziError::serialization("Failed to serialize YAML", e))?,
        };

        fs::write(output_path, content).await
            .map_err(|e| CziError::Repository(format!("Failed to export configuration {}: {}", output_path.display(), e)))?;
    }

    /// Import configuration from file
    pub async fn import_config(&self, import_path: &Path, merge: bool) -> Result<()> {
        let content = fs::read_to_string(import_path).await
            .map_err(|e| CziError::Repository(format!("Failed to read import file {}: {}", import_path.display(), e)))?;

        let imported_config: CziConfig = match import_path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&content)
                    .map_err(|e| CziError::serialization("Failed to parse YAML", e))?
            }
            Some("json") | _ => {
                serde_json::from_str(&content)
                    .map_err(|e| CziError::serialization("Failed to parse JSON", e))?
            }
        };

        if merge {
            let mut existing_config = self.load_config().await?;
            // Merge repositories (append, avoiding duplicates)
            for repo in imported_config.repositories {
                if !existing_config.repositories.iter().any(|r| r.id == repo.id) {
                    existing_config.repositories.push(repo);
                }
            }
            self.save_config(&existing_config).await?;
        } else {
            self.save_config(&imported_config).await?;
        }

        Ok(())
    }
}

/// Configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
}

/// In-memory configuration cache
#[derive(Debug)]
pub struct ConfigCache {
    storage: Arc<ConfigStorage>,
    config: tokio::sync::RwLock<Option<CziConfig>>,
}

impl ConfigCache {
    /// Create a new configuration cache
    pub fn new(storage: Arc<ConfigStorage>) -> Self {
        Self {
            storage,
            config: tokio::sync::RwLock::new(None),
        }
    }

    /// Get configuration (cached or loaded)
    pub async fn get(&self) -> Result<CziConfig> {
        let config_read = self.config.read().await;
        if let Some(ref config) = *config_read {
            return Ok(config.clone());
        }
        drop(config_read);

        let config = self.storage.load_config().await?;
        let mut config_write = self.config.write().await;
        *config_write = Some(config.clone());
        Ok(config)
    }

    /// Set configuration (updates cache and storage)
    pub async fn set(&self, config: &CziConfig) -> Result<()> {
        self.storage.save_config(config).await?;
        let mut config_write = self.config.write().await;
        *config_write = Some(config.clone());
        Ok(())
    }

    /// Invalidate cache
    pub async fn invalidate(&self) {
        let mut config_write = self.config.write().await;
        *config_write = None;
    }

    /// Force reload from storage
    pub async fn reload(&self) -> Result<CziConfig> {
        self.invalidate().await;
        self.get().await
    }
}

impl Default for ConfigStorage {
    fn default() -> Self {
        Self::new("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_config_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ConfigStorage::new(temp_dir.path().join("config.json"));

        assert_eq!(storage.config_path, temp_dir.path().join("config.json"));
        assert_eq!(storage.backup_path, temp_dir.path().join("config.bak"));
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ConfigStorage::new(temp_dir.path().join("config.json"));

        let config = CziConfig::default();
        storage.save_config(&config).await.unwrap();

        let loaded_config = storage.load_config().await.unwrap();
        assert_eq!(config.repositories.len(), loaded_config.repositories.len());
    }

    #[tokio::test]
    async fn test_config_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ConfigStorage::new(temp_dir.path().join("nonexistent.json"));

        let config = storage.load_config().await.unwrap();
        assert_eq!(config.repositories.len(), 0); // Default config
    }

    #[tokio::test]
    async fn test_repository_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ConfigStorage::new(temp_dir.path().join("config.json"));

        let repo = RepositoryConfiguration {
            id: "test_repo".to_string(),
            name: "Test Repository".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            local_path: None,
            branch: "main".to_string(),
            auth_type: crate::config::AuthType::None,
            auth_config: crate::config::AuthConfig::None,
            last_sync: None,
            status: crate::config::RepositoryStatus::Active,
        };

        // Add repository
        storage.add_repository(repo).await.unwrap();

        // Load repositories
        let repositories = storage.load_repositories().await.unwrap();
        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0].id, "test_repo");

        // Remove repository
        let removed = storage.remove_repository("test_repo").await.unwrap();
        assert!(removed);

        let repositories = storage.load_repositories().await.unwrap();
        assert_eq!(repositories.len(), 0);
    }

    #[tokio::test]
    async fn test_config_cache() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(ConfigStorage::new(temp_dir.path().join("config.json")));
        let cache = ConfigCache::new(storage);

        // First get should load from storage
        let config1 = cache.get().await.unwrap();

        // Second get should use cache
        let config2 = cache.get().await.unwrap();
        assert_eq!(config1.repositories.len(), config2.repositories.len());

        // Invalidate and reload
        cache.invalidate().await;
        let config3 = cache.reload().await.unwrap();
        assert_eq!(config1.repositories.len(), config3.repositories.len());
    }
}