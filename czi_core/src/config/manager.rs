//! Configuration management for CodeZombiesInvestigator
//!
//! Handles loading, validation, and persistence of configuration files
//! in JSON and YAML formats, plus environment variable support.

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug, instrument};

/// Main configuration structure for CZI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CziConfig {
    /// Application settings
    pub app: AppConfig,
    /// Repository configurations
    pub repositories: Vec<RepositoryConfig>,
    /// Active root nodes configuration
    pub active_root_nodes: Vec<RootNodeConfig>,
    /// Analysis configuration
    pub analysis: AnalysisConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// Application-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Data directory for storing analysis results
    pub data_dir: PathBuf,
    /// Cache directory for repositories
    pub cache_dir: PathBuf,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Enable debug mode
    pub debug: bool,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Unique repository identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Git repository URL
    pub url: String,
    /// Local cache path
    pub local_path: Option<PathBuf>,
    /// Branch to analyze (default: main)
    pub branch: String,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Enable this repository for analysis
    pub enabled: bool,
    /// Last synchronization timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    /// No authentication (public repositories)
    None,
    /// SSH key authentication
    SshKey {
        /// Path to SSH private key
        key_path: PathBuf,
        /// Optional key passphrase
        passphrase: Option<String>,
    },
    /// Personal access token
    Token {
        /// Authentication token
        token: String,
        /// Optional username for token auth
        username: Option<String>,
    },
    /// Basic username/password authentication
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

/// Root node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootNodeConfig {
    /// Unique identifier
    pub id: String,
    /// Repository this node belongs to
    pub repository_id: String,
    /// Type of root node
    pub node_type: RootNodeType,
    /// Symbol path (e.g., "com.example.Controller.method")
    pub symbol_path: String,
    /// File path relative to repository root
    pub file_path: String,
    /// Line number where symbol is defined
    pub line_number: Option<u32>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of root nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RootNodeType {
    /// HTTP/API endpoint controllers
    Controller,
    /// Scheduled jobs
    Scheduler,
    /// Message queue listeners
    Listener,
    /// Application entry points
    Main,
    /// Other custom entry points
    Custom,
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Languages to analyze (empty = auto-detect)
    pub languages: Vec<String>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Maximum file size to analyze (in bytes)
    pub max_file_size_bytes: u64,
    /// Enable semantic link detection
    pub enable_semantic_links: bool,
    /// Minimum confidence threshold for zombie detection
    pub min_confidence_threshold: f64,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum memory usage (in MB)
    pub max_memory_mb: u64,
    /// Maximum analysis time (in minutes)
    pub max_analysis_time_minutes: u64,
    /// Batch size for processing files
    pub file_batch_size: usize,
    /// Enable incremental analysis
    pub enable_incremental: bool,
    /// Number of parser threads (0 = auto)
    pub parser_threads: usize,
}

impl Default for CziConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            repositories: Vec::new(),
            active_root_nodes: Vec::new(),
            analysis: AnalysisConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            data_dir: PathBuf::from("./data"),
            cache_dir: PathBuf::from("./cache"),
            max_concurrent_operations: 4,
            debug: false,
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                "java".to_string(),
                "javascript".to_string(),
                "python".to_string(),
                "shell".to_string(),
            ],
            include_patterns: vec![
                "**/*.java".to_string(),
                "**/*.js".to_string(),
                "**/*.mjs".to_string(),
                "**/*.py".to_string(),
                "**/*.sh".to_string(),
                "**/*.bash".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
                "**/.git/**".to_string(),
                "**/*.min.js".to_string(),
            ],
            max_file_size_bytes: 1024 * 1024, // 1MB
            enable_semantic_links: true,
            min_confidence_threshold: 0.7,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048, // 2GB
            max_analysis_time_minutes: 120, // 2 hours
            file_batch_size: 100,
            enable_incremental: true,
            parser_threads: 0, // Auto-detect
        }
    }
}

