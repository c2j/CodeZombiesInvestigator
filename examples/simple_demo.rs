//! Simple demonstration of the CodeZombiesInvestigator concept
//! This example shows how the zombie code analysis would work without requiring
//! the full complex infrastructure to be compiled.

use std::collections::HashMap;

// Simplified types for demonstration
#[derive(Debug, Clone)]
pub struct CodeSymbol {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub exported: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZombieType {
    DeadCode,      // Completely isolated
    Orphaned,      // Has dependencies but no references
    Unreachable,   // Not reachable from entry points
}

#[derive(Debug, Clone)]
pub struct ZombieCodeItem {
    pub symbol: CodeSymbol,
    pub zombie_type: ZombieType,
    pub confidence: f64,
    pub isolation_distance: usize,
}

/// Simple zombie code analyzer for demonstration
pub struct SimpleZombieAnalyzer {
    symbols: Vec<CodeSymbol>,
    dependencies: HashMap<String, Vec<String>>, // symbol_id -> dependencies
}

impl SimpleZombieAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Add a code symbol to the analysis
    pub fn add_symbol(&mut self, symbol: CodeSymbol) {
        self.symbols.push(symbol.clone());
        self.dependencies.entry(symbol.id).or_insert_with(Vec::new);
    }

    /// Add a dependency relationship: `from` depends on `to`
    pub fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies.entry(from).or_insert_with(Vec::new).push(to);
        self.dependencies.entry(to).or_insert_with(Vec::new);
    }

    /// Analyze symbols to find zombie code
    pub fn analyze_zombie_code(&self) -> Vec<ZombieCodeItem> {
        let mut zombie_items = Vec::new();

        // Find entry points (exported symbols)
        let entry_points: Vec<String> = self.symbols
            .iter()
            .filter(|s| s.exported)
            .map(|s| s.id.clone())
            .collect();

        // Find reachable symbols from entry points
        let reachable = self.find_reachable_symbols(&entry_points);

        // Classify unreachable symbols as zombie code
        for symbol in &self.symbols {
            if !reachable.contains(&symbol.id) {
                let zombie_type = self.classify_zombie_type(symbol);
                let confidence = self.calculate_confidence(symbol, &reachable);
                let isolation_distance = self.calculate_isolation_distance(symbol, &reachable);

                zombie_items.push(ZombieCodeItem {
                    symbol: symbol.clone(),
                    zombie_type,
                    confidence,
                    isolation_distance,
                });
            }
        }

        // Sort by confidence (highest first)
        zombie_items.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        zombie_items
    }

    /// Find all symbols reachable from entry points using simple DFS
    fn find_reachable_symbols(&self, entry_points: &[String]) -> Vec<String> {
        let mut reachable = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for entry_point in entry_points {
            self.dfs_reachable(entry_point, &mut reachable, &mut visited);
        }

        reachable
    }

    /// Depth-first search to find reachable symbols
    fn dfs_reachable(&self, symbol_id: &str, reachable: &mut Vec<String>, visited: &mut std::collections::HashSet<String>) {
        if visited.contains(symbol_id) {
            return;
        }

        visited.insert(symbol_id.to_string());
        reachable.push(symbol_id.to_string());

        if let Some(dependencies) = self.dependencies.get(symbol_id) {
            for dep in dependencies {
                self.dfs_reachable(dep, reachable, visited);
            }
        }
    }

    /// Classify the type of zombie code
    fn classify_zombie_type(&self, symbol: &CodeSymbol) -> ZombieType {
        let dependencies = self.dependencies.get(&symbol.id).unwrap_or(&vec![]);
        let has_dependencies = !dependencies.is_empty();

        // Check if any other symbols depend on this one
        let has_references = self.symbols.iter().any(|s| {
            self.dependencies.get(&s.id).unwrap_or(&vec![]).contains(&symbol.id)
        });

        match (has_dependencies, has_references) {
            (false, false) => ZombieType::DeadCode,      // Completely isolated
            (true, false) => ZombieType::Orphaned,       // Has dependencies but no references
            _ => ZombieType::Unreachable,               // Default case
        }
    }

    /// Calculate confidence score for zombie classification (0.0 to 1.0)
    fn calculate_confidence(&self, symbol: &CodeSymbol, reachable: &[String]) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for symbols with complex names (might be important)
        if symbol.name.len() > 20 || symbol.name.contains("_") {
            confidence -= 0.1;
        }

        // Reduce confidence for symbols in test files
        if symbol.file_path.contains("test") || symbol.file_path.contains("spec") {
            confidence -= 0.3;
        }

        // Increase confidence for completely isolated symbols
        let dependencies = self.dependencies.get(&symbol.id).unwrap_or(&vec![]);
        if dependencies.is_empty() && !self.symbols.iter().any(|s| {
            self.dependencies.get(&s.id).unwrap_or(&vec![]).contains(&symbol.id)
        }) {
            confidence += 0.2;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Calculate isolation distance (how far from reachable code)
    fn calculate_isolation_distance(&self, symbol: &CodeSymbol, reachable: &[String]) -> usize {
        // Simple heuristic: count dependencies as isolation distance
        self.dependencies.get(&symbol.id).unwrap_or(&vec![]).len()
    }
}

