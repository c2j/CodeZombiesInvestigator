//! Active root node entity model for zombie code analysis
//!
//! Active root nodes represent entry points in the codebase from which
//! reachability analysis is performed. These are the starting points
//! for determining which code is "alive" vs "zombie" (unreachable).

use crate::{CziError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tracing::{debug, trace, instrument};
use uuid::Uuid;

/// Active root node representing an entry point for analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveRootNode {
    /// Unique identifier for this root node
    pub id: String,

    /// Repository this node belongs to
    pub repository_id: String,

    /// Type of root node (controller, main, scheduler, etc.)
    pub node_type: RootNodeType,

    /// Fully qualified symbol path (e.g., "com.example.Controller.main")
    pub symbol_path: String,

    /// File path where this symbol is defined
    pub file_path: String,

    /// Line number where symbol is defined
    pub line_number: Option<u32>,

    /// Programming language of this symbol
    pub language: String,

    /// Metadata about this root node
    pub metadata: HashMap<String, String>,

    /// Whether this node is currently active for analysis
    pub active: bool,

    /// When this root node was discovered/created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this root node was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Types of root nodes that can serve as analysis entry points
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RootNodeType {
    /// HTTP/API endpoint controllers
    Controller,

    /// Scheduled jobs and cron tasks
    Scheduler,

    /// Message queue listeners and event handlers
    Listener,

    /// Application main entry points
    Main,

    /// CLI command entry points
    CommandLine,

    /// Test entry points
    Test,

    /// Library/public API entry points
    Library,

    /// Other custom entry points
    Custom(String),
}

impl Hash for RootNodeType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RootNodeType::Controller => 0.hash(state),
            RootNodeType::Scheduler => 1.hash(state),
            RootNodeType::Listener => 2.hash(state),
            RootNodeType::Main => 3.hash(state),
            RootNodeType::CommandLine => 4.hash(state),
            RootNodeType::Test => 5.hash(state),
            RootNodeType::Library => 6.hash(state),
            RootNodeType::Custom(s) => {
                7.hash(state);
                s.hash(state);
            }
        }
    }
}

/// Active root node detector for finding potential entry points
pub struct RootNodeDetector {
    /// Patterns for detecting different types of root nodes
    patterns: HashMap<RootNodeType, Vec<String>>,

    /// Language-specific patterns
    language_patterns: HashMap<String, HashMap<RootNodeType, Vec<String>>>,
}

impl RootNodeDetector {
    /// Create a new root node detector with default patterns
    pub fn new() -> Self {
        let patterns = HashMap::new();
        let mut language_patterns = HashMap::new();

        // Java patterns
        let mut java_patterns = HashMap::new();
        java_patterns.insert(RootNodeType::Controller, vec![
            "@Controller".to_string(),
            "@RestController".to_string(),
            "@RequestMapping".to_string(),
            "@GetMapping".to_string(),
            "@PostMapping".to_string(),
            "@PutMapping".to_string(),
            "@DeleteMapping".to_string(),
        ]);
        java_patterns.insert(RootNodeType::Scheduler, vec![
            "@Scheduled".to_string(),
            "TimerTask".to_string(),
            "Job".to_string(),
        ]);
        java_patterns.insert(RootNodeType::Main, vec![
            "public static void main".to_string(),
        ]);
        java_patterns.insert(RootNodeType::Test, vec![
            "@Test".to_string(),
            "@BeforeEach".to_string(),
            "@AfterEach".to_string(),
        ]);
        language_patterns.insert("java".to_string(), java_patterns);

        // Python patterns
        let mut python_patterns = HashMap::new();
        python_patterns.insert(RootNodeType::Controller, vec![
            "@app.route".to_string(),
            "@bp.route".to_string(),
            "def test_".to_string(),
        ]);
        python_patterns.insert(RootNodeType::Main, vec![
            "if __name__ == \"__main__\"".to_string(),
            "def main()".to_string(),
        ]);
        python_patterns.insert(RootNodeType::Test, vec![
            "def test_".to_string(),
            "class Test".to_string(),
        ]);
        language_patterns.insert("python".to_string(), python_patterns);

        // JavaScript patterns
        let mut js_patterns = HashMap::new();
        js_patterns.insert(RootNodeType::Controller, vec![
            "app.get".to_string(),
            "app.post".to_string(),
            "router.get".to_string(),
            "router.post".to_string(),
            "express()".to_string(),
        ]);
        js_patterns.insert(RootNodeType::Main, vec![
            "function main".to_string(),
            "const main =".to_string(),
        ]);
        language_patterns.insert("javascript".to_string(), js_patterns);

        Self {
            patterns,
            language_patterns,
        }
    }

