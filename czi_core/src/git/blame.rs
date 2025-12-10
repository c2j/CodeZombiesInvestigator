//! Git blame analysis for tracking code ownership and changes

use crate::Result;
use git2::{BlameHunk as GitBlameHunk, Signature};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Information about a single blame hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameHunk {
    /// Start line number in the final file
    pub start_line: u32,
    /// Number of lines in this hunk
    pub line_count: u32,
    /// Final commit ID that introduced these lines
    pub final_commit_id: String,
    /// Original commit ID that originally introduced these lines
    pub orig_commit_id: String,
    /// Author information
    pub author: AuthorInfo,
    /// Committer information
    pub committer: AuthorInfo,
    /// Summary message of the commit
    pub summary: String,
    /// File path in the original commit
    pub orig_path: PathBuf,
    /// Timestamp when this hunk was authored
    pub authored_time: DateTime<Utc>,
    /// Timestamp when this hunk was committed
    pub committed_time: DateTime<Utc>,
}

/// Author/committer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Author name
    pub name: String,
    /// Author email
    pub email: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Complete blame information for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameInfo {
    /// File path
    pub file_path: PathBuf,
    /// Total lines in the file
    pub total_lines: u32,
    /// Blame hunks
    pub hunks: Vec<BlameHunk>,
    /// Analysis timestamp
    pub analysis_timestamp: DateTime<Utc>,
    /// Repository path
    pub repository_path: PathBuf,
}

/// Blame analysis results with author statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameAnalysis {
    /// Blame information for each analyzed file
    pub file_blames: Vec<BlameInfo>,
    /// Author contribution statistics
    pub author_stats: AuthorStats,
    /// Temporal analysis
    pub temporal_analysis: TemporalAnalysis,
    /// Analysis metadata
    pub metadata: BlameMetadata,
}

/// Author contribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorStats {
    /// Total lines contributed per author
    pub lines_by_author: HashMap<String, u32>,
    /// Number of files touched per author
    pub files_by_author: HashMap<String, usize>,
    /// Most recent contribution per author
    pub last_contribution: HashMap<String, DateTime<Utc>>,
    /// Average contribution age per author
    pub avg_contribution_age: HashMap<String, chrono::Duration>,
}

/// Temporal analysis of blame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    /// Commits by year/month
    pub commits_by_month: HashMap<String, usize>,
    /// Line age distribution
    pub line_age_distribution: LineAgeDistribution,
    /// Oldest line timestamp
    pub oldest_line: Option<DateTime<Utc>>,
    /// Newest line timestamp
    pub newest_line: Option<DateTime<Utc>>,
    /// Average line age
    pub avg_line_age: Option<chrono::Duration>,
}

/// Line age distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAgeDistribution {
    /// Lines added in last week
    pub last_week: u32,
    /// Lines added in last month
    pub last_month: u32,
    /// Lines added in last 3 months
    pub last_quarter: u32,
    /// Lines added in last year
    pub last_year: u32,
    /// Lines added more than a year ago
    pub older_than_year: u32,
}

/// Blame analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameMetadata {
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    /// Repository path
    pub repository_path: PathBuf,
    /// Branch analyzed
    pub branch: Option<String>,
    /// Commit analyzed
    pub commit: Option<String>,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Total lines analyzed
    pub total_lines_analyzed: u32,
}

/// Git blame analyzer
pub struct BlameAnalyzer<'r> {
    repository: &'r git2::Repository,
    repository_path: PathBuf,
}

impl<'r> BlameAnalyzer<'r> {
    /// Create a new blame analyzer for a repository
    pub fn new(repository: &'r crate::git::GitRepository) -> Result<Self> {
        Ok(Self {
            repository: repository.inner(),
            repository_path: repository.path().to_path_buf(),
        })
    }

    /// Analyze blame information for a single file
    pub fn analyze_file(&self, file_path: &Path) -> Result<BlameInfo> {
        let blame = self.repository.blame_file(file_path, Some(&mut git2::BlameOptions::new()))
            .map_err(|e| crate::CziError::git(format!("Failed to blame file: {}", e)))?;

        let total_lines = blame.len() as u32;
        let mut hunks = Vec::new();

        for hunk_index in 0..blame.len() {
            let blame_hunk = blame.get_index(hunk_index)
                .ok_or_else(|| crate::CziError::git("Failed to get blame hunk".to_string()))?;

            let hunk = self.convert_blame_hunk(&blame_hunk)?;
            hunks.push(hunk);
        }

        Ok(BlameInfo {
            file_path: file_path.to_path_buf(),
            total_lines,
            hunks,
            analysis_timestamp: Utc::now(),
            repository_path: self.repository_path.clone(),
        })
    }

