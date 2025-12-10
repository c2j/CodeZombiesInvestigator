//! Core library integration tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CziConfig, ConfigManager, TreeSitterManager, SupportedLanguage};

    #[test]
    fn test_error_handling() {
        // Test CziError creation
        let config_error = crate::CziError::config("Test config error");
        assert!(config_error.to_string().contains("Configuration error"));

        // Test From conversions
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test file");
        let czi_error: crate::CziError = io_error.into();
        assert!(matches!(czi_error, crate::CziError::Io { .. }));
    }

    #[test]
    fn test_configuration_management() {
        // Test default config creation
        let config = CziConfig::default();
        assert_eq!(config.app.log_level, "info");
        assert_eq!(config.app.max_concurrent_operations, 4);

        // Test config validation
        let mut invalid_config = CziConfig::default();
        invalid_config.app.max_concurrent_operations = 0;
        let manager = ConfigManager::new("test_config.json");
        assert!(manager.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_language_detection() {
        // Test file extension detection
        assert_eq!(SupportedLanguage::from_path("test.java"), Some(SupportedLanguage::Java));
        assert_eq!(SupportedLanguage::from_path("test.js"), Some(SupportedLanguage::JavaScript));
        assert_eq!(SupportedLanguage::from_path("test.py"), Some(SupportedLanguage::Python));
        assert_eq!(SupportedLanguage::from_path("test.sh"), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_path("test.unknown"), None);

        // Test file name detection
        assert_eq!(SupportedLanguage::from_file_name("Dockerfile"), Some(SupportedLanguage::Shell));
        assert_eq!(SupportedLanguage::from_file_name("unknown"), None);
    }

    #[test]
    fn test_tree_sitter_manager() {
        // Test Tree-sitter manager creation and language support
        let manager = TreeSitterManager::new();
        let supported_languages = manager.supported_languages();

        assert!(supported_languages.contains(&SupportedLanguage::Java));
        assert!(supported_languages.contains(&SupportedLanguage::JavaScript));
        assert!(supported_languages.contains(&SupportedLanguage::Python));
        assert!(supported_languages.contains(&SupportedLanguage::Shell));

        // Test file support checking
        assert!(manager.is_file_supported("test.java"));
        assert!(manager.is_file_supported("test.js"));
        assert!(manager.is_file_supported("test.py"));
        assert!(manager.is_file_supported("test.sh"));
        assert!(!manager.is_file_supported("test.unknown"));
    }

    #[test]
    fn test_tree_sitter_parsing() {
        // Test basic parsing functionality
        let manager = TreeSitterManager::new();

        // Test Java parsing
        let java_code = r#"
public class Test {
    public void method() {
        System.out.println("Hello");
    }
}
"#;
        let java_tree = manager.parse(java_code, SupportedLanguage::Java).unwrap();
        assert!(!java_tree.root_node().has_error());

        // Test JavaScript parsing
        let js_code = r#"
function test() {
    console.log("Hello");
    return 42;
}
"#;
        let js_tree = manager.parse(js_code, SupportedLanguage::JavaScript).unwrap();
        assert!(!js_tree.root_node().has_error());

        // Test Python parsing
        let py_code = r#"
def test():
    print("Hello")
    return 42
"#;
        let py_tree = manager.parse(py_code, SupportedLanguage::Python).unwrap();
        assert!(!py_tree.root_node().has_error());

        // Test Shell parsing
        let sh_code = r#"
#!/bin/bash
echo "Hello"
function test() {
    echo "Function"
}
test
"#;
        let sh_tree = manager.parse(sh_code, SupportedLanguage::Shell).unwrap();
        assert!(!sh_tree.root_node().has_error());
    }

    #[test]
    fn test_language_queries() {
        // Test that query strings are properly defined
        use crate::parser::LanguageQueries;

        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Java).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::JavaScript).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Python).is_empty());
        assert!(!LanguageQueries::function_definitions(SupportedLanguage::Shell).is_empty());

        assert!(!LanguageQueries::imports(SupportedLanguage::Java).is_empty());
        assert!(!LanguageQueries::imports(SupportedLanguage::JavaScript).is_empty());
        assert!(!LanguageQueries::imports(SupportedLanguage::Python).is_empty());
        assert!(!LanguageQueries::imports(SupportedLanguage::Shell).is_empty());
    }

    #[test]
    fn test_config_serialization() {
        use crate::{RepositoryConfiguration, config::AuthType, config::AuthConfig};

        // Test repository configuration serialization
        let repo_config = RepositoryConfiguration {
            id: "test_repo".to_string(),
            name: "Test Repository".to_string(),
            url: "https://github.com/test/repo.git".to_string(),
            local_path: Some("/tmp/test".into()),
            branch: "main".to_string(),
            auth_type: AuthType::Token,
            auth_config: AuthConfig::Token {
                token: "test_token".to_string(),
                username: Some("test_user".to_string()),
            },
            last_sync: Some(chrono::Utc::now()),
            status: crate::config::RepositoryStatus::Active,
        };

        // Test JSON serialization
        let json_str = serde_json::to_string(&repo_config).unwrap();
        let parsed: RepositoryConfiguration = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.id, repo_config.id);
        assert_eq!(parsed.name, repo_config.name);

        // Test YAML serialization
        let yaml_str = serde_yaml::to_string(&repo_config).unwrap();
        let parsed_yaml: RepositoryConfiguration = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(parsed_yaml.id, repo_config.id);
        assert_eq!(parsed_yaml.name, repo_config.name);
    }

    #[test]
    fn test_analysis_types() {
        use crate::{AnalysisResult, ZombieCodeItem, CodeSymbol, SymbolType, DependencyEdge, EdgeType};

        // Test AnalysisResult
        let analysis_result = AnalysisResult {
            id: "analysis_1".to_string(),
            total_symbols: 1000,
            zombie_symbols: 150,
        };
        assert_eq!(analysis_result.id, "analysis_1");
        assert_eq!(analysis_result.summary.total_symbols, 1000);
        assert_eq!(analysis_result.summary.zombie_symbols, 150);

        // Test ZombieCodeItem
        let zombie_item = ZombieCodeItem {
            id: "zombie_1".to_string(),
            symbol_id: "symbol_1".to_string(),
            removal_confidence: 0.9,
        };
        assert_eq!(zombie_item.id, "zombie_1");
        assert_eq!(zombie_item.removal_confidence, 0.9);

        // Test CodeSymbol
        let code_symbol = CodeSymbol {
            id: "symbol_1".to_string(),
            name: "testFunction".to_string(),
            symbol_type: SymbolType::Function,
            file_path: "test.java".to_string(),
            line_number: 10,
        };
        assert_eq!(code_symbol.name, "testFunction");
        assert_eq!(code_symbol.symbol_type, SymbolType::Function);

        // Test DependencyEdge
        let dep_edge = DependencyEdge {
            id: "edge_1".to_string(),
            source_symbol_id: "symbol_1".to_string(),
            target_symbol_id: "symbol_2".to_string(),
            edge_type: EdgeType::Calls,
        };
        assert_eq!(dep_edge.edge_type, EdgeType::Calls);
    }
}