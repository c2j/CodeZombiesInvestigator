# 📝 CodeZombiesInvestigator (CZI) 需求说明书

## 1. 引言 (Introduction)

### 1.1 项目目标
**CodeZombiesInvestigator (CZI)** 的目标是解决大型遗留系统中的**“僵尸代码”（Dead Code）**和**“幽灵模块”（Isolated Modules）**问题。通过高性能的静态分析和图论方法，CZI 能够准确识别与系统有效入口（Active Root Nodes）不存在调用关联的代码资产，并生成可操作的清理报告。

### 1.2 范围定义 (MVP Scope)
本 MVP 需求说明书涵盖了实现**多仓库代码依赖图构建**和**核心可达性分析算法**所需的所有功能。展示层（前端 UI）将采用 Tauri + Web 技术，功能需通过 IPC 调用 Rust 核心 API 实现。

---

## 2. 目标用户与价值 (Users & Value Proposition)

| 目标用户 | 主要痛点 | CZI 提供的价值 |
| :--- | :--- | :--- |
| **技术负责人/架构师 (TL/Architect)** | 不敢轻易删除老代码，代码库膨胀，技术债务高企。 | 提供**可信赖的、数据驱动的清理依据**，降低技术债务。 |
| **Code Reviewer (开发者)** | 无法判断 PR 中删除的代码是否影响其他模块。 | **跨仓库、跨语言的依赖影响分析**，提高审查效率和安全性。 |
| **运维/项目经理** | 系统复杂性高，上线风险大。 | **可视化系统核心路径**，快速识别维护成本高但无业务价值的模块。 |

---

## 3. 功能需求 (Functional Requirements - FR)

功能需求按分析流程分为四大模块：配置与数据摄取、图构建与链接、核心分析、报告输出。

### 3.1 FR-A：配置与数据摄取 (Ingestion)

| 编号 | 描述 | 关键点 |
| :--- | :--- | :--- |
| **FR-A.1** | **多仓库配置输入** | 必须支持通过配置文件（如 JSON/YAML）或 UI 界面输入**至少两个 Git 仓库 URL、本地路径和认证信息**。 |
| **FR-A.2** | **代码拉取与同步** | 能够异步地对配置的 Git 仓库执行 `fetch` 或 `shallow clone` 操作，并保持本地代码缓存与最新指定分支同步。 |
| **FR-A.3** | **入口清单定义** | 允许用户输入和持久化**活跃根节点（Active Root Nodes）**清单，包括：**调度脚本路径**、**Java Controller 注解方法签名**、**消息队列监听器方法签名**。 |
| **FR-A.4** | **文件解析并行化** | 使用 Rust **Tokio 运行时**和 **Tree-sitter** 驱动的并发解析池，对不同语言文件进行并行 AST 解析。 |

### 3.2 FR-B：图构建与语义链接 (Linking)

该模块负责将代码抽象为图结构，是 CZI 的核心。

| 编号 | 描述 | 关键链路/技术 |
| :--- | :--- | :--- |
| **FR-B.1** | **符号与节点创建** | 遍历所有 ASTs，创建 `File`, `Function`, `Struct` 等节点，并确保跨仓库的 **唯一 ID** (`[RepoName]::[Path]::[Symbol]`)。 |
| **FR-B.2** | **核心调用边创建** | 识别 Java/JS/Scripts 等语言中的**方法调用**和**依赖引入**，创建 `Calls` 和 `Imports` 边。 |
| **FR-B.3** | **链路一：MyBatis 隐式链接** | 必须解析 XML 文件的 `namespace` 和 `id`，并创建 `Java Method` 🎯 `XML Statement` 的 **`Implements`** 边。 |
| **FR-B.4** | **链路二：存储过程 Invocation** | 必须通过对 **Java JDBC/MyBatis API** 调用字符串参数进行解析，创建 `Java Method` 🎯 `Stored_Procedure` 的 **`Invokes`** 边。 |
| **FR-B.5** | **链路三：脚本触发链接** | 必须解析 Shell/Python 脚本的命令行参数，识别数据库连接和存储过程名，创建 `Scheduler_Script` 🎯 `Stored_Procedure` 的 **`Triggers`** 边。 |
| **FR-B.6** | **数据库表访问链接** | 必须解析所有 SQL 语句（包括 XML 内嵌 SQL），识别 `SELECT`, `INSERT`, `UPDATE` 等操作，创建 `SQL` 🎯 `DB_Table` 的 **`Reads/Writes`** 边。 |

