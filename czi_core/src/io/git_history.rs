//! Git history extraction service
//!
//! This module provides functionality for extracting Git history information
//! including last modification dates, contributor information, and file change
//! statistics for zombie code analysis.

use crate::{Result, CziError};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Contributor information for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorInfo {
    /// Name of the contributor
    pub name: String,

    /// Email address of the contributor
    pub email: String,

    /// Number of commits contributed to this file
    pub commit_count: usize,

    /// First contribution date
    pub first_commit_date: Option<DateTime<Utc>>,

    /// Last contribution date
    pub last_commit_date: Option<DateTime<Utc>>,

    /// Lines added by this contributor
    pub lines_added: usize,

    /// Lines deleted by this contributor
    pub lines_deleted: usize,
}

/// File change statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeStats {
    /// Total number of commits that modified this file
    pub total_commits: usize,

    /// Total lines added across all commits
    pub total_lines_added: usize,

    /// Total lines deleted across all commits
    pub total_lines_deleted: usize,

    /// First commit date for this file
    pub first_commit_date: Option<DateTime<Utc>>,

    /// Last commit date for this file
    pub last_commit_date: Option<DateTime<Utc>>,

    /// List of contributors to this file
    pub contributors: Vec<ContributorInfo>,

    /// File creation commit hash (if available)
    pub creation_commit_hash: Option<String>,

    /// File creation commit message (if available)
    pub creation_commit_message: Option<String>,

    /// Last modification commit hash
    pub last_commit_hash: Option<String>,

    /// Last modification commit message
    pub last_commit_message: Option<String>,
}

/// Git history extraction service
pub struct GitHistoryService {
    /// Path to the Git repository
    repo_path: String,

    /// Cache for file statistics to avoid repeated Git operations
    stats_cache: HashMap<String, FileChangeStats>,

    /// Whether to use verbose Git output
    verbose: bool,
}

impl GitHistoryService {
    /// Create a new Git history service for a repository
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let path = repo_path.as_ref().to_string_lossy().to_string();

        // Verify this is a Git repository
        let git_dir = Path::new(&path).join(".git");
        if !git_dir.exists() {
            return Err(CziError::validation(&format!(
                "Path '{}' is not a Git repository", path
            )));
        }

