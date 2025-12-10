//! CodeZombiesInvestigator Core Library
//!
//! This library provides the core analysis engine for identifying dead code
//! and isolated modules in multi-repository codebases.

pub mod error;
pub mod logging;
pub mod config;
pub mod io;
pub mod parser;
pub mod graph;
pub mod git;
pub mod runtime;
pub mod analysis;

#[cfg(test)]
mod tests;

// Re-export common types for convenience
pub use error::{CziError, Result};
pub use logging::{init_logging, OperationTimer, PerformanceMetrics};
pub use config::{
    RepositoryConfiguration, RepositoryConfig as ConfigRepositoryConfig, RootNodeConfig,
    AuthType, RepositoryStatus, CziConfig, ConfigManager
};
pub use graph::edge::{DependencyEdge, EdgeType};
pub use parser::{TreeSitterManager, SupportedLanguage, LanguageQueries, CodeSymbol, SymbolType};
pub use git::{
    GitWrapper, GitRepository, GitOperations, DiffAnalyzer, BlameAnalyzer,
    GitRepositoryConfig, repository::GitAuthConfig, CloneOptions, GitFetchOptions, PullOptions,
    GitUrl, GitUtils, FileChange, ChangeType, BlameInfo
};
pub use runtime::{
    CziRuntime, RuntimeConfig, RuntimeStats, TaskPool, TaskScheduler,
    RuntimeMonitor, TaskWorker, CziExecutor
};
pub use analysis::results::{AnalysisResult, ZombieCodeItem};