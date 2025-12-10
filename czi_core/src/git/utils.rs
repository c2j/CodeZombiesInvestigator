//! Git utility functions and URL handling

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Git URL parsing and normalization utilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitUrl {
    /// Original URL string
    pub original: String,
    /// Normalized URL string
    pub normalized: String,
    /// Protocol type
    pub protocol: GitProtocol,
    /// Host name
    pub host: Option<String>,
    /// Repository owner/organization
    pub owner: Option<String>,
    /// Repository name
    pub repository: Option<String>,
    /// URL path components
    pub path_components: Vec<String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Fragment
    pub fragment: Option<String>,
}

/// Git protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitProtocol {
    /// HTTPS protocol
    Https,
    /// SSH protocol
    Ssh,
    /// Git protocol (native git://)
    Git,
    /// File protocol (local filesystem)
    File,
    /// Unknown protocol
    Unknown,
}

/// Git utility functions
pub struct GitUtils;

impl GitUtils {
    /// Parse and normalize a Git URL
    pub fn parse_url(url: &str) -> Result<GitUrl> {
        let mut git_url = GitUrl {
            original: url.to_string(),
            normalized: url.to_string(),
            protocol: GitProtocol::Unknown,
            host: None,
            owner: None,
            repository: None,
            path_components: Vec::new(),
            query_params: HashMap::new(),
            fragment: None,
        };

        // Determine protocol
        if url.starts_with("https://") {
            git_url.protocol = GitProtocol::Https;
            Self::parse_https_url(url, &mut git_url)?;
        } else if url.starts_with("git@") || url.starts_with("ssh://") {
            git_url.protocol = GitProtocol::Ssh;
            Self::parse_ssh_url(url, &mut git_url)?;
        } else if url.starts_with("git://") {
            git_url.protocol = GitProtocol::Git;
            Self::parse_git_url(url, &mut git_url)?;
        } else if url.starts_with("file://") || Path::new(url).exists() {
            git_url.protocol = GitProtocol::File;
            Self::parse_file_url(url, &mut git_url)?;
        }

        Ok(git_url)
    }

    /// Parse HTTPS URLs
    fn parse_https_url(url: &str, git_url: &mut GitUrl) -> Result<()> {
        // Remove protocol prefix
        let url_without_proto = url.strip_prefix("https://")
            .ok_or_else(|| crate::CziError::git("Invalid HTTPS URL".to_string()))?;

        // Split into path and fragment
        let (path_part, fragment_part) = url_without_proto.split_once('#').unwrap_or((url_without_proto, ""));

        if !fragment_part.is_empty() {
            git_url.fragment = Some(fragment_part.to_string());
        }

        // Split into path and query
        let (path_part, query_part) = path_part.split_once('?').unwrap_or((path_part, ""));

        if !query_part.is_empty() {
            Self::parse_query_params(query_part, &mut git_url.query_params);
        }

        // Parse path components
        let path_parts: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

        if path_parts.is_empty() {
            return Err(crate::CziError::git("Invalid HTTPS URL - no host".to_string()));
        }

        git_url.host = Some(path_parts[0].to_string());

        if path_parts.len() >= 3 {
            // Assume format: host/owner/repo
            git_url.owner = Some(path_parts[1].to_string());
            git_url.repository = Some(path_parts[2].to_string());
        }

        git_url.path_components = path_parts.into_iter().map(|s| s.to_string()).collect();

        Ok(())
    }

    /// Parse SSH URLs
    fn parse_ssh_url(url: &str, git_url: &mut GitUrl) -> Result<()> {
        let url_without_proto = if url.starts_with("ssh://") {
            url.strip_prefix("ssh://")
                .ok_or_else(|| crate::CziError::git("Invalid SSH URL".to_string()))?
        } else {
            url // Format: git@host:owner/repo
        };

        if let Some(at_pos) = url_without_proto.find('@') {
            // Format: git@host:owner/repo
            let after_at = &url_without_proto[at_pos + 1..];

            if let Some(colon_pos) = after_at.find(':') {
                let host = &after_at[..colon_pos];
                let path_part = &after_at[colon_pos + 1..];

                git_url.host = Some(host.to_string());

                let path_parts: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

                if path_parts.len() >= 2 {
                    git_url.owner = Some(path_parts[0].to_string());
                    git_url.repository = Some(path_parts[1].to_string());
                }

                git_url.path_components = vec![host.to_string()];
                git_url.path_components.extend(path_parts.into_iter().map(|s| s.to_string()));
            } else {
                return Err(crate::CziError::git("Invalid SSH URL - missing colon".to_string()));
            }
        } else {
            return Err(crate::CziError::git("Invalid SSH URL - missing @".to_string()));
        }

        Ok(())
    }

