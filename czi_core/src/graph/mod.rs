//! Graph operations for CodeZombiesInvestigator
//!
//! Provides dependency graph construction and analysis using petgraph.

pub mod edge;
pub mod builder;
pub mod serialization;
pub mod detectors;
pub mod semantic_links;

use crate::{Result, parser::CodeSymbol};
use petgraph::{graph::Graph, Directed, graph::NodeIndex, visit::EdgeRef};
use serde::{Deserialize, Serialize};

// Re-export main types
pub use edge::{DependencyEdge, EdgeType};
pub use builder::GraphBuilder;
pub use serialization::GraphSerializer;
pub use detectors::calls::CallDetector;
pub use detectors::imports::ImportDetector;
pub use semantic_links::SemanticLinkDetector;

/// Main dependency graph type alias
pub type DependencyGraph = Graph<CodeSymbol, DependencyEdge, Directed>;

/// Graph statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub orphan_nodes: usize,
    pub strongly_connected_components: usize,
    pub average_out_degree: f64,
    pub max_depth: usize,
}

/// Graph query interface
pub trait GraphQuery {
    type Output;

    fn execute(&self, graph: &DependencyGraph) -> Result<Self::Output>;
}

/// Symbol query by ID
pub struct SymbolQuery {
    pub symbol_id: String,
}

impl GraphQuery for SymbolQuery {
    type Output = Option<NodeIndex>;

    fn execute(&self, graph: &DependencyGraph) -> Result<Self::Output> {
        for node_index in graph.node_indices() {
            if let Some(symbol) = graph.node_weight(node_index) {
                if symbol.id == self.symbol_id {
                    return Ok(Some(node_index));
                }
            }
        }
        Ok(None)
    }
}

/// Dependency query for a symbol
pub struct DependencyQuery {
    pub symbol_id: String,
    pub include_indirect: bool,
}

impl GraphQuery for DependencyQuery {
    type Output = Vec<(NodeIndex, DependencyEdge)>;

    fn execute(&self, graph: &DependencyGraph) -> Result<Self::Output> {
        let symbol_query = SymbolQuery {
            symbol_id: self.symbol_id.clone(),
        };

        if let Some(node_index) = symbol_query.execute(graph)? {
            if self.include_indirect {
                // Return all reachable nodes
                let mut dependencies = Vec::new();
                let mut visited = std::collections::HashSet::new();
                let mut stack = vec![node_index];

                while let Some(current) = stack.pop() {
                    if !visited.contains(&current) {
                        visited.insert(current);

                        for edge in graph.edges(current) {
                            let target = edge.target();
                            dependencies.push((target, edge.weight().clone()));
                            stack.push(target);
                        }
                    }
                }

                Ok(dependencies)
            } else {
                // Return only direct dependencies
                Ok(graph.edges(node_index).map(|e| (e.target(), e.weight().clone())).collect())
            }
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_symbol_creation() {
        use crate::parser::SymbolType;

        let mut symbol = CodeSymbol::new(
            "create_user".to_string(),
            "createUser".to_string(),
            SymbolType::Function,
            "src/UserService.java".to_string(),
            25,
            "user_service_repo".to_string(),
        );

        symbol.add_metadata("annotations".to_string(), "@Override".to_string());
        symbol.add_metadata("visibility".to_string(), "public".to_string());

        assert_eq!(symbol.name, "createUser");
        assert_eq!(symbol.symbol_type, SymbolType::Function);
        assert_eq!(symbol.get_metadata("annotations"), Some(&"@Override".to_string()));
    }

    #[test]
    fn test_symbol_query() {
        let graph = DependencyGraph::new();
        let query = SymbolQuery {
            symbol_id: "test_func".to_string(),
        };

        let result = query.execute(&graph).unwrap();
        assert!(result.is_none()); // Empty graph should return None
    }
}