//! Graph serialization

use crate::{Result, graph::DependencyGraph, graph::GraphMetrics};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Graph serializer for saving/loading dependency graphs
pub struct GraphSerializer;

impl GraphSerializer {
    /// Create a new graph serializer
    pub fn new() -> Self {
        Self
    }

    /// Serialize graph to JSON format
    pub fn serialize_to_json(&self, graph: &DependencyGraph, path: impl AsRef<Path>) -> Result<()> {
        let graph_data = SerializableGraph::from_graph(graph);
        let json_str = serde_json::to_string_pretty(&graph_data)?;
        fs::write(path, json_str)?;
        Ok(())
    }

    /// Serialize graph to binary format using bincode
    pub fn serialize_to_binary(&self, graph: &DependencyGraph, path: impl AsRef<Path>) -> Result<()> {
        let graph_data = SerializableGraph::from_graph(graph);
        let binary_data = bincode::serialize(&graph_data)?;
        fs::write(path, binary_data)?;
        Ok(())
    }

    /// Load graph from JSON format
    pub fn load_from_json(&self, path: impl AsRef<Path>) -> Result<DependencyGraph> {
        let json_str = fs::read_to_string(path)?;
        let graph_data: SerializableGraph = serde_json::from_str(&json_str)?;
        Ok(graph_data.to_graph()?)
    }

    /// Load graph from binary format
    pub fn load_from_binary(&self, path: impl AsRef<Path>) -> Result<DependencyGraph> {
        let binary_data = fs::read(path)?;
        let graph_data: SerializableGraph = bincode::deserialize(&binary_data)?;
        Ok(graph_data.to_graph()?)
    }

    /// Export graph to Graphviz DOT format for visualization
    pub fn export_to_dot(&self, graph: &DependencyGraph, path: impl AsRef<Path>) -> Result<()> {
        use petgraph::dot::{Dot, Config};

        let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
        let dot_str = format!("{}", dot);
        fs::write(path, dot_str)?;
        Ok(())
    }

    /// Calculate and return graph metrics
    pub fn calculate_metrics(&self, graph: &DependencyGraph) -> GraphMetrics {
        

        let total_nodes = graph.node_count();
        let total_edges = graph.edge_count();

        // Count orphan nodes
        let orphan_nodes = graph.node_indices()
            .filter(|&node| {
                graph.edges(node).next().is_none() &&
                graph.edges_directed(node, petgraph::Direction::Outgoing).next().is_none() &&
                graph.edges_directed(node, petgraph::Direction::Incoming).next().is_none()
            })
            .count();

        // Calculate strongly connected components (placeholder)
        let scc_count = 1; // Placeholder - would need proper SCC calculation

        // Calculate average out degree
        let average_out_degree = if total_nodes > 0 {
            total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };

        // Calculate maximum depth using BFS
        let max_depth = if graph.node_count() > 0 {
            // For now, use a simple calculation - this could be more sophisticated
            graph.node_count() as usize
        } else {
            0
        };

        GraphMetrics {
            total_nodes,
            total_edges,
            orphan_nodes,
            strongly_connected_components: scc_count,
            average_out_degree,
            max_depth,
        }
    }
}

impl Default for GraphSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable representation of a dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGraph {
    pub symbols: Vec<super::CodeSymbol>,
    pub edges: Vec<super::DependencyEdge>,
    pub metadata: GraphMetadata,
}

/// Metadata associated with the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub metrics: GraphMetrics,
}

impl SerializableGraph {
    /// Create a serializable graph from a dependency graph
    pub fn from_graph(graph: &DependencyGraph) -> Self {
        let symbols: Vec<super::CodeSymbol> = graph.node_weights().cloned().collect();
        let edges: Vec<super::DependencyEdge> = graph.edge_weights().cloned().collect();

        let serializer = GraphSerializer::new();
        let metrics = serializer.calculate_metrics(graph);

        Self {
            symbols,
            edges,
            metadata: GraphMetadata {
                version: "1.0".to_string(),
                created_at: chrono::Utc::now(),
                description: None,
                metrics,
            },
        }
    }

    /// Convert back to a dependency graph
    pub fn to_graph(&self) -> Result<DependencyGraph> {
        use petgraph::graph::Graph;
        

        let mut graph = Graph::<super::CodeSymbol, super::DependencyEdge, petgraph::Directed>::new();

        // Create a mapping from symbol IDs to node indices
        let mut symbol_to_index = std::collections::HashMap::new();

        // Add all symbols as nodes
        for symbol in &self.symbols {
            let index = graph.add_node(symbol.clone());
            symbol_to_index.insert(symbol.id.clone(), index);
        }

        // Add all edges
        for edge in &self.edges {
            if let (Some(&source_index), Some(&target_index)) = (
                symbol_to_index.get(&edge.source_symbol_id),
                symbol_to_index.get(&edge.target_symbol_id),
            ) {
                graph.add_edge(source_index, target_index, edge.clone());
            }
        }

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CodeSymbol, SymbolType, graph::GraphBuilder};

    #[test]
    fn test_serialization_roundtrip() {
        let serializer = GraphSerializer::new();
        let mut builder = GraphBuilder::new();

        // Create a test graph
        let symbol1 = CodeSymbol::new(
            "test_func".to_string(),
            "test".to_string(),
            SymbolType::Function,
            "test.js".to_string(),
            10,
            "repo1".to_string(),
        );

        let symbol2 = CodeSymbol::new(
            "helper_func".to_string(),
            "helper".to_string(),
            SymbolType::Function,
            "utils.js".to_string(),
            5,
            "repo1".to_string(),
        );

        builder.add_symbol(symbol1).unwrap();
        builder.add_symbol(symbol2).unwrap();

        let graph = builder.build();

        // Test serialization and deserialization
        let serializable = SerializableGraph::from_graph(&graph);
        let reconstructed_graph = serializable.to_graph().unwrap();

        assert_eq!(graph.node_count(), reconstructed_graph.node_count());
        assert_eq!(graph.edge_count(), reconstructed_graph.edge_count());
    }
}
