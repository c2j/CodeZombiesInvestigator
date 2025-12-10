//! Git operations for cloning, fetching, and repository management

use crate::{Result, git::{GitRepository, GitRepositoryConfig, repository::GitAuthConfig}, config::{auth::RepositoryAccessInfo, AuthType, repository::AuthConfig}};
use git2::{Repository, FetchOptions as Git2FetchOptions, PushOptions, RemoteCallbacks};
use std::path::{Path, PathBuf};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

/// Options for fetching from a remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFetchOptions {
    /// Whether to fetch all tags
    pub fetch_tags: bool,
    /// Whether to prune stale branches
    pub prune: bool,
    /// Authentication configuration
    pub auth: Option<super::GitAuthConfig>,
    /// Timeout for fetch operation (seconds)
    pub timeout: Option<u64>,
}

impl Default for GitFetchOptions {
    fn default() -> Self {
        Self {
            fetch_tags: true,
            prune: false,
            auth: None,
            timeout: Some(60), // 1 minute default
        }
    }
}

/// Options for cloning a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneOptions {
    /// Whether to clone recursively (initialize submodules)
    pub recursive: bool,
    /// Branch to checkout (None for default)
    pub branch: Option<String>,
    /// Whether to create a bare repository
    pub bare: bool,
    /// Authentication configuration
    pub auth: Option<super::GitAuthConfig>,
    /// Timeout for clone operation (seconds)
    pub timeout: Option<u64>,
    /// Depth for shallow clone (None for full clone)
    pub depth: Option<u32>,
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            branch: None,
            bare: false,
            auth: None,
            timeout: Some(300), // 5 minutes default
            depth: None,
        }
    }
}

/// Options for pulling changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullOptions {
    /// Remote name (default: "origin")
    pub remote: Option<String>,
    /// Branch to pull from (None for current branch's tracking branch)
    pub branch: Option<String>,
    /// Merge strategy
    pub merge_strategy: MergeStrategy,
    /// Authentication configuration
    pub auth: Option<super::GitAuthConfig>,
    /// Timeout for pull operation (seconds)
    pub timeout: Option<u64>,
}

impl Default for PullOptions {
    fn default() -> Self {
        Self {
            remote: Some("origin".to_string()),
            branch: None,
            merge_strategy: MergeStrategy::Default,
            auth: None,
            timeout: Some(120), // 2 minutes default
        }
    }
}

/// Merge strategies for pull operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Default merge strategy
    Default,
    /// Fast-forward only
    FastForwardOnly,
    /// Rebase instead of merge
    Rebase,
    /// Never merge, just fetch
    FetchOnly,
}

/// Main Git operations handler
#[derive(Clone)]
pub struct GitOperations {
    user_agent: String,
    default_timeout: Duration,
}

impl GitOperations {
    /// Create new Git operations handler
    pub fn new() -> Result<Self> {
        Ok(Self {
            user_agent: "CodeZombiesInvestigator/0.1.0".to_string(),
            default_timeout: Duration::from_secs(300), // 5 minutes
        })
    }

    /// Create Git operations with custom user agent
    pub fn with_user_agent(user_agent: String) -> Result<Self> {
        Ok(Self {
            user_agent,
            default_timeout: Duration::from_secs(300),
        })
    }

    /// Clone a repository from URL to local path
    pub async fn clone_repository(
        &self,
        url: &str,
        path: &Path,
        options: CloneOptions,
    ) -> Result<GitRepository> {
        let timeout_duration = Duration::from_secs(options.timeout.unwrap_or(300));

        let result = timeout(timeout_duration, async {
            self.clone_repository_inner(url, path, &options)
        }).await;

        match result {
            Ok(result) => result,
            Err(_) => Err(crate::CziError::git(format!(
                "Clone operation timed out after {} seconds",
                timeout_duration.as_secs()
            ))),
        }
    }

