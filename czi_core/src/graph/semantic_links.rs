//! Semantic link detection

use crate::{Result, graph::EdgeType, graph::GraphBuilder};
use std::collections::HashMap;

/// Detector for semantic links between code symbols
pub struct SemanticLinkDetector {
    config: SemanticLinkConfig,
}

/// Configuration for semantic link detection
#[derive(Debug, Clone)]
pub struct SemanticLinkConfig {
    /// Enable naming convention based links
    pub enable_naming_conventions: bool,
    /// Enable framework-specific link detection
    pub enable_framework_links: bool,
    /// Enable annotation-based links
    pub enable_annotation_links: bool,
}

impl Default for SemanticLinkConfig {
    fn default() -> Self {
        Self {
            enable_naming_conventions: true,
            enable_framework_links: true,
            enable_annotation_links: true,
        }
    }
}

/// Types of semantic links
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticLinkType {
    /// Naming convention suggests relationship
    NamingConvention,
    /// Framework-specific link (e.g., Spring annotations)
    FrameworkLink,
    /// Annotation-based link
    AnnotationLink,
    /// File-based relationship (same file, package)
    FileBased,
}

impl SemanticLinkDetector {
    /// Create a new semantic link detector
    pub fn new() -> Self {
        Self::with_config(SemanticLinkConfig::default())
    }

    /// Create a detector with custom configuration
    pub fn with_config(config: SemanticLinkConfig) -> Self {
        Self { config }
    }

    /// Detect semantic links and add them to the graph
    pub fn detect_semantic_links(&self, graph: &mut GraphBuilder) -> Result<usize> {
        let mut links_added = 0;

        links_added += self.detect_naming_convention_links(graph)?;
        links_added += self.detect_framework_links(graph)?;
        links_added += self.detect_annotation_links(graph)?;
        links_added += self.detect_file_based_links(graph)?;

        Ok(links_added)
    }

    /// Detect links based on naming conventions
    fn detect_naming_convention_links(&self, graph: &mut GraphBuilder) -> Result<usize> {
        if !self.config.enable_naming_conventions {
            return Ok(0);
        }

        let symbol_index_map = self.create_symbol_index_map(graph);
        let mut links_to_add = Vec::new();

        for (symbol1_id, &index1) in &symbol_index_map {
            let symbol1 = graph.get_symbol(symbol1_id).unwrap();

            // Look for symbols with related names
            for (symbol2_id, &index2) in &symbol_index_map {
                if index1 == index2 {
                    continue;
                }

                let symbol2 = graph.get_symbol(symbol2_id).unwrap();

                // Check for naming convention patterns
                if self.has_naming_convention_relationship(symbol1, symbol2) {
                    let edge_type = self.determine_edge_type_by_convention(symbol1, symbol2);
                    links_to_add.push((symbol1_id.clone(), symbol2_id.clone(), edge_type));
                }
            }
        }

        // Add the links after collecting them
        let links_added = links_to_add.len();
        for (source_id, target_id, edge_type) in links_to_add {
            graph.add_dependency(&source_id, &target_id, edge_type)?;
        }

        Ok(links_added)
    }

    /// Detect framework-specific links
    fn detect_framework_links(&self, graph: &mut GraphBuilder) -> Result<usize> {
        if !self.config.enable_framework_links {
            return Ok(0);
        }

        let mut links_added = 0;

        // Framework-specific patterns would be implemented here
        // For example:
        // - Spring Boot: @RestController patterns
        // - Express.js: router patterns
        // - Django: @api_view decorators
        // - Spring: @Service, @Repository patterns

        // Placeholder implementation
        Ok(links_added)
    }

    /// Detect annotation-based links
    fn detect_annotation_links(&self, graph: &mut GraphBuilder) -> Result<usize> {
        if !self.config.enable_annotation_links {
            return Ok(0);
        }

        let mut links_added = 0;

        // Annotation-based patterns would be implemented here
        // For example:
        // - @Autowired dependency injection
        // - @Inject annotations
        // - @Component stereotypes

        // Placeholder implementation
        Ok(links_added)
    }

    /// Detect file-based links (symbols in the same file or package)
    fn detect_file_based_links(&self, graph: &mut GraphBuilder) -> Result<usize> {
        let mut links_added = 0;
        let file_groups = self.group_symbols_by_file(graph);

        for symbols_in_file in file_groups.values() {
            // Add edges between symbols in the same file
            for (i, symbol1_id) in symbols_in_file.iter().enumerate() {
                for symbol2_id in symbols_in_file.iter().skip(i + 1) {
                    graph.add_dependency(symbol1_id, symbol2_id, EdgeType::Implements)?;
                    links_added += 1;
                }
            }
        }

        Ok(links_added)
    }

