//! Demo program to verify CZI functionality

use czi_core::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("🚀 CodeZombiesInvestigator Demo - 功能验证");
    println!("==================================");

    // 1. 测试错误处理
    println!("\n1️⃣ 测试错误处理系统");
    test_error_handling();

    // 2. 测试配置管理
    println!("\n2️⃣ 测试配置管理");
    test_configuration_management()?;

    // 3. 测试Tree-sitter集成
    println!("\n3️⃣ 测试Tree-sitter解析器");
    test_tree_sitter_parsing()?;

    // 4. 测试语言检测
    println!("\n4️⃣ 测试语言检测");
    test_language_detection();

    // 5. 测试序列化
    println!("\n5️⃣ 测试序列化功能");
    test_serialization()?;

    println!("\n✅ 所有功能验证完成！");
    Ok(())
}

fn test_error_handling() {
    // 测试不同类型的错误创建
    let config_error = CziError::config("配置文件格式错误");
    println!("   配置错误: {}", config_error);

    let parse_error = CziError::parse("test.java", 10, "语法错误");
    println!("   解析错误: {}", parse_error);

    let analysis_error = CziError::analysis("分析过程中内存不足");
    println!("   分析错误: {}", analysis_error);

    // 测试错误分类
    println!("   错误类别: {}", config_error.category());
    println!("   是否可恢复: {}", config_error.is_recoverable());
}

fn test_configuration_management() -> Result<()> {
    // 创建配置管理器
    let config_path = PathBuf::from("demo_config.json");
    let manager = ConfigManager::new(&config_path);

    // 测试默认配置
    let config = CziConfig::default();
    println!("   默认日志级别: {}", config.app.log_level);
    println!("   最大并发操作: {}", config.app.max_concurrent_operations);
    println!("   支持的语言数: {}", config.analysis.languages.len());

    // 测试配置验证
    let mut invalid_config = CziConfig::default();
    invalid_config.app.max_concurrent_operations = 0;

    // 注意：validate_config是私有方法，这里直接测试其他功能
    println!("   配置管理器创建成功");

    // 保存配置
    manager.save_config(&config)?;
    println!("   配置已保存到: {:?}", config_path);

    // 加载配置
    let loaded_config = manager.load_config()?;
    println!("   配置加载成功，日志级别: {}", loaded_config.app.log_level);

    Ok(())
}

fn test_tree_sitter_parsing() -> Result<()> {
    let manager = TreeSitterManager::new()?;

    // 测试支持的语言
    let supported_languages = manager.supported_languages();
    println!("   支持的语言数量: {}", supported_languages.len());
    println!("   支持的文件扩展名: {:?}", manager.supported_extensions());

    // 测试Java解析
    let java_code = r#"
public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }

    public static void main(String[] args) {
        Calculator calc = new Calculator();
        System.out.println("5 + 3 = " + calc.add(5, 3));
    }
}
"#;

    let java_tree = manager.parse(java_code, SupportedLanguage::Java)?;
    println!("   Java解析成功:");
    println!("     - 根节点类型: {}", java_tree.root_node().kind());
    println!("     - 是否有错误: {}", java_tree.root_node().has_error());
    println!("     - 节点数量: {}", java_tree.root_node().child_count());

    // 测试JavaScript解析
    let js_code = r#"
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const result = fibonacci(10);
console.log(`Fibonacci(10) = ${result}`);
"#;

    let js_tree = manager.parse(js_code, SupportedLanguage::JavaScript)?;
    println!("   JavaScript解析成功:");
    println!("     - 根节点类型: {}", js_tree.root_node().kind());
    println!("     - 是否有错误: {}", js_tree.root_node().has_error());

    Ok(())
}

fn test_language_detection() {
    let test_files = vec![
        ("src/main.java", "Java"),
        ("app.js", "JavaScript"),
        ("utils.py", "Python"),
        ("deploy.sh", "Shell"),
        ("Dockerfile", "Shell"),
        ("unknown.xyz", "Unknown"),
    ];

    for (file, expected) in test_files {
        let path = std::path::Path::new(file);
        let detected = SupportedLanguage::from_path(path)
            .or_else(|| SupportedLanguage::from_file_name(
                path.file_name().and_then(|n| n.to_str()).unwrap_or("")
            ));

        let detected_name = detected.map(|l| l.name()).unwrap_or("Unknown");
        println!("   {} -> {} (预期: {})", file, detected_name, expected);
    }
}

fn test_serialization() -> Result<()> {
    // 测试RepositoryConfiguration序列化
    use crate::{RepositoryConfiguration, AuthType, AuthConfig, RepositoryStatus};

    let repo_config = RepositoryConfiguration {
        id: "demo_repo".to_string(),
        name: "演示仓库".to_string(),
        url: "https://github.com/demo/repo.git".to_string(),
        local_path: PathBuf::from("./cache/demo_repo"),
        branch: "main".to_string(),
        auth_type: AuthType::Token,
        auth_config: Some(AuthConfig::Token {
            token: "demo_token".to_string(),
            username: Some("demo_user".to_string()),
        }),
        last_sync: Some(chrono::Utc::now()),
        status: RepositoryStatus::Active,
    };

    // JSON序列化
    let json_str = serde_json::to_string_pretty(&repo_config)?;
    println!("   JSON序列化成功，长度: {} 字符", json_str.len());

    let parsed_json: RepositoryConfiguration = serde_json::from_str(&json_str)?;
    println!("   JSON反序列化成功，仓库名: {}", parsed_json.name);

    // YAML序列化
    let yaml_str = serde_yaml::to_string(&repo_config)?;
    println!("   YAML序列化成功，长度: {} 字符", yaml_str.len());

    let parsed_yaml: RepositoryConfiguration = serde_yaml::from_str(&yaml_str)?;
    println!("   YAML反序列化成功，状态: {:?}", parsed_yaml.status);

    Ok(())
}