//! Configuration management for CodeZombiesInvestigator

pub mod repository;
pub mod auth;
pub mod manager;
pub mod root_node;

// Re-export main types
pub use repository::{RepositoryConfiguration, RepositoryStatus, AuthType, AuthConfig};
pub use manager::{CziConfig, ConfigManager, AppConfig, RepositoryConfig as RepositoryConfigFull, RootNodeConfig, RootNodeType, AnalysisConfig, PerformanceConfig};
pub use root_node::{ActiveRootNode, RootNodeDetector, RootNodeType as ActiveRootNodeType};