    /// Analyze blame information for multiple files
    pub fn analyze_files(&self, file_paths: &[&Path]) -> Result<BlameAnalysis> {
        let mut file_blames = Vec::new();
        let mut author_stats = AuthorStats {
            lines_by_author: HashMap::new(),
            files_by_author: HashMap::new(),
            last_contribution: HashMap::new(),
            avg_contribution_age: HashMap::new(),
        };

        let mut all_hunks = Vec::new();
        let mut timestamps = Vec::new();

        for &file_path in file_paths {
            let blame_info = self.analyze_file(file_path)?;

            // Update author statistics
            self.update_author_stats(&blame_info, &mut author_stats);

            // Collect all hunks for temporal analysis
            all_hunks.extend(blame_info.hunks.clone());

            // Collect timestamps
            for hunk in &blame_info.hunks {
                timestamps.push(hunk.committed_time);
                timestamps.push(hunk.authored_time);
            }

            file_blames.push(blame_info);
        }

        let temporal_analysis = self.create_temporal_analysis(&all_hunks)?;
        let total_lines_analyzed = file_blames.iter().map(|info| info.total_lines).sum();

        let metadata = BlameMetadata {
            timestamp: Utc::now(),
            repository_path: self.repository_path.clone(),
            branch: self.get_current_branch()?,
            commit: self.get_head_commit()?,
            files_analyzed: file_paths.len(),
            total_lines_analyzed,
        };

        Ok(BlameAnalysis {
            file_blames,
            author_stats,
            temporal_analysis,
            metadata,
        })
    }

