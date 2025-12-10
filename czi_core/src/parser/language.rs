//! Language support definitions and queries for Tree-sitter parsing

use crate::{CziError, Result};
use std::sync::Arc;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Java,
    JavaScript,
    Python,
    Shell,
}

impl SupportedLanguage {
    /// Get language name as string
    pub fn name(&self) -> &'static str {
        match self {
            SupportedLanguage::Java => "java",
            SupportedLanguage::JavaScript => "javascript",
            SupportedLanguage::Python => "python",
            SupportedLanguage::Shell => "shell",
        }
    }

    /// Get file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            SupportedLanguage::Java => &["java"],
            SupportedLanguage::JavaScript => &["js", "mjs", "jsx"],
            SupportedLanguage::Python => &["py", "pyw"],
            SupportedLanguage::Shell => &["sh", "bash", "zsh", "fish"],
        }
    }

    /// Detect language from file path
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            for lang in [Self::Java, Self::JavaScript, Self::Python, Self::Shell] {
                if lang.extensions().contains(&extension) {
                    return Some(lang);
                }
            }
        }
        None
    }
}

/// Language-specific queries for symbol extraction
#[derive(Debug)]
pub struct LanguageQueries {
    /// Function/method definitions
    pub function_query: Arc<tree_sitter::Query>,
    /// Class/struct definitions
    pub class_query: Arc<tree_sitter::Query>,
    /// Import/include statements
    pub import_query: Arc<tree_sitter::Query>,
    /// Function calls
    pub call_query: Arc<tree_sitter::Query>,
    /// Variable declarations
    pub variable_query: Arc<tree_sitter::Query>,
}