    /// Detect root nodes in source code content
    #[instrument(skip(self, content))]
    pub fn detect_root_nodes(
        &self,
        content: &str,
        file_path: &str,
        language: &str,
        repository_id: &str,
    ) -> Result<Vec<ActiveRootNode>> {
        let mut root_nodes = Vec::new();
        let now = chrono::Utc::now();

        if let Some(lang_patterns) = self.language_patterns.get(language) {
            for (node_type, patterns) in lang_patterns {
                for pattern in patterns {
                    if content.contains(pattern) {
                        let root_node = ActiveRootNode {
                            id: Uuid::new_v4().to_string(),
                            repository_id: repository_id.to_string(),
                            node_type: node_type.clone(),
                            symbol_path: format!("{}.{}",
                                file_path.replace('/', ".").replace(".java", "").replace(".py", "").replace(".js", ""),
                                "entry"
                            ),
                            file_path: file_path.to_string(),
                            line_number: self.find_pattern_line(content, pattern),
                            language: language.to_string(),
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("pattern".to_string(), pattern.clone());
                                meta.insert("file_size".to_string(), content.len().to_string());
                                meta
                            },
                            active: true,
                            created_at: now,
                            updated_at: now,
                        };

                        root_nodes.push(root_node);
                        trace!("Detected {:?} root node in {}", node_type, file_path);
                    }
                }
            }
        }

        debug!("Detected {} root nodes in {}", root_nodes.len(), file_path);
        Ok(root_nodes)
    }

    /// Find the line number where a pattern occurs
    fn find_pattern_line(&self, content: &str, pattern: &str) -> Option<u32> {
        for (line_num, line) in content.lines().enumerate() {
            if line.contains(pattern) {
                return Some(line_num as u32 + 1);
            }
        }
        None
    }

    /// Add custom pattern for root node detection
    pub fn add_pattern(
        &mut self,
        language: &str,
        node_type: RootNodeType,
        pattern: String,
    ) {
        let lang_patterns = self.language_patterns
            .entry(language.to_string())
            .or_insert_with(HashMap::new);

        let patterns = lang_patterns
            .entry(node_type)
            .or_insert_with(Vec::new);

        patterns.push(pattern);
    }
}

impl Default for RootNodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveRootNode {
    /// Create a new active root node
    pub fn new(
        repository_id: String,
        node_type: RootNodeType,
        symbol_path: String,
        file_path: String,
        language: String,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            repository_id,
            node_type,
            symbol_path,
            file_path,
            line_number: None,
            language,
            metadata: HashMap::new(),
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the root node
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(CziError::validation("Root node ID cannot be empty"));
        }

        if self.repository_id.is_empty() {
            return Err(CziError::validation("Repository ID cannot be empty"));
        }

        if self.symbol_path.is_empty() {
            return Err(CziError::validation("Symbol path cannot be empty"));
        }

        if self.file_path.is_empty() {
            return Err(CziError::validation("File path cannot be empty"));
        }

        if self.language.is_empty() {
            return Err(CziError::validation("Language cannot be empty"));
        }