    /// Analyze blame information for files with specific extensions
    pub fn analyze_by_extension(&self, extensions: &[&str]) -> Result<BlameAnalysis> {
        let workdir = self.repository.workdir()
            .ok_or_else(|| crate::CziError::git("Repository has no working directory".to_string()))?;

        let mut files_to_analyze = Vec::new();

        // Find files with matching extensions
        for entry in walkdir::WalkDir::new(workdir) {
            let entry = entry.map_err(|e| crate::CziError::git(format!("Failed to walk directory: {}", e)))?;

            if entry.file_type().is_file() {
                if let Some(path) = entry.path().strip_prefix(workdir).ok() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if extensions.contains(&ext_str.as_ref()) {
                            files_to_analyze.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        // Convert to references for analysis
        let file_refs: Vec<&Path> = files_to_analyze.iter().map(|p| p.as_path()).collect();
        self.analyze_files(&file_refs)
    }

    /// Get top contributors by lines of code
    pub fn get_top_contributors(&self, analysis: &BlameAnalysis, limit: usize) -> Vec<(String, u32)> {
        let mut contributors: Vec<(String, u32)> = analysis.author_stats.lines_by_author
            .iter()
            .map(|(author, lines)| (author.clone(), *lines))
            .collect();

        contributors.sort_by(|a, b| b.1.cmp(&a.1));
        contributors.truncate(limit);
        contributors
    }

    /// Get files with the most diverse authorship
    pub fn get_most_collaborative_files(&self, analysis: &BlameAnalysis, limit: usize) -> Vec<(PathBuf, usize)> {
        let mut file_author_counts: Vec<(PathBuf, usize)> = analysis.file_blames
            .iter()
            .map(|file_blame| {
                let unique_authors: std::collections::HashSet<_> = file_blame.hunks
                    .iter()
                    .map(|hunk| hunk.author.name.clone())
                    .collect();
                (file_blame.file_path.clone(), unique_authors.len())
            })
            .collect();

        file_author_counts.sort_by(|a, b| b.1.cmp(&a.1));
        file_author_counts.truncate(limit);
        file_author_counts
    }

    /// Get files that need attention (old lines, single author, etc.)
    pub fn get_attention_needed_files(&self, analysis: &BlameAnalysis) -> Vec<AttentionFile> {
        let mut attention_files = Vec::new();

        for file_blame in &analysis.file_blames {
            let mut reasons = Vec::new();

            // Check for old files (average age > 1 year)
            if let Some(avg_age) = self.calculate_file_avg_age(file_blame) {
                if avg_age > chrono::Duration::days(365) {
                    reasons.push(AttentionReason::OldCode);
                }
            }

            // Check for single-authored files
            let unique_authors: std::collections::HashSet<_> = file_blame.hunks
                .iter()
                .map(|hunk| hunk.author.name.clone())
                .collect();

            if unique_authors.len() == 1 && file_blame.total_lines > 50 {
                reasons.push(AttentionReason::SingleAuthored);
            }

            // Check for files with many commits (complex history)
            let unique_commits: std::collections::HashSet<_> = file_blame.hunks
                .iter()
                .map(|hunk| hunk.final_commit_id.clone())
                .collect();

            if unique_commits.len() > 20 {
                reasons.push(AttentionReason::HighChurn);
            }

            if !reasons.is_empty() {
                attention_files.push(AttentionFile {
                    file_path: file_blame.file_path.clone(),
                    total_lines: file_blame.total_lines,
                    reasons,
                });
            }
        }

        attention_files.sort_by(|a, b| b.total_lines.cmp(&a.total_lines));
        attention_files
    }

    /// Convert git2 blame hunk to our format
    fn convert_blame_hunk(&self, blame_hunk: &GitBlameHunk) -> Result<BlameHunk> {
        let final_commit_id = blame_hunk.final_commit_id().to_string();
        let orig_commit_id = blame_hunk.orig_commit_id().to_string();

        let author = self.convert_signature(&blame_hunk.final_signature())?;
        // Note: final_committer() not available in git2 0.18, use final_signature instead
        let committer = self.convert_signature(&blame_hunk.final_signature())?;

        let summary = self.get_commit_message(&final_commit_id)?;
        // Note: orig_path() not available in git2 0.18, use file_path instead
        let orig_path = PathBuf::from("");

        let authored_time = DateTime::from_timestamp(
            blame_hunk.final_signature().when().seconds() as i64,
            (blame_hunk.final_signature().when().offset_minutes() * 60) as u32,
        ).ok_or_else(|| crate::CziError::git("Invalid timestamp".to_string()))?;

        // Note: final_committer() not available in git2 0.18, use final_signature instead
        let committed_time = DateTime::from_timestamp(
            blame_hunk.final_signature().when().seconds() as i64,
            (blame_hunk.final_signature().when().offset_minutes() * 60) as u32,
        ).ok_or_else(|| crate::CziError::git("Invalid timestamp".to_string()))?;

        Ok(BlameHunk {
            start_line: blame_hunk.final_start_line() as u32,
            line_count: blame_hunk.lines_in_hunk() as u32,
            final_commit_id,
            orig_commit_id,
            author,
            committer,
            summary,
            orig_path,
            authored_time,
            committed_time,
        })
    }

    /// Convert git2 signature to our format
    fn convert_signature(&self, signature: &Signature) -> Result<AuthorInfo> {
        let timestamp = DateTime::from_timestamp(
            signature.when().seconds() as i64,
            (signature.when().offset_minutes() * 60) as u32,
        ).ok_or_else(|| crate::CziError::git("Invalid timestamp".to_string()))?;

        Ok(AuthorInfo {
            name: signature.name().unwrap_or("Unknown").to_string(),
            email: signature.email().unwrap_or("unknown@example.com").to_string(),
            timestamp,
        })
    }

    /// Get commit message for a commit ID
    fn get_commit_message(&self, commit_id: &str) -> Result<String> {
        let commit = self.find_commit(commit_id)?;
        let message = commit.message().unwrap_or("No message");
        let summary = message.lines().next().unwrap_or("No summary");
        Ok(summary.to_string())
    }

    /// Find a commit by ID
    fn find_commit(&self, id: &str) -> Result<git2::Commit<'_>> {
        let obj = self.repository.revparse_single(id)
            .map_err(|e| crate::CziError::git(format!("Failed to find commit: {}", e)))?;

        obj.peel_to_commit()
            .map_err(|e| crate::CziError::git(format!("Failed to peel to commit: {}", e)))
    }

    /// Update author statistics with blame information
    fn update_author_stats(&self, blame_info: &BlameInfo, stats: &mut AuthorStats) {
        let mut file_authors = std::collections::HashSet::new();

        for hunk in &blame_info.hunks {
            let author_name = hunk.author.name.clone();

            // Update line count
            *stats.lines_by_author.entry(author_name.clone()).or_insert(0) += hunk.line_count;

            // Track file contributions
            file_authors.insert(author_name.clone());

            // Update last contribution time
            let entry = stats.last_contribution.entry(author_name.clone()).or_insert_with(|| hunk.authored_time);
            if hunk.authored_time > *entry {
                *entry = hunk.authored_time;
            }
        }

        // Update file count per author
        for author in file_authors {
            *stats.files_by_author.entry(author).or_insert(0) += 1;
        }

        // Calculate average contribution age
        self.calculate_avg_contribution_ages(stats);
    }

    /// Calculate average contribution ages for all authors
    fn calculate_avg_contribution_ages(&self, stats: &mut AuthorStats) {
        let now = Utc::now();

        for (author, _) in &stats.lines_by_author {
            if let Some(last_contribution) = stats.last_contribution.get(author) {
                let age = now.signed_duration_since(*last_contribution);
                stats.avg_contribution_age.insert(author.clone(), age);
            }
        }
    }

    /// Create temporal analysis from blame hunks
    fn create_temporal_analysis(&self, hunks: &[BlameHunk]) -> Result<TemporalAnalysis> {
        let mut commits_by_month = HashMap::new();
        let mut timestamps = Vec::new();

        for hunk in hunks {
            timestamps.push(hunk.committed_time);

            // Count commits by month
            let month_key = hunk.committed_time.format("%Y-%m").to_string();
            *commits_by_month.entry(month_key).or_insert(0) += 1;
        }

        let line_age_distribution = self.calculate_line_age_distribution(&timestamps)?;

        let oldest_line = timestamps.iter().min().cloned();
        let newest_line = timestamps.iter().max().cloned();

        let avg_line_age = if !timestamps.is_empty() {
            let now = Utc::now();
            let total_age: chrono::Duration = timestamps.iter()
                .map(|&ts| now.signed_duration_since(ts))
                .sum();
            Some(total_age / timestamps.len() as i32)
        } else {
            None
        };

        Ok(TemporalAnalysis {
            commits_by_month,
            line_age_distribution,
            oldest_line,
            newest_line,
            avg_line_age,
        })
    }

    /// Calculate line age distribution
    fn calculate_line_age_distribution(&self, timestamps: &[DateTime<Utc>]) -> Result<LineAgeDistribution> {
        let now = Utc::now();
        let one_week = chrono::Duration::weeks(1);
        let one_month = chrono::Duration::weeks(4);
        let one_quarter = chrono::Duration::weeks(12);
        let one_year = chrono::Duration::weeks(52);

        let mut last_week = 0;
        let mut last_month = 0;
        let mut last_quarter = 0;
        let mut last_year = 0;
        let mut older_than_year = 0;

        for &timestamp in timestamps {
            let age = now.signed_duration_since(timestamp);

            if age <= one_week {
                last_week += 1;
            } else if age <= one_month {
                last_month += 1;
            } else if age <= one_quarter {
                last_quarter += 1;
            } else if age <= one_year {
                last_year += 1;
            } else {
                older_than_year += 1;
            }
        }

        Ok(LineAgeDistribution {
            last_week,
            last_month,
            last_quarter,
            last_year,
            older_than_year,
        })
    }

    /// Calculate average age for a file
    fn calculate_file_avg_age(&self, blame_info: &BlameInfo) -> Option<chrono::Duration> {
        if blame_info.hunks.is_empty() {
            return None;
        }

        let now = Utc::now();
        let total_age: chrono::Duration = blame_info.hunks.iter()
            .map(|hunk| now.signed_duration_since(hunk.authored_time))
            .sum();

        Some(total_age / blame_info.hunks.len() as i32)
    }

    /// Get current branch name
    fn get_current_branch(&self) -> Result<Option<String>> {
        let head = match self.repository.head() {
            Ok(head) => head,
            Err(_) => return Ok(None),
        };
        let branch_name = head.shorthand().map(|s| s.to_string());
        Ok(branch_name)
    }

    /// Get HEAD commit ID
    fn get_head_commit(&self) -> Result<Option<String>> {
        let head = match self.repository.head() {
            Ok(head) => head,
            Err(_) => return Ok(None),
        };
        let commit = match head.peel_to_commit() {
            Ok(commit) => commit,
            Err(_) => return Ok(None),
        };
        Ok(Some(commit.id().to_string()))
    }
}

/// Files that need attention with reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionFile {
    pub file_path: PathBuf,
    pub total_lines: u32,
    pub reasons: Vec<AttentionReason>,
}

/// Reasons a file might need attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttentionReason {
    /// Code is old (not recently modified)
    OldCode,
    /// Single author only
    SingleAuthored,
    /// High churn (many commits)
    HighChurn,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::GitOperations;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_blame_analyzer_creation() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = BlameAnalyzer::new(&repo);
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_file_blame_analysis() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = BlameAnalyzer::new(&repo).unwrap();

        // Create a test file
        let test_file_path = repo_path.join("test.txt");
        fs::write(&test_file_path, "line1\nline2\nline3\n").unwrap();

        // Analyze the file (this will work but won't have meaningful blame info yet)
        let blame_info = analyzer.analyze_file(&test_file_path.join("test.txt"));
        assert!(blame_info.is_ok());

        let blame_result = blame_info.unwrap();
        assert_eq!(blame_result.file_path, Path::new("test.txt"));
        assert_eq!(blame_result.total_lines, 3);
    }

    #[test]
    fn test_author_info_conversion() {
        let ops = GitOperations::new().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let repo = ops.init_repository(repo_path, false).unwrap();
        let analyzer = BlameAnalyzer::new(&repo).unwrap();

        // Create a signature to test conversion
        let time = git2::Time::new(1609459200, 0); // 2021-01-01 00:00:00 UTC
        let signature = git2::Signature::new("Test Author", "test@example.com", &time)
            .unwrap();

        let author_info = analyzer.convert_signature(&signature).unwrap();
        assert_eq!(author_info.name, "Test Author");
        assert_eq!(author_info.email, "test@example.com");
    }
}