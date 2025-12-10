//! Dependency graph builder

use crate::{Result, CodeSymbol, DependencyEdge, SymbolType, EdgeType, graph::DependencyGraph};
use petgraph::{graph::Graph, Directed, graph::NodeIndex};
use std::collections::HashMap;

/// Builder for creating dependency graphs
pub struct GraphBuilder {
    graph: DependencyGraph,
    symbol_index_map: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            symbol_index_map: HashMap::new(),
        }
    }

    /// Add a symbol to the graph
    pub fn add_symbol(&mut self, symbol: CodeSymbol) -> Result<NodeIndex> {
        let unique_id = symbol.id.clone();

        // Check if symbol already exists
        if let Some(&existing_index) = self.symbol_index_map.get(&unique_id) {
            return Ok(existing_index);
        }

        // Add new symbol to graph
        let node_index = self.graph.add_node(symbol);
        self.symbol_index_map.insert(unique_id, node_index);

        Ok(node_index)
    }

    /// Add a dependency edge between two symbols
    pub fn add_dependency(
        &mut self,
        source_id: &str,
        target_id: &str,
        edge_type: super::EdgeType,
    ) -> Result<()> {
        let source_index = self.symbol_index_map.get(source_id)
            .ok_or_else(|| crate::CziError::graph(
                format!("Source symbol not found: {}", source_id)
            ))?;

        let target_index = self.symbol_index_map.get(target_id)
            .ok_or_else(|| crate::CziError::graph(
                format!("Target symbol not found: {}", target_id)
            ))?;

        let edge = DependencyEdge {
            id: format!("{}_{}", source_id, target_id),
            source_symbol_id: source_id.to_string(),
            target_symbol_id: target_id.to_string(),
            edge_type,
            confidence: 0.8, // Default confidence
            strong: true,    // Default to strong dependency
            line_number: None,
            file_path: None,
            metadata: std::collections::HashMap::new(),
            discovered_at: chrono::Utc::now(),
        };

        self.graph.add_edge(*source_index, *target_index, edge);
        Ok(())
    }

    /// Get the node index for a symbol
    pub fn get_symbol_index(&self, symbol_id: &str) -> Option<NodeIndex> {
        self.symbol_index_map.get(symbol_id).copied()
    }

    /// Get a reference to the symbol by ID
    pub fn get_symbol(&self, symbol_id: &str) -> Option<&CodeSymbol> {
        self.symbol_index_map.get(symbol_id)
            .and_then(|&index| self.graph.node_weight(index))
    }

    /// Get a mutable reference to the symbol by ID
    pub fn get_symbol_mut(&mut self, symbol_id: &str) -> Option<&mut CodeSymbol> {
        self.symbol_index_map.get(symbol_id)
            .and_then(|&index| self.graph.node_weight_mut(index))
    }

    /// Get all symbols in the graph
    pub fn get_all_symbols(&self) -> Vec<&CodeSymbol> {
        self.graph.node_weights().collect()
    }

    /// Get all outgoing edges for a symbol
    pub fn get_dependencies(&self, symbol_id: &str) -> Vec<&DependencyEdge> {
        if let Some(&node_index) = self.symbol_index_map.get(symbol_id) {
            self.graph.edges(node_index)
                .map(|edge_ref| edge_ref.weight())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Build and return the completed graph
    pub fn build(self) -> DependencyGraph {
        self.graph
    }

    /// Get current graph statistics
    pub fn get_metrics(&self) -> super::GraphMetrics {
        let total_nodes = self.graph.node_count();
        let total_edges = self.graph.edge_count();

        // Count orphan nodes (nodes with no edges)
        let orphan_nodes = self.graph.node_indices()
            .filter(|&node| {
                self.graph.edges(node).next().is_none() &&
                self.graph.edges_directed(node, petgraph::Direction::Incoming).next().is_none() &&
                self.graph.edges_directed(node, petgraph::Direction::Outgoing).next().is_none()
            })
            .count();

        // Calculate average out degree
        let average_out_degree = if total_nodes > 0 {
            total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };

        // For now, use placeholder values for SCC count and max depth
        // These would require more complex graph analysis
        super::GraphMetrics {
            total_nodes,
            total_edges,
            orphan_nodes,
            strongly_connected_components: 1, // Placeholder
            average_out_degree,
            max_depth: 0, // Placeholder - would require graph traversal
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();

        // Add symbols
        let symbol1 = CodeSymbol::new(
            "func1".to_string(),
            "test_function".to_string(),
            SymbolType::Function,
            "test.js".to_string(),
            10,
            "repo1".to_string(),
        );

        let symbol2 = CodeSymbol::new(
            "func2".to_string(),
            "helper_function".to_string(),
            SymbolType::Function,
            "test.js".to_string(),
            20,
            "repo1".to_string(),
        );

        let idx1 = builder.add_symbol(symbol1).unwrap();
        let idx2 = builder.add_symbol(symbol2).unwrap();

        // Add dependency
        builder.add_dependency("repo1::test.js::func1", "repo1::test.js::func2", EdgeType::Calls)
            .unwrap();

        // Verify structure
        let metrics = builder.get_metrics();
        assert_eq!(metrics.total_nodes, 2);
        assert_eq!(metrics.total_edges, 1);
        assert_eq!(metrics.orphan_nodes, 1); // symbol2 has no outgoing edges

        // Build graph
        let graph = builder.build();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}
