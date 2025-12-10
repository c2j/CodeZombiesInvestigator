//! Repository configuration types

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::Utc;

/// Repository status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryStatus {
    Active,
    Syncing,
    Error,
    Disabled,
}

/// Repository configuration for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfiguration {
    pub id: String,
    pub name: String,
    pub url: String,
    pub local_path: Option<PathBuf>, // Make optional as per tests
    pub branch: String,
    pub auth_type: AuthType,
    pub auth_config: AuthConfig, // Make required as per tests
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub status: RepositoryStatus,
}

/// Authentication types for repository access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    SshKey,
    Token,
    Basic,
}

/// Authentication configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // Use untagged to match test expectations
pub enum AuthConfig {
    None,
    SshKey {
        key_path: PathBuf,
        passphrase: Option<String>,
    },
    Token {
        token: String,
        username: Option<String>,
    },
    Basic {
        username: String,
        password: String,
    },
}

impl RepositoryConfiguration {
    /// Validate the repository configuration
    pub fn validate(&self) -> Result<()> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err(CziError::validation("Repository name cannot be empty"));
        }

        // Validate URL
        if self.url.trim().is_empty() {
            return Err(CziError::validation("Repository URL cannot be empty"));
        }

        // Validate URL format
        if !self.is_valid_url(&self.url) {
            return Err(CziError::validation("Invalid repository URL format"));
        }

        // Validate branch
        if self.branch.trim().is_empty() {
            return Err(CziError::validation("Branch name cannot be empty"));
        }

        // Validate authentication configuration
        self.validate_auth_config()?;

        Ok(())
    }

    /// Check if URL is valid
    fn is_valid_url(&self, url: &str) -> bool {
        // Basic URL validation for Git repositories
        if url.starts_with("https://") || url.starts_with("http://") {
            // HTTPS/HTTP URL
            url.contains(".git") || url.contains("github.com") || url.contains("gitlab.com")
        } else if url.starts_with("git@") {
            // SSH URL format
            url.contains(":") && (url.contains("/") || url.contains(".git"))
        } else {
            false
        }
    }

    /// Validate authentication configuration
    fn validate_auth_config(&self) -> Result<()> {
        match (&self.auth_type, &self.auth_config) {
            (AuthType::None, AuthConfig::None) => Ok(()),
            (AuthType::SshKey, AuthConfig::SshKey { key_path, .. }) => {
                if key_path.as_os_str().is_empty() {
                    return Err(CziError::validation("SSH key path cannot be empty"));
                }
                Ok(())
            }
            (AuthType::Token, AuthConfig::Token { token, .. }) => {
                if token.trim().is_empty() {
                    return Err(CziError::validation("Authentication token cannot be empty"));
                }
                Ok(())
            }
            (AuthType::Basic, AuthConfig::Basic { username, password }) => {
                if username.trim().is_empty() {
                    return Err(CziError::validation("Username cannot be empty"));
                }
                if password.trim().is_empty() {
                    return Err(CziError::validation("Password cannot be empty"));
                }
                Ok(())
            }
            // Mismatched auth type and config
            (AuthType::SshKey, _) => {
                Err(CziError::validation("SSH authentication requires SSH configuration"))
            }
            (AuthType::Token, _) => {
                Err(CziError::validation("Token authentication requires token configuration"))
            }
            (AuthType::Basic, _) => {
                Err(CziError::validation("Basic authentication requires username and password"))
            }
            _ => {
                // Additional validation for URL/auth type mismatches
                if self.url.starts_with("git@") && !matches!(self.auth_type, AuthType::SshKey) {
                    Err(CziError::validation("SSH URLs require SSH key authentication"))
                } else if (self.url.starts_with("https://") || self.url.starts_with("http://"))
                    && matches!(self.auth_type, AuthType::SshKey) {
                    Err(CziError::validation("HTTPS/HTTP URLs cannot use SSH key authentication"))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Transition repository to a new status
    pub fn transition_to(&mut self, new_status: RepositoryStatus) -> Result<()> {
        // Validate state transitions
        match (self.status, new_status) {
            (RepositoryStatus::Active, RepositoryStatus::Syncing) => {
                self.status = new_status;
                Ok(())
            }
            (RepositoryStatus::Syncing, RepositoryStatus::Active) => {
                self.status = new_status;
                self.last_sync = Some(Utc::now());
                Ok(())
            }
            (RepositoryStatus::Syncing, RepositoryStatus::Error) => {
                self.status = new_status;
                Ok(())
            }
            (RepositoryStatus::Error, RepositoryStatus::Active) => {
                // Allow manual re-enabling
                self.status = new_status;
                Ok(())
            }
            (current, new) => {
                Err(CziError::validation(format!("Invalid state transition from {:?} to {:?}", current, new)))
            }
        }
    }

    /// Check if repository is ready for analysis
    pub fn is_ready_for_analysis(&self) -> bool {
        matches!(self.status, RepositoryStatus::Active)
            && self.last_sync.is_some()
    }

    /// Get repository status as string
    pub fn status_string(&self) -> &'static str {
        match self.status {
            RepositoryStatus::Active => "active",
            RepositoryStatus::Syncing => "syncing",
            RepositoryStatus::Error => "error",
            RepositoryStatus::Disabled => "disabled",
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig::None
    }
}

impl Default for RepositoryStatus {
    fn default() -> Self {
        RepositoryStatus::Active
    }
}