impl LanguageQueries {
    /// Create Java language queries
    pub fn java() -> Result<Self> {
        let language = tree_sitter_java::language();

        let function_query = tree_sitter::Query::new(language, r#"
            (method_declaration
                name: (identifier) @function.name
                parameters: (formal_parameters) @function.params
            ) @function
        "#).map_err(|e| CziError::parse(format!("Failed to create Java function query: {}", e)))?;

        let class_query = tree_sitter::Query::new(language, r#"
            (class_declaration
                name: (identifier) @class.name
            ) @class
        "#).map_err(|e| CziError::parse(format!("Failed to create Java class query: {}", e)))?;

        let import_query = tree_sitter::Query::new(language, r#"
            (import_declaration
                (scoped_identifier) @import.name
            ) @import
        "#).map_err(|e| CziError::parse(format!("Failed to create Java import query: {}", e)))?;

        let call_query = tree_sitter::Query::new(language, r#"
            (method_invocation
                name: (identifier) @call.name
            ) @call
        "#).map_err(|e| CziError::parse(format!("Failed to create Java call query: {}", e)))?;

        let variable_query = tree_sitter::Query::new(language, r#"
            (field_declaration
                (variable_declarator
                    name: (identifier) @variable.name
                )
            ) @variable
        "#).map_err(|e| CziError::parse(format!("Failed to create Java variable query: {}", e)))?;

        Ok(LanguageQueries {
            function_query: Arc::new(function_query),
            class_query: Arc::new(class_query),
            import_query: Arc::new(import_query),
            call_query: Arc::new(call_query),
            variable_query: Arc::new(variable_query),
        })
    }

    /// Create JavaScript language queries
    pub fn javascript() -> Result<Self> {
        let language = tree_sitter_javascript::language();

        let function_query = tree_sitter::Query::new(language, r#"
            (function_declaration
                name: (identifier) @function.name
                parameters: (formal_parameters) @function.params
            ) @function
            (method_definition
                name: (property_identifier) @function.name
                parameters: (formal_parameters) @function.params
            ) @function
        "#).map_err(|e| CziError::parse(format!("Failed to create JavaScript function query: {}", e)))?;

        let class_query = tree_sitter::Query::new(language, r#"
            (class_declaration
                name: (identifier) @class.name
            ) @class
        "#).map_err(|e| CziError::parse(format!("Failed to create JavaScript class query: {}", e)))?;

        let import_query = tree_sitter::Query::new(language, r#"
            (import_statement
                source: (string) @import.source
            ) @import
        "#).map_err(|e| CziError::parse(format!("Failed to create JavaScript import query: {}", e)))?;

        let call_query = tree_sitter::Query::new(language, r#"
            (call_expression
                function: (identifier) @call.name
            ) @call
            (call_expression
                function: (member_expression
                    property: (property_identifier) @call.name
                )
            ) @call
        "#).map_err(|e| CziError::parse(format!("Failed to create JavaScript call query: {}", e)))?;

        let variable_query = tree_sitter::Query::new(language, r#"
            (variable_declaration
                (variable_declarator
                    name: (identifier) @variable.name
                )
            ) @variable
        "#).map_err(|e| CziError::parse(format!("Failed to create JavaScript variable query: {}", e)))?;

        Ok(LanguageQueries {
            function_query: Arc::new(function_query),
            class_query: Arc::new(class_query),
            import_query: Arc::new(import_query),
            call_query: Arc::new(call_query),
            variable_query: Arc::new(variable_query),
        })
    }

    /// Create Python language queries
    pub fn python() -> Result<Self> {
        let language = tree_sitter_python::language();

        let function_query = tree_sitter::Query::new(language, r#"
            (function_definition
                name: (identifier) @function.name
                parameters: (parameters) @function.params
            ) @function
        "#).map_err(|e| CziError::parse(format!("Failed to create Python function query: {}", e)))?;

        let class_query = tree_sitter::Query::new(language, r#"
            (class_definition
                name: (identifier) @class.name
            ) @class
        "#).map_err(|e| CziError::parse(format!("Failed to create Python class query: {}", e)))?;

        let import_query = tree_sitter::Query::new(language, r#"
            (import_statement
                name: (dotted_name) @import.name
            ) @import
            (import_from_statement
                module_name: (dotted_name) @import.module
            ) @import
        "#).map_err(|e| CziError::parse(format!("Failed to create Python import query: {}", e)))?;

        let call_query = tree_sitter::Query::new(language, r#"
            (call
                function: (identifier) @call.name
            ) @call
        "#).map_err(|e| CziError::parse(format!("Failed to create Python call query: {}", e)))?;

        let variable_query = tree_sitter::Query::new(language, r#"
            (assignment
                left: (identifier) @variable.name
            ) @variable
        "#).map_err(|e| CziError::parse(format!("Failed to create Python variable query: {}", e)))?;

        Ok(LanguageQueries {
            function_query: Arc::new(function_query),
            class_query: Arc::new(class_query),
            import_query: Arc::new(import_query),
            call_query: Arc::new(call_query),
            variable_query: Arc::new(variable_query),
        })
    }

    /// Create Shell language queries
    pub fn shell() -> Result<Self> {
        let language = tree_sitter_bash::language();

        let function_query = tree_sitter::Query::new(language, r#"
            (function_definition
                name: (word) @function.name
            ) @function
        "#).map_err(|e| CziError::parse(format!("Failed to create Shell function query: {}", e)))?;

        let class_query = tree_sitter::Query::new(language, r#"
            # Shell doesn't have classes, so we use a dummy query
            (program) @class
        "#).map_err(|e| CziError::parse(format!("Failed to create Shell class query: {}", e)))?;

        let import_query = tree_sitter::Query::new(language, r#"
            (command
                name: (command_name) @import.name
                (#match? @import.name "^(source|\.)")
            ) @import
        "#).map_err(|e| CziError::parse(format!("Failed to create Shell import query: {}", e)))?;

        let call_query = tree_sitter::Query::new(language, r#"
            (command
                name: (command_name) @call.name
            ) @call
        "#).map_err(|e| CziError::parse(format!("Failed to create Shell call query: {}", e)))?;

        let variable_query = tree_sitter::Query::new(language, r#"
            (variable_assignment
                name: (variable_name) @variable.name
            ) @variable
        "#).map_err(|e| CziError::parse(format!("Failed to create Shell variable query: {}", e)))?;

        Ok(LanguageQueries {
            function_query: Arc::new(function_query),
            class_query: Arc::new(class_query),
            import_query: Arc::new(import_query),
            call_query: Arc::new(call_query),
            variable_query: Arc::new(variable_query),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        use std::path::Path;

        assert_eq!(SupportedLanguage::from_path(Path::new("test.java")), Some(SupportedLanguage::Java));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.js")), Some(SupportedLanguage::JavaScript));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.py")), Some(SupportedLanguage::Python));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.sh")), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_language_queries_creation() -> Result<()> {
        // Test that we can create queries for all languages without errors
        let _java_queries = LanguageQueries::java()?;
        let _js_queries = LanguageQueries::javascript()?;
        let _python_queries = LanguageQueries::python()?;
        let _shell_queries = LanguageQueries::shell()?;
        Ok(())
    }
}