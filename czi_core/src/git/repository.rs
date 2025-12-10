//! Git repository abstraction and management

use crate::Result;
use git2::{Repository, RepositoryOpenFlags, Status, StatusOptions};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for repository operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepositoryConfig {
    /// Repository URL
    pub url: String,
    /// Local path for the repository
    pub path: PathBuf,
    /// Default branch name
    pub default_branch: Option<String>,
    /// Authentication configuration
    pub auth: Option<GitAuthConfig>,
    /// Repository metadata
    pub metadata: RepositoryMetadata,
}

/// Authentication configuration for Git operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAuthConfig {
    /// SSH key path
    pub ssh_key_path: Option<PathBuf>,
    /// SSH passphrase
    pub ssh_passphrase: Option<String>,
    /// Username for HTTPS authentication
    pub username: Option<String>,
    /// Password/token for HTTPS authentication
    pub password: Option<String>,
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    /// Unique identifier for the repository
    pub id: Uuid,
    /// Repository name
    pub name: String,
    /// Repository description
    pub description: Option<String>,
    /// Owner information
    pub owner: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Language detected in the repository
    pub language: Option<String>,
    /// Total lines of code
    pub lines_of_code: Option<u64>,
    /// Repository size in bytes
    pub size_bytes: Option<u64>,
}

/// Repository status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Current branch name
    pub current_branch: String,
    /// Current commit hash
    pub current_commit: String,
    /// Number of modified files
    pub modified_files: usize,
    /// Number of added files
    pub added_files: usize,
    /// Number of deleted files
    pub deleted_files: usize,
    /// Number of untracked files
    pub untracked_files: usize,
    /// Is the repository clean (no uncommitted changes)
    pub is_clean: bool,
    /// Has unpushed commits
    pub has_unpushed_commits: bool,
    /// Is ahead of remote
    pub ahead_commits: usize,
    /// Is behind remote
    pub behind_commits: usize,
}

/// Wrapper for git2::Repository providing high-level operations
pub struct GitRepository {
    repository: Repository,
    config: GitRepositoryConfig,
}

impl GitRepository {
    /// Open a repository from the given path
    pub fn open(path: &Path) -> Result<Self> {
        let repository = Repository::open_ext(path, RepositoryOpenFlags::all(), Vec::<&Path>::new())
            .map_err(|e| crate::CziError::git(format!("Failed to open repository: {}", e)))?;

        let config = Self::create_config_from_repo(&repository, path)?;

        Ok(Self {
            repository,
            config,
        })
    }

    /// Create a new repository from git2::Repository and config
    pub fn from_repository(repository: Repository, config: GitRepositoryConfig) -> Self {
        Self {
            repository,
            config,
        }
    }

    /// Get repository configuration
    pub fn config(&self) -> &GitRepositoryConfig {
        &self.config
    }

    /// Get mutable repository configuration
    pub fn config_mut(&mut self) -> &mut GitRepositoryConfig {
        &mut self.config
    }

    /// Get the underlying git2::Repository
    pub fn inner(&self) -> &Repository {
        &self.repository
    }

    /// Get mutable reference to the underlying git2::Repository
    pub fn inner_mut(&mut self) -> &mut Repository {
        &mut self.repository
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Get the current working directory
    pub fn workdir(&self) -> Option<&Path> {
        self.repository.workdir()
    }

    /// Get the current HEAD reference
    pub fn head(&self) -> Result<git2::Reference<'_>> {
        self.repository.head()
            .map_err(|e| crate::CziError::git(format!("Failed to get HEAD: {}", e)))
    }