    /// Parse Git protocol URLs
    fn parse_git_url(url: &str, git_url: &mut GitUrl) -> Result<()> {
        let url_without_proto = url.strip_prefix("git://")
            .ok_or_else(|| crate::CziError::git("Invalid Git protocol URL".to_string()))?;

        let path_parts: Vec<&str> = url_without_proto.split('/').filter(|s| !s.is_empty()).collect();

        if path_parts.is_empty() {
            return Err(crate::CziError::git("Invalid Git protocol URL - no host".to_string()));
        }

        git_url.host = Some(path_parts[0].to_string());

        if path_parts.len() >= 3 {
            git_url.owner = Some(path_parts[1].to_string());
            git_url.repository = Some(path_parts[2].to_string());
        }

        git_url.path_components = path_parts.into_iter().map(|s| s.to_string()).collect();

        Ok(())
    }

    /// Parse file URLs
    fn parse_file_url(url: &str, git_url: &mut GitUrl) -> Result<()> {
        let path = if url.starts_with("file://") {
            url.strip_prefix("file://")
                .ok_or_else(|| crate::CziError::git("Invalid file URL".to_string()))?
        } else {
            url
        };

        let path_buf = PathBuf::from(path);

        git_url.path_components = path_buf.components()
            .filter_map(|c| c.as_os_str().to_str())
            .map(|s| s.to_string())
            .collect();

        git_url.repository = path_buf.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        Ok(())
    }

