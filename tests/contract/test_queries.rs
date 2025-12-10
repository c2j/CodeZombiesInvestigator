//! Contract tests for dependency query endpoints
//!
//! These tests verify the contract between frontend and backend for
//! dependency query operations in the zombie code analysis system.

use czi_ipc::{IpcCommand, IpcResponse, commands::queries::QueryCommandHandler};
use serde_json::json;

/// Test query_dependencies contract
#[test]
fn test_query_dependencies_contract() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "query_deps_1".to_string(),
        name: "query_dependencies".to_string(),
        params: json!({
            "symbol_id": "com.example.UserService.createUser",
            "include_indirect": false
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.command_id, "query_deps_1");
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("symbol_id").is_some());
    assert!(data.get("dependencies").is_some());
    assert!(data.get("direct_count").is_some());
    assert!(data.get("total_count").is_some());
}

/// Test query_dependents contract
#[test]
fn test_query_dependents_contract() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "query_dependents_1".to_string(),
        name: "query_dependents".to_string(),
        params: json!({
            "symbol_id": "com.example.UserService.createUser",
            "include_indirect": true
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.command_id, "query_dependents_1");
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("symbol_id").is_some());
    assert!(data.get("dependents").is_some());
    assert!(data.get("direct_count").is_some());
    assert!(data.get("total_count").is_some());
}

/// Test get_symbol_info contract
#[test]
fn test_get_symbol_info_contract() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "symbol_info_1".to_string(),
        name: "get_symbol_info".to_string(),
        params: json!({
            "symbol_id": "com.example.UserService.createUser"
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.command_id, "symbol_info_1");
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("symbol").is_some());

    let symbol = data.get("symbol").unwrap();
    assert!(symbol.get("id").is_some());
    assert!(symbol.get("name").is_some());
    assert!(symbol.get("qualified_name").is_some());
    assert!(symbol.get("symbol_type").is_some());
    assert!(symbol.get("file_path").is_some());
    assert!(symbol.get("line_number").is_some());
    assert!(symbol.get("language").is_some());
    assert!(symbol.get("visibility").is_some());
    assert!(symbol.get("exported").is_some());
}

/// Test find_path_between_symbols contract
#[test]
fn test_find_path_between_symbols_contract() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "find_path_1".to_string(),
        name: "find_path_between_symbols".to_string(),
        params: json!({
            "from_symbol_id": "com.example.Controller.handleRequest",
            "to_symbol_id": "com.example.Database.saveUser",
            "max_depth": 10
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.command_id, "find_path_1");
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("from_symbol_id").is_some());
    assert!(data.get("to_symbol_id").is_some());
    assert!(data.get("path_found").is_some());
    assert!(data.get("path_length").is_some());

    if data.get("path_found").unwrap().as_bool().unwrap() {
        assert!(data.get("path").is_some());
    }
}

/// Test get_isolation_boundary contract
#[test]
fn test_get_isolation_boundary_contract() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "isolation_boundary_1".to_string(),
        name: "get_isolation_boundary".to_string(),
        params: json!({
            "symbol_id": "com.example.LegacyUtils.oldFunction"
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.command_id, "isolation_boundary_1");
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("symbol_id").is_some());
    assert!(data.get("isolation_distance").is_some());
    assert!(data.get("boundary_symbols").is_some());
    assert!(data.get("nearest_active_symbol").is_some());
}

/// Test query validation errors
#[test]
fn test_query_validation_errors() {
    let handler = QueryCommandHandler::new();

    // Test missing required parameter
    let command = IpcCommand {
        id: "error_test".to_string(),
        name: "query_dependencies".to_string(),
        params: json!({
            // Missing symbol_id
            "include_indirect": false
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, "error");
    assert!(response.error.is_some());
}

/// Test unknown query command
#[test]
fn test_unknown_query_command() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "unknown_test".to_string(),
        name: "unknown_query_command".to_string(),
        params: json!({}),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, "error");
    assert!(response.error.unwrap().contains("Unknown query command"));
}

/// Test pagination for dependency queries
#[test]
fn test_dependency_query_pagination() {
    let handler = QueryCommandHandler::new();

    let command = IpcCommand {
        id: "pagination_test".to_string(),
        name: "query_dependencies".to_string(),
        params: json!({
            "symbol_id": "com.example.BigService.processData",
            "include_indirect": true,
            "page": 1,
            "page_size": 50
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = handler.execute(command);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status, "success");

    let data = response.data.unwrap();
    assert!(data.get("dependencies").is_some());
    assert!(data.get("pagination").is_some());

    let pagination = data.get("pagination").unwrap();
    assert!(pagination.get("page").is_some());
    assert!(pagination.get("page_size").is_some());
    assert!(pagination.get("total_items").is_some());
    assert!(pagination.get("total_pages").is_some());
}