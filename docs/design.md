# ⚙️ CodeZombiesInvestigator (CZI) 技术设计规格书

## 1\. 架构总览与技术栈 (Architecture and Stack)

CZI 采用分离式架构，确保高性能计算与灵活的前端展示解耦。

### 1.1 核心技术栈

| 组件 | 技术 | 用途 |
| :--- | :--- | :--- |
| **分析核心 (CZI Core)** | **Rust** (Tokio) | 负责 I/O、并行解析、图构建和分析算法。 |
| **图引擎** | **`petgraph::DiGraph`** | 内存中的有向图结构，用于可达性分析。 |
| **解析引擎** | **Tree-sitter** (Rust Bindings) | 统一的 AST 解析，支持多语言。 |
| **持久化** | **`bincode`** / **`serde`** | 快速序列化/反序列化图结构，实现秒级加载。 |
| **桌面框架** | **Tauri** | 轻量级 WebView 容器，负责前端与 Rust 核心的桥接。 |
| **前端/可视化** | **React/Vue** + **D3.js/vis.js** | 负责展示报告、交互式依赖图。 |

### 1.2 架构分层

1.  **I/O 层 (Rust):** 负责多仓库的 Git 操作、文件系统抽象。
2.  **Ingestion/Parser 层 (Rust):** 包含 Tree-sitter 驱动的并发解析器池，负责生成原始符号和调用关系。
3.  **Graph Engine 层 (Rust):** 基于 `petgraph`，负责语义链接和图构建。
4.  **Analysis/Query 层 (Rust):** 负责执行可达性算法和依赖查询。
5.  **Presentation 层 (Tauri/Web):** 负责 UI 渲染和用户输入。

-----

## 2\. 数据模型设计 (Data Model Design)

### 2.1 图模型：节点 (Node) 设计

所有节点均存储在 `petgraph::DiGraph` 中，并由一个全局的 `SymbolIndex: HashMap<String, NodeIndex>` 映射，实现快速查找。

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| **`id`** | `String` | **唯一标识符**：`[RepoName]::[Path]::[Symbol]`，用于跨仓库引用。 |
| **`kind`** | `enum NodeType` | 实体类型：`Function`, `Class`, `File`, `DB_Table`, `XML_Statement`, `Scheduler_Script`。 |
| **`repo_name`** | `String` | 节点所在的 Git 仓库名称。 |
| **`is_active_root`** | `bool` | 是否为用户定义的系统入口点（Controller, Scheduler）。 |
| **`is_reachable`** | `bool` | **分析结果**：是否被活跃根节点触及（默认 `false`）。 |
| **`metadata`** | `HashMap<String, String>` | 附加信息：`last_modified_date`, `contributor`, `line_number`。 |

### 2.2 图模型：边 (Edge) 设计

边是带方向的（`Source` 🎯 `Target`），代表依赖关系。

| 边类型 | `enum EdgeType` | 描述 | 对应 CZI 链路 |
| :--- | :--- | :--- | :--- |
| **`CALLS`** | Direct Call | 方法调用 (e.g., `ServiceA` 🎯 `ServiceB`)。 | 核心 |
| **`IMPLEMENTS`** | Semantic Link | Java 接口方法 🎯 MyBatis XML Statement。 | 链路一 |
| **`INVOKES`** | External Call | Java/Scripts 🎯 存储过程。 | 链路二、三 |
| **`ACCESSES`** | Data I/O | SQL 语句 🎯 数据库表（或存储过程 🎯 表）。 | 链路一、二、三 |
| **`IMPORTS`** | File Dependency | 文件 A 依赖/引入 文件 B。 | 核心 |

### 2.3 报告输出模型 (CodeZombiesReport.json)

前端展示的最终 JSON 报告格式：