    /// Parse query parameters
    fn parse_query_params(query: &str, params: &mut HashMap<String, String>) {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), value.to_string());
            } else {
                params.insert(pair.to_string(), String::new());
            }
        }
    }

    /// Validate if a string looks like a Git URL
    pub fn is_git_url(url: &str) -> bool {
        // Check for common Git URL patterns
        url.starts_with("https://") && url.contains(".git") ||
        url.starts_with("git@") ||
        url.starts_with("ssh://") ||
        url.starts_with("git://") ||
        url.starts_with("file://") ||
        url.ends_with(".git")
    }

    /// Convert Git URL to HTTPS format
    pub fn to_https_url(url: &str) -> Result<String> {
        let git_url = Self::parse_url(url)?;

        match git_url.protocol {
            GitProtocol::Https => Ok(url.to_string()),
            GitProtocol::Ssh => {
                if let (Some(host), Some(owner), Some(repo)) = (&git_url.host, &git_url.owner, &git_url.repository) {
                    Ok(format!("https://{}/{}/{}.git", host, owner, repo))
                } else {
                    Err(crate::CziError::git("Cannot convert SSH URL to HTTPS - missing components".to_string()))
                }
            }
            GitProtocol::Git => {
                if let (Some(host), Some(owner), Some(repo)) = (&git_url.host, &git_url.owner, &git_url.repository) {
                    Ok(format!("https://{}/{}/{}.git", host, owner, repo))
                } else {
                    Err(crate::CziError::git("Cannot convert Git protocol URL to HTTPS - missing components".to_string()))
                }
            }
            GitProtocol::File | GitProtocol::Unknown => {
                Err(crate::CziError::git("Cannot convert URL to HTTPS".to_string()))
            }
        }
    }

    /// Convert Git URL to SSH format
    pub fn to_ssh_url(url: &str) -> Result<String> {
        let git_url = Self::parse_url(url)?;

        match git_url.protocol {
            GitProtocol::Ssh => Ok(url.to_string()),
            GitProtocol::Https => {
                if let (Some(host), Some(owner), Some(repo)) = (&git_url.host, &git_url.owner, &git_url.repository) {
                    Ok(format!("git@{}:{}/{}.git", host, owner, repo))
                } else {
                    Err(crate::CziError::git("Cannot convert HTTPS URL to SSH - missing components".to_string()))
                }
            }
            GitProtocol::Git => {
                if let (Some(host), Some(owner), Some(repo)) = (&git_url.host, &git_url.owner, &git_url.repository) {
                    Ok(format!("git@{}:{}/{}.git", host, owner, repo))
                } else {
                    Err(crate::CziError::git("Cannot convert Git protocol URL to SSH - missing components".to_string()))
                }
            }
            GitProtocol::File | GitProtocol::Unknown => {
                Err(crate::CziError::git("Cannot convert URL to SSH".to_string()))
            }
        }
    }

    /// Extract repository name from URL
    pub fn extract_repository_name(url: &str) -> Option<String> {
        let git_url = Self::parse_url(url).ok()?;
        git_url.repository
    }

    /// Extract repository owner from URL
    pub fn extract_repository_owner(url: &str) -> Option<String> {
        let git_url = Self::parse_url(url).ok()?;
        git_url.owner
    }

    /// Extract host from URL
    pub fn extract_host(url: &str) -> Option<String> {
        let git_url = Self::parse_url(url).ok()?;
        git_url.host
    }

    /// Generate a default local directory name from a Git URL
    pub fn generate_directory_name(url: &str) -> String {
        Self::extract_repository_name(url)
            .unwrap_or_else(|| {
                // Fallback: use the last path component
                let git_url = Self::parse_url(url).unwrap_or_else(|_| GitUrl {
                    original: url.to_string(),
                    normalized: url.to_string(),
                    protocol: GitProtocol::Unknown,
                    host: None,
                    owner: None,
                    repository: None,
                    path_components: vec![],
                    query_params: HashMap::new(),
                    fragment: None,
                });

                git_url.path_components
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "repository".to_string())
            })
            .trim_end_matches(".git")
            .to_string()
    }

    /// Check if a URL is accessible (basic connectivity check)
    pub async fn check_url_accessible(url: &str) -> Result<bool> {
        let git_url = Self::parse_url(url)?;

        match git_url.protocol {
            GitProtocol::File => {
                // Check if local path exists
                Ok(Path::new(&url.replace("file://", "")).exists())
            }
            GitProtocol::Https | GitProtocol::Ssh | GitProtocol::Git => {
                // For remote URLs, we could implement a connectivity check here
                // For now, just validate the URL format
                Ok(git_url.host.is_some() && git_url.repository.is_some())
            }
            GitProtocol::Unknown => Ok(false),
        }
    }

    /// Sanitize a string for use as a directory name
    pub fn sanitize_directory_name(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c if c.is_control() => '_',
                c => c,
            })
            .collect::<String>()
            .trim_end_matches(".git")
            .to_string()
    }

    /// Get common Git host domains
    pub fn common_git_hosts() -> Vec<&'static str> {
        vec![
            "github.com",
            "gitlab.com",
            "bitbucket.org",
            "git.sr.ht",
            "codeberg.org",
            "gitea.com",
        ]
    }

    /// Check if a host is a known Git hosting service
    pub fn is_known_git_host(host: &str) -> bool {
        Self::common_git_hosts().contains(&host)
    }

    /// Determine if a repository URL is likely public
    pub fn is_likely_public_url(url: &str) -> bool {
        if let Ok(git_url) = Self::parse_url(url) {
            match git_url.protocol {
                GitProtocol::Https => {
                    // HTTPS URLs are typically public unless they use authentication
                    !url.contains("@") && !url.contains("token")
                }
                GitProtocol::Git => true, // Git protocol is typically public
                GitProtocol::Ssh => false, // SSH requires authentication
                GitProtocol::File => true, // Local repositories are always accessible
                GitProtocol::Unknown => false,
            }
        } else {
            false
        }
    }

    /// Convert URL to a canonical form for comparison
    pub fn canonicalize_url(url: &str) -> Result<String> {
        let git_url = Self::parse_url(url)?;

        match git_url.protocol {
            GitProtocol::Https | GitProtocol::Git => {
                if let (Some(host), Some(owner), Some(repo)) = (&git_url.host, &git_url.owner, &git_url.repository) {
                    let repo_name = repo.trim_end_matches(".git");
                    Ok(format!("https://{}/{}/{}", host, owner, repo_name))
                } else {
                    Err(crate::CziError::git("Cannot canonicalize URL - missing components".to_string()))
                }
            }
            _ => Ok(url.to_string()), // Return as-is for other protocols
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_url() {
        let url = "https://github.com/owner/repo.git";
        let git_url = GitUtils::parse_url(url).unwrap();

        assert_eq!(git_url.protocol, GitProtocol::Https);
        assert_eq!(git_url.host, Some("github.com".to_string()));
        assert_eq!(git_url.owner, Some("owner".to_string()));
        assert_eq!(git_url.repository, Some("repo.git".to_string()));
    }

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@github.com:owner/repo.git";
        let git_url = GitUtils::parse_url(url).unwrap();

        assert_eq!(git_url.protocol, GitProtocol::Ssh);
        assert_eq!(git_url.host, Some("github.com".to_string()));
        assert_eq!(git_url.owner, Some("owner".to_string()));
        assert_eq!(git_url.repository, Some("repo.git".to_string()));
    }

    #[test]
    fn test_extract_repository_name() {
        assert_eq!(
            GitUtils::extract_repository_name("https://github.com/owner/repo.git"),
            Some("repo.git".to_string())
        );

        assert_eq!(
            GitUtils::extract_repository_name("git@github.com:owner/another-repo"),
            Some("another-repo".to_string())
        );
    }

    #[test]
    fn test_generate_directory_name() {
        assert_eq!(
            GitUtils::generate_directory_name("https://github.com/owner/repo.git"),
            "repo"
        );

        assert_eq!(
            GitUtils::generate_directory_name("https://github.com/owner/another-repo"),
            "another-repo"
        );
    }

    #[test]
    fn test_is_git_url() {
        assert!(GitUtils::is_git_url("https://github.com/owner/repo.git"));
        assert!(GitUtils::is_git_url("git@github.com:owner/repo.git"));
        assert!(GitUtils::is_git_url("git://github.com/owner/repo.git"));
        assert!(GitUtils::is_git_url("/path/to/local/repo.git"));
        assert!(!GitUtils::is_git_url("https://example.com/not-a-repo"));
    }

    #[test]
    fn test_to_https_url() {
        let ssh_url = "git@github.com:owner/repo.git";
        let https_url = GitUtils::to_https_url(ssh_url).unwrap();
        assert_eq!(https_url, "https://github.com/owner/repo.git");

        let existing_https = "https://github.com/owner/repo.git";
        assert_eq!(GitUtils::to_https_url(existing_https).unwrap(), existing_https);
    }

    #[test]
    fn test_sanitize_directory_name() {
        assert_eq!(GitUtils::sanitize_directory_name("my/repo"), "my_repo");
        assert_eq!(GitUtils::sanitize_directory_name("my:repo"), "my_repo");
        assert_eq!(GitUtils::sanitize_directory_name("repo.git"), "repo");
        assert_eq!(GitUtils::sanitize_directory_name("repo"), "repo");
    }

    #[test]
    fn test_is_known_git_host() {
        assert!(GitUtils::is_known_git_host("github.com"));
        assert!(GitUtils::is_known_git_host("gitlab.com"));
        assert!(GitUtils::is_known_git_host("bitbucket.org"));
        assert!(!GitUtils::is_known_git_host("unknown-host.com"));
    }

    #[test]
    fn test_is_likely_public_url() {
        assert!(GitUtils::is_likely_public_url("https://github.com/owner/repo.git"));
        assert!(GitUtils::is_likely_public_url("git://github.com/owner/repo.git"));
        assert!(!GitUtils::is_likely_public_url("git@github.com:owner/repo.git"));
        assert!(GitUtils::is_likely_public_url("/path/to/local/repo.git"));
    }

    #[tokio::test]
    async fn test_check_url_accessible() {
        let local_path = std::env::current_dir().unwrap();
        let file_url = format!("file://{}", local_path.display());

        assert!(GitUtils::check_url_accessible(&file_url).await.unwrap());
        assert!(!GitUtils::check_url_accessible("https://invalid-url-that-does-not-exist.com/repo.git").await.unwrap());
    }
}