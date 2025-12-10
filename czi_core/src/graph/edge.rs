//! Dependency edge definitions for code analysis graphs
//!
//! Dependency edges represent relationships between code symbols in the analysis graph.
//! These relationships are used to determine code reachability and identify zombie code.

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, trace};
use uuid::Uuid;

/// Dependency edge representing a relationship between two code symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyEdge {
    /// Unique identifier for this edge
    pub id: String,

    /// Source symbol ID (the dependent)
    pub source_symbol_id: String,

    /// Target symbol ID (the dependency)
    pub target_symbol_id: String,

    /// Type of dependency relationship
    pub edge_type: EdgeType,

    /// Confidence score for this dependency (0.0 to 1.0)
    pub confidence: f64,

    /// Whether this dependency is strong (likely to be preserved)
    pub strong: bool,

    /// Line number where this dependency occurs (if applicable)
    pub line_number: Option<u32>,

    /// File where this dependency occurs
    pub file_path: Option<String>,

    /// Additional metadata about this dependency
    pub metadata: HashMap<String, String>,

    /// When this dependency was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Types of dependency edges between code symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Function or method call relationship
    Calls,

    /// Import/require/include relationship
    Imports,

    /// Class inheritance relationship
    Extends,

    /// Interface implementation relationship
    Implements,

    /// Class composition/field reference
    Uses,

    /// Type annotation/reference
    References,

    /// Variable assignment
    Assigns,

    /// Data flow relationship
    DataFlow,

    /// Control flow relationship
    ControlFlow,

    /// Configuration/binding relationship
    Binds,

    /// Event subscription/listener relationship
    ListensTo,

    /// API endpoint routing
    RoutesTo,

    /// Database/table relationship
    Queries,

    /// File I/O relationship
    Reads,

    /// File I/O relationship
    Writes,

    /// HTTP request/response relationship
    Requests,

    /// Message queue relationship
    Publishes,

    /// Message queue relationship
    Consumes,

    /// Other custom dependency type
    Other(String),
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Calls => write!(f, "calls"),
            EdgeType::Imports => write!(f, "imports"),
            EdgeType::Extends => write!(f, "extends"),
            EdgeType::Implements => write!(f, "implements"),
            EdgeType::Uses => write!(f, "uses"),
            EdgeType::References => write!(f, "references"),
            EdgeType::Assigns => write!(f, "assigns"),
            EdgeType::DataFlow => write!(f, "data_flow"),
            EdgeType::ControlFlow => write!(f, "control_flow"),
            EdgeType::Binds => write!(f, "binds"),
            EdgeType::ListensTo => write!(f, "listens_to"),
            EdgeType::RoutesTo => write!(f, "routes_to"),
            EdgeType::Queries => write!(f, "queries"),
            EdgeType::Reads => write!(f, "reads"),
            EdgeType::Writes => write!(f, "writes"),
            EdgeType::Requests => write!(f, "requests"),
            EdgeType::Publishes => write!(f, "publishes"),
            EdgeType::Consumes => write!(f, "consumes"),
            EdgeType::Other(s) => write!(f, "other({})", s),
        }
    }
}

impl std::fmt::Display for DependencyEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.source_symbol_id, self.target_symbol_id)
    }
}

/// Edge metadata builder
pub struct EdgeMetadataBuilder {
    metadata: HashMap<String, String>,
}

