//! Repository synchronization service

use crate::{CziError, Result, config::{RepositoryConfiguration, repository::AuthConfig}, git::repository::GitAuthConfig};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::{debug, info, warn, error, instrument};
use chrono::{DateTime, Utc};

/// Repository synchronization service
pub struct RepositorySyncService {
    git_operations: crate::git::GitOperations,
    cache_dir: PathBuf,
}

impl RepositorySyncService {
    /// Create a new synchronization service
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| CziError::Io(e))?;

        Ok(Self {
            git_operations: crate::git::GitOperations::new()?,
            cache_dir,
        })
    }

    /// Synchronize a repository (clone or fetch)
    #[instrument(skip(self))]
    pub async fn synchronize_repository(&self, repo_config: &RepositoryConfiguration) -> Result<SyncResult> {
        info!("Starting synchronization for repository: {}", repo_config.name);

        // Determine local path
        let local_path = self.get_local_path(repo_config)?;

        // Check if repository already exists locally
        if local_path.exists() && local_path.join(".git").exists() {
            info!("Repository exists locally, fetching updates: {}", local_path.display());
            self.fetch_repository(repo_config, &local_path).await
        } else {
            info!("Cloning repository to: {}", local_path.display());
            self.clone_repository(repo_config, &local_path).await
        }
    }

    /// Clone repository
    async fn clone_repository(&self, repo_config: &RepositoryConfiguration, local_path: &Path) -> Result<SyncResult> {
        let repo_config = repo_config.clone();
        let local_path = local_path.to_path_buf();
        let git_operations = self.git_operations.clone();

        task::spawn_blocking(move || {
            // Remove existing directory if it exists (for clean clone)
            if local_path.exists() {
                std::fs::remove_dir_all(&local_path)
                    .map_err(|e| CziError::Io(e))?;
            }

            // Create parent directory
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CziError::Io(e))?;
            }

            // Clone repository
            let git_auth_config = match repo_config.auth_config {
                AuthConfig::None => None,
                AuthConfig::SshKey { key_path, passphrase } => Some(GitAuthConfig {
                    ssh_key_path: Some(key_path.clone()),
                    ssh_passphrase: passphrase.clone(),
                    username: None,
                    password: None,
                }),
                AuthConfig::Token { token, username } => Some(GitAuthConfig {
                    ssh_key_path: None,
                    ssh_passphrase: None,
                    username: username.clone(),
                    password: Some(token.clone()),
                }),
                AuthConfig::Basic { username, password } => Some(GitAuthConfig {
                    ssh_key_path: None,
                    ssh_passphrase: None,
                    username: Some(username.clone()),
                    password: Some(password.clone()),
                }),
            };

            let clone_options = crate::git::CloneOptions {
                recursive: false, // Shallow clone for faster sync
                branch: Some(repo_config.branch.clone()),
                bare: false,
                auth: git_auth_config,
                timeout: Some(600), // 10 minutes timeout
                depth: Some(1), // Shallow clone with depth 1
            };

            let start_time = std::time::Instant::now();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    git_operations.clone_repository(&repo_config.url, &local_path, clone_options).await
                })
            });
            let duration = start_time.elapsed();

            match result {
                Ok(repository) => {
                    // Get repository information
                    let branch_info = Self::get_repository_info(&repository)?;
                    let sync_stats = SyncStats {
                        duration,
                        objects_received: 0, // TODO: Get actual stats from git operations
                        bytes_received: 0,
                        pack_size: 0,
                    };

                    Ok(SyncResult {
                        success: true,
                        local_path,
                        default_branch: branch_info.default_branch,
                        available_branches: branch_info.branches,
                        last_commit: branch_info.last_commit,
                        sync_stats,
                        error: None,
                    })
                }
                Err(e) => {
                    error!("Failed to clone repository {}: {}", repo_config.url, e);
                    Ok(SyncResult {
                        success: false,
                        local_path,
                        default_branch: None,
                        available_branches: vec![],
                        last_commit: None,
                        sync_stats: SyncStats::default(),
                        error: Some(e.to_string()),
                    })
                }
            }
        }).await
        .map_err(|e| CziError::runtime(format!("Task execution failed: {}", e)))?
    }

    /// Fetch repository updates
    async fn fetch_repository(&self, repo_config: &RepositoryConfiguration, local_path: &Path) -> Result<SyncResult> {
        let repo_config = repo_config.clone();
        let local_path = local_path.to_path_buf();
        let git_operations = self.git_operations.clone();

        task::spawn_blocking(move || {
            // Open existing repository
            let repository = git_operations.open_repository(&local_path)?;

            // Fetch updates
            let git_auth_config = match &repo_config.auth_config {
                AuthConfig::None => None,
                AuthConfig::SshKey { key_path, passphrase } => Some(GitAuthConfig {
                    ssh_key_path: Some(key_path.clone()),
                    ssh_passphrase: passphrase.clone(),
                    username: None,
                    password: None,
                }),
                AuthConfig::Token { token, username } => Some(GitAuthConfig {
                    ssh_key_path: None,
                    ssh_passphrase: None,
                    username: username.clone(),
                    password: Some(token.clone()),
                }),
                AuthConfig::Basic { username, password } => Some(GitAuthConfig {
                    ssh_key_path: None,
                    ssh_passphrase: None,
                    username: Some(username.clone()),
                    password: Some(password.clone()),
                }),
            };

            let fetch_options = crate::git::GitFetchOptions {
                fetch_tags: true,
                prune: false,
                auth: git_auth_config,
                timeout: Some(300), // 5 minutes timeout
            };

            let start_time = std::time::Instant::now();
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    git_operations.fetch(&repository, fetch_options).await
                })
            });
            let duration = start_time.elapsed();

            match result {
                Ok(_) => {
                    // Get updated repository information
                    let branch_info = Self::get_repository_info(&repository)?;
                    let sync_stats = SyncStats {
                        duration,
                        objects_received: 0, // TODO: Get actual stats from git operations
                        bytes_received: 0,
                        pack_size: 0,
                    };

                    Ok(SyncResult {
                        success: true,
                        local_path,
                        default_branch: branch_info.default_branch,
                        available_branches: branch_info.branches,
                        last_commit: branch_info.last_commit,
                        sync_stats,
                        error: None,
                    })
                }
                Err(e) => {
                    warn!("Failed to fetch repository updates {}: {}", repo_config.url, e);
                    Ok(SyncResult {
                        success: false,
                        local_path,
                        default_branch: None,
                        available_branches: vec![],
                        last_commit: None,
                        sync_stats: SyncStats::default(),
                        error: Some(e.to_string()),
                    })
                }
            }
        }).await
        .map_err(|e| CziError::runtime(format!("Task execution failed: {}", e)))?
    }

    /// Get repository information
    fn get_repository_info(_repository: &crate::git::GitRepository) -> Result<RepositoryInfo> {
        // TODO: Implement actual repository info extraction
        // This would use git2 to get branch information, last commit, etc.
        Ok(RepositoryInfo {
            default_branch: Some("main".to_string()),
            branches: vec!["main".to_string(), "develop".to_string()],
            last_commit: Some(CommitInfo {
                hash: "abc123".to_string(),
                message: "Latest commit".to_string(),
                author: "Test Author".to_string(),
                timestamp: Utc::now(),
            }),
        })
    }

    /// Get local path for repository
    fn get_local_path(&self, repo_config: &RepositoryConfiguration) -> Result<PathBuf> {
        let path = if let Some(local_path) = &repo_config.local_path {
            local_path.clone()
        } else {
            // Generate path from URL
            let url_parts = repo_config.url.split('/').collect::<Vec<_>>();
            let repo_name = url_parts.last()
                .and_then(|s| s.strip_suffix(".git"))
                .unwrap_or("repository");

            let host = if repo_config.url.starts_with("git@") {
                repo_config.url.split('@').nth(1)
                    .and_then(|s| s.split(':').next())
            } else {
                repo_config.url.split("://").nth(1)
                    .and_then(|s| s.split('/').next())
            }.unwrap_or("unknown");

            self.cache_dir.join(host).join(repo_name)
        };

        Ok(path)
    }

    /// Remove repository from local cache
    #[instrument(skip(self))]
    pub async fn remove_repository(&self, repo_config: &RepositoryConfiguration) -> Result<()> {
        let local_path = self.get_local_path(repo_config)?;

        if local_path.exists() {
            info!("Removing repository from cache: {}", local_path.display());

            task::spawn_blocking(move || {
                std::fs::remove_dir_all(&local_path)
                    .map_err(|e| CziError::Io(e))
            }).await
        .map_err(|e| CziError::runtime(format!("Task execution failed: {}", e)))??
        } else {
            warn!("Repository not found in cache: {}", local_path.display());
        }

        Ok(())
    }

    /// Get synchronization status for multiple repositories
    pub async fn get_sync_status(&self, repositories: &[RepositoryConfiguration]) -> Result<Vec<SyncStatus>> {
        let mut statuses = Vec::with_capacity(repositories.len());

        for repo_config in repositories {
            let local_path = self.get_local_path(repo_config)?;
            let status = if local_path.exists() && local_path.join(".git").exists() {
                // Check if repository needs update
                match self.check_repository_status(repo_config, &local_path).await {
                    Ok(needs_update) => {
                        let state = if needs_update {
                            RepositorySyncState::NeedsUpdate
                        } else {
                            RepositorySyncState::UpToDate
                        };
                        SyncStatus {
                            repository_id: repo_config.id.clone(),
                            status: state,
                            local_path: local_path.clone(),
                            last_sync: repo_config.last_sync,
                        }
                    }
                    Err(_) => SyncStatus {
                        repository_id: repo_config.id.clone(),
                        status: RepositorySyncState::Error,
                        local_path: local_path.clone(),
                        last_sync: repo_config.last_sync,
                    },
                }
            } else {
                SyncStatus {
                    repository_id: repo_config.id.clone(),
                    status: RepositorySyncState::NotCloned,
                    local_path: local_path.clone(),
                    last_sync: repo_config.last_sync,
                }
            };

            statuses.push(status);
        }

        Ok(statuses)
    }

    /// Check if repository needs update
    async fn check_repository_status(&self, repo_config: &RepositoryConfiguration, local_path: &Path) -> Result<bool> {
        let repo_config = repo_config.clone();
        let local_path = local_path.to_path_buf();
        let git_operations = self.git_operations.clone();

        task::spawn_blocking(move || {
            let repository = git_operations.open_repository(&local_path)?;

            // TODO: Implement actual remote checking
            // For now, assume repository needs update if last sync is older than 1 hour
            let needs_update = repo_config.last_sync
                .map(|last_sync| {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(last_sync);
                    duration.num_hours() >= 1
                })
                .unwrap_or(true);

            Ok(needs_update)
        }).await
        .map_err(|e| CziError::runtime(format!("Task execution failed: {}", e)))?
    }

    /// Legacy method for backward compatibility
    pub fn sync_repository(&self, url: &str, path: &str) -> Result<()> {
        // Create a temporary repo config for compatibility
        let repo_config = RepositoryConfiguration {
            id: format!("legacy_{}", uuid::Uuid::new_v4()),
            name: "Legacy Sync".to_string(),
            url: url.to_string(),
            local_path: Some(PathBuf::from(path)),
            branch: "main".to_string(),
            auth_type: crate::config::AuthType::None,
            auth_config: crate::config::AuthConfig::None,
            last_sync: None,
            status: crate::config::RepositoryStatus::Active,
        };

        // Use new sync method in blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.synchronize_repository(&repo_config))
            .map(|_| ())
    }
}

/// Synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub local_path: PathBuf,
    pub default_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub last_commit: Option<CommitInfo>,
    pub sync_stats: SyncStats,
    pub error: Option<String>,
}

/// Synchronization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub duration: std::time::Duration,
    pub objects_received: u64,
    pub bytes_received: u64,
    pub pack_size: u64,
}

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub default_branch: Option<String>,
    pub branches: Vec<String>,
    pub last_commit: Option<CommitInfo>,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub repository_id: String,
    pub status: RepositorySyncState,
    pub local_path: PathBuf,
    pub last_sync: Option<DateTime<Utc>>,
}

/// Repository synchronization state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositorySyncState {
    NotCloned,
    NeedsUpdate,
    UpToDate,
    Error,
}

impl Default for SyncStats {
    fn default() -> Self {
        Self {
            duration: std::time::Duration::from_secs(0),
            objects_received: 0,
            bytes_received: 0,
            pack_size: 0,
        }
    }
}

impl Default for RepositorySyncService {
    fn default() -> Self {
        Self::new("./cache").expect("Failed to create default sync service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_service_creation() {
        let service = RepositorySyncService::new("./test_cache");
        assert!(service.is_ok(), "Should create sync service with valid cache directory");
    }

    #[test]
    fn test_local_path_generation() {
        let service = RepositorySyncService::new("./test_cache").unwrap();

        let repo_config = RepositoryConfiguration {
            id: "test_repo".to_string(),
            name: "Test Repository".to_string(),
            url: "https://github.com/user/repo.git".to_string(),
            local_path: None,
            branch: "main".to_string(),
            auth_type: crate::config::AuthType::None,
            auth_config: crate::config::AuthConfig::None,
            last_sync: None,
            status: crate::config::RepositoryStatus::Active,
        };

        let path = service.get_local_path(&repo_config).unwrap();
        assert!(path.to_string_lossy().contains("github.com"));
        assert!(path.to_string_lossy().contains("repo"));
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult {
            success: true,
            local_path: PathBuf::from("/tmp/repo"),
            default_branch: Some("main".to_string()),
            available_branches: vec!["main".to_string()],
            last_commit: None,
            sync_stats: SyncStats::default(),
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.default_branch.unwrap(), "main");
        assert_eq!(result.available_branches.len(), 1);
    }

    #[test]
    fn test_sync_status_creation() {
        let status = SyncStatus {
            repository_id: "test_repo".to_string(),
            status: RepositorySyncState::UpToDate,
            local_path: PathBuf::from("/tmp/repo"),
            last_sync: Some(Utc::now()),
        };

        assert_eq!(status.repository_id, "test_repo");
        assert_eq!(status.status, RepositorySyncState::UpToDate);
        assert!(status.last_sync.is_some());
    }
}