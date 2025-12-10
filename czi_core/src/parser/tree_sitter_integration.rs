//! Tree-sitter integration for multi-language parsing
//!
//! Provides a unified interface for parsing multiple programming languages
//! using tree-sitter grammars.

use crate::{CziError, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn, instrument};
use tree_sitter::{Language, Parser};

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
    pub fn from_path(path: &Path) -> Option<Self> {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            for lang in [Self::Java, Self::JavaScript, Self::Python, Self::Shell] {
                if lang.extensions().contains(&extension) {
                    return Some(lang);
                }
            }
        }
        None
    }

    /// Detect language from file name (for files without extensions)
    pub fn from_file_name(file_name: &str) -> Option<Self> {
        // Check for common patterns
        if file_name.ends_with(".java") {
            Some(Self::Java)
        } else if file_name.ends_with(".js") || file_name.ends_with(".mjs") {
            Some(Self::JavaScript)
        } else if file_name.ends_with(".py") {
            Some(Self::Python)
        } else if file_name.ends_with(".sh") || file_name.ends_with(".bash") {
            Some(Self::Shell)
        } else {
            // Check for shebang lines
            if file_name == "Dockerfile" || file_name == "Makefile" {
                Some(Self::Shell)
            } else {
                None
            }
        }
    }
}

/// Tree-sitter parser manager
pub struct TreeSitterManager {
    languages: Arc<RwLock<HashMap<SupportedLanguage, Language>>>,
    parsers: Arc<RwLock<HashMap<SupportedLanguage, Parser>>>,
}