impl EdgeMetadataBuilder {
    /// Create a new metadata builder
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Add a metadata key-value pair
    pub fn add(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add call frequency
    pub fn call_frequency(mut self, frequency: u32) -> Self {
        self.metadata.insert("call_frequency".to_string(), frequency.to_string());
        self
    }

    /// Add call depth
    pub fn call_depth(mut self, depth: u32) -> Self {
        self.metadata.insert("call_depth".to_string(), depth.to_string());
        self
    }

    /// Add import type (e.g., "static", "dynamic")
    pub fn import_type(mut self, import_type: String) -> Self {
        self.metadata.insert("import_type".to_string(), import_type);
        self
    }

    /// Add relationship strength
    pub fn relationship_strength(mut self, strength: f64) -> Self {
        self.metadata.insert("relationship_strength".to_string(), strength.to_string());
        self
    }

    /// Add context information
    pub fn context(mut self, context: String) -> Self {
        self.metadata.insert("context".to_string(), context);
        self
    }

    /// Add discovery method (e.g., "static_analysis", "runtime", "manual")
    pub fn discovery_method(mut self, method: String) -> Self {
        self.metadata.insert("discovery_method".to_string(), method);
        self
    }

    /// Build the metadata
    pub fn build(self) -> HashMap<String, String> {
        self.metadata
    }
}

impl Default for EdgeMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyEdge {
    /// Create a new dependency edge
    pub fn new(
        id: String,
        source_symbol_id: String,
        target_symbol_id: String,
        edge_type: EdgeType,
    ) -> Self {
        Self {
            id,
            source_symbol_id,
            target_symbol_id,
            edge_type,
            confidence: 1.0,
            strong: false,
            line_number: None,
            file_path: None,
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
        }
    }

    /// Create an edge with additional details
    pub fn with_details(
        id: String,
        source_symbol_id: String,
        target_symbol_id: String,
        edge_type: EdgeType,
        confidence: f64,
        line_number: Option<u32>,
        file_path: Option<String>,
    ) -> Self {
        Self {
            id,
            source_symbol_id,
            target_symbol_id,
            edge_type,
            confidence,
            strong: confidence >= 0.7,
            line_number,
            file_path,
            metadata: HashMap::new(),
            discovered_at: chrono::Utc::now(),
        }
    }

    /// Validate the edge
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(CziError::validation("Edge ID cannot be empty"));
        }

        if self.source_symbol_id.is_empty() {
            return Err(CziError::validation("Source symbol ID cannot be empty"));
        }

        if self.target_symbol_id.is_empty() {
            return Err(CziError::validation("Target symbol ID cannot be empty"));
        }

        if self.source_symbol_id == self.target_symbol_id {
            return Err(CziError::validation("Source and target symbol IDs cannot be the same"));
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(CziError::validation("Confidence must be between 0.0 and 1.0"));
        }