/// Configuration manager for loading and saving configurations
pub struct ConfigManager {
    config_path: PathBuf,
    env_prefix: String,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            env_prefix: "CZI_".to_string(),
        }
    }

    /// Set custom environment variable prefix
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Load configuration from file and environment variables
    #[instrument(skip(self))]
    pub fn load_config(&self) -> Result<CziConfig> {
        let mut config = if self.config_path.exists() {
            info!("Loading configuration from: {:?}", self.config_path);
            self.load_from_file(&self.config_path)?
        } else {
            warn!("Configuration file not found, using defaults: {:?}", self.config_path);
            CziConfig::default()
        };

        // Override with environment variables
        self.apply_env_overrides(&mut config);

        // Validate configuration
        self.validate_config(&config)?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to file
    #[instrument(skip(self, config))]
    pub fn save_config(&self, config: &CziConfig) -> Result<()> {
        // Validate before saving
        self.validate_config(config)?;

        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = if self.config_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref() == Some("yaml") ||
            self.config_path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref() == Some("yml") {
            serde_yaml::to_string(config)?
        } else {
            serde_json::to_string_pretty(config)?
        };

        fs::write(&self.config_path, content)?;
        info!("Configuration saved to: {:?}", self.config_path);
        Ok(())
    }

    /// Load configuration from file
    fn load_from_file(&self, path: &Path) -> Result<CziConfig> {
        let content = fs::read_to_string(path)?;

        let config = if path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref() == Some("yaml") ||
            path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref() == Some("yml") {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };

        debug!("Configuration loaded from file: {:?}", path);
        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&self, config: &mut CziConfig) {
        // App configuration overrides
        if let Ok(log_level) = std::env::var(&format!("{}LOG_LEVEL", self.env_prefix)) {
            config.app.log_level = log_level;
        }

        if let Ok(data_dir) = std::env::var(&format!("{}DATA_DIR", self.env_prefix)) {
            config.app.data_dir = PathBuf::from(data_dir);
        }

        if let Ok(cache_dir) = std::env::var(&format!("{}CACHE_DIR", self.env_prefix)) {
            config.app.cache_dir = PathBuf::from(cache_dir);
        }

        if let Ok(max_concurrent) = std::env::var(&format!("{}MAX_CONCURRENT", self.env_prefix)) {
            if let Ok(val) = max_concurrent.parse() {
                config.app.max_concurrent_operations = val;
            }
        }

        // Performance configuration overrides
        if let Ok(max_memory) = std::env::var(&format!("{}MAX_MEMORY_MB", self.env_prefix)) {
            if let Ok(val) = max_memory.parse() {
                config.performance.max_memory_mb = val;
            }
        }

        if let Ok(max_time) = std::env::var(&format!("{}MAX_ANALYSIS_TIME", self.env_prefix)) {
            if let Ok(val) = max_time.parse() {
                config.performance.max_analysis_time_minutes = val;
            }
        }

        debug!("Environment variable overrides applied");
    }

    /// Validate configuration
    fn validate_config(&self, config: &CziConfig) -> Result<()> {
        // Validate app configuration
        if config.app.max_concurrent_operations == 0 {
            return Err(CziError::validation("Max concurrent operations must be greater than 0"));
        }

        // Validate repository configurations
        for repo in &config.repositories {
            if repo.id.is_empty() {
                return Err(CziError::validation("Repository ID cannot be empty"));
            }
            if repo.name.is_empty() {
                return Err(CziError::validation("Repository name cannot be empty"));
            }
            if repo.url.is_empty() {
                return Err(CziError::validation("Repository URL cannot be empty"));
            }
        }

        // Validate root node configurations
        for root in &config.active_root_nodes {
            if root.id.is_empty() {
                return Err(CziError::validation("Root node ID cannot be empty"));
            }
            if root.repository_id.is_empty() {
                return Err(CziError::validation("Root node repository ID cannot be empty"));
            }
            if root.symbol_path.is_empty() {
                return Err(CziError::validation("Root node symbol path cannot be empty"));
            }
        }

        // Validate performance configuration
        if config.performance.max_memory_mb == 0 {
            return Err(CziError::validation("Max memory MB must be greater than 0"));
        }

        if config.performance.max_analysis_time_minutes == 0 {
            return Err(CziError::validation("Max analysis time minutes must be greater than 0"));
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Get default configuration file path
    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("czi")
            .join("config.json")
    }

    /// Ensure configuration directories exist
    pub fn ensure_config_dirs(&self, config: &CziConfig) -> Result<()> {
        // Create data directory
        if !config.app.data_dir.exists() {
            fs::create_dir_all(&config.app.data_dir)?;
            info!("Created data directory: {:?}", config.app.data_dir);
        }

        // Create cache directory
        if !config.app.cache_dir.exists() {
            fs::create_dir_all(&config.app.cache_dir)?;
            info!("Created cache directory: {:?}", config.app.cache_dir);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = CziConfig::default();
        assert_eq!(config.app.log_level, "info");
        assert_eq!(config.app.max_concurrent_operations, 4);
        assert!(!config.repositories.is_empty()); // Actually empty by default
        assert!(!config.analysis.languages.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = CziConfig::default();
        config.app.max_concurrent_operations = 0;

        let manager = ConfigManager::new("test.json");
        assert!(manager.validate_config(&config).is_err());

        config.app.max_concurrent_operations = 4;
        assert!(manager.validate_config(&config).is_ok());
    }

    #[test]
    fn test_config_save_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.json");
        let manager = ConfigManager::new(&config_path);

        let mut config = CziConfig::default();
        config.app.debug = true;
        config.app.max_concurrent_operations = 8;

        manager.save_config(&config)?;
        let loaded_config = manager.load_config()?;

        assert_eq!(loaded_config.app.debug, true);
        assert_eq!(loaded_config.app.max_concurrent_operations, 8);

        Ok(())
    }

    #[test]
    fn test_env_overrides() {
        let mut config = CziConfig::default();
        config.app.log_level = "debug".to_string();

        std::env::set_var("CZI_LOG_LEVEL", "error");
        let manager = ConfigManager::new("test.json");
        manager.apply_env_overrides(&mut config);

        assert_eq!(config.app.log_level, "error");
        std::env::remove_var("CZI_LOG_LEVEL");
    }
}