impl TreeSitterManager {
    /// Create a new tree-sitter manager
    pub fn new() -> Result<Self> {
        let manager = Self {
            languages: Arc::new(RwLock::new(HashMap::new())),
            parsers: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize all supported languages
        manager.initialize_languages()?;

        Ok(manager)
    }

    /// Initialize language grammars
    fn initialize_languages(&self) -> Result<()> {
        let mut languages = self.languages.write().map_err(|_| {
            CziError::internal("Failed to acquire write lock for languages")
        })?;

        info!("Initializing tree-sitter language grammars");

        // Java
        languages.insert(SupportedLanguage::Java, unsafe {
            tree_sitter_java::language()
        });
        debug!("Initialized Java language grammar");

        // JavaScript
        languages.insert(SupportedLanguage::JavaScript, unsafe {
            tree_sitter_javascript::language()
        });
        debug!("Initialized JavaScript language grammar");

        // Python
        languages.insert(SupportedLanguage::Python, unsafe {
            tree_sitter_python::language()
        });
        debug!("Initialized Python language grammar");

        // Shell
        languages.insert(SupportedLanguage::Shell, unsafe {
            tree_sitter_bash::language()
        });
        debug!("Initialized Shell language grammar");

        info!("All language grammars initialized successfully");
        Ok(())
    }

    /// Get or create a parser for the specified language
    #[instrument(skip(self))]
    pub fn get_parser(&self, language: SupportedLanguage) -> Result<Parser> {
        let _parsers = self.parsers.write().map_err(|_| {
            CziError::internal("Failed to acquire write lock for parsers")
        })?;

        // Note: Parser doesn't implement Clone, so we create a new one each time
        // This is fine since Parser creation is cheap

        // Create new parser
        let mut parser = Parser::new();
        let languages = self.languages.read().map_err(|_| {
            CziError::internal("Failed to acquire read lock for languages")
        })?;

        let language_obj = languages.get(&language)
            .ok_or_else(|| CziError::tree_sitter(format!("Language {:?} not initialized", language)))?;

        parser.set_language(*language_obj)
            .map_err(|e| CziError::tree_sitter(format!("Failed to set language for parser: {}", e)))?;

        // Note: Not storing parser since it doesn't implement Clone
        // parsers.insert(language, parser);

        debug!("Created new parser for language: {:?}", language);
        Ok(parser)
    }

    /// Parse source code with the specified language
    #[instrument(skip(self, source_code))]
    pub fn parse(&self, source_code: &str, language: SupportedLanguage) -> Result<tree_sitter::Tree> {
        let mut parser = self.get_parser(language)?;

        parser.parse(source_code, None)
            .ok_or_else(|| CziError::tree_sitter("Failed to parse source code"))
    }

    /// Parse source code by auto-detecting the language from file path
    #[instrument(skip(self, source_code))]
    pub fn parse_file(&self, source_code: &str, file_path: &Path) -> Result<tree_sitter::Tree> {
        let language = SupportedLanguage::from_path(file_path)
            .or_else(|| SupportedLanguage::from_file_name(
                file_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
            ))
            .ok_or_else(|| {
                CziError::parse(
                    file_path.to_string_lossy(),
                    0,
                    "Unsupported or unknown file type"
                )
            })?;

        debug!("Detected language {:?} for file: {:?}", language, file_path);
        self.parse(source_code, language)
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<SupportedLanguage> {
        vec![
            SupportedLanguage::Java,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Shell,
        ]
    }

    /// Check if a file type is supported
    pub fn is_file_supported(&self, file_path: &Path) -> bool {
        SupportedLanguage::from_path(file_path).is_some() ||
        SupportedLanguage::from_file_name(
            file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
        ).is_some()
    }

    /// Get supported file extensions
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        let mut extensions = Vec::new();
        for lang in self.supported_languages() {
            extensions.extend(lang.extensions());
        }
        extensions.sort();
        extensions.dedup();
        extensions
    }
}

impl Default for TreeSitterManager {
    fn default() -> Self {
        Self::new().expect("Failed to create TreeSitterManager")
    }
}

/// Language-specific query definitions
pub struct LanguageQueries;

impl LanguageQueries {
    /// Get function/method definition queries for a language
    pub fn function_definitions(language: SupportedLanguage) -> &'static str {
        match language {
            SupportedLanguage::Java => r#"
                [
                    (method_declaration
                        name: (identifier) @method.name
                        parameters: (formal_parameters) @method.params
                        body: (block) @method.body)
                    (constructor_declaration
                        name: (identifier) @constructor.name
                        parameters: (formal_parameters) @constructor.params
                        body: (block) @constructor.body)
                ]
            "#,
            SupportedLanguage::JavaScript => r#"
                [
                    (function_declaration
                        name: (identifier) @function.name
                        parameters: (formal_parameters) @function.params
                        body: (statement_block) @function.body)
                    (function_expression
                        name: (identifier) @function.name
                        parameters: (formal_parameters) @function.params
                        body: (statement_block) @function.body)
                    (arrow_function
                        parameters: (formal_parameters) @arrow.params
                        body: (statement_block) @arrow.body)
                    (method_definition
                        name: (property_identifier) @method.name
                        parameters: (formal_parameters) @method.params
                        body: (statement_block) @method.body)
                ]
            "#,
            SupportedLanguage::Python => r#"
                [
                    (function_definition
                        name: (identifier) @function.name
                        parameters: (parameters) @function.params
                        body: (block) @function.body)
                    (class_definition
                        name: (identifier) @class.name
                        body: (block) @class.body)
                    (async_function_definition
                        name: (identifier) @async_function.name
                        parameters: (parameters) @async_function.params
                        body: (block) @async_function.body)
                ]
            "#,
            SupportedLanguage::Shell => r#"
                [
                    (function_definition
                        name: (word) @function.name
                        body: (compound_statement) @function.body)
                ]
            "#,
        }
    }

