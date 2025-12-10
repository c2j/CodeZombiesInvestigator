//! I/O operations for CodeZombiesInvestigator

pub mod validator;
pub mod sync;
pub mod git_history;

// Re-export commonly used types
pub use validator::RepositoryValidator;
pub use sync::{RepositorySyncService, SyncResult, SyncStats};