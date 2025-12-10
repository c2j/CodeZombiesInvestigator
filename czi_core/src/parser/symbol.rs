//! Code symbol extraction and modeling
//!
//! Code symbols represent extracted code entities (functions, classes, methods, variables)
//! that are tracked in the dependency graph for zombie code analysis.

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code symbol representing an extracted code entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeSymbol {
    /// Unique identifier for this symbol
    pub id: String,

    /// Symbol name (e.g., "methodName", "ClassName")
    pub name: String,

    /// Fully qualified symbol name (e.g., "com.example.Controller.methodName")
    pub qualified_name: String,

    /// Type of symbol
    pub symbol_type: SymbolType,

    /// Programming language of this symbol
    pub language: String,

    /// File path where symbol is defined
    pub file_path: String,

    /// Line number where symbol is defined
    pub line_number: u32,

    /// Column number where symbol starts
    pub column: Option<u32>,

    /// Symbol visibility/access level
    pub visibility: SymbolVisibility,

    /// Whether this symbol is exported/public
    pub exported: bool,

    /// Repository this symbol belongs to
    pub repository_id: String,

    /// Symbol metadata
    pub metadata: HashMap<String, String>,

    /// When this symbol was extracted
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for CodeSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.qualified_name, self.symbol_type)
    }
}

/// Types of code symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SymbolType {
    /// Function or method
    Function,

    /// Class or interface
    Class,

    /// Variable or field
    Variable,

    /// Method (function within a class)
    Method,

    /// Interface
    Interface,

    /// Enum
    Enum,

    /// Module or namespace
    Module,

    /// Package
    Package,

    /// Constructor
    Constructor,

    /// Destructor
    Destructor,

    /// Property/attribute
    Property,

    /// Constant
    Constant,

    /// Type alias
    TypeAlias,

    /// Trait/mixin
    Trait,

    /// Annotation/decorator
    Annotation,

    /// Other custom symbol type
    Other(String),
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "function"),
            SymbolType::Class => write!(f, "class"),
            SymbolType::Variable => write!(f, "variable"),
            SymbolType::Method => write!(f, "method"),
            SymbolType::Interface => write!(f, "interface"),
            SymbolType::Enum => write!(f, "enum"),
            SymbolType::Module => write!(f, "module"),
            SymbolType::Package => write!(f, "package"),
            SymbolType::Constructor => write!(f, "constructor"),
            SymbolType::Destructor => write!(f, "destructor"),
            SymbolType::Property => write!(f, "property"),
            SymbolType::Constant => write!(f, "constant"),
            SymbolType::TypeAlias => write!(f, "type_alias"),
            SymbolType::Trait => write!(f, "trait"),
            SymbolType::Annotation => write!(f, "annotation"),
            SymbolType::Other(s) => write!(f, "other({})", s),
        }
    }
}

/// Symbol visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SymbolVisibility {
    /// Public symbol
    Public,

    /// Protected symbol
    Protected,

    /// Private symbol
    Private,

    /// Internal/package-private
    Internal,

    /// File-level visibility
    File,

    /// Unknown visibility
    Unknown,
}

/// Symbol metadata builder
pub struct SymbolMetadataBuilder {
    metadata: HashMap<String, String>,
}

impl SymbolMetadataBuilder {
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

    /// Add complexity metric
    pub fn complexity(mut self, complexity: u32) -> Self {
        self.metadata.insert("complexity".to_string(), complexity.to_string());
        self
    }

    /// Add lines of code metric
    pub fn lines_of_code(mut self, loc: u32) -> Self {
        self.metadata.insert("lines_of_code".to_string(), loc.to_string());
        self
    }

    /// Add cyclomatic complexity
    pub fn cyclomatic_complexity(mut self, cc: u32) -> Self {
        self.metadata.insert("cyclomatic_complexity".to_string(), cc.to_string());
        self
    }

    /// Add parameter count
    pub fn parameter_count(mut self, count: u32) -> Self {
        self.metadata.insert("parameter_count".to_string(), count.to_string());
        self
    }

    /// Add return type
    pub fn return_type(mut self, return_type: String) -> Self {
        self.metadata.insert("return_type".to_string(), return_type);
        self
    }

    /// Add parent class/interface
    pub fn parent_class(mut self, parent: String) -> Self {
        self.metadata.insert("parent_class".to_string(), parent);
        self
    }

    /// Add implemented interfaces
    pub fn interfaces(mut self, interfaces: Vec<String>) -> Self {
        self.metadata.insert("interfaces".to_string(), interfaces.join(","));
        self
    }

    /// Build the metadata
    pub fn build(self) -> HashMap<String, String> {
        self.metadata
    }
}

impl Default for SymbolMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSymbol {
    /// Create a new code symbol
    pub fn new(
        id: String,
        name: String,
        symbol_type: SymbolType,
        file_path: String,
        line_number: u32,
        repository_id: String,
    ) -> Self {
        let qualified_name = name.clone();
        Self {
            id,
            name,
            qualified_name,
            symbol_type,
            language: "unknown".to_string(),
            file_path,
            line_number,
            column: None,
            visibility: SymbolVisibility::Unknown,
            exported: false,
            repository_id,
            metadata: HashMap::new(),
            extracted_at: chrono::Utc::now(),
        }
    }

