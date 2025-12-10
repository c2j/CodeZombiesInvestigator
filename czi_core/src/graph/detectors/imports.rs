//! Import detection

use crate::{Result, parser::{TreeSitterManager, SupportedLanguage}};
use crate::parser::LanguageQueries;
use tree_sitter::{Language, Query, QueryCursor};
use std::sync::Arc;

/// Detector for import statements and module dependencies
pub struct ImportDetector {
    manager: TreeSitterManager,
    queries: Vec<Arc<Query>>,
}

impl ImportDetector {
    /// Create a new import detector
    pub fn new() -> Result<Self> {
        let manager = TreeSitterManager::new()?;
        let mut queries = Vec::new();

        // Load queries for all supported languages
        for language in TreeSitterManager::supported_languages() {
            let language_queries = match language {
                SupportedLanguage::Java => LanguageQueries::java()?,
                SupportedLanguage::JavaScript => LanguageQueries::javascript()?,
                SupportedLanguage::Python => LanguageQueries::python()?,
                SupportedLanguage::Shell => LanguageQueries::shell()?,
            };

            // Clone the Arc<Query>
            let query = language_queries.import_query.clone();
            queries.push(query);
        }

        Ok(Self { manager, queries })
    }

    /// Detect imports in source code
    pub fn detect_imports(&mut self, source_code: &str, language: SupportedLanguage) -> Result<Vec<ImportStatement>> {
        let tree = self.manager.parse(source_code, language)?;
        let query_index = self.get_query_index(language)?;
        let query = &self.queries[query_index];

        let mut cursor = QueryCursor::new();
        let mut imports = Vec::new();

        let matches = cursor.matches(query.as_ref(), tree.root_node(), source_code.as_bytes());
        for m in matches {
            if let Some(capture_source) = m.captures.get(0) {
                let node = capture_source.node;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let module_name = &source_code[start_byte..end_byte];

                imports.push(ImportStatement {
                    module: module_name.to_string(),
                    import_type: self.determine_import_type(language, &m),
                    line_number: Self::get_line_number(source_code, start_byte),
                    alias: None,
                });
            }
        }

        Ok(imports)
    }

    /// Determine the type of import statement
    fn determine_import_type(&self, language: SupportedLanguage, match_info: &tree_sitter::QueryMatch) -> ImportType {
        // This is a simplified determination - in practice, you'd use the specific node types
        match language {
            SupportedLanguage::Java => ImportType::ClassImport,
            SupportedLanguage::JavaScript => ImportType::DefaultImport,
            SupportedLanguage::Python => ImportType::Import,
            SupportedLanguage::Shell => ImportType::Source,
        }
    }

    /// Get the language object for a supported language
    fn get_language_for_supported(language: SupportedLanguage) -> Language {
        match language {
            SupportedLanguage::Java => unsafe { tree_sitter_java::language() },
            SupportedLanguage::JavaScript => unsafe { tree_sitter_javascript::language() },
            SupportedLanguage::Python => unsafe { tree_sitter_python::language() },
            SupportedLanguage::Shell => unsafe { tree_sitter_bash::language() },
        }
    }

    /// Get query index for a language
    fn get_query_index(&self, language: SupportedLanguage) -> Result<usize> {
        let supported = TreeSitterManager::supported_languages();
        supported.iter()
            .position(|&l| l == language)
            .ok_or_else(|| crate::CziError::analysis("Unsupported language"))
    }

    /// Extract line number from byte offset
    fn get_line_number(source: &str, byte_offset: usize) -> u32 {
        source[..byte_offset]
            .matches('\n')
            .count() as u32 + 1
    }
}

/// Types of import statements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportType {
    /// Default import (import module)
    Import,
    /// Named import (import { name } from 'module')
    Named,
    /// Default import with alias (import * as alias from 'module')
    DefaultImport,
    /// Class import (for Java)
    ClassImport,
    /// Source command (for shell)
    Source,
}

/// Represents a detected import statement
#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub module: String,
    pub import_type: ImportType,
    pub line_number: u32,
    pub alias: Option<String>,
}

impl Default for ImportDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create ImportDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_detector_creation() {
        let detector = ImportDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_import_detection() {
        let detector = ImportDetector::new().unwrap();
        let js_code = r#"
import React from 'react';
import { useState, useEffect } from 'react';
const helper = require('./helper');
"#;

        let imports = detector.detect_imports(js_code, SupportedLanguage::JavaScript).unwrap();
        assert!(!imports.is_empty());
        assert_eq!(imports.len(), 3);
    }
}