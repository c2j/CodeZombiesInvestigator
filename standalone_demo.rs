//! Standalone demonstration of the CodeZombiesInvestigator concept
//! This example shows how zombie code analysis works without dependencies.

use std::collections::{HashMap, HashSet};

// Simplified types for demonstration
#[derive(Debug, Clone)]
pub struct CodeSymbol {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub exported: bool,
}

impl CodeSymbol {
    pub fn new(id: String, name: String, file_path: String, exported: bool) -> Self {
        Self {
            id,
            name,
            file_path,
            exported,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ZombieType {
    DeadCode,      // Completely isolated
    Orphaned,      // Has dependencies but no references
    Unreachable,   // Not reachable from entry points
}

impl std::fmt::Display for ZombieType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZombieType::DeadCode => write!(f, "Dead Code"),
            ZombieType::Orphaned => write!(f, "Orphaned"),
            ZombieType::Unreachable => write!(f, "Unreachable"),
        }
    }
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
        self.dependencies.entry(from.clone()).or_insert_with(Vec::new).push(to.clone());
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

        // If no entry points, all code is considered zombie
        if entry_points.is_empty() {
            for symbol in &self.symbols {
                zombie_items.push(ZombieCodeItem {
                    symbol: symbol.clone(),
                    zombie_type: ZombieType::Unreachable,
                    confidence: 0.5,
                    isolation_distance: 0,
                });
            }
            return zombie_items;
        }

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
        let mut visited = HashSet::new();

        for entry_point in entry_points {
            self.dfs_reachable(entry_point, &mut reachable, &mut visited);
        }

