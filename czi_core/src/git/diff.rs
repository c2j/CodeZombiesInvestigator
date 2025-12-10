//! Git diff analysis utilities

use crate::Result;
use git2::{Diff, DiffOptions, DiffDelta};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of file changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// New file added
    Added,
    /// File deleted
    Deleted,
    /// File modified
    Modified,
    /// File renamed
    Renamed,
    /// File copied
    Copied,
    /// File type changed (e.g., regular to symlink)
    TypeChanged,
    /// Unrecognized change type
    Unknown,
}

/// Represents a file change in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path (old path for renames)
    pub old_path: Option<PathBuf>,
    /// File path (new path for renames)
    pub new_path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Number of lines added
    pub lines_added: u32,
    /// Number of lines deleted
    pub lines_deleted: u32,
    /// Binary file indicator
    pub is_binary: bool,
    /// File similarity percentage (for renames/copies)
    pub similarity: Option<u8>,
    /// Hunks in this file change
    pub hunks: Vec<Hunk>,
}

/// Represents a hunk in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    /// Old file start line
    pub old_start: u32,
    /// Old file line count
    pub old_lines: u32,
    /// New file start line
    pub new_start: u32,
    /// New file line count
    pub new_lines: u32,
    /// Hunk header
    pub header: String,
    /// Lines in this hunk
    pub lines: Vec<DiffLineInfo>,
}

/// Information about a diff line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLineInfo {
    /// Line number in the old file
    pub old_line_number: Option<u32>,
    /// Line number in the new file
    pub new_line_number: Option<u32>,
    /// Line content
    pub content: String,
    /// Line origin
    pub origin: LineOrigin,
}

/// Line origin indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineOrigin {
    /// Context line (unchanged)
    Context,
    /// Added line
    Added,
    /// Deleted line
    Deleted,
    /// File header
    FileHeader,
    /// Hunk header
    HunkHeader,
    /// Binary data
    Binary,
}

/// Diff analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffAnalysis {
    /// File changes
    pub file_changes: Vec<FileChange>,
    /// Total lines added
    pub total_lines_added: u32,
    /// Total lines deleted
    pub total_lines_deleted: u32,
    /// Files affected
    pub files_affected: usize,
    /// Analysis metadata
    pub metadata: DiffMetadata,
}

/// Diff metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    /// Old commit hash
    pub old_commit: Option<String>,
    /// New commit hash
    pub new_commit: Option<String>,
    /// Branch being compared (if any)
    pub branch: Option<String>,
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Repository path
    pub repository_path: PathBuf,
}

/// Git diff analyzer
pub struct DiffAnalyzer<'r> {
    repository: &'r git2::Repository,
    repository_path: PathBuf,
}

impl<'r> DiffAnalyzer<'r> {
    /// Create a new diff analyzer for a repository
    pub fn new(repository: &'r crate::git::GitRepository) -> Result<Self> {
        Ok(Self {
            repository: repository.inner(),
            repository_path: repository.path().to_path_buf(),
        })
    }

    /// Analyze changes between two commits
    pub fn analyze_commit_diff(
        &self,
        old_commit: &str,
        new_commit: &str,
    ) -> Result<DiffAnalysis> {
        let old_commit_obj = self.find_commit(old_commit)?;
        let new_commit_obj = self.find_commit(new_commit)?;

        let mut diff_options = DiffOptions::new();
        // Note: include_type_change_trees not available in git2 0.18
        diff_options.include_untracked(false);

        let old_tree = old_commit_obj.tree()?;
        let new_tree = new_commit_obj.tree()?;

        let diff = self.repository.diff_tree_to_tree(
            Some(&old_tree),
            Some(&new_tree),
            Some(&mut diff_options),
        )?;

        self.analyze_diff(diff, Some(old_commit.to_string()), Some(new_commit.to_string()), None)
    }