        Ok(Self {
            repo_path: path,
            stats_cache: HashMap::new(),
            verbose: false,
        })
    }

    /// Create a new Git history service with verbose output
    pub fn with_verbose<P: AsRef<Path>>(repo_path: P, verbose: bool) -> Result<Self> {
        let mut service = Self::new(repo_path)?;
        service.verbose = verbose;
        Ok(service)
    }

    /// Get the last modification date for a file
    pub fn get_last_modified(&mut self, file_path: &str) -> Result<DateTime<Utc>> {
        let stats = self.get_file_stats(file_path)?;
        stats.last_commit_date
            .ok_or_else(|| CziError::not_found(&format!(
                "No Git history found for file: {}", file_path
            )))
    }

    /// Get comprehensive file change statistics
    pub fn get_file_stats(&mut self, file_path: &str) -> Result<FileChangeStats> {
        // Check cache first
        if let Some(stats) = self.stats_cache.get(file_path) {
            return Ok(stats.clone());
        }

        let stats = self.compute_file_stats(file_path)?;
        self.stats_cache.insert(file_path.to_string(), stats.clone());
        Ok(stats)
    }

    /// Get contributor information for a file
    pub fn get_file_contributors(&mut self, file_path: &str) -> Result<Vec<ContributorInfo>> {
        let stats = self.get_file_stats(file_path)?;
        Ok(stats.contributors)
    }

    /// Check if a file has been modified since a given date
    pub fn is_modified_since(&mut self, file_path: &str, since: DateTime<Utc>) -> Result<bool> {
        let last_modified = self.get_last_modified(file_path)?;
        Ok(last_modified > since)
    }

    /// Get files modified within a date range
    pub fn get_files_modified_in_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        file_pattern: Option<&str>,
    ) -> Result<Vec<String>> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_path)
            .args(&["log", "--name-only", "--pretty=format:%ct"])
            .args(&[format!("--since={}", start_date.timestamp())])
            .args(&[format!("--until={}", end_date.timestamp())]);

        if let Some(pattern) = file_pattern {
            cmd.args(&["--", pattern]);
        }

        let output = cmd.output()
            .map_err(|e| CziError::internal(&format!("Failed to execute git log: {}", e)))?;

        if !output.status.success() {
            return Err(CziError::internal(&format!(
                "Git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();
        let mut in_file_list = false;

        for line in stdout.lines() {
            if line.is_empty() {
                in_file_list = true;
                continue;
            }

            if line.parse::<u64>().is_ok() {
                // This is a timestamp, start of new commit
                in_file_list = false;
                continue;
            }

            if in_file_list && !line.trim().is_empty() {
                files.push(line.to_string());
            }
        }

        // Remove duplicates and sort
        files.sort();
        files.dedup();

        Ok(files)
    }

    /// Get commit history for a specific file
    pub fn get_file_commit_history(&self, file_path: &str, limit: Option<usize>) -> Result<Vec<GitCommit>> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_path)
            .args(&["log", "--pretty=format:%H|%ct|%an|%ae|%s"])
            .args(&["--", file_path]);

        if let Some(limit) = limit {
            cmd.args(&[format!("-{}", limit)]);
        }

        let output = cmd.output()
            .map_err(|e| CziError::internal(&format!("Failed to execute git log: {}", e)))?;

        if !output.status.success() {
            return Err(CziError::internal(&format!(
                "Git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            if let Some(parts) = line.split('|').collect::<Vec<_>>().get(0..5) {
                if let (Ok(hash), Ok(timestamp), name, email, message) = (
                    parts[0].parse::<String>(),
                    parts[1].parse::<i64>(),
                    parts[2],
                    parts[3],
                    parts[4]
                ) {
                    let commit = GitCommit {
                        hash,
                        author_name: name.to_string(),
                        author_email: email.to_string(),
                        message: message.to_string(),
                        commit_date: DateTime::from_timestamp(timestamp, 0)
                            .unwrap_or_else(|| Utc::now()),
                    };
                    commits.push(commit);
                }
            }
        }

        Ok(commits)
    }

    /// Clear the statistics cache
    pub fn clear_cache(&mut self) {
        self.stats_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.stats_cache.len(), self.stats_cache.capacity())
    }

    // Private helper methods

    /// Compute file statistics by running Git commands
    fn compute_file_stats(&self, file_path: &str) -> Result<FileChangeStats> {
        let full_path = Path::new(&self.repo_path).join(file_path);
        let relative_path = full_path.strip_prefix(&self.repo_path)
            .map_err(|_| CziError::validation("File path is not within repository"))?;
        let relative_str = relative_path.to_string_lossy();

        // Get commit history with statistics
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&[
                "log", "--numstat", "--pretty=format:%H|%ct|%an|%ae",
                "--", &relative_str
            ])
            .output()
            .map_err(|e| CziError::internal(&format!("Failed to execute git log: {}", e)))?;

        if !output.status.success() {
            return Err(CziError::internal(&format!(
                "Git log failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut stats = FileChangeStats {
            total_commits: 0,
            total_lines_added: 0,
            total_lines_deleted: 0,
            first_commit_date: None,
            last_commit_date: None,
            contributors: Vec::new(),
            creation_commit_hash: None,
            creation_commit_message: None,
            last_commit_hash: None,
            last_commit_message: None,
        };

        let mut contributor_map: HashMap<String, ContributorInfo> = HashMap::new();
        let mut lines = stdout.lines().peekable();
        let mut is_first_commit = true;

        while let Some(line) = lines.next() {
            if line.contains('|') {
                // This is a commit header line
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    let commit_hash = parts[0].to_string();
                    let timestamp = parts[1].parse::<i64>().unwrap_or(0);
                    let author_name = parts[2].to_string();
                    let author_email = parts[3].to_string();

                    let commit_date = DateTime::from_timestamp(timestamp, 0)
                        .unwrap_or_else(|| Utc::now());

                    // Update commit tracking
                    stats.total_commits += 1;

                    if is_first_commit {
                        stats.creation_commit_hash = Some(commit_hash.clone());
                        stats.first_commit_date = Some(commit_date);
                        is_first_commit = false;
                    }

                    stats.last_commit_hash = Some(commit_hash.clone());
                    stats.last_commit_date = Some(commit_date);

                    // Update contributor information
                    let contributor_key = format!("{} <{}>", author_name, author_email);
                    let contributor = contributor_map.entry(contributor_key.clone()).or_insert(
                        ContributorInfo {
                            name: author_name,
                            email: author_email,
                            commit_count: 0,
                            first_commit_date: None,
                            last_commit_date: None,
                            lines_added: 0,
                            lines_deleted: 0,
                        }
                    );

                    contributor.commit_count += 1;
                    if contributor.first_commit_date.is_none() {
                        contributor.first_commit_date = Some(commit_date);
                    }
                    contributor.last_commit_date = Some(commit_date);

                    // Look ahead for file change statistics
                    while let Some(next_line) = lines.peek() {
                        if next_line.contains('|') {
                            break; // Next commit header
                        }

                        let change_line = lines.next().unwrap();
                        let change_parts: Vec<&str> = change_line.split_whitespace().collect();

                        if change_parts.len() >= 3 {
                            let added = change_parts[0].parse::<usize>().unwrap_or(0);
                            let deleted = change_parts[1].parse::<usize>().unwrap_or(0);

                            stats.total_lines_added += added;
                            stats.total_lines_deleted += deleted;
                            contributor.lines_added += added;
                            contributor.lines_deleted += deleted;
                        }
                    }
                }
            }
        }

        // Convert contributor map to vector and sort by commit count
        stats.contributors = contributor_map.into_values()
            .collect::<Vec<_>>();
        stats.contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

        Ok(stats)
    }
}

impl Default for GitHistoryService {
    fn default() -> Self {
        Self::new(".").unwrap_or_else(|_| Self {
            repo_path: ".".to_string(),
            stats_cache: HashMap::new(),
            verbose: false,
        })
    }
}

/// Represents a Git commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    /// Commit hash
    pub hash: String,

    /// Author name
    pub author_name: String,

    /// Author email
    pub author_email: String,

    /// Commit message
    pub message: String,

    /// Commit date and time
    pub commit_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_git_repo() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .current_dir(repo_path)
            .args(&["init"])
            .output()?;

        // Configure git user
        Command::new("git")
            .current_dir(repo_path)
            .args(&["config", "user.name", "Test User"])
            .output()?;

        Command::new("git")
            .current_dir(repo_path)
            .args(&["config", "user.email", "test@example.com"])
            .output()?;

        // Create a test file
        let test_file = repo_path.join("test.rs");
        fs::write(&test_file, "fn test() { println!(\"Hello\"); }")?;

        // Add and commit the file
        Command::new("git")
            .current_dir(repo_path)
            .args(&["add", "test.rs"])
            .output()?;

        Command::new("git")
            .current_dir(repo_path)
            .args(&["commit", "-m", "Initial commit"])
            .output()?;

        Ok(temp_dir)
    }

    #[test]
    fn test_git_history_service_creation() {
        let temp_dir = create_test_git_repo().unwrap();
        let service = GitHistoryService::new(temp_dir.path());
        assert!(service.is_ok());
    }

    #[test]
    fn test_get_last_modified() {
        let temp_dir = create_test_git_repo().unwrap();
        let service = GitHistoryService::new(temp_dir.path()).unwrap();
        let last_modified = service.get_last_modified("test.rs");
        assert!(last_modified.is_ok());
    }

    #[test]
    fn test_non_git_repo_error() {
        let temp_dir = TempDir::new().unwrap();
        let service = GitHistoryService::new(temp_dir.path());
        assert!(service.is_err());
    }
}