    /// Get the current commit
    pub fn current_commit(&self) -> Result<git2::Commit<'_>> {
        let head = self.head()?;
        head.peel_to_commit()
            .map_err(|e| crate::CziError::git(format!("Failed to peel to commit: {}", e)))
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.head()?;
        match head.shorthand() {
            Some(name) => Ok(name.to_string()),
            None => {
                // HEAD is detached, try to get commit hash
                let commit = self.current_commit()?;
                Ok(format!("HEAD-{}", commit.id()))
            }
        }
    }

    /// Get repository status
    pub fn get_status(&self) -> Result<RepositoryStatus> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_ignored(false)
            .recurse_untracked_dirs(true);

        let statuses = self.repository.statuses(Some(&mut opts))
            .map_err(|e| crate::CziError::git(format!("Failed to get repository status: {}", e)))?;

        let mut modified_files = 0;
        let mut added_files = 0;
        let mut deleted_files = 0;
        let mut untracked_files = 0;

        for entry in statuses.iter() {
            let status = entry.status();
            if status.contains(Status::WT_MODIFIED) || status.contains(Status::INDEX_MODIFIED) {
                modified_files += 1;
            }
            if status.contains(Status::WT_NEW) || status.contains(Status::INDEX_NEW) {
                added_files += 1;
            }
            if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
                deleted_files += 1;
            }
            if status.contains(Status::WT_NEW) {
                untracked_files += 1;
            }
        }

        let is_clean = statuses.is_empty();
        let current_branch = self.current_branch().unwrap_or_else(|_| "unknown".to_string());
        let current_commit = self.current_commit()
            .map(|c| c.id().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Check for unpushed commits (simplified - in practice would compare with remote)
        let has_unpushed_commits = false; // Placeholder
        let ahead_commits = 0; // Placeholder
        let behind_commits = 0; // Placeholder

        Ok(RepositoryStatus {
            current_branch,
            current_commit,
            modified_files,
            added_files,
            deleted_files,
            untracked_files,
            is_clean,
            has_unpushed_commits,
            ahead_commits,
            behind_commits,
        })
    }

    /// Get all branches in the repository
    pub fn get_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        let branch_refs = self.repository.branches(Some(git2::BranchType::Local))
            .map_err(|e| crate::CziError::git(format!("Failed to get branches: {}", e)))?;

        for branch_result in branch_refs {
            let (branch, _branch_type) = branch_result.map_err(|e| crate::CziError::git(format!("Error processing branch: {}", e)))?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    /// Get all remote URLs
    pub fn get_remotes(&self) -> Result<Vec<String>> {
        let mut remotes = Vec::new();
        let remote_names = self.repository.remotes()
            .map_err(|e| crate::CziError::git(format!("Failed to get remotes: {}", e)))?;

        for i in 0..remote_names.len() {
            if let Some(name) = remote_names.get(i) {
                if let Ok(remote) = self.repository.find_remote(name) {
                    if let Some(url) = remote.url() {
                        remotes.push(url.to_string());
                    }
                }
            }
        }

        Ok(remotes)
    }

    /// Create configuration from existing repository
    fn create_config_from_repo(repository: &Repository, path: &Path) -> Result<GitRepositoryConfig> {
        let url = Self::extract_repo_url(repository).unwrap_or_else(|| "unknown".to_string());
        let default_branch = Self::get_default_branch(repository)?;

        let metadata = RepositoryMetadata {
            id: Uuid::new_v4(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            description: None,
            owner: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            language: None,
            lines_of_code: None,
            size_bytes: None,
        };

        Ok(GitRepositoryConfig {
            url,
            path: path.to_path_buf(),
            default_branch,
            auth: None,
            metadata,
        })
    }

    /// Extract repository URL from git config
    fn extract_repo_url(repository: &Repository) -> Option<String> {
        let remote = repository.find_remote("origin").ok()?;
        remote.url().map(|url| url.to_string())
    }

    /// Get default branch name
    fn get_default_branch(repository: &Repository) -> Result<Option<String>> {
        let remote = match repository.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => return Ok(None),
        };

        let default_branch = match remote.default_branch() {
            Ok(branch) => branch,
            Err(_) => return Ok(None),
        };

        // Convert Buf to String
        let branch_str = String::from_utf8_lossy(&default_branch).to_string();

        // Remove "refs/heads/" prefix if present
        let branch_name = if branch_str.starts_with("refs/heads/") {
            &branch_str["refs/heads/".len()..]
        } else {
            &branch_str
        };

        Ok(Some(branch_name.to_string()))
    }

    /// Update repository metadata
    pub fn update_metadata(&mut self) -> Result<()> {
        self.config.metadata.updated_at = Utc::now();
        // In a real implementation, you would analyze the repository here
        // to extract language, lines of code, etc.
        Ok(())
    }

    /// Check if repository is a bare repository
    pub fn is_bare(&self) -> bool {
        self.repository.is_bare()
    }

    /// Check if repository is empty (no commits)
    pub fn is_empty(&self) -> Result<bool> {
        let head_ref = self.repository.head();
        match head_ref {
            Ok(_) => Ok(false),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch || e.code() == git2::ErrorCode::NotFound => Ok(true),
            Err(e) => Err(crate::CziError::git(format!("Error checking if repository is empty: {}", e))),
        }
    }

    /// Get repository statistics
    pub fn get_statistics(&self) -> Result<RepositoryStatistics> {
        let total_commits = self.get_total_commits()?;
        let total_branches = self.get_branches()?.len();
        let total_tags = self.get_total_tags()?;

        Ok(RepositoryStatistics {
            total_commits,
            total_branches,
            total_tags,
            total_files: self.count_files()?,
        })
    }

    /// Get total number of commits
    fn get_total_commits(&self) -> Result<u64> {
        let mut revwalk = self.repository.revwalk()
            .map_err(|e| crate::CziError::git(format!("Failed to create revwalk: {}", e)))?;

        revwalk.push_head()
            .map_err(|e| crate::CziError::git(format!("Failed to push HEAD to revwalk: {}", e)))?;

        Ok(revwalk.count() as u64)
    }

    /// Get total number of tags
    fn get_total_tags(&self) -> Result<usize> {
        let tag_names = self.repository.tag_names(None)
            .map_err(|e| crate::CziError::git(format!("Failed to get tag names: {}", e)))?;

        Ok(tag_names.len())
    }

    /// Count tracked files in the repository
    fn count_files(&self) -> Result<usize> {
        let head_tree = match self.repository.head() {
            Ok(head_ref) => {
                let head_commit = head_ref.peel_to_commit()
                    .map_err(|e| crate::CziError::git(format!("Failed to peel to commit: {}", e)))?;
                head_commit.tree()
                    .map_err(|e| crate::CziError::git(format!("Failed to get tree: {}", e)))?
            }
            Err(_) => {
                // Repository has no commits, return 0
                return Ok(0);
            }
        };

        Ok(self.count_tree_files(&head_tree))
    }

    /// Recursively count files in a tree
    fn count_tree_files(&self, tree: &git2::Tree) -> usize {
        let mut file_count = 0;

        for entry in tree.iter() {
            match entry.kind() {
                Some(git2::ObjectType::Blob) => file_count += 1,
                Some(git2::ObjectType::Tree) => {
                    if let Ok(subtree) = entry.to_object(&self.repository).and_then(|obj| obj.into_tree().map_err(|_| git2::Error::from_str("Failed to convert to tree"))) {
                        file_count += self.count_tree_files(&subtree);
                    }
                }
                _ => {}
            }
        }

        file_count
    }
}