        Ok(())
    }

    /// Add metadata to this edge
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set confidence score
    pub fn set_confidence(&mut self, confidence: f64) {
        self.confidence = confidence.clamp(0.0, 1.0);
        self.strong = self.confidence >= 0.7;
    }

    /// Set line number
    pub fn set_line_number(&mut self, line_number: u32) {
        self.line_number = Some(line_number);
    }

    /// Set file path
    pub fn set_file_path(&mut self, file_path: String) {
        self.file_path = Some(file_path);
    }

    /// Mark as strong dependency
    pub fn mark_strong(&mut self) {
        self.strong = true;
        self.confidence = self.confidence.max(0.7);
    }

    /// Mark as weak dependency
    pub fn mark_weak(&mut self) {
        self.strong = false;
        self.confidence = self.confidence.min(0.6);
    }

    /// Check if this is a strong dependency
    pub fn is_strong(&self) -> bool {
        self.strong || self.confidence >= 0.7
    }

    /// Get edge display name
    pub fn display_name(&self) -> String {
        match &self.edge_type {
            EdgeType::Calls => format!("{} calls {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Imports => format!("{} imports {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Extends => format!("{} extends {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Implements => format!("{} implements {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Uses => format!("{} uses {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::References => format!("{} references {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Assigns => format!("{} assigns to {}", self.source_symbol_id, self.target_symbol_id),
            EdgeType::Other(custom_type) => format!("{} {} {}", self.source_symbol_id, custom_type, self.target_symbol_id),
            _ => format!("{} {} {}", self.source_symbol_id, self.edge_type, self.target_symbol_id),
        }
    }

    /// Get edge location string
    pub fn location(&self) -> Option<String> {
        if let Some(file_path) = &self.file_path {
            if let Some(line_number) = self.line_number {
                Some(format!("{}:{}", file_path, line_number))
            } else {
                Some(file_path.clone())
            }
        } else {
            None
        }
    }

    /// Check if this is a transitive dependency (can be removed in analysis)
    pub fn is_transitive(&self) -> bool {
        match &self.edge_type {
            EdgeType::DataFlow | EdgeType::ControlFlow => false, // Keep these
            EdgeType::Calls | EdgeType::Uses | EdgeType::References => !self.is_strong(),
            EdgeType::Imports | EdgeType::Extends | EdgeType::Implements => true, // Keep these
            _ => !self.is_strong(),
        }
    }

    /// Check if this edge represents a critical dependency
    pub fn is_critical(&self) -> bool {
        match &self.edge_type {
            EdgeType::Extends | EdgeType::Implements => true,
            EdgeType::Calls | EdgeType::Uses => self.is_strong(),
            _ => false,
        }
    }
}

impl Default for EdgeType {
    fn default() -> Self {
        Self::Other("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_edge_creation() {
        let edge = DependencyEdge::new(
            "edge_1".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Calls,
        );

        assert_eq!(edge.id, "edge_1");
        assert_eq!(edge.source_symbol_id, "source_sym");
        assert_eq!(edge.target_symbol_id, "target_sym");
        assert_eq!(edge.edge_type, EdgeType::Calls);
        assert_eq!(edge.confidence, 1.0);
        assert!(edge.is_strong());
        assert!(edge.validate().is_ok());
    }

    #[test]
    fn test_dependency_edge_with_details() {
        let edge = DependencyEdge::with_details(
            "edge_2".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Imports,
            0.8,
            Some(15),
            Some("src/main/java/Test.java".to_string()),
        );

        assert_eq!(edge.confidence, 0.8);
        assert_eq!(edge.line_number, Some(15));
        assert_eq!(edge.file_path, Some("src/main/java/Test.java".to_string()));
        assert!(edge.is_strong());
    }

    #[test]
    fn test_dependency_edge_validation() {
        let edge = DependencyEdge::new(
            "edge_1".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Calls,
        );

        // Valid edge should pass validation
        assert!(edge.validate().is_ok());

        // Invalid edge (same source and target) should fail validation
        let invalid_edge = DependencyEdge::new(
            "edge_invalid".to_string(),
            "same_sym".to_string(),
            "same_sym".to_string(),
            EdgeType::Calls,
        );
        assert!(invalid_edge.validate().is_err());
    }

    #[test]
    fn test_dependency_edge_metadata() {
        let mut edge = DependencyEdge::new(
            "edge_1".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Calls,
        );

        edge.add_metadata("call_frequency".to_string(), "5".to_string());
        edge.add_metadata("context".to_string(), "method_call".to_string());

        assert_eq!(edge.get_metadata("call_frequency"), Some(&"5".to_string()));
        assert_eq!(edge.get_metadata("context"), Some(&"method_call".to_string()));
        assert_eq!(edge.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_edge_metadata_builder() {
        let metadata = EdgeMetadataBuilder::new()
            .call_frequency(10)
            .call_depth(3)
            .import_type("static".to_string())
            .relationship_strength(0.9)
            .context("function_call".to_string())
            .discovery_method("static_analysis".to_string())
            .build();

        assert_eq!(metadata.get("call_frequency"), Some(&"10".to_string()));
        assert_eq!(metadata.get("call_depth"), Some(&"3".to_string()));
        assert_eq!(metadata.get("import_type"), Some(&"static".to_string()));
        assert_eq!(metadata.get("relationship_strength"), Some(&"0.9".to_string()));
        assert_eq!(metadata.get("context"), Some(&"function_call".to_string()));
        assert_eq!(metadata.get("discovery_method"), Some(&"static_analysis".to_string()));
    }

    #[test]
    fn test_edge_confidence() {
        let mut edge = DependencyEdge::new(
            "edge_1".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Calls,
        );

        // High confidence should make it strong
        edge.set_confidence(0.8);
        assert_eq!(edge.confidence, 0.8);
        assert!(edge.is_strong());

        // Low confidence should make it weak
        edge.set_confidence(0.5);
        assert_eq!(edge.confidence, 0.5);
        assert!(!edge.is_strong());

        // Clamping should work
        edge.set_confidence(1.5);
        assert_eq!(edge.confidence, 1.0);

        edge.set_confidence(-0.5);
        assert_eq!(edge.confidence, 0.0);
    }

    #[test]
    fn test_edge_display_name() {
        let edge = DependencyEdge::new(
            "edge_1".to_string(),
            "ClassA.method".to_string(),
            "ClassB.method".to_string(),
            EdgeType::Calls,
        );

        assert_eq!(edge.display_name(), "ClassA.method calls ClassB.method");

        let import_edge = DependencyEdge::new(
            "edge_2".to_string(),
            "ClassA".to_string(),
            "package.ClassB".to_string(),
            EdgeType::Imports,
        );

        assert_eq!(import_edge.display_name(), "ClassA imports package.ClassB");

        let extends_edge = DependencyEdge::new(
            "edge_3".to_string(),
            "ChildClass".to_string(),
            "ParentClass".to_string(),
            EdgeType::Extends,
        );

        assert_eq!(extends_edge.display_name(), "ChildClass extends ParentClass");
    }

    #[test]
    fn test_edge_location() {
        let mut edge = DependencyEdge::new(
            "edge_1".to_string(),
            "source_sym".to_string(),
            "target_sym".to_string(),
            EdgeType::Calls,
        );

        assert_eq!(edge.location(), None);

        edge.set_file_path("src/main/java/Test.java".to_string());
        assert_eq!(edge.location(), Some("src/main/java/Test.java".to_string()));

        edge.set_line_number(25);
        assert_eq!(edge.location(), Some("src/main/java/Test.java:25".to_string()));
    }

    #[test]
    fn test_edge_criticality() {
        let extends_edge = DependencyEdge::new(
            "edge_1".to_string(),
            "Child".to_string(),
            "Parent".to_string(),
            EdgeType::Extends,
        );
        assert!(extends_edge.is_critical());

        let implements_edge = DependencyEdge::new(
            "edge_2".to_string(),
            "Class".to_string(),
            "Interface".to_string(),
            EdgeType::Implements,
        );
        assert!(implements_edge.is_critical());

        let mut weak_call_edge = DependencyEdge::new(
            "edge_3".to_string(),
            "Caller".to_string(),
            "Callee".to_string(),
            EdgeType::Calls,
        );
        weak_call_edge.set_confidence(0.5);
        assert!(!weak_call_edge.is_critical());

        let mut strong_call_edge = DependencyEdge::new(
            "edge_4".to_string(),
            "Caller".to_string(),
            "Callee".to_string(),
            EdgeType::Calls,
        );
        strong_call_edge.set_confidence(0.8);
        assert!(strong_call_edge.is_critical());
    }

    #[test]
    fn test_edge_type_equality() {
        assert_eq!(EdgeType::Calls, EdgeType::Calls);
        assert_eq!(EdgeType::Imports, EdgeType::Imports);
        assert_eq!(EdgeType::Other("custom".to_string()), EdgeType::Other("custom".to_string()));
        assert_ne!(EdgeType::Calls, EdgeType::Imports);
    }

    #[test]
    fn test_custom_edge_type() {
        let custom_edge = DependencyEdge::new(
            "edge_custom".to_string(),
            "source".to_string(),
            "target".to_string(),
            EdgeType::Other("custom_dependency".to_string()),
        );

        assert_eq!(custom_edge.edge_type, EdgeType::Other("custom_dependency".to_string()));
        assert_eq!(custom_edge.display_name(), "source custom_dependency target");
    }
}