    /// Check if two symbols have a naming convention relationship
    fn has_naming_convention_relationship(&self, symbol1: &super::CodeSymbol, symbol2: &super::CodeSymbol) -> bool {
        // Helper method to check naming conventions
        let name1 = &symbol1.name;
        let name2 = &symbol2.name;

        // Check for common naming patterns
        self.is_helper_method(name1, name2) ||
        self.is_factory_method(name1, name2) ||
        self.is_utility_method(name1, name2) ||
        self.is_related_class(name1, name2) ||
        self.is_package_level_relationship(name1, name2)
    }

    /// Determine edge type based on naming convention
    fn determine_edge_type_by_convention(&self, symbol1: &super::CodeSymbol, symbol2: &super::CodeSymbol) -> EdgeType {
        if self.is_factory_method(&symbol1.name, &symbol2.name) {
            EdgeType::Calls
        } else if self.is_utility_method(&symbol1.name, &symbol2.name) {
            EdgeType::Calls
        } else if self.is_related_class(&symbol1.name, &symbol2.name) {
            EdgeType::Implements
        } else {
            EdgeType::Calls
        }
    }

    /// Check if one method is a helper for another
    fn is_helper_method(&self, method_name: &str, potential_helper: &str) -> bool {
        potential_helper.starts_with("helper") && method_name != potential_helper ||
        potential_helper.contains("Util") && method_name.contains("Util") ||
        potential_helper.starts_with("_") && !method_name.starts_with("_")
    }

    /// Check if one method is a factory for another
    fn is_factory_method(&self, method_name: &str, potential_product: &str) -> bool {
        potential_product.starts_with("create") && method_name != potential_product ||
        potential_product.starts_with("new") && method_name != potential_product ||
        potential_product.starts_with("build") && method_name.contains("Builder")
    }

    /// Check if one method is a utility for another
    fn is_utility_method(&self, method_name: &str, potential_utility: &str) -> bool {
        potential_utility.contains("Util") && method_name.contains("Util") ||
        potential_utility.starts_with("to") ||
        method_name.contains("parse") && potential_utility.contains("String")
    }

    /// Check if two names are related by class hierarchy
    fn is_related_class(&self, name1: &str, name2: &str) -> bool {
        // Simple heuristic: check if names share a common prefix
        let common_prefix = Self::common_prefix(name1, name2);
        common_prefix.len() > 2
    }

    /// Check for package-level relationships
    fn is_package_level_relationship(&self, name1: &str, name2: &str) -> bool {
        // Simple heuristic: check if names suggest package/module relationships
        name1.contains("::") && name2.contains("::") &&
        name1.split("::").next().is_some() && name1.split("::").next() == name2.split("::").next()
    }

    /// Find common prefix between two strings
    fn common_prefix<'a>(s1: &'a str, s2: &'a str) -> &'a str {
        let end = s1.chars().zip(s2.chars())
            .take_while(|(c1, c2)| c1 == c2)
            .count();

        &s1[..end]
    }

    /// Create a mapping of symbol IDs to their node indices
    fn create_symbol_index_map(&self, graph: &GraphBuilder) -> HashMap<String, petgraph::graph::NodeIndex> {
        let mut map = HashMap::new();

        for symbol in graph.get_all_symbols() {
            map.insert(symbol.id.clone(), graph.get_symbol_index(&symbol.id).unwrap());
        }

        map
    }

    /// Group symbols by their file path
    fn group_symbols_by_file(&self, graph: &GraphBuilder) -> HashMap<String, Vec<String>> {
        let mut file_groups = HashMap::new();

        for symbol in graph.get_all_symbols() {
            let file_key = format!("{}:{}", symbol.repository_id, symbol.file_path);
            file_groups.entry(file_key)
                .or_insert_with(Vec::new)
                .push(symbol.id.clone());
        }

        file_groups
    }
}

impl Default for SemanticLinkDetector {
    fn default() -> Self {
        Self::with_config(SemanticLinkConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SymbolType;

    #[test]
    fn test_semantic_link_detector_creation() {
        let detector = SemanticLinkDetector::new();
        assert!(detector.config.enable_naming_conventions);
    }

    #[test]
    fn test_naming_convention_detection() {
        let detector = SemanticLinkDetector::new();

        assert!(detector.is_helper_method("mainMethod", "helperMethod"));
        assert!(detector.is_factory_method("createUser", "UserFactory"));
        assert!(detector.is_utility_method("parseString", "StringUtil"));
    }

    #[test]
    fn test_common_prefix() {
        assert_eq!(
            SemanticLinkDetector::common_prefix("UserService", "UserManager"),
            "User"
        );
        assert_eq!(
            SemanticLinkDetector::common_prefix("com.example", "com.example.util"),
            "com.example."
        );
    }
}