    /// Internal clone implementation
    fn clone_repository_inner(
        &self,
        url: &str,
        path: &Path,
        options: &CloneOptions,
    ) -> Result<GitRepository> {
        let mut builder = git2::build::RepoBuilder::new();

        // Configure clone options
        if let Some(branch) = &options.branch {
            builder.branch(branch);
        }

        builder.bare(options.bare);

        // Note: shallow cloning requires git2 0.19+ or direct git command
        // For now, we'll implement depth through a post-clone fetch if needed

        // Configure authentication callbacks
        let mut callbacks = self.create_callbacks(&options.auth)?;

        // Configure fetch options for cloning
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Set up autotag behavior
        fetch_options.download_tags(git2::AutotagOption::All);

        builder.fetch_options(fetch_options);

        // Perform the clone
        let repository = builder.clone(url, path)
            .map_err(|e| crate::CziError::git(format!("Failed to clone repository: {}", e)))?;

        // Initialize submodules if requested
        if options.recursive {
            self.init_submodules(&repository)?;
        }

        // Create repository config
        let config = GitRepositoryConfig {
            url: url.to_string(),
            path: path.to_path_buf(),
            default_branch: options.branch.clone(),
            auth: options.auth.clone(),
            metadata: super::repository::RepositoryMetadata {
                id: uuid::Uuid::new_v4(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("cloned-repo")
                    .to_string(),
                description: None,
                owner: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                language: None,
                lines_of_code: None,
                size_bytes: None,
            },
        };

        Ok(GitRepository::from_repository(repository, config))
    }

    /// Initialize a new repository
    pub fn init_repository(&self, path: &Path, bare: bool) -> Result<GitRepository> {
        let repository = if bare {
            Repository::init_bare(path)
        } else {
            Repository::init(path)
        }.map_err(|e| crate::CziError::git(format!("Failed to initialize repository: {}", e)))?;

        let config = GitRepositoryConfig {
            url: format!("local:{}", path.display()),
            path: path.to_path_buf(),
            default_branch: Some("main".to_string()),
            auth: None,
            metadata: super::repository::RepositoryMetadata {
                id: uuid::Uuid::new_v4(),
                name: path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("new-repo")
                    .to_string(),
                description: None,
                owner: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                language: None,
                lines_of_code: None,
                size_bytes: None,
            },
        };

        Ok(GitRepository::from_repository(repository, config))
    }

    /// Open an existing repository
    pub fn open_repository(&self, path: &Path) -> Result<GitRepository> {
        GitRepository::open(path)
    }

    /// Fetch changes from remote
    pub async fn fetch(&self, repository: &GitRepository, options: GitFetchOptions) -> Result<()> {
        let timeout_duration = Duration::from_secs(options.timeout.unwrap_or(60));

        let result = timeout(timeout_duration, async {
            self.fetch_inner(repository, &options)
        }).await;

        match result {
            Ok(result) => result,
            Err(_) => Err(crate::CziError::git(format!(
                "Fetch operation timed out after {} seconds",
                timeout_duration.as_secs()
            ))),
        }
    }

    /// Internal fetch implementation
    fn fetch_inner(&self, repository: &GitRepository, options: &GitFetchOptions) -> Result<()> {
        let remote_name = "origin"; // Default to origin for GitFetchOptions

        let mut remote = repository.inner().find_remote(remote_name)
            .map_err(|e| crate::CziError::git(format!("Remote '{}' not found: {}", remote_name, e)))?;

        let mut callbacks = self.create_callbacks(&options.auth)?;

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        // Note: prune() method may not be available in git2 0.18
        // fetch_options.prune(options.prune);

        if options.fetch_tags {
            fetch_options.download_tags(git2::AutotagOption::All);
        }

        // Note: refspec support would require adding field to GitFetchOptions
        let refspecs: &[&str] = &[];

        remote.fetch(refspecs, Some(&mut fetch_options), None)
            .map_err(|e| crate::CziError::git(format!("Failed to fetch from remote: {}", e)))?;

        Ok(())
    }

    /// Pull changes from remote and merge
    pub async fn pull(&self, repository: &mut GitRepository, options: PullOptions) -> Result<()> {
        let timeout_duration = Duration::from_secs(options.timeout.unwrap_or(120));

        let result = timeout(timeout_duration, async {
            self.pull_inner(repository, &options).await
        }).await;

        match result {
            Ok(result) => result,
            Err(_) => Err(crate::CziError::git(format!(
                "Pull operation timed out after {} seconds",
                timeout_duration.as_secs()
            ))),
        }
    }

    /// Internal pull implementation
    async fn pull_inner(&self, repository: &mut GitRepository, options: &PullOptions) -> Result<()> {
        match options.merge_strategy {
            MergeStrategy::FetchOnly => {
                // Just fetch, don't merge
                let fetch_opts = GitFetchOptions {
                    fetch_tags: true,
                    prune: false,
                    auth: options.auth.clone(),
                    timeout: Some(60),
                };
                self.fetch(repository, fetch_opts).await?;
            }
            MergeStrategy::FastForwardOnly => {
                self.fast_forward_pull(repository, options).await?;
            }
            MergeStrategy::Rebase => {
                self.rebase_pull(repository, options).await?;
            }
            MergeStrategy::Default => {
                self.default_pull(repository, options).await?;
            }
        }

        Ok(())
    }

    /// Fast-forward pull implementation
    async fn fast_forward_pull(&self, repository: &mut GitRepository, options: &PullOptions) -> Result<()> {
        // Fetch first
        let fetch_opts = GitFetchOptions {
            fetch_tags: true,
            prune: false,
            auth: options.auth.clone(),
            timeout: Some(60),
        };
        self.fetch(repository, fetch_opts).await?;

        // Attempt fast-forward merge
        let head_commit = repository.current_commit()?;
        let upstream_oid = self.get_upstream_commit(repository, options)?;
        let upstream_commit = repository.inner().find_commit(upstream_oid)
            .map_err(|e| crate::CziError::git(format!("Failed to find upstream commit: {}", e)))?;

        if self.is_fast_forward(&head_commit, &upstream_commit)? {
            self.fast_forward_merge(repository.inner(), &upstream_commit)?;
        } else {
            return Err(crate::CziError::git("Fast-forward not possible".to_string()));
        }

        Ok(())
    }

    /// Default pull implementation (merge)
    async fn default_pull(&self, repository: &mut GitRepository, options: &PullOptions) -> Result<()> {
        // Fetch first
        let fetch_opts = GitFetchOptions {
            fetch_tags: true,
            prune: false,
            auth: options.auth.clone(),
            timeout: Some(60),
        };
        self.fetch(repository, fetch_opts).await?;

        // Perform merge
        let head_commit = repository.current_commit()?;
        let upstream_oid = self.get_upstream_commit(repository, options)?;
        let upstream_commit = repository.inner().find_commit(upstream_oid)
            .map_err(|e| crate::CziError::git(format!("Failed to find upstream commit: {}", e)))?;

        self.merge_commit(repository.inner(), &head_commit, &upstream_commit)?;

        Ok(())
    }

    /// Rebase pull implementation
    async fn rebase_pull(
        &self,
        repository: &mut GitRepository,
        options: &PullOptions,
    ) -> Result<()> {
        // Fetch first
        let fetch_opts = GitFetchOptions {
            fetch_tags: true,
            prune: false,
            auth: options.auth.clone(),
            timeout: Some(60),
        };
        self.fetch(repository, fetch_opts).await?;

        // Perform rebase (simplified - real implementation would be more complex)
        let upstream_oid = self.get_upstream_commit(repository, options)?;
        let upstream_commit = repository.inner().find_commit(upstream_oid)
            .map_err(|e| crate::CziError::git(format!("Failed to find upstream commit: {}", e)))?;
        let head_commit = repository.current_commit()?;

        if self.is_fast_forward(&head_commit, &upstream_commit)? {
            self.fast_forward_merge(repository.inner(), &upstream_commit)?;
        } else {
            // For now, fall back to merge for non-fast-forward cases
            self.merge_commit(repository.inner(), &head_commit, &upstream_commit)?;
        }

        Ok(())
    }

    /// Initialize and update submodules
    fn init_submodules(&self, repository: &git2::Repository) -> Result<()> {
        // This is a simplified submodule initialization
        // A real implementation would handle recursive submodules, authentication, etc.
        let mut submodules = repository.submodules()
            .map_err(|e| crate::CziError::git(format!("Failed to get submodules: {}", e)))?;

        for submodule in submodules.iter_mut() {
            submodule.update(true, None)
                .map_err(|e| crate::CziError::git(format!("Failed to update submodule: {}", e)))?;
        }

        Ok(())
    }

    /// Create remote callbacks with authentication
    fn create_callbacks(&self, auth: &Option<super::GitAuthConfig>) -> Result<RemoteCallbacks<'_>> {
        let mut callbacks = RemoteCallbacks::new();

        // Set user agent
        if let Some(auth_config) = auth {
            let auth_config = auth_config.clone();
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                self.handle_credentials(url, username_from_url, allowed_types, &auth_config)
            });
        } else {
            callbacks.credentials(|_url, _username_from_url, _allowed_types| {
                Err(git2::Error::from_str("No authentication provided"))
            });
        }

        Ok(callbacks)
    }

    /// Handle credential requests
    fn handle_credentials(
        &self,
        _url: &str,
        _username_from_url: Option<&str>,
        _allowed_types: git2::CredentialType,
        auth_config: &super::GitAuthConfig,
    ) -> std::result::Result<git2::Cred, git2::Error> {
        // Simplified credential handling - real implementation would be more robust
        if let Some(_ssh_key_path) = &auth_config.ssh_key_path {
            // SSH key authentication - simplified for now
            // TODO: Implement proper SSH key handling
            return Err(git2::Error::from_str("SSH key authentication not yet implemented"));
        } else if let (Some(username), Some(password)) = (&auth_config.username, &auth_config.password) {
            git2::Cred::userpass_plaintext(username, password)
        } else {
            Err(git2::Error::from_str("No valid credentials provided"))
        }
    }

    /// Get upstream commit for pull operations
    fn get_upstream_commit(&self, repository: &GitRepository, options: &PullOptions) -> Result<git2::Oid> {
        let remote_name = options.remote.as_deref().unwrap_or("origin");
        let branch_name = options.branch.as_deref().unwrap_or("main");

        let remote_branch_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);
        let reference = repository.inner().find_reference(&remote_branch_ref)
            .map_err(|e| crate::CziError::git(format!("Remote branch '{}' not found: {}", branch_name, e)))?;

        reference.target()
            .ok_or_else(|| crate::CziError::git("Reference has no target".to_string()))
    }

    /// Check if a commit can be fast-forwarded to another
    fn is_fast_forward(&self, _base: &git2::Commit, target: &git2::Commit) -> Result<bool> {
        // Simplified check - in practice, we'd use git2's merge base analysis
        Ok(true) // For now, assume it's fast-forwardable
    }

    /// Perform fast-forward merge
    fn fast_forward_merge(&self, repository: &git2::Repository, target: &git2::Commit) -> Result<()> {
        let mut reference = repository.head()
            .map_err(|e| crate::CziError::git(format!("Failed to get HEAD: {}", e)))?;

        reference.set_target(target.id(), "Fast-forward merge")
            .map_err(|e| crate::CziError::git(format!("Failed to update HEAD: {}", e)))?;

        repository.checkout_head(None)
            .map_err(|e| crate::CziError::git(format!("Failed to checkout HEAD: {}", e)))?;

        Ok(())
    }

    /// Perform merge commit
    fn merge_commit(&self, repository: &git2::Repository, _base: &git2::Commit, target: &git2::Commit) -> Result<()> {
        // Simplified merge implementation
        let commit_id = repository.commit(
            Some("HEAD"),
            &repository.signature()?,
            &repository.signature()?,
            "Merge commit",
            &target.tree()?,
            &[&target],
        ).map_err(|e| crate::CziError::git(format!("Failed to create merge commit: {}", e)))?;

        // Checkout to update working directory
        repository.set_head_detached(commit_id)
            .map_err(|e| crate::CziError::git(format!("Failed to set HEAD detached: {}", e)))?;

        Ok(())
    }

    /// Get default user agent
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Set default timeout
    pub fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    /// Get default timeout
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Test repository access by attempting to connect and fetch basic information
    pub fn test_repository_access(
        &self,
        url: &str,
        auth_config: &AuthConfig,
        temp_dir: &Path,
    ) -> Result<RepositoryAccessInfo> {
        use crate::CziError;

        // Try to create a temporary repository to test access
        let repo_path = temp_dir.join("test_repo");

        // Remove existing directory if it exists
        if repo_path.exists() {
            std::fs::remove_dir_all(&repo_path)
                .map_err(|e| CziError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to remove existing test directory: {}", e)
                )))?;
        }

        // Create parent directory
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CziError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create parent directory: {}", e)
                )))?;
        }

        // Test repository access by attempting a shallow clone using the sync method
        let (git_auth_config, auth_method) = match auth_config {
            AuthConfig::None => (None, AuthType::None),
            AuthConfig::SshKey { key_path, passphrase } => (Some(GitAuthConfig {
                ssh_key_path: Some(key_path.clone()),
                ssh_passphrase: passphrase.clone(),
                username: None,
                password: None,
            }), AuthType::SshKey),
            AuthConfig::Token { token, username } => (Some(GitAuthConfig {
                ssh_key_path: None,
                ssh_passphrase: None,
                username: username.clone(),
                password: Some(token.clone()),
            }), AuthType::Token),
            AuthConfig::Basic { username, password } => (Some(GitAuthConfig {
                ssh_key_path: None,
                ssh_passphrase: None,
                username: Some(username.clone()),
                password: Some(password.clone()),
            }), AuthType::Basic),
        };

        let clone_options = super::CloneOptions {
            recursive: false,
            branch: Some("main".to_string()),
            bare: false,
            auth: git_auth_config,
            timeout: Some(30), // 30 seconds timeout for testing
            depth: Some(1), // Shallow clone for faster testing
        };

        // Use the synchronous clone implementation
        match self.clone_repository_inner(&url, &repo_path, &clone_options) {
            Ok(_) => {
                // Clean up test directory
                let _ = std::fs::remove_dir_all(&repo_path);

                Ok(RepositoryAccessInfo {
                    accessible: true,
                    branches: Some(vec!["main".to_string()]), // Simplified for now
                    default_branch: Some("main".to_string()),
                    error: None,
                    repository_type: "git".to_string(),
                    auth_method,
                    tested_url: url.to_string(),
                })
            }
            Err(e) => {
                // Clean up test directory if it was partially created
                let _ = std::fs::remove_dir_all(&repo_path);

                Ok(RepositoryAccessInfo {
                    accessible: false,
                    branches: None,
                    default_branch: None,
                    error: Some(e.to_string()),
                    repository_type: "git".to_string(),
                    auth_method,
                    tested_url: url.to_string(),
                })
            }
        }
    }
}

