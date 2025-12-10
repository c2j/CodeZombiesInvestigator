//! Tree-sitter-based parser pipeline for multi-language code analysis
//!
//! This module provides the main parsing pipeline that processes source code files
//! and extracts code symbols and their dependencies using Tree-sitter parsers.

use crate::{
    Result, CziError,
    parser::{CodeSymbol, SymbolType, SymbolVisibility, SymbolMetadataBuilder, language::SupportedLanguage},
    graph::{DependencyEdge, EdgeType, builder::GraphBuilder},
    config::root_node::RootNodeDetector,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

/// Configuration for parser pipeline
#[derive(Debug, Clone)]
pub struct ParserPipelineConfig {
    /// Languages to parse (empty = auto-detect)
    pub languages: Vec<String>,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Maximum file size to parse (in bytes)
    pub max_file_size_bytes: u64,

    /// Whether to extract dependencies
    pub extract_dependencies: bool,

    /// Whether to detect root nodes
    pub detect_root_nodes: bool,

    /// Number of parsing threads (0 = auto-detect)
    pub parsing_threads: usize,

    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
}

impl Default for ParserPipelineConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                "java".to_string(),
                "javascript".to_string(),
                "python".to_string(),
                "shell".to_string(),
            ],
            include_patterns: vec![
                "**/*.java".to_string(),
                "**/*.js".to_string(),
                "**/*.mjs".to_string(),
                "**/*.py".to_string(),
                "**/*.sh".to_string(),
                "**/*.bash".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
                "**/.git/**".to_string(),
                "**/*.min.js".to_string(),
                "**/*.test.js".to_string(),
                "**/*_test.py".to_string(),
            ],
            max_file_size_bytes: 1024 * 1024, // 1MB
            extract_dependencies: true,
            detect_root_nodes: true,
            parsing_threads: 0, // Auto-detect
            follow_symlinks: false,
        }
    }
}

/// Result of parsing a file
#[derive(Debug, Clone)]
pub struct FileParseResult {
    /// File path
    pub file_path: PathBuf,

    /// Language detected
    pub language: String,

    /// Symbols extracted from the file
    pub symbols: Vec<CodeSymbol>,

    /// Dependencies between symbols
    pub dependencies: Vec<DependencyEdge>,

    /// Root nodes detected in the file
    pub root_nodes: Vec<crate::config::ActiveRootNode>,

    /// Whether parsing was successful
    pub success: bool,

    /// Error message if parsing failed
    pub error: Option<String>,

    /// Parsing statistics
    pub stats: ParseStats,
}

/// Parsing statistics for a file
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Number of lines parsed
    pub lines_count: usize,

    /// Number of symbols extracted
    pub symbols_count: usize,

    /// Number of dependencies found
    pub dependencies_count: usize,

    /// Time taken to parse (in milliseconds)
    pub parse_time_ms: u64,
}

/// Tree-sitter-based parser pipeline
pub struct ParserPipeline {
    /// Configuration
    config: ParserPipelineConfig,

    /// Tree-sitter manager
    tree_sitter: Arc<tokio::sync::RwLock<crate::parser::TreeSitterManager>>,

    /// Root node detector
    root_node_detector: RootNodeDetector,

    /// Statistics
    stats: Arc<RwLock<PipelineStats>>,
}

/// Overall pipeline statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    /// Total files processed
    pub files_processed: usize,

    /// Total symbols extracted
    pub symbols_extracted: usize,

    /// Total dependencies found
    pub dependencies_found: usize,

    /// Files with errors
    pub files_with_errors: usize,

    /// Total processing time (in milliseconds)
    pub total_time_ms: u64,
}

