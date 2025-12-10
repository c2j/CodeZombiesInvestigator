//! Multi-language parsing support using Tree-sitter

use std::collections::HashMap;
use tree_sitter::{Language, Parser};

use crate::error::{CziError, Result};

mod language;
mod symbol;
mod extractors;
mod pipeline;

pub use language::{SupportedLanguage, LanguageQueries};
pub use symbol::{CodeSymbol, SymbolType, SymbolVisibility, SymbolMetadataBuilder};
pub use pipeline::{ParserPipeline, ParserPipelineConfig, FileParseResult, ParseStats, PipelineStats};

/// Tree-sitter manager for multi-language parsing
pub struct TreeSitterManager {
    parsers: HashMap<SupportedLanguage, Parser>,
    languages: HashMap<SupportedLanguage, Language>,
    queries: HashMap<SupportedLanguage, LanguageQueries>,
}

impl TreeSitterManager {
    /// Create a new Tree-sitter manager
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            parsers: HashMap::new(),
            languages: HashMap::new(),
            queries: HashMap::new(),
        };

        manager.initialize_languages()?;
        Ok(manager)
    }

    /// Initialize all supported languages
    fn initialize_languages(&mut self) -> Result<()> {
        // Java
        let java_language = tree_sitter_java::language();
        self.languages.insert(SupportedLanguage::Java, java_language);
        self.queries.insert(SupportedLanguage::Java, LanguageQueries::java()?);

        // JavaScript
        let js_language = tree_sitter_javascript::language();
        self.languages.insert(SupportedLanguage::JavaScript, js_language);
        self.queries.insert(SupportedLanguage::JavaScript, LanguageQueries::javascript()?);

        // Python
        let python_language = tree_sitter_python::language();
        self.languages.insert(SupportedLanguage::Python, python_language);
        self.queries.insert(SupportedLanguage::Python, LanguageQueries::python()?);

        // Shell
        let bash_language = tree_sitter_bash::language();
        self.languages.insert(SupportedLanguage::Shell, bash_language);
        self.queries.insert(SupportedLanguage::Shell, LanguageQueries::shell()?);

        Ok(())
    }

    /// Get parser for a specific language
    pub fn get_parser(&mut self, language: SupportedLanguage) -> Result<&mut Parser> {
        if !self.parsers.contains_key(&language) {
            let lang = self.languages.get(&language)
                .ok_or_else(|| CziError::parse(format!("Language {:?} not supported", language)))?;

            let mut parser = Parser::new();
            parser.set_language(*lang)
                .map_err(|e| CziError::parse(format!("Failed to set language: {}", e)))?;

            self.parsers.insert(language, parser);
        }

        Ok(self.parsers.get_mut(&language).unwrap())
    }

    /// Get language queries
    pub fn get_queries(&self, language: SupportedLanguage) -> Result<&LanguageQueries> {
        self.queries.get(&language)
            .ok_or_else(|| CziError::parse(format!("Queries for language {:?} not found", language)))
    }

    /// Parse source code and return the syntax tree
    pub fn parse(&mut self, content: &str, language: SupportedLanguage) -> Result<tree_sitter::Tree> {
        let parser = self.get_parser(language)?;
        let tree = parser.parse(content, None)
            .ok_or_else(|| CziError::parse("Failed to parse source code".to_string()))?;
        Ok(tree)
    }

    /// Parse a file and extract symbols
    pub fn parse_file(&mut self, content: &str, language: SupportedLanguage, file_path: &str) -> Result<Vec<CodeSymbol>> {
        let queries = self.get_queries(language)?;
        let parser = self.get_parser(language)?;

        let tree = parser.parse(content, None)
            .ok_or_else(|| CziError::parse(format!("Failed to parse file: {}", file_path)))?;

        let root_node = tree.root_node();
        let mut symbols = Vec::new();

        // Extract symbols based on language - TODO: implement proper extraction
        match language {
            SupportedLanguage::Java => {
                // TODO: Implement Java symbol extraction
                symbols = Vec::new();
            },
            SupportedLanguage::JavaScript => {
                // TODO: Implement JavaScript symbol extraction
                symbols = Vec::new();
            },
            SupportedLanguage::Python => {
                // TODO: Implement Python symbol extraction
                symbols = Vec::new();
            },
            SupportedLanguage::Shell => {
                // TODO: Implement Shell symbol extraction
                symbols = Vec::new();
            },
        }

        Ok(symbols)
    }

    /// Detect language from file extension
    pub fn detect_language(file_path: &str) -> Result<SupportedLanguage> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| CziError::parse(format!("Cannot determine file extension: {}", file_path)))?;

        match extension.to_lowercase().as_str() {
            "java" => Ok(SupportedLanguage::Java),
            "js" | "jsx" | "ts" | "tsx" | "mjs" => Ok(SupportedLanguage::JavaScript),
            "py" | "pyw" | "pyi" => Ok(SupportedLanguage::Python),
            "sh" | "bash" | "zsh" | "fish" => Ok(SupportedLanguage::Shell),
            _ => Err(CziError::parse(format!("Unsupported file extension: {}", extension))),
        }
    }

    /// Get supported languages
    pub fn supported_languages() -> Vec<SupportedLanguage> {
        vec![
            SupportedLanguage::Java,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Shell,
        ]
    }

    /// Get supported file extensions
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        vec![
            "java", "js", "jsx", "ts", "tsx", "mjs",  // JavaScript/TypeScript
            "py", "pyw", "pyi",  // Python
            "sh", "bash", "zsh", "fish",  // Shell
        ]
    }
}

impl Default for TreeSitterManager {
    fn default() -> Self {
        Self::new().expect("Failed to create TreeSitterManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sitter_manager_creation() -> Result<()> {
        let manager = TreeSitterManager::new()?;
        assert_eq!(manager.languages.len(), 4);
        Ok(())
    }

    #[test]
    fn test_language_detection() -> Result<()> {
        assert_eq!(TreeSitterManager::detect_language("test.java")?, SupportedLanguage::Java);
        assert_eq!(TreeSitterManager::detect_language("test.js")?, SupportedLanguage::JavaScript);
        assert_eq!(TreeSitterManager::detect_language("test.py")?, SupportedLanguage::Python);
        assert_eq!(TreeSitterManager::detect_language("test.sh")?, SupportedLanguage::Shell);
        Ok(())
    }

    #[test]
    fn test_unsupported_language() {
        let result = TreeSitterManager::detect_language("test.unknown");
        assert!(result.is_err());
    }
}