```json
[
  {
    "symbol_id": "repoA::src/Service.java::unusedMethod()",
    "status": "DEAD_UNREACHABLE",
    "reason": "Not connected to any active root node.",
    "kind": "Function",
    "repo_name": "RepoA",
    "last_modified": "2018-03-15",
    "contributor": "John Doe"
  },
  // ... 其他僵尸节点
]
```

-----

## 3\. 核心引擎设计 (Core Engine Implementation)

### 3.1 符号解析器 (Ingestion Pipeline)

  * **并行解析：** 利用 `tokio::task::spawn` 对不同语言文件并行执行 `tree-sitter` 解析。
  * **语义链接服务：** 专门的 Rust 模块（例如 `SemanticLinker`）处理隐式依赖：
      * **Java 字符串分析：** 使用正则表达式或更精确的 AST 遍历，在 `jdbcTemplate.call("...")` 的字符串参数中提取存储过程名称。
      * **MyBatis 匹配：** 维护一个 **`XML_ID_Map`**，将 `namespace` 和 `id` 预先索引，在解析 Java 接口时快速查找并创建 `IMPLEMENTS` 边。

### 3.2 可达性分析算法

  * **算法选择：** **广度优先搜索 (BFS)**。BFS 简单高效，适合在大规模稀疏图（代码图通常是稀疏图）中进行可达性判断。
  * **执行流程：**
    1.  **初始化：** 创建一个 `VecDeque` 队列，装载所有 `is_active_root: true` 的节点索引。
    2.  **遍历：** 循环弹出节点，将其 `is_reachable` 设为 `true`。
    3.  **扩散：** 遍历该节点的所有**出边**。如果目标节点尚未标记为 `is_reachable`，则标记并加入队列。
  * **内存优化：** 由于图在内存中，分析速度极快。通过序列化/反序列化机制，可避免每次启动都重新解析代码。

-----

## 4\. API/接口定义 (Tauri IPC Bridge)

Tauri 前端与 Rust 核心之间的通信必须通过定义清晰的命令（Command）。

| IPC 命令 (Tauri Invoke) | 职责 | 核心模块调用 |
| :--- | :--- | :--- |
| **`run_analysis(config: String)`** | 启动全量图构建、解析和可达性分析。 | **Ingestion Pipeline** |
| **`get_zombie_report()`** | 返回最终的 JSON 格式僵尸代码报告。 | **Analysis/Query Layer** |
| **`get_symbol_info(symbol_id: String)`** | 根据 ID 获取节点元数据（如文件路径、行号、Commit 历史）。 | **Analysis/Query Layer** |
| **`query_dependencies(symbol_id: String)`** | 核心查询：返回某个符号的所有**直接和间接调用者**（用于路径分析）。 | **Analysis/Query Layer** |
| **`get_config()`** | 获取当前加载的多仓库配置。 | **I/O Layer** |

-----

## 5\. 交互与可视化 (UI/Interaction Design)

虽然前端不是基础阶段的重点，但需要确定核心视图。

### 5.1 核心视图 (Main Views)

1.  **配置/状态视图 (Configuration View):**
      * 用于输入多仓库配置。
      * 展示上次分析时间、图节点总数、僵尸代码数量。
2.  **报告视图 (Report View):**
      * 主界面，以表格形式展示 `CodeZombiesReport.json` 内容。
      * 提供筛选器：按仓库、按语言、按最后修改日期（例如：“三年以上未修改的僵尸”）。
3.  **路径分析视图 (Path Analysis View):**
      * 用于展示僵尸代码或普通代码的**依赖图**。
      * **关键交互：** 用户点击僵尸节点，前端调用 `query_dependencies` API，然后使用 D3.js 或 Vis.js 可视化展示其到最近**活跃节点**或**孤岛边界**的调用链。

### 5.2 交互原则 (Interaction Principles)

  * **Keyboard First:** 沿袭之前的设计，报告视图应支持键盘导航和筛选。
  * **Zero-Delay Query:** 所有依赖查询（FR-C.3）必须通过高性能的 Rust 核心实现，确保前端交互无卡顿。
