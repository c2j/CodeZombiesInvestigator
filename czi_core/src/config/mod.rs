//! Configuration management for CodeZombiesInvestigator

pub mod repository;
pub mod auth;
pub mod manager;
pub mod root_node;
pub mod storage;

// Re-export main types
pub use repository::{RepositoryConfiguration, RepositoryStatus, AuthType};
pub use manager::{CziConfig, ConfigManager, AppConfig, RepositoryConfig, RootNodeConfig, RootNodeType, AnalysisConfig, PerformanceConfig, AuthConfig};
pub use root_node::{ActiveRootNode, RootNodeDetector, RootNodeType as ActiveRootNodeType};
pub use storage::ConfigStorage;