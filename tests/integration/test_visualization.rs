//! Integration tests for result visualization workflow
//!
//! These tests verify the end-to-end workflow for visualizing zombie code
//! analysis results and exploring dependency relationships.

use czi_ipc::{IpcCommand, IpcResponse, commands::{queries::QueryCommandHandler, analysis::AnalysisCommandHandler}};
use serde_json::json;
use std::collections::HashMap;

/// Test complete visualization workflow for zombie code analysis
#[test]
fn test_complete_visualization_workflow() {
    let analysis_handler = AnalysisCommandHandler::new();
    let query_handler = QueryCommandHandler::new();

    // Step 1: Run analysis
    let analysis_command = IpcCommand {
        id: "workflow_analysis".to_string(),
        name: "run_analysis".to_string(),
        params: json!({
            "repository_ids": ["test_repo_1", "test_repo_2"],
            "analysis_options": {
                "include_tests": false,
                "max_depth": 50
            }
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let analysis_result = analysis_handler.execute(analysis_command).unwrap();
    assert_eq!(analysis_result.status, "success");

    let analysis_data = analysis_result.data.unwrap();
    let analysis_id = analysis_data.get("id").unwrap().as_str().unwrap();

    // Step 2: Check analysis status
    let status_command = IpcCommand {
        id: "workflow_status".to_string(),
        name: "get_analysis_status".to_string(),
        params: json!({
            "id": analysis_id
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let status_result = analysis_handler.execute(status_command).unwrap();
    assert_eq!(status_result.status, "success");

    // Step 3: Get analysis results
    let results_command = IpcCommand {
        id: "workflow_results".to_string(),
        name: "get_analysis_results".to_string(),
        params: json!({
            "id": analysis_id
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let results_result = analysis_handler.execute(results_command).unwrap();
    assert_eq!(results_result.status, "success");

    let results_data = results_result.data.unwrap();
    assert_eq!(results_data.get("status").unwrap().as_str().unwrap(), "Completed");

    // Step 4: Extract zombie code items for visualization
    let zombie_items = results_data.get("zombie_code_items").unwrap().as_array().unwrap();
    assert!(!zombie_items.is_empty());

    // Step 5: For each zombie item, query its dependencies and dependents
    for zombie_item in zombie_items {
        let symbol_id = zombie_item.get("symbol_id").unwrap().as_str().unwrap();

        // Query dependencies
        let deps_command = IpcCommand {
            id: format!("deps_{}", symbol_id),
            name: "query_dependencies".to_string(),
            params: json!({
                "symbol_id": symbol_id,
                "include_indirect": true
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let deps_result = query_handler.execute(deps_command).unwrap();
        assert_eq!(deps_result.status, "success");

        // Query dependents
        let dependents_command = IpcCommand {
            id: format!("dependents_{}", symbol_id),
            name: "query_dependents".to_string(),
            params: json!({
                "symbol_id": symbol_id,
                "include_indirect": false
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let dependents_result = query_handler.execute(dependents_command).unwrap();
        assert_eq!(dependents_result.status, "success");

        // Get detailed symbol info
        let symbol_info_command = IpcCommand {
            id: format!("info_{}", symbol_id),
            name: "get_symbol_info".to_string(),
            params: json!({
                "symbol_id": symbol_id
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let info_result = query_handler.execute(symbol_info_command).unwrap();
        assert_eq!(info_result.status, "success");
    }
}

/// Test dependency path visualization
#[test]
fn test_dependency_path_visualization() {
    let query_handler = QueryCommandHandler::new();

    // Find path between a root node and a zombie symbol
    let path_command = IpcCommand {
        id: "path_viz_test".to_string(),
        name: "find_path_between_symbols".to_string(),
        params: json!({
            "from_symbol_id": "com.example.MainController.handleRequest",
            "to_symbol_id": "com.example.LegacyUtils.processData",
            "max_depth": 20
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = query_handler.execute(path_command).unwrap();
    assert_eq!(result.status, "success");

    let data = result.data.unwrap();
    let path_found = data.get("path_found").unwrap().as_bool().unwrap();

    if path_found {
        let path = data.get("path").unwrap().as_array().unwrap();
        assert!(!path.is_empty());

        // Verify each path item has required fields
        for path_item in path {
            assert!(path_item.get("symbol_id").is_some());
            assert!(path_item.get("symbol_name").is_some());
            assert!(path_item.get("file_path").is_some());
            assert!(path_item.get("relationship_type").is_some());
        }
    }
}

/// Test isolation boundary visualization
#[test]
fn test_isolation_boundary_visualization() {
    let query_handler = QueryCommandHandler::new();

    let boundary_command = IpcCommand {
        id: "boundary_viz_test".to_string(),
        name: "get_isolation_boundary".to_string(),
        params: json!({
            "symbol_id": "com.example.OrphanedFunction.calculate"
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = query_handler.execute(boundary_command).unwrap();
    assert_eq!(result.status, "success");

    let data = result.data.unwrap();

    assert!(data.get("isolation_distance").is_some());
    assert!(data.get("boundary_symbols").is_some());
    assert!(data.get("nearest_active_symbol").is_some());

    let boundary_symbols = data.get("boundary_symbols").unwrap().as_array().unwrap();

    // Verify boundary symbols have required visualization data
    for symbol in boundary_symbols {
        assert!(symbol.get("symbol_id").is_some());
        assert!(symbol.get("symbol_name").is_some());
        assert!(symbol.get("distance").is_some());
        assert!(symbol.get("path").is_some());
    }
}

/// Test graph structure for visualization
#[test]
fn test_graph_structure_for_visualization() {
    let query_handler = QueryCommandHandler::new();

    // Get a symbol with known dependencies
    let symbol_info_command = IpcCommand {
        id: "graph_structure_test".to_string(),
        name: "get_symbol_info".to_string(),
        params: json!({
            "symbol_id": "com.example.ServiceLayer.processOrder"
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = query_handler.execute(symbol_info_command).unwrap();
    assert_eq!(result.status, "success");

    let data = result.data.unwrap();
    let symbol = data.get("symbol").unwrap();

    // Verify symbol has fields needed for graph visualization
    assert!(symbol.get("id").is_some());
    assert!(symbol.get("name").is_some());
    assert!(symbol.get("symbol_type").is_some());
    assert!(symbol.get("file_path").is_some());
    assert!(symbol.get("line_number").is_some());
    assert!(symbol.get("visibility").is_some());
    assert!(symbol.get("exported").is_some());

    // Get dependencies for graph edges
    let deps_command = IpcCommand {
        id: "graph_edges_test".to_string(),
        name: "query_dependencies".to_string(),
        params: json!({
            "symbol_id": "com.example.ServiceLayer.processOrder",
            "include_indirect": false
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let deps_result = query_handler.execute(deps_command).unwrap();
    assert_eq!(deps_result.status, "success");

    let deps_data = deps_result.data.unwrap();
    let dependencies = deps_data.get("dependencies").unwrap().as_array().unwrap();

    // Verify dependency edges have visualization data
    for dep in dependencies {
        assert!(dep.get("target_symbol_id").is_some());
        assert!(dep.get("target_symbol_name").is_some());
        assert!(dep.get("relationship_type").is_some());
        assert!(dep.get("confidence").is_some());
        assert!(dep.get("line_number").is_some());
    }
}

/// Test filtering and search for visualization
#[test]
fn test_filtering_and_search_for_visualization() {
    let analysis_handler = AnalysisCommandHandler::new();

    // Start an analysis that we can filter results from
    let analysis_command = IpcCommand {
        id: "filter_test_analysis".to_string(),
        name: "run_analysis".to_string(),
        params: json!({
            "repository_ids": ["large_repo"],
            "filters": {
                "languages": ["java", "python"],
                "min_confidence": 0.7
            }
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let analysis_result = analysis_handler.execute(analysis_command).unwrap();
    let analysis_id = analysis_result.data.unwrap()
        .get("id").unwrap().as_str().unwrap();

    // Get filtered results
    let results_command = IpcCommand {
        id: "filter_test_results".to_string(),
        name: "get_analysis_results".to_string(),
        params: json!({
            "id": analysis_id,
            "filters": {
                "zombie_types": ["DeadCode", "Orphaned"],
                "confidence_range": [0.8, 1.0],
                "file_patterns": ["src/legacy/*", "src/old/*"]
            }
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let results_result = analysis_handler.execute(results_command).unwrap();
    assert_eq!(results_result.status, "success");

    let data = results_result.data.unwrap();
    let zombie_items = data.get("zombie_code_items").unwrap().as_array().unwrap();

    // Verify filtered results meet criteria
    for item in zombie_items {
        let confidence = item.get("confidence").unwrap().as_f64().unwrap();
        assert!(confidence >= 0.8);

        let zombie_type = item.get("zombie_type").unwrap().as_str().unwrap();
        assert!(zombie_type == "DeadCode" || zombie_type == "Orphaned");
    }
}

/// Test performance metrics for visualization
#[test]
fn test_performance_metrics_for_visualization() {
    let query_handler = QueryCommandHandler::new();

    // Test query response time
    let start_time = std::time::Instant::now();

    let command = IpcCommand {
        id: "perf_test".to_string(),
        name: "query_dependencies".to_string(),
        params: json!({
            "symbol_id": "com.example.HighVolumeService.processData",
            "include_indirect": true
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = query_handler.execute(command).unwrap();
    let elapsed = start_time.elapsed();

    // Should respond within performance requirements (50ms target)
    assert!(elapsed.as_millis() < 100); // Allow some tolerance for test environment
    assert_eq!(result.status, "success");

    // Check for performance metadata in response
    let data = result.data.unwrap();
    assert!(data.get("query_time_ms").is_some());
    assert!(data.get("dependency_count").is_some());
}

/// Create a test codebase for visualization testing
struct TestCodebase {
    repositories: HashMap<String, Vec<String>>,
}

impl TestCodebase {
    fn new() -> Self {
        let mut repositories = HashMap::new();

        // Repository 1: Main application
        repositories.insert("main_app".to_string(), vec![
            "src/main/java/Main.java".to_string(),
            "src/main/java/controller/UserController.java".to_string(),
            "src/main/java/service/UserService.java".to_string(),
            "src/main/java/database/UserRepository.java".to_string(),
            "src/main/java/legacy/OldAuthSystem.java".to_string(),
            "src/main/java/utils/UnusedHelper.java".to_string(),
        ]);

        // Repository 2: Utilities
        repositories.insert("utils".to_string(), vec![
            "src/main/java/utils/CommonUtils.java".to_string(),
            "src/main/java/utils/DateTimeHelper.java".to_string(),
            "src/main/java/legacy/LegacyFormatter.java".to_string(),
        ]);

        Self { repositories }
    }

    fn get_file_content(&self, repo: &str, file: &str) -> String {
        match (repo, file) {
            ("main_app", "src/main/java/controller/UserController.java") => {
                r#"
package controller;

import service.UserService;
import legacy.OldAuthSystem;

public class UserController {
    private UserService userService;

    public UserController() {
        this.userService = new UserService();
    }

    public String createUser(String name) {
        return userService.createUser(name);
    }

    public String deleteUser(String id) {
        return userService.deleteUser(id);
    }
}
                "#.to_string()
            }
            ("main_app", "src/main/java/legacy/OldAuthSystem.java") => {
                r#"
package legacy;

import database.UserRepository;

public class OldAuthSystem {
    private UserRepository repo;

    public OldAuthSystem() {
        this.repo = new UserRepository();
    }

    public boolean authenticate(String token) {
        // This is no longer used - replaced by JWT auth
        return validateToken(token);
    }

    private boolean validateToken(String token) {
        // Legacy token validation
        return token != null && token.length() > 10;
    }
}
                "#.to_string()
            }
            _ => "// Sample code content".to_string(),
        }
    }
}