        Ok(())
    }

    /// Add metadata to this root node
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set line number
    pub fn set_line_number(&mut self, line_number: u32) {
        self.line_number = Some(line_number);
        self.updated_at = chrono::Utc::now();
    }

    /// Activate this root node for analysis
    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = chrono::Utc::now();
    }

    /// Deactivate this root node from analysis
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = chrono::Utc::now();
    }

    /// Check if this root node is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get a display name for this root node
    pub fn display_name(&self) -> String {
        match &self.node_type {
            RootNodeType::Custom(name) => format!("{}: {}", name, self.symbol_path),
            _ => format!("{:?}: {}", self.node_type, self.symbol_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_root_node_creation() {
        let node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Main,
            "com.example.Main.main".to_string(),
            "src/main/java/Main.java".to_string(),
            "java".to_string(),
        );

        assert_eq!(node.repository_id, "repo_1");
        assert_eq!(node.node_type, RootNodeType::Main);
        assert_eq!(node.symbol_path, "com.example.Main.main");
        assert_eq!(node.file_path, "src/main/java/Main.java");
        assert_eq!(node.language, "java");
        assert!(node.is_active());
        assert!(node.validate().is_ok());
    }

    #[test]
    fn test_active_root_node_validation() {
        let mut node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Main,
            "com.example.Main.main".to_string(),
            "src/main/java/Main.java".to_string(),
            "java".to_string(),
        );

        // Valid node should pass validation
        assert!(node.validate().is_ok());

        // Invalid node (empty repository_id) should fail validation
        node.repository_id = String::new();
        assert!(node.validate().is_err());
    }

    #[test]
    fn test_root_node_detector_java() {
        let detector = RootNodeDetector::new();
        let java_code = r#"
@Controller
public class UserController {
    @GetMapping("/users")
    public String getUsers() {
        return "users";
    }
}

public class Main {
    public static void main(String[] args) {
        System.out.println("Hello World");
    }
}
"#;

        let root_nodes = detector.detect_root_nodes(
            java_code,
            "src/main/java/UserController.java",
            "java",
            "test_repo"
        ).unwrap();

        assert!(!root_nodes.is_empty());

        // Should detect both Controller and Main root nodes
        let controller_found = root_nodes.iter().any(|n| n.node_type == RootNodeType::Controller);
        let main_found = root_nodes.iter().any(|n| n.node_type == RootNodeType::Main);

        assert!(controller_found);
        assert!(main_found);
    }

    #[test]
    fn test_root_node_detector_python() {
        let detector = RootNodeDetector::new();
        let python_code = r#"
from flask import Flask

app = Flask(__name__)

@app.route('/hello')
def hello():
    return "Hello World!"

def main():
    print("Hello from main")

if __name__ == "__main__":
    main()
"#;

        let root_nodes = detector.detect_root_nodes(
            python_code,
            "app.py",
            "python",
            "test_repo"
        ).unwrap();

        assert!(!root_nodes.is_empty());

        // Should detect both Controller and Main root nodes
        let controller_found = root_nodes.iter().any(|n| n.node_type == RootNodeType::Controller);
        let main_found = root_nodes.iter().any(|n| n.node_type == RootNodeType::Main);

        assert!(controller_found);
        assert!(main_found);
    }

    #[test]
    fn test_root_node_metadata() {
        let mut node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Main,
            "com.example.Main.main".to_string(),
            "src/main/java/Main.java".to_string(),
            "java".to_string(),
        );

        node.add_metadata("author".to_string(), "John Doe".to_string());
        node.add_metadata("version".to_string(), "1.0".to_string());

        assert_eq!(node.get_metadata("author"), Some(&"John Doe".to_string()));
        assert_eq!(node.get_metadata("version"), Some(&"1.0".to_string()));
        assert_eq!(node.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_root_node_activation() {
        let mut node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Main,
            "com.example.Main.main".to_string(),
            "src/main/java/Main.java".to_string(),
            "java".to_string(),
        );

        assert!(node.is_active());

        node.deactivate();
        assert!(!node.is_active());

        node.activate();
        assert!(node.is_active());
    }

    #[test]
    fn test_root_node_display_name() {
        let node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Controller,
            "com.example.UserController.getUsers".to_string(),
            "src/main/java/UserController.java".to_string(),
            "java".to_string(),
        );

        assert_eq!(node.display_name(), "Controller: com.example.UserController.getUsers");

        let custom_node = ActiveRootNode::new(
            "repo_1".to_string(),
            RootNodeType::Custom("API Endpoint".to_string()),
            "com.example.Api.handleRequest".to_string(),
            "src/main/java/Api.java".to_string(),
            "java".to_string(),
        );

        assert_eq!(custom_node.display_name(), "API Endpoint: com.example.Api.handleRequest");
    }
}