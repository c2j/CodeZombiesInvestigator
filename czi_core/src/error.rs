//! Error handling for CodeZombiesInvestigator core library

use thiserror::Error;

/// Main error type for the CZI core library
#[derive(Error, Debug)]
pub enum CziError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("YAML serialization error: {0}")]
    YamlSerialization(#[from] serde_yaml::Error),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {operation}")]
    Timeout { operation: String },

    #[error("Rate limited: {operation}")]
    RateLimited { operation: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for CZI operations
pub type Result<T> = std::result::Result<T, CziError>;

impl CziError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        CziError::Config(msg.into())
    }

    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        CziError::Parse(msg.into())
    }

    /// Create a graph error
    pub fn graph<S: Into<String>>(msg: S) -> Self {
        CziError::Graph(msg.into())
    }

    /// Create an analysis error
    pub fn analysis<S: Into<String>>(msg: S) -> Self {
        CziError::Analysis(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        CziError::Validation(msg.into())
    }

    /// Create a repository error
    pub fn repository<S: Into<String>>(msg: S) -> Self {
        CziError::Repository(msg.into())
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        CziError::Auth(msg.into())
    }

    /// Create a "not found" error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        CziError::NotFound {
            resource: resource.into(),
        }
    }

    /// Create an "already exists" error
    pub fn already_exists<S: Into<String>>(resource: S) -> Self {
        CziError::AlreadyExists {
            resource: resource.into(),
        }
    }

    /// Create an "invalid state" error
    pub fn invalid_state<S: Into<String>>(msg: S) -> Self {
        CziError::InvalidState(msg.into())
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        CziError::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a rate limited error
    pub fn rate_limited<S: Into<String>>(operation: S) -> Self {
        CziError::RateLimited {
            operation: operation.into(),
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        CziError::Network {
            message: msg.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(operation: S) -> Self {
        CziError::PermissionDenied {
            operation: operation.into(),
        }
    }

    /// Create an unsupported operation error
    pub fn unsupported<S: Into<String>>(msg: S) -> Self {
        CziError::Unsupported(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        CziError::Internal(msg.into())
    }

    /// Create a runtime error
    pub fn runtime<S: Into<String>>(msg: S) -> Self {
        CziError::Runtime(msg.into())
    }

    /// Create a git error
    pub fn git<S: Into<String>>(msg: S) -> Self {
        CziError::Git(git2::Error::from_str(&msg.into()))
    }

    /// Create an IPC error
    pub fn ipc<S: Into<String>>(msg: S) -> Self {
        CziError::Ipc(msg.into())
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        CziError::Other(msg.into())
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CziError::Timeout { .. }
                | CziError::RateLimited { .. }
                | CziError::Network { .. }
                | CziError::Repository(_)
        )
    }

    /// Get error category for grouping/metrics
    pub fn category(&self) -> &'static str {
        match self {
            CziError::Config(_) => "config",
            CziError::Io(_) => "io",
            CziError::Git(_) => "git",
            CziError::Parse(_) => "parse",
            CziError::Graph(_) => "graph",
            CziError::Analysis(_) => "analysis",
            CziError::Validation(_) => "validation",
            CziError::Serialization(_) => "serialization",
            CziError::YamlSerialization(_) => "yaml_serialization",
            CziError::JsonSerialization(_) => "json_serialization",
            CziError::UrlParse(_) => "url_parse",
            CziError::Repository(_) => "repository",
            CziError::Auth(_) => "auth",
            CziError::NotFound { .. } => "not_found",
            CziError::AlreadyExists { .. } => "already_exists",
            CziError::InvalidState(_) => "invalid_state",
            CziError::Timeout { .. } => "timeout",
            CziError::RateLimited { .. } => "rate_limited",
            CziError::Network { .. } => "network",
            CziError::PermissionDenied { .. } => "permission_denied",
            CziError::Unsupported(_) => "unsupported",
            CziError::Internal(_) => "internal",
            CziError::Runtime(_) => "runtime",
            CziError::Ipc(_) => "ipc",
            CziError::Other(_) => "other",
        }
    }
}

/// Extension trait for adding context to errors
pub trait ResultExt<T> {
    /// Add context to an error
    fn context<C: Into<String>>(self, context: C) -> Result<T>;

    /// Add context to an error with a function
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Into<String>,
        F: FnOnce() -> C;
}

impl<T> ResultExt<T> for Result<T> {
    fn context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| CziError::Other(format!("{}: {}", context.into(), e)))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Into<String>,
        F: FnOnce() -> C,
    {
        self.map_err(|e| CziError::Other(format!("{}: {}", f().into(), e)))
    }
}

/// Macro for creating errors with file context
#[macro_export]
macro_rules! file_context {
    ($operation:expr, $path:expr) => {
        $crate::error::CziError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{}: {}", $operation, $path.display()),
        ))
    };
}

/// Macro for validation errors
#[macro_export]
macro_rules! validation_error {
    ($($arg:tt)*) => {
        $crate::error::CziError::Validation(format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CziError::config("test config error");
        assert!(matches!(err, CziError::Config(_)));
        assert_eq!(err.category(), "config");

        let err = CziError::not_found("test resource");
        assert!(matches!(err, CziError::NotFound { .. }));
        assert_eq!(err.category(), "not_found");
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(CziError::timeout("test").is_recoverable());
        assert!(CziError::rate_limited("test").is_recoverable());
        assert!(CziError::network("test").is_recoverable());
        assert!(CziError::repository("test").is_recoverable());

        assert!(!CziError::config("test").is_recoverable());
        assert!(!CziError::validation("test").is_recoverable());
    }

    #[test]
    fn test_context_extension() {
        let result: Result<i32> = Err(CziError::other("base error"));
        let result = result.context("additional context");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("additional context"));
        assert!(err.to_string().contains("base error"));
    }
}