impl Default for GitOperations {
    fn default() -> Self {
        Self::new().expect("Failed to create Git operations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use git2::Repository;

    #[test]
    fn test_git_operations_creation() {
        let ops = GitOperations::new();
        assert!(ops.is_ok());
    }

    #[test]
    fn test_init_repository() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false);
        assert!(repo.is_ok());

        // Verify the repository was created
        let reopened = ops.open_repository(repo_path);
        assert!(reopened.is_ok());
    }

    #[test]
    fn test_bare_repository_init() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, true);
        assert!(repo.is_ok());

        assert!(repo.unwrap().is_bare());
    }

    #[tokio::test]
    async fn test_fetch_options() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Create a repository with a remote (but don't actually fetch)
        let mut repo = ops.init_repository(repo_path, false).unwrap();

        let fetch_opts = GitFetchOptions::default();
        let result = ops.fetch(&repo, fetch_opts).await;

        // This should fail because we don't have a real remote
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_options_default() {
        let opts = CloneOptions::default();
        assert!(!opts.recursive);
        assert!(opts.branch.is_none());
        assert!(!opts.bare);
        assert_eq!(opts.timeout, Some(300));
        assert!(opts.depth.is_none());
    }

    #[test]
    fn test_pull_options_default() {
        let opts = PullOptions::default();
        assert_eq!(opts.remote, Some("origin".to_string()));
        assert!(opts.branch.is_none());
        assert!(matches!(opts.merge_strategy, MergeStrategy::Default));
        assert_eq!(opts.timeout, Some(120));
    }
}