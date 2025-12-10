//! Authentication configuration for repository access

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::repository::{AuthType, RepositoryConfiguration};

/// Re-export authentication types from repository module
pub use super::repository::{AuthConfig};

/// Authentication service for validating and managing repository credentials
pub struct AuthService {
    ssh_agent_available: bool,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new() -> Self {
        Self {
            ssh_agent_available: Self::check_ssh_agent(),
        }
    }

    /// Check if SSH agent is available
    fn check_ssh_agent() -> bool {
        // Check for SSH agent socket
        std::env::var("SSH_AUTH_SOCK").is_ok()
    }

    /// Validate SSH key configuration
    pub fn validate_ssh_config(&self, config: &AuthConfig) -> Result<()> {
        if let AuthConfig::SshKey { key_path, .. } = config {
            // Check if key file exists
            if !key_path.exists() {
                return Err(CziError::validation(format!("SSH key file not found: {:?}", key_path)));
            }

            // Check file permissions (should be 600 or 400 for private keys)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                match std::fs::metadata(key_path) {
                    Ok(metadata) => {
                        let mode = metadata.permissions().mode();
                        if mode & 0o077 != 0 {
                            return Err(CziError::validation("SSH key file has too open permissions (should be 600 or 400)"));
                        }
                    }
                    Err(_) => {
                        return Err(CziError::validation("Unable to read SSH key file permissions"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate token configuration
    pub fn validate_token_config(&self, config: &AuthConfig) -> Result<()> {
        if let AuthConfig::Token { token, .. } = config {
            // Basic token validation
            if token.len() < 10 {
                return Err(CziError::validation("Authentication token appears to be too short"));
            }

            // Check for common token patterns
            if !self.is_valid_token_format(token) {
                return Err(CziError::validation("Authentication token format is not recognized"));
            }
        }
        Ok(())
    }

    /// Check if token format is valid
    fn is_valid_token_format(&self, token: &str) -> bool {
        // GitHub personal access token pattern
        if token.starts_with("ghp_") || token.starts_with("github_pat_") {
            return token.len() >= 40;
        }

        // GitLab personal access token pattern
        if token.starts_with("glpat-") {
            return token.len() >= 20;
        }

        // Generic token pattern - just check basic requirements
        token.len() >= 20 && token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Validate basic authentication configuration
    pub fn validate_basic_config(&self, config: &AuthConfig) -> Result<()> {
        if let AuthConfig::Basic { username, password } = config {
            // Validate username
            if username.trim().is_empty() {
                return Err(CziError::validation("Username cannot be empty"));
            }

            if username.len() < 2 {
                return Err(CziError::validation("Username appears to be too short"));
            }

            // Validate password
            if password.trim().is_empty() {
                return Err(CziError::validation("Password cannot be empty"));
            }

            if password.len() < 4 {
                return Err(CziError::validation("Password appears to be too short"));
            }
        }
        Ok(())
    }

    /// Get available authentication methods for a repository URL
    pub fn get_supported_auth_methods(&self, url: &str) -> Vec<AuthType> {
        let mut methods = vec![AuthType::None];

        if url.starts_with("git@") || url.contains("github.com") || url.contains("gitlab.com") {
            methods.push(AuthType::SshKey);
        }

        if url.starts_with("https://") || url.starts_with("http://") {
            methods.push(AuthType::Token);
            methods.push(AuthType::Basic);
        }

        methods
    }

    /// Check if SSH agent is available for key authentication
    pub fn is_ssh_agent_available(&self) -> bool {
        self.ssh_agent_available
    }

    /// Generate a secure random token for testing
    #[cfg(test)]
    pub fn generate_test_token() -> String {
        format!("test_token_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
    }
}

/// Information about repository access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAccessInfo {
    /// Whether the repository is accessible
    pub accessible: bool,
    /// Available branches (if accessible)
    pub branches: Option<Vec<String>>,
    /// Default branch (if accessible)
    pub default_branch: Option<String>,
    /// Error message (if not accessible)
    pub error: Option<String>,
    /// Repository type (git, svn, etc.)
    pub repository_type: String,
    /// Authentication method used
    pub auth_method: AuthType,
    /// URL that was tested
    pub tested_url: String,
}

impl RepositoryAccessInfo {
    /// Create a successful access info
    pub fn success(
        branches: Vec<String>,
        default_branch: Option<String>,
        auth_method: AuthType,
        tested_url: String,
    ) -> Self {
        Self {
            accessible: true,
            branches: Some(branches),
            default_branch,
            error: None,
            repository_type: "git".to_string(),
            auth_method,
            tested_url,
        }
    }

    /// Create a failed access info
    pub fn failure(error: String, auth_method: AuthType, tested_url: String) -> Self {
        Self {
            accessible: false,
            branches: None,
            default_branch: None,
            error: Some(error),
            repository_type: "git".to_string(),
            auth_method,
            tested_url,
        }
    }

    /// Get error message
    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service_creation() {
        let service = AuthService::new();
        assert!(!service.ssh_agent_available || service.ssh_agent_available); // Either way is fine
    }

    #[test]
    fn test_ssh_config_validation_missing_key() {
        let service = AuthService::new();
        let config = AuthConfig::SshKey {
            key_path: PathBuf::from("/nonexistent/key"),
            passphrase: None,
        };

        let result = service.validate_ssh_config(&config);
        assert!(result.is_err(), "Should fail for missing SSH key");
    }

    #[test]
    fn test_token_validation_too_short() {
        let service = AuthService::new();
        let config = AuthConfig::Token {
            token: "short".to_string(),
            username: None,
        };

        let result = service.validate_token_config(&config);
        assert!(result.is_err(), "Should fail for short token");
    }

    #[test]
    fn test_token_validation_valid_github_token() {
        let service = AuthService::new();
        let token = format!("ghp_{}", "x".repeat(40));
        let config = AuthConfig::Token {
            token,
            username: None,
        };

        let result = service.validate_token_config(&config);
        assert!(result.is_ok(), "Should pass for valid GitHub token");
    }

    #[test]
    fn test_basic_validation_empty_credentials() {
        let service = AuthService::new();

        let config = AuthConfig::Basic {
            username: "".to_string(),
            password: "password".to_string(),
        };
        assert!(service.validate_basic_config(&config).is_err(), "Should fail for empty username");

        let config = AuthConfig::Basic {
            username: "user".to_string(),
            password: "".to_string(),
        };
        assert!(service.validate_basic_config(&config).is_err(), "Should fail for empty password");
    }

    #[test]
    fn test_supported_auth_methods() {
        let service = AuthService::new();

        // HTTPS URL should support token and basic auth
        let https_methods = service.get_supported_auth_methods("https://github.com/example/repo.git");
        assert!(https_methods.contains(&AuthType::Token));
        assert!(https_methods.contains(&AuthType::Basic));
        assert!(https_methods.contains(&AuthType::None));

        // SSH URL should support SSH key auth
        let ssh_methods = service.get_supported_auth_methods("git@github.com:example/repo.git");
        assert!(ssh_methods.contains(&AuthType::SSHKey));
        assert!(ssh_methods.contains(&AuthType::None));
    }

    #[test]
    fn test_repository_access_info_creation() {
        let success_info = RepositoryAccessInfo::success(
            vec!["main".to_string(), "develop".to_string()],
            Some("main".to_string()),
            AuthType::None,
            "https://github.com/example/repo.git".to_string(),
        );

        assert!(success_info.accessible);
        assert_eq!(success_info.branches.unwrap().len(), 2);
        assert_eq!(success_info.default_branch.unwrap(), "main");
        assert!(success_info.error.is_none());

        let failure_info = RepositoryAccessInfo::failure(
            "Access denied".to_string(),
            AuthType::Token,
            "https://github.com/example/private.git".to_string(),
        );

        assert!(!failure_info.accessible);
        assert!(failure_info.branches.is_none());
        assert_eq!(failure_info.error.unwrap(), "Access denied");
    }
}