    /// Create a symbol with additional details
    pub fn with_details(
        id: String,
        name: String,
        qualified_name: String,
        symbol_type: SymbolType,
        language: String,
        file_path: String,
        line_number: u32,
        repository_id: String,
    ) -> Self {
        Self {
            id,
            name,
            qualified_name,
            symbol_type,
            language,
            file_path,
            line_number,
            column: None,
            visibility: SymbolVisibility::Unknown,
            exported: false,
            repository_id,
            metadata: HashMap::new(),
            extracted_at: chrono::Utc::now(),
        }
    }

    /// Validate the symbol
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(CziError::validation("Symbol ID cannot be empty"));
        }

        if self.name.is_empty() {
            return Err(CziError::validation("Symbol name cannot be empty"));
        }

        if self.qualified_name.is_empty() {
            return Err(CziError::validation("Qualified name cannot be empty"));
        }

        if self.file_path.is_empty() {
            return Err(CziError::validation("File path cannot be empty"));
        }

        if self.repository_id.is_empty() {
            return Err(CziError::validation("Repository ID cannot be empty"));
        }

        if self.line_number == 0 {
            return Err(CziError::validation("Line number must be greater than 0"));
        }

        Ok(())
    }

    /// Add metadata to this symbol
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set qualified name
    pub fn set_qualified_name(&mut self, qualified_name: String) {
        self.qualified_name = qualified_name;
    }

    /// Set language
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    /// Set column
    pub fn set_column(&mut self, column: u32) {
        self.column = Some(column);
    }

    /// Set visibility
    pub fn set_visibility(&mut self, visibility: SymbolVisibility) {
        self.visibility = visibility;
    }

    /// Set exported flag
    pub fn set_exported(&mut self, exported: bool) {
        self.exported = exported;
    }

    /// Get symbol display name
    pub fn display_name(&self) -> String {
        match &self.symbol_type {
            SymbolType::Function | SymbolType::Method => {
                format!("{}() - {}", self.name, self.symbol_type)
            }
            SymbolType::Class | SymbolType::Interface | SymbolType::Enum => {
                format!("{} - {}", self.name, self.symbol_type)
            }
            SymbolType::Variable | SymbolType::Property | SymbolType::Constant => {
                format!("{} - {}", self.name, self.symbol_type)
            }
            SymbolType::Other(custom_type) => {
                format!("{} - {}", self.name, custom_type)
            }
            _ => {
                format!("{} - {}", self.name, self.symbol_type)
            }
        }
    }

    /// Get symbol signature
    pub fn signature(&self) -> String {
        match &self.symbol_type {
            SymbolType::Function | SymbolType::Method => {
                let params = self.get_metadata("parameters")
                    .cloned()
                    .unwrap_or_else(|| "...".to_string());
                format!("{}({})", self.qualified_name, params)
            }
            SymbolType::Class | SymbolType::Interface | SymbolType::Enum => {
                format!("{} {}", self.symbol_type, self.qualified_name)
            }
            _ => self.qualified_name.clone(),
        }
    }

    /// Check if this symbol is a root node candidate
    pub fn is_root_candidate(&self) -> bool {
        match &self.symbol_type {
            SymbolType::Function | SymbolType::Method => {
                // Public methods/functions are root candidates
                self.visibility == SymbolVisibility::Public || self.exported
            }
            SymbolType::Class | SymbolType::Interface => {
                // Public classes are root candidates
                self.visibility == SymbolVisibility::Public || self.exported
            }
            _ => false,
        }
    }

    /// Get file location string
    pub fn location(&self) -> String {
        if let Some(column) = self.column {
            format!("{}:{}:{}", self.file_path, self.line_number, column)
        } else {
            format!("{}:{}", self.file_path, self.line_number)
        }
    }
}

impl Default for SymbolVisibility {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for SymbolType {
    fn default() -> Self {
        Self::Other("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_symbol_creation() {
        let symbol = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );

        assert_eq!(symbol.id, "sym_1");
        assert_eq!(symbol.name, "testMethod");
        assert_eq!(symbol.symbol_type, SymbolType::Method);
        assert_eq!(symbol.file_path, "src/main/java/Test.java");
        assert_eq!(symbol.line_number, 10);
        assert_eq!(symbol.repository_id, "repo_1");
        assert!(symbol.validate().is_ok());
    }

    #[test]
    fn test_code_symbol_with_details() {
        let symbol = CodeSymbol::with_details(
            "sym_2".to_string(),
            "Test".to_string(),
            "com.example.Test".to_string(),
            SymbolType::Class,
            "java".to_string(),
            "src/main/java/Test.java".to_string(),
            1,
            "repo_1".to_string(),
        );

        assert_eq!(symbol.qualified_name, "com.example.Test");
        assert_eq!(symbol.language, "java");
        assert_eq!(symbol.symbol_type, SymbolType::Class);
    }