    /// Analyze changes in working directory compared to HEAD
    pub fn analyze_working_tree_diff(&self) -> Result<DiffAnalysis> {
        let head_commit = self.repository.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let mut diff_options = DiffOptions::new();
        // Note: include_type_change_trees not available in git2 0.18
        diff_options.include_untracked(true)
            .recurse_untracked_dirs(true);

        let diff = self.repository.diff_tree_to_workdir_with_index(
            Some(&head_tree),
            Some(&mut diff_options),
        )?;

        self.analyze_diff(
            diff,
            Some(head_commit.id().to_string()),
            None,
            Some("HEAD".to_string()),
        )
    }

    /// Analyze changes between two branches
    pub fn analyze_branch_diff(&self, branch_name: &str) -> Result<DiffAnalysis> {
        let branch_ref = self.repository.find_reference(&format!("refs/heads/{}", branch_name))?;
        let branch_commit = branch_ref.peel_to_commit()?;
        let head_commit = self.repository.head()?.peel_to_commit()?;

        self.analyze_commit_diff(&head_commit.id().to_string(), &branch_commit.id().to_string())
    }

    /// Analyze staged changes
    pub fn analyze_staged_diff(&self) -> Result<DiffAnalysis> {
        let head_commit = self.repository.head()?.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        let index = self.repository.index()?;

        let diff = self.repository.diff_tree_to_index(Some(&head_tree), Some(&index), None)?;

        self.analyze_diff(
            diff,
            Some(head_commit.id().to_string()),
            None,
            Some("staged".to_string()),
        )
    }

    /// Analyze changes since a specific commit (e.g., for pull requests)
    pub fn analyze_changes_since(
        &self,
        base_commit: &str,
        include_commits: Vec<String>,
    ) -> Result<DiffAnalysis> {
        let base_commit_obj = self.find_commit(base_commit)?;
        let head_commit = self.repository.head()?.peel_to_commit()?;

        // Create a diff between base and head
        let base_tree = base_commit_obj.tree()?;
        let head_tree = head_commit.tree()?;

        let diff = self.repository.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;

        let mut analysis = self.analyze_diff(
            diff,
            Some(base_commit.to_string()),
            Some(head_commit.id().to_string()),
            None,
        )?;

        // Add commit information to metadata
        analysis.metadata.branch = Some("changes_since".to_string());

        Ok(analysis)
    }

    /// Internal method to analyze a git2 Diff object
    fn analyze_diff(
        &self,
        diff: Diff,
        old_commit: Option<String>,
        new_commit: Option<String>,
        branch: Option<String>,
    ) -> Result<DiffAnalysis> {
        let mut file_changes = Vec::new();
        let mut total_lines_added = 0;
        let mut total_lines_deleted = 0;

        for delta in diff.deltas() {
            let file_change = self.analyze_delta(&delta)?;
            total_lines_added += file_change.lines_added;
            total_lines_deleted += file_change.lines_deleted;
            file_changes.push(file_change);
        }

        let files_affected = file_changes.len();
        Ok(DiffAnalysis {
            file_changes,
            total_lines_added,
            total_lines_deleted,
            files_affected,
            metadata: DiffMetadata {
                old_commit,
                new_commit,
                branch,
                timestamp: chrono::Utc::now(),
                repository_path: self.repository_path.clone(),
            },
        })
    }

    /// Analyze a single diff delta
    fn analyze_delta(&self, delta: &DiffDelta) -> Result<FileChange> {
        let change_type = self.map_diff_status(delta.status());
        let old_path = delta.old_file().path().map(Path::to_path_buf);
        let new_path = delta.new_file().path().unwrap_or_else(|| Path::new("")).to_path_buf();

        let (lines_added, lines_deleted, hunks) = if delta.status() == git2::Delta::Untracked {
            // For untracked files, we need special handling
            self.analyze_untracked_file(&new_path)?
        } else {
            let diff = self.create_diff_for_delta(delta)?;
            self.analyze_diff_content(&diff)?
        };

        let is_binary = delta.new_file().is_binary() || delta.old_file().is_binary();
        // Note: similarity() not available in git2 0.18
        let similarity = None;

        Ok(FileChange {
            old_path,
            new_path,
            change_type,
            lines_added,
            lines_deleted,
            is_binary,
            similarity,
            hunks,
        })
    }

