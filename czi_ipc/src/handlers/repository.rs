//! Repository command handlers

use crate::{Result, IpcCommand, IpcResponse, commands::BaseCommandHandler};
use serde_json::Value;
use czi_core::{
    config::{RepositoryConfiguration, AuthType, AuthConfig},
    io::{RepositoryValidator, RepositorySyncService},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

/// Repository command handler
pub struct RepositoryHandler {
    base: BaseCommandHandler,
    config_storage: Arc<czi_core::config::ConfigStorage>,
    validator: RepositoryValidator,
    sync_service: RepositorySyncService,
    repositories: Arc<RwLock<Vec<RepositoryConfiguration>>>,
}

impl RepositoryHandler {
    /// Create a new repository handler
    pub fn new(config_storage: Arc<czi_core::config::ConfigStorage>) -> Result<Self> {
        let sync_service = RepositorySyncService::new("./repositories")?;

        Ok(Self {
            base: BaseCommandHandler,
            config_storage,
            validator: RepositoryValidator::new()?,
            sync_service,
            repositories: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Handle list_repositories command
    #[instrument(skip(self))]
    pub async fn handle_list_repositories(&self, command: IpcCommand) -> Result<IpcResponse> {
        debug!("Handling list_repositories command");

        let repositories = self.repositories.read().await;

        let repo_list: Vec<Value> = repositories.iter().map(|repo| {
            json!({
                "id": repo.id,
                "name": repo.name,
                "url": repo.url,
                "branch": repo.branch,
                "auth_type": repo.auth_type,
                "status": repo.status_string(),
                "last_sync": repo.last_sync,
                "local_path": repo.local_path
            })
        }).collect();

        Ok(self.base.success_response(command.id, Some(Value::Array(repo_list))))
    }

    /// Handle add_repository command
    #[instrument(skip(self))]
    pub async fn handle_add_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        debug!("Handling add_repository command");

        // Validate required parameters
        self.base.validate_params(&command, &["name", "url"])?;

        let name: String = self.base.get_param(&command, "name")?;
        let url: String = self.base.get_param(&command, "url")?;
        let branch: Option<String> = self.base.get_optional_param(&command, "branch")?;
        let auth_type: String = self.base.get_param(&command, "auth_type")
            .unwrap_or_else(|_| "none".to_string());

        // Parse auth type
        let auth_type = match auth_type.as_str() {
            "none" => AuthType::None,
            "ssh_key" => AuthType::SSHKey,
            "token" => AuthType::Token,
            "basic" => AuthType::Basic,
            _ => return Err(czi_core::CziError::validation(
                "auth_type",
                "Invalid authentication type"
            )),
        };

        // Create auth config
        let auth_config = match auth_type {
            AuthType::None => AuthConfig::None,
            AuthType::SSHKey => {
                let key_path: String = self.base.get_param(&command, "key_path")?;
                let passphrase: Option<String> = self.base.get_optional_param(&command, "passphrase")?;
                AuthConfig::SshKey {
                    key_path: key_path.into(),
                    passphrase,
                }
            }
            AuthType::Token => {
                let token: String = self.base.get_param(&command, "token")?;
                let username: Option<String> = self.base.get_optional_param(&command, "username")?;
                AuthConfig::Token { token, username }
            }
            AuthType::Basic => {
                let username: String = self.base.get_param(&command, "username")?;
                let password: String = self.base.get_param(&command, "password")?;
                AuthConfig::Basic { username, password }
            }
        };

        // Create repository configuration
        let repo_id = format!("repo_{}", uuid::Uuid::new_v4());
        let repository = RepositoryConfiguration {
            id: repo_id.clone(),
            name,
            url,
            local_path: None,
            branch: branch.unwrap_or_else(|| "main".to_string()),
            auth_type,
            auth_config,
            last_sync: None,
            status: czi_core::config::RepositoryStatus::Active,
        };

        // Validate repository
        repository.validate()?;

        // Add to in-memory list
        {
            let mut repositories = self.repositories.write().await;
            repositories.push(repository.clone());
        }

        // Save to storage
        self.config_storage.save_repositories(
            self.repositories.read().await.clone()
        ).await?;

        let response_data = json!({
            "id": repo_id,
            "status": "added",
            "message": "Repository added successfully"
        });

        Ok(self.base.success_response(command.id, Some(response_data)))
    }

    /// Handle remove_repository command
    #[instrument(skip(self))]
    pub async fn handle_remove_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        debug!("Handling remove_repository command");

        let repository_id: String = self.base.get_param(&command, "id")?;

        // Remove from in-memory list
        let mut repositories = self.repositories.write().await;
        let initial_len = repositories.len();
        repositories.retain(|r| r.id != repository_id);
        let removed = repositories.len() < initial_len;

        if removed {
            // Save to storage
            self.config_storage.save_repositories(repositories.clone()).await?;

            let response_data = json!({
                "id": repository_id,
                "status": "removed",
                "message": "Repository removed successfully"
            });

            Ok(self.base.success_response(command.id, Some(response_data)))
        } else {
            Err(czi_core::CziError::validation(
                "id",
                "Repository not found"
            ))
        }
    }

    /// Handle sync_repository command
    #[instrument(skip(self))]
    pub async fn handle_sync_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        debug!("Handling sync_repository command");

        let repository_id: String = self.base.get_param(&command, "id")?;

        // Find repository
        let repositories = self.repositories.read().await;
        let repository = repositories.iter()
            .find(|r| r.id == repository_id)
            .ok_or_else(|| czi_core::CziError::validation("id", "Repository not found"))?
            .clone();
        drop(repositories);

        // Synchronize repository
        let sync_result = self.sync_service.synchronize_repository(&repository).await;

        match sync_result {
            Ok(result) => {
                // Update last sync timestamp
                {
                    let mut repositories = self.repositories.write().await;
                    if let Some(repo) = repositories.iter_mut().find(|r| r.id == repository_id) {
                        repo.last_sync = Some(chrono::Utc::now());
                        if result.success {
                            repo.transition_to(czi_core::config::RepositoryStatus::Active)?;
                        } else {
                            repo.transition_to(czi_core::config::RepositoryStatus::Error)?;
                        }
                    }
                }

                let response_data = json!({
                    "id": repository_id,
                    "success": result.success,
                    "local_path": result.local_path,
                    "default_branch": result.default_branch,
                    "available_branches": result.available_branches,
                    "sync_stats": {
                        "duration_ms": result.sync_stats.duration.as_millis(),
                        "objects_received": result.sync_stats.objects_received
                    },
                    "error": result.error
                });

                Ok(self.base.success_response(command.id, Some(response_data)))
            }
            Err(e) => {
                error!("Failed to synchronize repository {}: {}", repository_id, e);

                let response_data = json!({
                    "id": repository_id,
                    "success": false,
                    "error": e.to_string()
                });

                Ok(self.base.success_response(command.id, Some(response_data)))
            }
        }
    }

    /// Handle validate_repository command
    #[instrument(skip(self))]
    pub async fn handle_validate_repository(&self, command: IpcCommand) -> Result<IpcResponse> {
        debug!("Handling validate_repository command");

        let url: String = self.base.get_param(&command, "url")?;
        let auth_type: String = self.base.get_param(&command, "auth_type")
            .unwrap_or_else(|_| "none".to_string());

        // Parse auth type
        let auth_type = match auth_type.as_str() {
            "none" => AuthType::None,
            "ssh_key" => AuthType::SSHKey,
            "token" => AuthType::Token,
            "basic" => AuthType::Basic,
            _ => return Err(czi_core::CziError::validation(
                "auth_type",
                "Invalid authentication type"
            )),
        };

        // Create auth config from parameters
        let auth_config = match auth_type {
            AuthType::None => Some(AuthConfig::None),
            AuthType::SSHKey => {
                let key_path: Option<String> = self.base.get_optional_param(&command, "key_path")?;
                let passphrase: Option<String> = self.base.get_optional_param(&command, "passphrase")?;
                key_path.map(|kp| AuthConfig::SshKey {
                    key_path: kp.into(),
                    passphrase,
                })
            }
            AuthType::Token => {
                let token: Option<String> = self.base.get_optional_param(&command, "token")?;
                let username: Option<String> = self.base.get_optional_param(&command, "username")?;
                token.map(|t| AuthConfig::Token { token: t, username })
            }
            AuthType::Basic => {
                let username: Option<String> = self.base.get_optional_param(&command, "username")?;
                let password: Option<String> = self.base.get_optional_param(&command, "password")?;
                match (username, password) {
                    (Some(u), Some(p)) => Some(AuthConfig::Basic { username: u, password: p }),
                    _ => None,
                }
            }
        };

        // Validate repository
        let validation_result = self.validator.validate_repository(&url, auth_type, auth_config).await?;

        let response_data = json!({
            "url": url,
            "accessible": validation_result.accessible,
            "repository_type": validation_result.repository_type,
            "default_branch": validation_result.default_branch,
            "available_branches": validation_result.available_branches,
            "auth_method": validation_result.auth_method,
            "error": validation_result.error,
            "validation_metadata": {
                "url_type": validation_result.validation_metadata.url_type,
                "hosting_platform": validation_result.validation_metadata.hosting_platform,
                "requires_authentication": validation_result.validation_metadata.requires_authentication,
                "supported_auth_methods": validation_result.validation_metadata.supported_auth_methods
            }
        });

        Ok(self.base.success_response(command.id, Some(response_data)))
    }

    /// Load repositories from storage
    pub async fn load_repositories(&self) -> Result<()> {
        debug!("Loading repositories from storage");

        let repositories = self.config_storage.load_repositories().await?;
        let mut repos_lock = self.repositories.write().await;
        *repos_lock = repositories;

        info!("Loaded {} repositories from storage", repos_lock.len());
        Ok(())
    }
}