impl ParserPipeline {
    /// Create a new parser pipeline with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ParserPipelineConfig::default())
    }

    /// Create a parser pipeline with custom configuration
    pub fn with_config(config: ParserPipelineConfig) -> Result<Self> {
        let tree_sitter = Arc::new(tokio::sync::RwLock::new(crate::parser::TreeSitterManager::new()?));
        let root_node_detector = RootNodeDetector::new();

        Ok(Self {
            config,
            tree_sitter,
            root_node_detector,
            stats: Arc::new(RwLock::new(PipelineStats::default())),
        })
    }

    /// Parse files from a directory recursively
    #[instrument(skip(self, directory_path))]
    pub async fn parse_directory(&self, directory_path: &Path, repository_id: &str) -> Result<Vec<FileParseResult>> {
        info!("Starting to parse directory: {:?}", directory_path);

        let files = self.discover_files(directory_path)?;
        info!("Discovered {} files to parse", files.len());

        self.parse_files(files, repository_id).await
    }

    /// Parse specific files
    #[instrument(skip(self, files))]
    pub async fn parse_files(&self, files: Vec<PathBuf>, repository_id: &str) -> Result<Vec<FileParseResult>> {
        let start_time = std::time::Instant::now();

        // Filter files based on configuration
        let eligible_files = self.filter_eligible_files(&files)?;
        info!("{} files eligible for parsing", eligible_files.len());

        // Parse files in parallel
        let results = self.parse_files_parallel(eligible_files, repository_id).await?;

        // Update statistics
        let duration = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.files_processed = results.len();
        stats.symbols_extracted = results.iter().map(|r| r.symbols.len()).sum();
        stats.dependencies_found = results.iter().map(|r| r.dependencies.len()).sum();
        stats.files_with_errors = results.iter().filter(|r| !r.success).count();
        stats.total_time_ms = duration.as_millis() as u64;

        info!(
            "Parsing completed: {} files, {} symbols, {} dependencies in {}ms",
            stats.files_processed,
            stats.symbols_extracted,
            stats.dependencies_found,
            stats.total_time_ms
        );

        Ok(results)
    }

    /// Parse a single file
    #[instrument(skip(self))]
    pub async fn parse_file(&self, file_path: &Path, repository_id: &str) -> Result<FileParseResult> {
        let start_time = std::time::Instant::now();
        debug!("Parsing file: {:?}", file_path);

        // Read file content
        let content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| CziError::Io(e))?;

        // Check file size
        if content.len() > self.config.max_file_size_bytes as usize {
            return Ok(FileParseResult {
                file_path: file_path.to_path_buf(),
                language: "unknown".to_string(),
                symbols: Vec::new(),
                dependencies: Vec::new(),
                root_nodes: Vec::new(),
                success: false,
                error: Some(format!("File too large: {} bytes", content.len())),
                stats: ParseStats::default(),
            });
        }

        // Detect language
        let language_str = self.detect_language(file_path, &content)?;
        let language = match language_str.as_str() {
            "java" => SupportedLanguage::Java,
            "javascript" => SupportedLanguage::JavaScript,
            "python" => SupportedLanguage::Python,
            "shell" => SupportedLanguage::Shell,
            _ => SupportedLanguage::JavaScript, // Default fallback
        };

        // Parse with Tree-sitter
        let parse_result = match self.tree_sitter.write().await.parse_file(&content, language, file_path.to_str().unwrap_or("unknown")) {
            Ok(symbols) => {
                // Extract symbols
                let symbols = symbols;

                // Extract dependencies
                let dependencies = Vec::new(); // TODO: Fix dependency extraction after Tree-sitter API changes

                // Detect root nodes
                let root_nodes = if self.config.detect_root_nodes {
                    let language_str = match language {
                        SupportedLanguage::Java => "java",
                        SupportedLanguage::JavaScript => "javascript",
                        SupportedLanguage::Python => "python",
                        SupportedLanguage::Shell => "shell",
                    };
                    self.root_node_detector.detect_root_nodes(&content, &file_path.to_string_lossy(), language_str, repository_id)?
                } else {
                    Vec::new()
                };

                FileParseResult {
                    file_path: file_path.to_path_buf(),
                    language: language_str.to_string(),
                    symbols,
                    dependencies,
                    root_nodes,
                    success: true,
                    error: None,
                    stats: ParseStats {
                        lines_count: content.lines().count(),
                        symbols_count: 0, // Will be set later
                        dependencies_count: 0, // Will be set later
                        parse_time_ms: start_time.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => {
                let language_str = match language {
                    SupportedLanguage::Java => "java",
                    SupportedLanguage::JavaScript => "javascript",
                    SupportedLanguage::Python => "python",
                    SupportedLanguage::Shell => "shell",
                };
                FileParseResult {
                    file_path: file_path.to_path_buf(),
                    language: language_str.to_string(),
                    symbols: Vec::new(),
                    dependencies: Vec::new(),
                    root_nodes: Vec::new(),
                    success: false,
                    error: Some(format!("Parse error: {}", e)),
                    stats: ParseStats {
                        lines_count: content.lines().count(),
                        symbols_count: 0,
                        dependencies_count: 0,
                        parse_time_ms: start_time.elapsed().as_millis() as u64,
                    },
                }
            },
        };

        Ok(parse_result)
    }

    /// Build a dependency graph from parse results
    pub async fn build_graph(&self, parse_results: &[FileParseResult]) -> Result<GraphBuilder> {
        let mut graph_builder = GraphBuilder::new();

        // Add all symbols to the graph
        for result in parse_results {
            for symbol in &result.symbols {
                graph_builder.add_symbol(symbol.clone())?;
            }
        }

        // Add all dependencies to the graph
        for result in parse_results {
            for dependency in &result.dependencies {
                // Add dependency using symbol IDs
                if let Err(e) = graph_builder.add_dependency(
                    &dependency.source_symbol_id,
                    &dependency.target_symbol_id,
                    dependency.edge_type.clone()
                ) {
                    warn!(
                        "Failed to add dependency: {} -> {}: {}",
                        dependency.source_symbol_id,
                        dependency.target_symbol_id,
                        e
                    );
                }
            }
        }

        Ok(graph_builder)
    }

    /// Get pipeline statistics
    pub async fn get_stats(&self) -> PipelineStats {
        self.stats.read().await.clone()
    }

    // Private helper methods

    /// Discover files in a directory
    fn discover_files(&self, directory_path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(directory_path)
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Filter files based on configuration
    fn filter_eligible_files(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut eligible = Vec::new();

        for file_path in files {
            let path_str = file_path.to_string_lossy();

            // Check exclude patterns
            let excluded = self.config.exclude_patterns.iter()
                .any(|pattern| glob::Pattern::new(pattern).map_or(false, |p| p.matches(&path_str)));

            if excluded {
                continue;
            }

            // Check include patterns
            let included = if self.config.include_patterns.is_empty() {
                true
            } else {
                self.config.include_patterns.iter()
                    .any(|pattern| glob::Pattern::new(pattern).map_or(false, |p| p.matches(&path_str)))
            };

            if included {
                eligible.push(file_path.clone());
            }
        }

        Ok(eligible)
    }

    /// Parse files in parallel
    async fn parse_files_parallel(&self, files: Vec<PathBuf>, repository_id: &str) -> Result<Vec<FileParseResult>> {
        let thread_count = if self.config.parsing_threads == 0 {
            num_cpus::get()
        } else {
            self.config.parsing_threads
        };

        info!("Parsing {} files using {} threads", files.len(), thread_count);

        // Create batches for parallel processing
        let batch_size = (files.len() + thread_count - 1) / thread_count;
        let batches: Vec<_> = files.chunks(batch_size).collect();

        let mut results = Vec::new();

        for batch in batches {
            let batch_results: Vec<_> = futures::future::join_all(
                batch.iter().map(|file_path| {
                    let pipeline = self;
                    let repository_id = repository_id.to_string();
                    async move {
                        pipeline.parse_file(file_path, &repository_id).await
                    }
                })
            ).await;

            for result in batch_results {
                results.push(result?);
            }
        }

        Ok(results)
    }

    /// Detect language for a file
    fn detect_language(&self, file_path: &Path, content: &str) -> Result<String> {
        // First try to detect from file extension
        if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
            let language = match extension.to_lowercase().as_str() {
                "java" => "java".to_string(),
                "js" | "mjs" => "javascript".to_string(),
                "py" => "python".to_string(),
                "sh" | "bash" => "shell".to_string(),
                _ => {
                    // Try to detect from content if extension is not recognized
                    self.detect_language_from_content(content)
                }
            };
            return Ok(language);
        }

        // Try to detect from content
        Ok(self.detect_language_from_content(content))
    }

    /// Detect language from file content
    fn detect_language_from_content(&self, content: &str) -> String {
        // Simple heuristics for language detection
        if content.contains("import java") || content.contains("public class") {
            "java".to_string()
        } else if content.contains("function") || content.contains("const ") || content.contains("let ") {
            "javascript".to_string()
        } else if content.contains("def ") || content.contains("import ") || content.contains("from ") {
            "python".to_string()
        } else if content.contains("#!/bin/bash") || content.contains("#!/bin/sh") {
            "shell".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Extract symbols from a parsed tree
    fn extract_symbols(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_path: &Path,
        language: &str,
        repository_id: &str,
    ) -> Result<Vec<CodeSymbol>> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();

        self.extract_symbols_recursive(&mut symbols, &root_node, content, file_path, language, repository_id)?;

        Ok(symbols)
    }

    /// Recursively extract symbols from tree nodes
    fn extract_symbols_recursive(
        &self,
        symbols: &mut Vec<CodeSymbol>,
        node: &tree_sitter::Node,
        content: &str,
        file_path: &Path,
        language: &str,
        repository_id: &str,
    ) -> Result<()> {
        // Check if this node represents a symbol we want to extract
        if let Some(symbol) = self.node_to_symbol(node, content, file_path, language, repository_id)? {
            symbols.push(symbol);
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_recursive(symbols, &child, content, file_path, language, repository_id)?;
        }

        Ok(())
    }

    /// Convert a tree-sitter node to a code symbol
    fn node_to_symbol(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        file_path: &Path,
        language: &str,
        repository_id: &str,
    ) -> Result<Option<CodeSymbol>> {
        let node_kind = node.kind();
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();

        // Extract the text for this node
        let node_text = &content[start_byte..end_byte];

        // Determine symbol type and extract information based on language
        match language {
            "java" => self.extract_java_symbol(node, node_text, content, file_path, repository_id),
            "javascript" => self.extract_javascript_symbol(node, node_text, content, file_path, repository_id),
            "python" => self.extract_python_symbol(node, node_text, content, file_path, repository_id),
            "shell" => self.extract_shell_symbol(node, node_text, content, file_path, repository_id),
            _ => Ok(None),
        }
    }

    /// Extract Java symbol from node
    fn extract_java_symbol(
        &self,
        node: &tree_sitter::Node,
        node_text: &str,
        content: &str,
        file_path: &Path,
        repository_id: &str,
    ) -> Result<Option<CodeSymbol>> {
        let kind = node.kind();

        match kind {
            "method_declaration" | "constructor_declaration" => {
                // Extract method name and details
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);
                    let symbol_type = if kind == "constructor_declaration" {
                        SymbolType::Constructor
                    } else {
                        SymbolType::Method
                    };

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        symbol_type,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("java".to_string());
                    symbol.set_visibility(self.extract_java_visibility(node)?);
                    symbol.set_column(node.start_position().column as u32);

                    // Add metadata
                    let metadata = SymbolMetadataBuilder::new()
                        .lines_of_code((node.end_position().row - node.start_position().row + 1) as u32)
                        .build();
                    for (key, value) in metadata {
                        symbol.add_metadata(key, value);
                    }

                    return Ok(Some(symbol));
                }
            }
            "class_declaration" | "interface_declaration" | "enum_declaration" => {
                // Extract class/interface/enum name
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);
                    let symbol_type = match kind {
                        "class_declaration" => SymbolType::Class,
                        "interface_declaration" => SymbolType::Interface,
                        "enum_declaration" => SymbolType::Enum,
                        _ => SymbolType::Class,
                    };

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        symbol_type,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("java".to_string());
                    symbol.set_visibility(self.extract_java_visibility(node)?);
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Extract JavaScript symbol from node
    fn extract_javascript_symbol(
        &self,
        node: &tree_sitter::Node,
        node_text: &str,
        content: &str,
        file_path: &Path,
        repository_id: &str,
    ) -> Result<Option<CodeSymbol>> {
        let kind = node.kind();

        match kind {
            "function_declaration" | "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);
                    let symbol_type = if kind == "function_declaration" {
                        SymbolType::Function
                    } else {
                        SymbolType::Method
                    };

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        symbol_type,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("javascript".to_string());
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        SymbolType::Class,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("javascript".to_string());
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Extract Python symbol from node
    fn extract_python_symbol(
        &self,
        node: &tree_sitter::Node,
        node_text: &str,
        content: &str,
        file_path: &Path,
        repository_id: &str,
    ) -> Result<Option<CodeSymbol>> {
        let kind = node.kind();

        match kind {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        SymbolType::Function,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("python".to_string());
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        SymbolType::Class,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("python".to_string());
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Extract Shell symbol from node
    fn extract_shell_symbol(
        &self,
        node: &tree_sitter::Node,
        node_text: &str,
        content: &str,
        file_path: &Path,
        repository_id: &str,
    ) -> Result<Option<CodeSymbol>> {
        let kind = node.kind();

        match kind {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, content);

                    let mut symbol = CodeSymbol::new(
                        Uuid::new_v4().to_string(),
                        name,
                        SymbolType::Function,
                        file_path.to_string_lossy().to_string(),
                        (node.start_position().row + 1) as u32,
                        repository_id.to_string(),
                    );
                    symbol.set_language("shell".to_string());
                    symbol.set_column(node.start_position().column as u32);

                    return Ok(Some(symbol));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Extract Java visibility modifier
    fn extract_java_visibility(&self, node: &tree_sitter::Node) -> Result<SymbolVisibility> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "public" => return Ok(SymbolVisibility::Public),
                "private" => return Ok(SymbolVisibility::Private),
                "protected" => return Ok(SymbolVisibility::Protected),
                _ => {}
            }
        }
        Ok(SymbolVisibility::Unknown)
    }

    /// Get text content of a node
    fn get_node_text(&self, node: &tree_sitter::Node, content: &str) -> String {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        content[start_byte..end_byte].trim().to_string()
    }

    /// Extract dependencies from a parsed tree
    fn extract_dependencies(
        &self,
        tree: &tree_sitter::Tree,
        content: &str,
        file_path: &Path,
        symbols: &[CodeSymbol],
    ) -> Result<Vec<DependencyEdge>> {
        // This is a simplified implementation
        // In a full implementation, you would analyze the tree to find:
        // - Function calls
        // - Class instantiations
        // - Import statements
        // - Variable references
        // etc.

        let mut dependencies = Vec::new();

        // For now, create simple example dependencies
        // In a real implementation, this would be much more sophisticated
        for (i, symbol) in symbols.iter().enumerate() {
            for other_symbol in symbols.iter().skip(i + 1) {
                // Create a sample dependency (this would be based on actual code analysis)
                if symbol.symbol_type == SymbolType::Function && other_symbol.symbol_type == SymbolType::Function {
                    let edge = DependencyEdge::new(
                        Uuid::new_v4().to_string(),
                        symbol.id.clone(),
                        other_symbol.id.clone(),
                        EdgeType::Calls,
                    );
                    dependencies.push(edge);
                }
            }
        }

        Ok(dependencies)
    }
}

impl Default for ParserPipeline {
    fn default() -> Self {
        Self::new().expect("Failed to create default parser pipeline")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_parser_pipeline_creation() {
        let pipeline = ParserPipeline::new();
        assert!(pipeline.is_ok());
    }

    #[tokio::test]
    async fn test_parse_simple_java_file() {
        // Create a temporary directory with a Java file
        let temp_dir = TempDir::new().unwrap();
        let java_file = temp_dir.path().join("Test.java");
        fs::write(&java_file, r#"
public class Test {
    public void testMethod() {
        System.out.println("Hello World");
    }
}
"#).unwrap();

        let pipeline = ParserPipeline::new().unwrap();
        let result = pipeline.parse_file(&java_file, "test_repo").await.unwrap();

        assert!(result.success);
        assert_eq!(result.language, "java");
        assert!(!result.symbols.is_empty());

        // Should have found the class and method
        let symbols: Vec<_> = result.symbols.iter().collect();
        assert!(symbols.iter().any(|s| s.symbol_type == SymbolType::Class));
        assert!(symbols.iter().any(|s| s.symbol_type == SymbolType::Method));
    }

    #[tokio::test]
    async fn test_parse_simple_python_file() {
        let temp_dir = TempDir::new().unwrap();
        let py_file = temp_dir.path().join("test.py");
        fs::write(&py_file, r#"
def test_function():
    print("Hello World")

class TestClass:
    def method(self):
        pass
"#).unwrap();

        let pipeline = ParserPipeline::new().unwrap();
        let result = pipeline.parse_file(&py_file, "test_repo").await.unwrap();

        assert!(result.success);
        assert_eq!(result.language, "python");
        assert!(!result.symbols.is_empty());

        // Should have found the function and class
        let symbols: Vec<_> = result.symbols.iter().collect();
        assert!(symbols.iter().any(|s| s.symbol_type == SymbolType::Function));
        assert!(symbols.iter().any(|s| s.symbol_type == SymbolType::Class));
    }

    #[tokio::test]
    async fn test_language_detection() {
        let pipeline = ParserPipeline::new().unwrap();

        // Test Java detection
        let java_content = "import java.util.List; public class Test {}";
        let detected = pipeline.detect_language_from_content(java_content);
        assert_eq!(detected, "java");

        // Test JavaScript detection
        let js_content = "function test() { const x = 1; }";
        let detected = pipeline.detect_language_from_content(js_content);
        assert_eq!(detected, "javascript");

        // Test Python detection
        let py_content = "def test(): import os; pass";
        let detected = pipeline.detect_language_from_content(py_content);
        assert_eq!(detected, "python");
    }

    #[tokio::test]
    async fn test_build_graph_from_parse_results() {
        let temp_dir = TempDir::new().unwrap();
        let java_file = temp_dir.path().join("Test.java");
        fs::write(&java_file, r#"
public class Test {
    public void method1() {}
    public void method2() { method1(); }
}
"#).unwrap();

        let pipeline = ParserPipeline::new().unwrap();
        let results = pipeline.parse_file(&java_file, "test_repo").await.unwrap();

        let graph = pipeline.build_graph(&[results]).await;
        assert!(graph.is_ok());

        let graph = graph.unwrap();
        // Should have symbols for class and methods
        assert!(graph.node_count() > 0);
    }

    #[test]
    fn test_pipeline_config_default() {
        let config = ParserPipelineConfig::default();
        assert!(!config.languages.is_empty());
        assert!(!config.include_patterns.is_empty());
        assert!(!config.exclude_patterns.is_empty());
        assert!(config.extract_dependencies);
        assert!(config.detect_root_nodes);
    }
}