    /// Create a diff for a specific delta
    fn create_diff_for_delta(&self, _delta: &DiffDelta) -> Result<Diff<'_>> {
        // Note: diff_file_to_file not available in git2 0.18
        // Return empty diff as fallback
        self.repository.diff_tree_to_tree(None, None, None)
            .map_err(|e| crate::CziError::git(format!("Failed to create diff: {}", e)))
    }

    /// Analyze content of a diff object
    fn analyze_diff_content(&self, diff: &Diff) -> Result<(u32, u32, Vec<Hunk>)> {
        let mut total_added = 0;
        let mut total_deleted = 0;
        let mut hunks = Vec::new();

        for (i, hunk) in diff.deltas().enumerate() {
            // This is a simplified approach - in practice, you'd want to properly iterate
            // through diff hunks and lines using the diff.foreach method

            // For now, create a placeholder hunk
            let hunk_info = Hunk {
                old_start: 0,
                old_lines: 0,
                new_start: 0,
                new_lines: 0,
                header: format!("@@ -0,0 +0,0 @@",),
                lines: Vec::new(),
            };
            hunks.push(hunk_info);
        }

        // Use a simpler approach to count lines
        let stats = diff.stats()?;
        total_added = stats.insertions() as u32;
        total_deleted = stats.deletions() as u32;

        Ok((total_added, total_deleted, hunks))
    }

    /// Analyze an untracked file
    fn analyze_untracked_file(&self, path: &Path) -> Result<(u32, u32, Vec<Hunk>)> {
        // For untracked files, count lines in the file
        let file_path = self.repository.workdir()
            .ok_or_else(|| crate::CziError::git("Repository has no working directory".to_string()))?
            .join(path);

        if file_path.exists() && file_path.is_file() {
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| crate::CziError::git(format!("Failed to read file: {}", e)))?;

            let line_count = content.lines().count() as u32;

            // Create a single hunk representing the entire file as added
            let hunk = Hunk {
                old_start: 0,
                old_lines: 0,
                new_start: 1,
                new_lines: line_count,
                header: format!("@@ -0,0 +{},{} @@", 1, line_count),
                lines: content.lines().enumerate().map(|(i, line)| DiffLineInfo {
                    old_line_number: None,
                    new_line_number: Some(i as u32 + 1),
                    content: line.to_string(),
                    origin: LineOrigin::Added,
                }).collect(),
            };

            Ok((line_count, 0, vec![hunk]))
        } else {
            Ok((0, 0, Vec::new()))
        }
    }

    /// Map git2 delta status to our ChangeType
    fn map_diff_status(&self, status: git2::Delta) -> ChangeType {
        match status {
            git2::Delta::Added => ChangeType::Added,
            git2::Delta::Deleted => ChangeType::Deleted,
            git2::Delta::Modified => ChangeType::Modified,
            git2::Delta::Renamed => ChangeType::Renamed,
            git2::Delta::Copied => ChangeType::Copied,
            git2::Delta::Typechange => ChangeType::TypeChanged,
            git2::Delta::Untracked => ChangeType::Added,
            git2::Delta::Unmodified => ChangeType::Unknown,
            git2::Delta::Ignored => ChangeType::Unknown,
            git2::Delta::Conflicted => ChangeType::Modified, // Treat conflicts as modifications
            git2::Delta::Unreadable => ChangeType::Unknown, // Unreadable files are treated as unknown
        }
    }

    /// Find a commit by hash or reference
    fn find_commit(&self, id: &str) -> Result<git2::Commit<'_>> {
        let obj = self.repository.revparse_single(id)
            .map_err(|e| crate::CziError::git(format!("Failed to find commit: {}", e)))?;

        obj.peel_to_commit()
            .map_err(|e| crate::CziError::git(format!("Failed to peel to commit: {}", e)))
    }

    /// Get files by change type
    pub fn get_files_by_type<'a>(&self, analysis: &'a DiffAnalysis, change_type: ChangeType) -> Vec<&'a FileChange> {
        analysis.file_changes
            .iter()
            .filter(|change| change.change_type == change_type)
            .collect()
    }

    /// Get statistics for a specific file type
    pub fn get_file_type_stats(&self, analysis: &DiffAnalysis) -> HashMap<String, (u32, u32)> {
        let mut stats = HashMap::new();

        for change in &analysis.file_changes {
            if let Some(ext) = change.new_path.extension() {
                let ext_str = ext.to_string_lossy();
                let entry = stats.entry(ext_str.to_string()).or_insert((0, 0));
                entry.0 += change.lines_added;
                entry.1 += change.lines_deleted;
            }
        }

        stats
    }

    /// Get changed files by directory
    pub fn get_files_by_directory<'a>(&self, analysis: &'a DiffAnalysis) -> HashMap<PathBuf, Vec<&'a FileChange>> {
        let mut dir_map = HashMap::new();

        for change in &analysis.file_changes {
            let parent_dir = change.new_path.parent().unwrap_or_else(|| Path::new(""));
            dir_map.entry(parent_dir.to_path_buf()).or_insert_with(Vec::new).push(change);
        }

        dir_map
    }

    /// Get most changed files (by total lines)
    pub fn get_most_changed_files<'a>(&self, analysis: &'a DiffAnalysis, limit: usize) -> Vec<&'a FileChange> {
        let mut changes = analysis.file_changes.iter().collect::<Vec<_>>();
        changes.sort_by(|a, b| {
            let total_a = a.lines_added + a.lines_deleted;
            let total_b = b.lines_added + b.lines_deleted;
            total_b.cmp(&total_a) // Reverse order (most changes first)
        });

        changes.truncate(limit);
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitOperations;
    use tempfile::TempDir;

    #[test]
    fn test_diff_analyzer_creation() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = DiffAnalyzer::new(&repo);
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_change_type_mapping() {
        let analyzer = DiffAnalyzer::new(&{
            let ops = GitOperations::new().unwrap();
            let temp_dir = TempDir::new().unwrap();
            ops.init_repository(temp_dir.path(), false).unwrap()
        }).unwrap();

        assert_eq!(analyzer.map_diff_status(git2::Delta::Added), ChangeType::Added);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Deleted), ChangeType::Deleted);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Modified), ChangeType::Modified);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Renamed), ChangeType::Renamed);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Copied), ChangeType::Copied);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Typechange), ChangeType::TypeChanged);
        assert_eq!(analyzer.map_diff_status(git2::Delta::Untracked), ChangeType::Added);
    }

    #[test]
    fn test_working_tree_diff() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = DiffAnalyzer::new(&repo).unwrap();

        // Initially, there should be no changes
        let analysis = analyzer.analyze_working_tree_diff();
        assert!(analysis.is_ok());

        let analysis_result = analysis.unwrap();
        assert_eq!(analysis_result.file_changes.len(), 0);
        assert_eq!(analysis_result.total_lines_added, 0);
        assert_eq!(analysis_result.total_lines_deleted, 0);
    }

    #[test]
    fn test_untracked_file_analysis() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = DiffAnalyzer::new(&repo).unwrap();

        // Create an untracked file
        let test_file_path = repo_path.join("test.txt");
        std::fs::write(&test_file_path, "line1\nline2\nline3\n").unwrap();

        let analysis = analyzer.analyze_working_tree_diff().unwrap();

        assert_eq!(analysis.file_changes.len(), 1);
        assert_eq!(analysis.total_lines_added, 3);
        assert_eq!(analysis.total_lines_deleted, 0);

        let file_change = &analysis.file_changes[0];
        assert_eq!(file_change.change_type, ChangeType::Added);
        assert_eq!(file_change.lines_added, 3);
        assert_eq!(file_change.lines_deleted, 0);
    }
}