### 3.3 FR-C：核心分析与查询 (Analysis)

| 编号 | 描述 | 算法/逻辑 |
| :--- | :--- | :--- |
| **FR-C.1** | **根节点标记** | 识别并标记所有来自 FR-A.3 清单的节点为 **`is_active_root: true`**。 |
| **FR-C.2** | **僵尸代码分析** | 运行 **广度优先搜索 (BFS) / 可达性算法**，从所有活跃根节点出发遍历图。标记所有未被访问的节点为 **`is_reachable: false`**。 |
| **FR-C.3** | **依赖查询 API (IPC)** | 提供 IPC API 供前端调用：`query_dependents(SymbolID)`（查询调用者）和 `query_dependencies(SymbolID)`（查询被调用者）。 |
| **FR-C.4** | **代码考古数据提取** | 对所有不可达节点，提取其 Git 历史数据：**最后修改日期 (Last Commit Date)** 和 **主要贡献者 (Top Contributor)**。 |

### 3.4 FR-D：报告与展示 (Reporting - Tauri Layer)

| 编号 | 描述 | 展示方式 |
| :--- | :--- | :--- |
| **FR-D.1** | **标准化 JSON 报告** | Rust 核心必须输出一个包含所有不可达节点的标准化 JSON 报告，包含：`ID`, `Type`, `RepoName`, `LastModifiedDate`。 |
| **FR-D.2** | **僵尸清单展示** | Tauri 前端必须以表格形式展示僵尸代码报告，支持按仓库、语言、最后修改时间进行筛选和排序。 |
| **FR-D.3** | **交互式路径分析** | 允许用户点击报告中的僵尸节点，前端调用 Rust 核心 API 查找并可视化展示**“最接近的调用者”**或**“孤立路径”**。 |

---

## 4. 非功能性需求 (Non-Functional Requirements - NFR)

### 4.1 性能与规模 (Performance & Scale)
* **NFR-P.1** | **启动时间：** 核心引擎加载序列化图结构，必须在 **2 秒内** 完成。
* **NFR-P.2** | **分析时间：** 对 50 万行代码规模的项目，完成全量图构建和可达性分析的时间控制在 **2 分钟以内**。
* **NFR-P.3** | **查询速度：** 前端通过 IPC 查询任意符号的依赖关系，响应时间必须在 **50 毫秒内**。
* **NFR-P.4** | **内存效率：** 即使图节点数达到 100 万以上，内存占用需优化至合理范围（例如，分析时不持续超过 2GB）。

### 4.2 技术与架构 (Technology & Architecture)
* **NFR-T.1** | **核心技术栈：** 必须使用 **Rust** (Tokio, Petgraph, Tree-sitter) 实现分析引擎。
* **NFR-T.2** | **UI 框架：** 必须使用 **Tauri** 作为桌面应用框架，前端使用成熟的 Web 技术（React/Vue）。
* **NFR-T.3** | **代码解耦：** 分析核心必须与 UI 完全解耦，通过 **Tauri IPC** 或 **RPC/TCP 接口** 进行通信。
* **NFR-T.4** | **跨平台支持：** MVP 阶段优先支持 **macOS 和 Windows** 平台。

---

## 5. 架构与流程总结 (Summary)



CZI 采用**双核架构**：

1.  **Rust 分析核心 (CZI Core):** 负责高性能计算、多仓库 Git I/O、Tree-sitter 解析、**Petgraph 图构建**和**可达性分析**。
2.  **Tauri 展示层 (CZI UI):** 负责用户交互、配置管理、**图表可视化**、报告展示，通过 **IPC** 触发核心计算。
