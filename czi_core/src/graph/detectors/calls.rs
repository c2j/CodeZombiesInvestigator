//! Function call detection

use crate::{Result, parser::{TreeSitterManager, SupportedLanguage}};
use crate::parser::LanguageQueries;
use tree_sitter::{Language, Query, QueryCursor};
use std::sync::Arc;

/// Detector for function calls and method invocations
pub struct CallDetector {
    manager: TreeSitterManager,
    queries: Vec<Arc<Query>>,
}

impl CallDetector {
    /// Create a new call detector
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

            // Keep the Arc<Query> instead of cloning the Query
            queries.push(language_queries.call_query.clone());
        }

        Ok(Self { manager, queries })
    }

    /// Detect function calls in source code
    pub fn detect_calls(&mut self, source_code: &str, language: SupportedLanguage) -> Result<Vec<FunctionCall>> {
        let tree = self.manager.parse(source_code, language)?;
        let query_index = self.get_query_index(language)?;
        let query = &self.queries[query_index];

        let mut cursor = QueryCursor::new();
        let mut calls = Vec::new();

        let matches = cursor.matches(query.as_ref(), tree.root_node(), source_code.as_bytes());
        for m in matches {
            if let Some(capture_name) = m.captures.get(0) {
                let node = capture_name.node;
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let name = &source_code[start_byte..end_byte];

                calls.push(FunctionCall {
                    name: name.to_string(),
                    caller_span: (start_byte, end_byte),
                    line_number: Self::get_line_number(source_code, start_byte),
                    context: self.extract_context(source_code, start_byte, end_byte),
                });
            }
        }

        Ok(calls)
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

    /// Extract surrounding context for a function call
    fn extract_context(&self, source: &str, start_byte: usize, end_byte: usize) -> String {
        let start = start_byte.saturating_sub(20);
        let end = (end_byte + 20).min(source.len());

        // Find word boundaries - iterate backwards/forwards to find whitespace or punctuation
        let context_start = source[..start]
            .bytes()
            .rposition(|b| b == b' ' || b == b'\t' || b == b'\n' || b == b';' || b == b'{' || b == b')')
            .map(|i| i + 1)
            .unwrap_or(0);

        let context_end = source[end..]
            .bytes()
            .position(|b| b == b' ' || b == b'\t' || b == b'\n' || b == b';' || b == b'{' || b == b'}')
            .map(|i| end + i)
            .unwrap_or(source.len());

        source[context_start..context_end].trim().to_string()
    }
}

/// Represents a detected function call
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub caller_span: (usize, usize),
    pub line_number: u32,
    pub context: String,
}

impl Default for CallDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create CallDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_detector_creation() {
        let detector = CallDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_function_call_detection() {
        let detector = CallDetector::new().unwrap();
        let js_code = r#"
function test() {
    console.log("Hello");
    helper();
    return helper();
}
"#;

        let calls = detector.detect_calls(js_code, SupportedLanguage::JavaScript).unwrap();
        assert!(!calls.is_empty());
    }
}