/// Repository statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatistics {
    /// Total number of commits
    pub total_commits: u64,
    /// Total number of branches
    pub total_branches: usize,
    /// Total number of tags
    pub total_tags: usize,
    /// Total number of files in the current HEAD
    pub total_files: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use git2::Repository;

    #[test]
    fn test_repository_init() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a bare repository
        let repo = Repository::init(repo_path).unwrap();
        assert!(repo.is_bare());

        // Open with our wrapper
        let wrapper = GitRepository::open(repo_path);
        assert!(wrapper.is_ok());
    }

    #[test]
    fn test_repository_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let _repo = Repository::init(repo_path).unwrap();
        let wrapper = GitRepository::open(repo_path).unwrap();

        let config = wrapper.config();
        assert_eq!(config.path, repo_path);
        assert!(!config.metadata.id.to_string().is_empty());
    }

    #[test]
    fn test_repository_status() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let _repo = Repository::init(repo_path).unwrap();
        let wrapper = GitRepository::open(repo_path).unwrap();

        let status = wrapper.get_status();
        assert!(status.is_ok());

        let status_info = status.unwrap();
        assert!(status_info.is_clean); // New repository should be clean
        assert_eq!(status_info.modified_files, 0);
        assert_eq!(status_info.untracked_files, 0);
    }
}