    /// Get class definition queries for a language
    pub fn class_definitions(language: SupportedLanguage) -> &'static str {
        match language {
            SupportedLanguage::Java => r#"
                (class_declaration
                    name: (identifier) @class.name
                    body: (class_body) @class.body)
            "#,
            SupportedLanguage::JavaScript => r#"
                [
                    (class_declaration
                        name: (identifier) @class.name
                        body: (class_body) @class.body)
                    (class_expression
                        name: (identifier) @class.name
                        body: (class_body) @class.body)
                ]
            "#,
            SupportedLanguage::Python => r#"
                (class_definition
                    name: (identifier) @class.name
                    body: (block) @class.body)
            "#,
            SupportedLanguage::Shell => {
                // Shell doesn't have classes
                ""
            }
        }
    }

    /// Get import/require queries for a language
    pub fn imports(language: SupportedLanguage) -> &'static str {
        match language {
            SupportedLanguage::Java => r#"
                [
                    (import_declaration
                        (scoped_identifier) @import.name)
                    (import_declaration
                        (import_list
                            (scoped_identifier) @import.name))
                ]
            "#,
            SupportedLanguage::JavaScript => r#"
                [
                    (import_statement
                        source: (string) @import.source)
                    (import_statement
                        import_clause: (namespace_import) @import.namespace)
                    (require
                        source: (string) @require.source)
                ]
            "#,
            SupportedLanguage::Python => r#"
                [
                    (import_statement
                        name: (dotted_name) @import.name)
                    (import_from_statement
                        module_name: (dotted_name) @import.module
                        name: (dotted_name) @import.name)
                    (future_import_statement
                        name: (dotted_name) @import.name)
                ]
            "#,
            SupportedLanguage::Shell => r#"
                [
                    (source_statement
                        file: (word) @source.file)
                ]
            "#,
        }
    }

    /// Get function call queries for a language
    pub fn function_calls(language: SupportedLanguage) -> &'static str {
        match language {
            SupportedLanguage::Java => r#"
                (method_invocation
                    name: (identifier) @call.name)
            "#,
            SupportedLanguage::JavaScript => r#"
                [
                    (call_expression
                        function: (identifier) @call.function)
                    (call_expression
                        function: (member_expression
                            property: (property_identifier) @call.method))
                    (new_expression
                        constructor: (identifier) @new.constructor)
                ]
            "#,
            SupportedLanguage::Python => r#"
                [
                    (call
                        function: (identifier) @call.function)
                    (call
                        function: (attribute
                            attribute: (identifier) @call.method))
                ]
            "#,
            SupportedLanguage::Shell => r#"
                (command
                    name: (word) @command.name)
            "#,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_language_detection() {
        assert_eq!(SupportedLanguage::from_path(Path::new("test.java")), Some(SupportedLanguage::Java));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.js")), Some(SupportedLanguage::JavaScript));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.py")), Some(SupportedLanguage::Python));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.sh")), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_path(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_file_name_detection() {
        assert_eq!(SupportedLanguage::from_file_name("Dockerfile"), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_file_name("Makefile"), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_file_name("unknown"), None);
    }

    #[test]
    fn test_tree_sitter_manager() {
        let manager = TreeSitterManager::new().unwrap();
        let supported_exts = manager.supported_extensions();

        assert!(supported_exts.contains(&"java"));
        assert!(supported_exts.contains(&"js"));
        assert!(supported_exts.contains(&"py"));
        assert!(supported_exts.contains(&"sh"));

        assert!(manager.is_file_supported(Path::new("test.java")));
        assert!(!manager.is_file_supported(Path::new("test.unknown")));
    }

    #[test]
    fn test_parse_simple_code() {
        let manager = TreeSitterManager::new().unwrap();

        let java_code = "public class Test { public void method() {} }";
        let tree = manager.parse(java_code, SupportedLanguage::Java).unwrap();
        assert!(!tree.root_node().has_error());

        let js_code = "function test() { return 42; }";
        let tree = manager.parse(js_code, SupportedLanguage::JavaScript).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_language_queries() {
        // Test that query strings are not empty for most languages
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Java).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::JavaScript).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Python).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Shell).is_empty());
    }
}