        reachable
    }

    /// Depth-first search to find reachable symbols
    fn dfs_reachable(&self, symbol_id: &str, reachable: &mut Vec<String>, visited: &mut HashSet<String>) {
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
        let empty_deps = vec![];
        let dependencies = self.dependencies.get(&symbol.id).unwrap_or(&empty_deps);
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
    fn calculate_confidence(&self, symbol: &CodeSymbol, _reachable: &[String]) -> f64 {
        let mut confidence: f64 = 1.0;

        // Reduce confidence for symbols with complex names (might be important)
        if symbol.name.len() > 20 || symbol.name.contains("_") {
            confidence -= 0.1;
        }

        // Reduce confidence for symbols in test files
        if symbol.file_path.contains("test") || symbol.file_path.contains("spec") {
            confidence -= 0.3;
        }

        // Increase confidence for completely isolated symbols
        let empty_deps = vec![];
        let dependencies = self.dependencies.get(&symbol.id).unwrap_or(&empty_deps);
        let has_references = self.symbols.iter().any(|s| {
            let empty_deps2 = vec![];
            self.dependencies.get(&s.id).unwrap_or(&empty_deps2).contains(&symbol.id)
        });

        if dependencies.is_empty() && !has_references {
            confidence += 0.2;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Calculate isolation distance (how far from reachable code)
    fn calculate_isolation_distance(&self, symbol: &CodeSymbol, _reachable: &[String]) -> usize {
        // Simple heuristic: count dependencies as isolation distance
        self.dependencies.get(&symbol.id).unwrap_or(&vec![]).len()
    }
}

fn main() {
    println!("🧟 CodeZombiesInvestigator - Standalone Demo");
    println!("=============================================");

    let mut analyzer = SimpleZombieAnalyzer::new();

    // Create a sample codebase with some symbols
    println!("\n📁 Creating sample codebase...");

    // Entry point symbols
    analyzer.add_symbol(CodeSymbol::new(
        "main".to_string(),
        "main".to_string(),
        "src/main.rs".to_string(),
        true,
    ));

    analyzer.add_symbol(CodeSymbol::new(
        "app_start".to_string(),
        "app_start".to_string(),
        "src/app.rs".to_string(),
        true,
    ));

    // Used symbols
    analyzer.add_symbol(CodeSymbol::new(
        "user_service".to_string(),
        "UserService".to_string(),
        "src/services/user.rs".to_string(),
        false,
    ));

    analyzer.add_symbol(CodeSymbol::new(
        "database".to_string(),
        "Database".to_string(),
        "src/db.rs".to_string(),
        false,
    ));

    // Zombie code symbols
    analyzer.add_symbol(CodeSymbol::new(
        "old_auth".to_string(),
        "OldAuthenticationSystem".to_string(),
        "src/auth/legacy.rs".to_string(),
        false,
    ));

    analyzer.add_symbol(CodeSymbol::new(
        "unused_util".to_string(),
        "unused_utility_function".to_string(),
        "src/utils/unused.rs".to_string(),
        false,
    ));

    analyzer.add_symbol(CodeSymbol::new(
        "test_helper".to_string(),
        "test_helper_only".to_string(),
        "tests/helpers.rs".to_string(),
        false,
    ));

    analyzer.add_symbol(CodeSymbol::new(
        "deprecated_feature".to_string(),
        "deprecated_feature_handler".to_string(),
        "src/features/old.rs".to_string(),
        false,
    ));

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

    // Deprecated feature depends on old auth (circular zombie dependency)
    analyzer.add_dependency("deprecated_feature".to_string(), "old_auth".to_string());

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
            println!("   🧟 Type: {}", item.zombie_type);
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

    println!("\n📈 Analysis Summary:");
    println!("- Total symbols analyzed: {}", analyzer.symbols.len());
    println!("- Zombie code identified: {}", zombie_items.len());
    println!("- Potential cleanup candidates: {}",
             zombie_items.iter().filter(|i| i.confidence >= 0.7).count());

    // Show breakdown by type
    let mut type_counts = HashMap::new();
    for item in &zombie_items {
        *type_counts.entry(item.zombie_type.clone()).or_insert(0) += 1;
    }

    println!("\n🧟‍♂️ Zombie Code Breakdown:");
    for (zombie_type, count) in type_counts {
        println!("- {}: {}", zombie_type, count);
    }

    println!("\n🎯 Core Concepts Demonstrated:");
    println!("   1. ✅ Build dependency graph of code symbols");
    println!("   2. ✅ Find reachable symbols from entry points");
    println!("   3. ✅ Classify unreachable symbols as zombie code");
    println!("   4. ✅ Calculate confidence for safe removal");
    println!("   5. ✅ Categorize zombie types (DeadCode, Orphaned, Unreachable)");

    println!("\n🏗️  Architecture Benefits:");
    println!("   • Helps identify unused code for cleanup");
    println!("   • Reduces technical debt and maintenance burden");
    println!("   • Improves codebase understanding and navigation");
    println!("   • Supports refactoring decisions with data");

    println!("\n🚀 Next Steps for Full Implementation:");
    println!("   • Parse real code files (Java, Python, JavaScript)");
    println!("   • Build complex dependency graphs with static analysis");
    println!("   • Detect entry points (main methods, controllers, tests)");
    println!("   • Generate detailed reports and visualizations");
    println!("   • Integrate with CI/CD pipelines for automated cleanup");

    println!("\n✨ Demo completed successfully! 🎉");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_zombie_analysis() {
        let mut analyzer = SimpleZombieAnalyzer::new();

        // Add entry point
        analyzer.add_symbol(CodeSymbol::new(
            "main".to_string(),
            "main".to_string(),
            "main.rs".to_string(),
            true,
        ));

        // Add used symbol
        analyzer.add_symbol(CodeSymbol::new(
            "helper".to_string(),
            "helper".to_string(),
            "helper.rs".to_string(),
            false,
        ));

        // Add zombie symbol
        analyzer.add_symbol(CodeSymbol::new(
            "unused".to_string(),
            "unused_function".to_string(),
            "unused.rs".to_string(),
            false,
        ));

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

        // Add entry point
        analyzer.add_symbol(CodeSymbol::new(
            "main".to_string(),
            "main".to_string(),
            "main.rs".to_string(),
            true,
        ));

        // Add zombie symbol that has dependencies
        analyzer.add_symbol(CodeSymbol::new(
            "orphaned".to_string(),
            "orphaned_function".to_string(),
            "orphaned.rs".to_string(),
            false,
        ));

        analyzer.add_symbol(CodeSymbol::new(
            "dependency".to_string(),
            "dependency_func".to_string(),
            "dep.rs".to_string(),
            false,
        ));

        // Add dependency making it orphaned (has deps but no references)
        analyzer.add_dependency("orphaned".to_string(), "dependency".to_string());

        let zombie_items = analyzer.analyze_zombie_code();
        assert_eq!(zombie_items.len(), 2);

        // Check that we have one Orphaned and one Unreachable symbol (order may vary)
        let orphaned_count = zombie_items.iter()
            .filter(|item| item.zombie_type == ZombieType::Orphaned)
            .count();
        let unreachable_count = zombie_items.iter()
            .filter(|item| item.zombie_type == ZombieType::Unreachable)
            .count();

        assert_eq!(orphaned_count, 1);
        assert_eq!(unreachable_count, 1);
    }

    #[test]
    fn test_no_entry_points() {
        let mut analyzer = SimpleZombieAnalyzer::new();

        analyzer.add_symbol(CodeSymbol::new(
            "func1".to_string(),
            "function1".to_string(),
            "file1.rs".to_string(),
            false,
        ));

        analyzer.add_symbol(CodeSymbol::new(
            "func2".to_string(),
            "function2".to_string(),
            "file2.rs".to_string(),
            false,
        ));

        let zombie_items = analyzer.analyze_zombie_code();
        // All symbols should be considered unreachable when no entry points exist
        assert_eq!(zombie_items.len(), 2);
        for item in &zombie_items {
            assert_eq!(item.zombie_type, ZombieType::Unreachable);
        }
    }
}