    #[test]
    fn test_code_symbol_validation() {
        let mut symbol = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );

        // Valid symbol should pass validation
        assert!(symbol.validate().is_ok());

        // Invalid symbol (empty name) should fail validation
        symbol.name = String::new();
        assert!(symbol.validate().is_err());

        // Invalid symbol (line number 0) should fail validation
        symbol.name = "testMethod".to_string();
        symbol.line_number = 0;
        assert!(symbol.validate().is_err());
    }

    #[test]
    fn test_symbol_metadata() {
        let mut symbol = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );

        symbol.add_metadata("complexity".to_string(), "5".to_string());
        symbol.add_metadata("test_coverage".to_string(), "80%".to_string());

        assert_eq!(symbol.get_metadata("complexity"), Some(&"5".to_string()));
        assert_eq!(symbol.get_metadata("test_coverage"), Some(&"80%".to_string()));
        assert_eq!(symbol.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_symbol_metadata_builder() {
        let metadata = SymbolMetadataBuilder::new()
            .complexity(10)
            .lines_of_code(50)
            .cyclomatic_complexity(8)
            .parameter_count(3)
            .return_type("String".to_string())
            .add("custom_field".to_string(), "custom_value".to_string())
            .build();

        assert_eq!(metadata.get("complexity"), Some(&"10".to_string()));
        assert_eq!(metadata.get("lines_of_code"), Some(&"50".to_string()));
        assert_eq!(metadata.get("cyclomatic_complexity"), Some(&"8".to_string()));
        assert_eq!(metadata.get("parameter_count"), Some(&"3".to_string()));
        assert_eq!(metadata.get("return_type"), Some(&"String".to_string()));
        assert_eq!(metadata.get("custom_field"), Some(&"custom_value".to_string()));
    }

    #[test]
    fn test_symbol_display_name() {
        let method = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );

        assert_eq!(method.display_name(), "testMethod() - Method");

        let class = CodeSymbol::new(
            "sym_2".to_string(),
            "Test".to_string(),
            SymbolType::Class,
            "src/main/java/Test.java".to_string(),
            1,
            "repo_1".to_string(),
        );

        assert_eq!(class.display_name(), "Test - Class");
    }

    #[test]
    fn test_symbol_signature() {
        let mut method = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );
        method.set_qualified_name("com.example.Test.testMethod".to_string());
        method.add_metadata("parameters".to_string(), "String arg1, int arg2".to_string());

        assert_eq!(method.signature(), "com.example.Test.testMethod(String arg1, int arg2)");

        let mut class = CodeSymbol::new(
            "sym_2".to_string(),
            "Test".to_string(),
            SymbolType::Class,
            "src/main/java/Test.java".to_string(),
            1,
            "repo_1".to_string(),
        );
        class.set_qualified_name("com.example.Test".to_string());

        assert_eq!(class.signature(), "Class com.example.Test");
    }

    #[test]
    fn test_symbol_location() {
        let mut symbol = CodeSymbol::new(
            "sym_1".to_string(),
            "testMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );

        assert_eq!(symbol.location(), "src/main/java/Test.java:10");

        symbol.set_column(15);
        assert_eq!(symbol.location(), "src/main/java/Test.java:10:15");
    }

    #[test]
    fn test_symbol_root_candidate() {
        let mut public_method = CodeSymbol::new(
            "sym_1".to_string(),
            "publicMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            10,
            "repo_1".to_string(),
        );
        public_method.set_visibility(SymbolVisibility::Public);

        assert!(public_method.is_root_candidate());

        let mut private_method = CodeSymbol::new(
            "sym_2".to_string(),
            "privateMethod".to_string(),
            SymbolType::Method,
            "src/main/java/Test.java".to_string(),
            15,
            "repo_1".to_string(),
        );
        private_method.set_visibility(SymbolVisibility::Private);

        assert!(!private_method.is_root_candidate());

        let mut exported_function = CodeSymbol::new(
            "sym_3".to_string(),
            "exportedFunction".to_string(),
            SymbolType::Function,
            "src/main.js".to_string(),
            20,
            "repo_1".to_string(),
        );
        exported_function.set_exported(true);

        assert!(exported_function.is_root_candidate());
    }

    #[test]
    fn test_symbol_type_equality() {
        assert_eq!(SymbolType::Function, SymbolType::Function);
        assert_eq!(SymbolType::Class, SymbolType::Class);
        assert_eq!(SymbolType::Other("custom".to_string()), SymbolType::Other("custom".to_string()));
        assert_ne!(SymbolType::Function, SymbolType::Method);
    }

    #[test]
    fn test_symbol_visibility_equality() {
        assert_eq!(SymbolVisibility::Public, SymbolVisibility::Public);
        assert_eq!(SymbolVisibility::Private, SymbolVisibility::Private);
        assert_ne!(SymbolVisibility::Public, SymbolVisibility::Private);
    }
}
