//! Unit tests for dependency graph building

use crate::{
    graph::{DependencyGraph, CodeSymbol, SymbolType, DependencyEdge, EdgeType, builder::GraphBuilder},
    parser::SupportedLanguage,
};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_builder_creation() {
        // Arrange & Act
        let builder = GraphBuilder::new();

        // Assert
        assert!(builder.is_ok(), "GraphBuilder should be created successfully");
    }

    #[test]
    fn test_add_single_symbol() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();
        let symbol = CodeSymbol::new(
            "test_func".to_string(),
            "testFunction".to_string(),
            SymbolType::Function,
            "src/test.java".to_string(),
            10,
            "test_repo".to_string(),
        );

        // Act
        let node_index = builder.add_symbol(symbol).unwrap();

        // Assert
        assert!(node_index.index() >= 0, "Should add symbol and return valid index");
    }

    #[test]
    fn test_add_multiple_symbols() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        let symbol1 = CodeSymbol::new(
            "func1".to_string(),
            "function1".to_string(),
            SymbolType::Function,
            "file1.java".to_string(),
            5,
            "repo1".to_string(),
        );

        let symbol2 = CodeSymbol::new(
            "func2".to_string(),
            "function2".to_string(),
            SymbolType::Function,
            "file2.java".to_string(),
            15,
            "repo1".to_string(),
        );

        // Act
        let index1 = builder.add_symbol(symbol1).unwrap();
        let index2 = builder.add_symbol(symbol2).unwrap();

        // Assert
        assert_ne!(index1, index2, "Should return different indices for different symbols");
        assert!(index1.index() < index2.index(), "First symbol should have lower index");
    }

    #[test]
    fn test_add_dependency_edge() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        let caller = CodeSymbol::new(
            "caller".to_string(),
            "caller".to_string(),
            SymbolType::Function,
            "caller.java".to_string(),
            10,
            "repo1".to_string(),
        );

        let callee = CodeSymbol::new(
            "callee".to_string(),
            "callee".to_string(),
            SymbolType::Function,
            "callee.java".to_string(),
            20,
            "repo1".to_string(),
        );

        let caller_index = builder.add_symbol(caller).unwrap();
        let callee_index = builder.add_symbol(callee).unwrap();

        // Act
        let edge = DependencyEdge::new(
            format!("edge_{}", Uuid::new_v4()),
            caller.id.clone(),
            callee.id.clone(),
            EdgeType::Calls,
        );

        builder.add_dependency_edge(caller_index, callee_index, edge).unwrap();

        // Build and verify graph
        let graph = builder.build();

        // Assert
        assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes");
        assert_eq!(graph.edge_count(), 1, "Graph should have 1 edge");

        // Verify the edge connects the correct nodes
        let edges = graph.edges(caller_index).collect::<Vec<_>>();
        assert_eq!(edges.len(), 1, "Should have one edge from caller");
        assert_eq!(edges[0].target(), callee_index, "Edge should target callee");
    }

    #[test]
    fn test_multiple_dependency_edges() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        // Create three symbols: A -> B -> C
        let symbol_a = CodeSymbol::new("func_a", "FuncA", SymbolType::Function, "a.java", 1, "repo1");
        let symbol_b = CodeSymbol::new("func_b", "FuncB", SymbolType::Function, "b.java", 1, "repo1");
        let symbol_c = CodeSymbol::new("func_c", "FuncC", SymbolType::Function, "c.java", 1, "repo1");

        let index_a = builder.add_symbol(symbol_a).unwrap();
        let index_b = builder.add_symbol(symbol_b).unwrap();
        let index_c = builder.add_symbol(symbol_c).unwrap();

        // Act: Create edges A->B and B->C
        let edge_ab = DependencyEdge::new("edge_ab".to_string(), "func_a".to_string(), "func_b".to_string(), EdgeType::Calls);
        let edge_bc = DependencyEdge::new("edge_bc".to_string(), "func_b".to_string(), "func_c".to_string(), EdgeType::Calls);

        builder.add_dependency_edge(index_a, index_b, edge_ab).unwrap();
        builder.add_dependency_edge(index_b, index_c, edge_bc).unwrap();

        // Build and verify
        let graph = builder.build();

        // Assert
        assert_eq!(graph.node_count(), 3, "Graph should have 3 nodes");
        assert_eq!(graph.edge_count(), 2, "Graph should have 2 edges");

        // Verify node A has 1 outgoing edge
        let edges_from_a = graph.edges(index_a).collect::<Vec<_>>();
        assert_eq!(edges_from_a.len(), 1, "Node A should have 1 outgoing edge");
        assert_eq!(edges_from_a[0].target(), index_b, "A should connect to B");

        // Verify node B has 1 outgoing edge
        let edges_from_b = graph.edges(index_b).collect::<Vec<_>>();
        assert_eq!(edges_from_b.len(), 1, "Node B should have 1 outgoing edge");
        assert_eq!(edges_from_b[0].target(), index_c, "B should connect to C");

        // Verify node C has no outgoing edges
        let edges_from_c = graph.edges(index_c).collect::<Vec<_>>();
        assert_eq!(edges_from_c.len(), 0, "Node C should have no outgoing edges");
    }

    #[test]
    fn test_different_edge_types() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        let symbol1 = CodeSymbol::new("class1", "Class1", SymbolType::Class, "class1.java", 1, "repo1");
        let symbol2 = CodeSymbol::new("interface1", "Interface1", SymbolType::Class, "interface1.java", 1, "repo1");
        let symbol3 = CodeSymbol::new("method1", "Method1", SymbolType::Function, "method1.java", 1, "repo1");

        let index1 = builder.add_symbol(symbol1).unwrap();
        let index2 = builder.add_symbol(symbol2).unwrap();
        let index3 = builder.add_symbol(symbol3).unwrap();

        // Act: Add different types of edges
        let impl_edge = DependencyEdge::new("impl_edge".to_string(), "class1".to_string(), "interface1".to_string(), EdgeType::Implements);
        let call_edge = DependencyEdge::new("call_edge".to_string(), "class1".to_string(), "method1".to_string(), EdgeType::Calls);
        let import_edge = DependencyEdge::new("import_edge".to_string(), "method1".to_string(), "external_lib".to_string(), EdgeType::Imports);

        builder.add_dependency_edge(index1, index2, impl_edge).unwrap();
        builder.add_dependency_edge(index1, index3, call_edge).unwrap();
        builder.add_dependency_edge(index3, index3, import_edge).unwrap();

        // Build and verify
        let graph = builder.build();

        // Assert
        assert_eq!(graph.node_count(), 3, "Graph should have 3 nodes");
        assert_eq!(graph.edge_count(), 3, "Graph should have 3 edges");

        // Verify edge types are preserved
        let edges = graph.edges(index1).collect::<Vec<_>>();
        let edge_types: Vec<EdgeType> = edges.iter().map(|e| e.weight().edge_type).collect();

        assert!(edge_types.contains(&EdgeType::Implements), "Should have Implements edge");
        assert!(edge_types.contains(&EdgeType::Calls), "Should have Calls edge");
    }

    #[test]
    fn test_duplicate_symbol_handling() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        let symbol = CodeSymbol::new(
            "duplicate_func".to_string(),
            "duplicateFunction".to_string(),
            SymbolType::Function,
            "dup.java".to_string(),
            5,
            "repo1".to_string(),
        );

        // Act: Add the same symbol twice
        let index1 = builder.add_symbol(symbol.clone()).unwrap();
        let index2 = builder.add_symbol(symbol).unwrap();

        // Assert
        assert_eq!(index1, index2, "Should return the same index for duplicate symbol");

        let graph = builder.build();
        assert_eq!(graph.node_count(), 1, "Graph should have only 1 node for duplicate symbol");
    }

    #[test]
    fn test_graph_serialization() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        let symbol = CodeSymbol::new("test_func", "testFunc", SymbolType::Function, "test.java", 10, "test_repo");
        let index = builder.add_symbol(symbol).unwrap();

        // Act: Add an edge and serialize
        let edge = DependencyEdge::new("test_edge".to_string(), "test_func".to_string(), "test_func".to_string(), EdgeType::Calls);
        builder.add_dependency_edge(index, index, edge).unwrap();

        let graph = builder.build();

        // Serialize and deserialize
        let serialized = serde_json::to_string(&graph).unwrap();
        let deserialized: DependencyGraph = serde_json::from_str(&serialized).unwrap();

        // Assert
        assert_eq!(graph.node_count(), deserialized.node_count());
        assert_eq!(graph.edge_count(), deserialized.edge_count());
    }

    #[test]
    fn test_symbol_validation() {
        // Test symbol creation with various parameters
        let symbol = CodeSymbol::new(
            "valid_id".to_string(),
            "Valid Name".to_string(),
            SymbolType::Function,
            "valid/path.java".to_string(),
            42,
            "repo_id".to_string(),
        );

        assert_eq!(symbol.id, "valid_id");
        assert_eq!(symbol.name, "Valid Name");
        assert_eq!(symbol.symbol_type, SymbolType::Function);
        assert_eq!(symbol.file_path, "valid/path.java");
        assert_eq!(symbol.line_number, 42);
        assert_eq!(symbol.repository_id, "repo_id");
    }

    #[test]
    fn test_symbol_metadata() {
        // Arrange
        let mut symbol = CodeSymbol::new("func1", "func1", SymbolType::Function, "file1.java", 1, "repo1");

        // Act
        symbol.add_metadata("complexity".to_string(), "high".to_string());
        symbol.add_metadata("test_coverage".to_string(), "80%".to_string());

        // Assert
        let metadata = symbol.metadata();
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("complexity"), Some(&"high".to_string()));
        assert_eq!(metadata.get("test_coverage"), Some(&"80%".to_string()));
    }

    #[test]
    fn test_edge_creation_and_properties() {
        // Arrange & Act
        let edge = DependencyEdge::new(
            "edge_123".to_string(),
            "caller_id".to_string(),
            "callee_id".to_string(),
            EdgeType::Calls,
        );

        // Assert
        assert_eq!(edge.id, "edge_123");
        assert_eq!(edge.source_symbol_id, "caller_id");
        assert_eq!(edge.target_symbol_id, "callee_id");
        assert_eq!(edge.edge_type, EdgeType::Calls);
    }

    #[test]
    fn test_graph_query_dependencies() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        // Create a simple dependency chain: A -> B -> C
        let symbol_a = CodeSymbol::new("a", "A", SymbolType::Function, "a.java", 1, "repo1");
        let symbol_b = CodeSymbol::new("b", "B", SymbolType::Function, "b.java", 1, "repo1");
        let symbol_c = CodeSymbol::new("c", "C", SymbolType::Function, "c.java", 1, "repo1");

        let index_a = builder.add_symbol(symbol_a).unwrap();
        let index_b = builder.add_symbol(symbol_b).unwrap();
        let index_c = builder.add_symbol(symbol_c).unwrap();

        let edge_ab = DependencyEdge::new("ab", "a", "b", EdgeType::Calls);
        let edge_bc = DependencyEdge::new("bc", "b", "c", EdgeType::Calls);

        builder.add_dependency_edge(index_a, index_b, edge_ab).unwrap();
        builder.add_dependency_edge(index_b, index_c, edge_bc).unwrap();

        let graph = builder.build();

        // Act
        use crate::graph::GraphQuery;
        let query = crate::graph::DependencyQuery {
            symbol_id: "a".to_string(),
            include_indirect: false,
        };

        let result = query.execute(&graph).unwrap();

        // Assert
        assert_eq!(result.len(), 1, "A should have 1 direct dependency");
        assert_eq!(result[0].0, index_a, "Should return correct caller index");
        assert_eq!(result[0].1.id, "ab", "Should return correct edge");
        assert_eq!(result[0].1.source_symbol_id, "a");
        assert_eq!(result[0].1.target_symbol_id, "b");
    }

    #[test]
    fn test_graph_query_indirect_dependencies() {
        // Arrange
        let mut builder = GraphBuilder::new().unwrap();

        // Create a simple dependency chain: A -> B -> C
        let symbol_a = CodeSymbol::new("a", "A", SymbolType::Function, "a.java", 1, "repo1");
        let symbol_b = CodeSymbol::new("b", "B", SymbolType::Function, "b.java", 1, "repo1");
        let symbol_c = CodeSymbol::new("c", "C", SymbolType::Function, "c.java", 1, "repo1");

        let index_a = builder.add_symbol(symbol_a).unwrap();
        let index_b = builder.add_symbol(symbol_b).unwrap();
        let index_c = builder.add_symbol(symbol_c).unwrap();

        let edge_ab = DependencyEdge::new("ab", "a", "b", EdgeType::Calls);
        let edge_bc = DependencyEdge::new("bc", "b", "c", EdgeType::Calls);

        builder.add_dependency_edge(index_a, index_b, edge_ab).unwrap();
        builder.add_dependency_edge(index_b, index_c, edge_bc).unwrap();

        let graph = builder.build();

        // Act
        use crate::graph::GraphQuery;
        let query = crate::graph::DependencyQuery {
            symbol_id: "a".to_string(),
            include_indirect: true,
        };

        let result = query.execute(&graph).unwrap();

        // Assert
        assert!(result.len() >= 2, "A should have at least 2 dependencies (direct + indirect)");

        let dependencies: HashMap<String, &(NodeIndex, DependencyEdge)> = result.iter()
            .map(|(index, edge)| (edge.target_symbol_id.clone(), (*index, edge.clone())))
            .collect();

        assert!(dependencies.contains_key("b"), "Should include direct dependency B");
        assert!(dependencies.contains_key("c"), "Should include indirect dependency C");
    }
}