//! Git operations wrapper for repository management and file system abstraction

pub mod repository;
pub mod operations;
pub mod diff;
pub mod blame;
pub mod utils;

pub use repository::{GitRepository, GitRepositoryConfig, RepositoryStatus, GitAuthConfig};
pub use operations::{GitOperations, CloneOptions, GitFetchOptions, PullOptions};
pub use diff::{DiffAnalyzer, FileChange, ChangeType, Hunk};
pub use blame::{BlameAnalyzer, BlameHunk, BlameInfo};
pub use utils::{GitUtils, GitUrl};

use crate::Result;

/// Main Git wrapper providing high-level operations
pub struct GitWrapper {
    operations: GitOperations,
}

impl GitWrapper {
    /// Create a new Git wrapper with default configuration
    pub fn new() -> Result<Self> {
        Ok(Self {
            operations: GitOperations::new()?,
        })
    }

    /// Create a Git wrapper with custom operations
    pub fn with_operations(operations: GitOperations) -> Self {
        Self { operations }
    }

    /// Get reference to the underlying operations
    pub fn operations(&self) -> &GitOperations {
        &self.operations
    }

    /// Get mutable reference to the underlying operations
    pub fn operations_mut(&mut self) -> &mut GitOperations {
        &mut self.operations
    }

    /// Open a repository from a path
    pub fn open_repository(&self, path: &std::path::Path) -> Result<GitRepository> {
        self.operations.open_repository(path)
    }

    /// Clone a repository from URL
    pub async fn clone_repository(&self, url: &str, path: &std::path::Path) -> Result<GitRepository> {
        self.operations.clone_repository(url, path, CloneOptions::default()).await
    }

    /// Initialize a new repository
    pub fn init_repository(&self, path: &std::path::Path, bare: bool) -> Result<GitRepository> {
        self.operations.init_repository(path, bare)
    }
}

impl Default for GitWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create Git wrapper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_wrapper_creation() {
        let wrapper = GitWrapper::new();
        assert!(wrapper.is_ok());
    }

    #[test]
    fn test_repository_init() {
        let wrapper = GitWrapper::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = wrapper.init_repository(repo_path, false);
        assert!(repo.is_ok());

        // Verify we can open the repository again
        let reopened = wrapper.open_repository(repo_path);
        assert!(reopened.is_ok());
    }
}