fn main() {
    println!("🧟 CodeZombiesInvestigator - Simple Demo");
    println!("========================================");

    let mut analyzer = SimpleZombieAnalyzer::new();

    // Create a sample codebase with some symbols
    println!("\n📁 Creating sample codebase...");

    // Entry point symbols
    analyzer.add_symbol(CodeSymbol {
        id: "main".to_string(),
        name: "main".to_string(),
        file_path: "src/main.rs".to_string(),
        exported: true,
    });

    analyzer.add_symbol(CodeSymbol {
        id: "app_start".to_string(),
        name: "app_start".to_string(),
        file_path: "src/app.rs".to_string(),
        exported: true,
    });

    // Used symbols
    analyzer.add_symbol(CodeSymbol {
        id: "user_service".to_string(),
        name: "UserService".to_string(),
        file_path: "src/services/user.rs".to_string(),
        exported: false,
    });

    analyzer.add_symbol(CodeSymbol {
        id: "database".to_string(),
        name: "Database".to_string(),
        file_path: "src/db.rs".to_string(),
        exported: false,
    });

    // Zombie code symbols
    analyzer.add_symbol(CodeSymbol {
        id: "old_auth".to_string(),
        name: "OldAuthenticationSystem".to_string(),
        file_path: "src/auth/legacy.rs".to_string(),
        exported: false,
    });

    analyzer.add_symbol(CodeSymbol {
        id: "unused_util".to_string(),
        name: "unused_utility_function".to_string(),
        file_path: "src/utils/unused.rs".to_string(),
        exported: false,
    });

    analyzer.add_symbol(CodeSymbol {
        id: "test_helper".to_string(),
        name: "test_helper_only".to_string(),
        file_path: "tests/helpers.rs".to_string(),
        exported: false,
    });

    // Add dependencies
    println!("📊 Adding dependency relationships...");

    // Main depends on app_start
    analyzer.add_dependency("main".to_string(), "app_start".to_string());

    // app_start depends on user_service
    analyzer.add_dependency("app_start".to_string(), "user_service".to_string());

    // user_service depends on database
    analyzer.add_dependency("user_service".to_string(), "database".to_string());

    // Old auth system has internal dependencies but is never called
    analyzer.add_dependency("old_auth".to_string(), "database".to_string());

    println!("🔍 Analyzing zombie code...");

    // Run the analysis
    let zombie_items = analyzer.analyze_zombie_code();

    println!("\n🧟 Found {} zombie code items:", zombie_items.len());
    println!("=====================================");

    if zombie_items.is_empty() {
        println!("✨ No zombie code found! Codebase is clean.");
    } else {
        for (index, item) in zombie_items.iter().enumerate() {
            let confidence_percent = (item.confidence * 100.0).round() as u32;
            let emoji = match item.zombie_type {
                ZombieType::DeadCode => "💀",
                ZombieType::Orphaned => "👻",
                ZombieType::Unreachable => "🗑️",
            };

            println!(
                "\n{} #{}: {} (Confidence: {}%)",
                emoji, index + 1, item.symbol.name, confidence_percent
            );
            println!("   📁 File: {}", item.symbol.file_path);
            println!("   🧟 Type: {:?}", item.zombie_type);
            println!("   📏 Isolation: {}", item.isolation_distance);

            if confidence_percent >= 80 {
                println!("   ✅ Safe to remove (high confidence)");
            } else if confidence_percent >= 60 {
                println!("   ⚠️  Review before removing (medium confidence)");
            } else {
                println!("   ❌ Manual review required (low confidence)");
            }
        }
    }

    println!("\n📈 Summary:");
    println!("- Total symbols analyzed: {}", analyzer.symbols.len());
    println!("- Zombie code identified: {}", zombie_items.len());
    println!("- Potential cleanup candidates: {}",
             zombie_items.iter().filter(|i| i.confidence >= 0.7).count());

    println!("\n🎯 This demonstrates the core concept of zombie code analysis:");
    println!("   1. Build dependency graph of code symbols");
    println!("   2. Find reachable symbols from entry points");
    println!("   3. Classify unreachable symbols as zombie code");
    println!("   4. Calculate confidence for safe removal");

    println!("\n✨ Demo completed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_zombie_analysis() {
        let mut analyzer = SimpleZombieAnalyzer::new();

        // Add entry point
        analyzer.add_symbol(CodeSymbol {
            id: "main".to_string(),
            name: "main".to_string(),
            file_path: "main.rs".to_string(),
            exported: true,
        });

        // Add used symbol
        analyzer.add_symbol(CodeSymbol {
            id: "helper".to_string(),
            name: "helper".to_string(),
            file_path: "helper.rs".to_string(),
            exported: false,
        });

        // Add zombie symbol
        analyzer.add_symbol(CodeSymbol {
            id: "unused".to_string(),
            name: "unused_function".to_string(),
            file_path: "unused.rs".to_string(),
            exported: false,
        });

        // Add dependency
        analyzer.add_dependency("main".to_string(), "helper".to_string());

        let zombie_items = analyzer.analyze_zombie_code();
        assert_eq!(zombie_items.len(), 1);
        assert_eq!(zombie_items[0].symbol.id, "unused");
        assert_eq!(zombie_items[0].zombie_type, ZombieType::DeadCode);
    }

    #[test]
    fn test_orphaned_classification() {
        let mut analyzer = SimpleZombieAnalyzer::new();

        // Add zombie symbol that has dependencies
        analyzer.add_symbol(CodeSymbol {
            id: "orphaned".to_string(),
            name: "orphaned_function".to_string(),
            file_path: "orphaned.rs".to_string(),
            exported: false,
        });

        analyzer.add_symbol(CodeSymbol {
            id: "dependency".to_string(),
            name: "dependency_func".to_string(),
            file_path: "dep.rs".to_string(),
            exported: false,
        });

        // Add dependency making it orphaned (has deps but no references)
        analyzer.add_dependency("orphaned".to_string(), "dependency".to_string());

        let zombie_items = analyzer.analyze_zombie_code();
        assert_eq!(zombie_items.len(), 2);
        assert_eq!(zombie_items[0].zombie_type, ZombieType::Orphaned);
    }
}