### CZI 前后端接口定义（IPC规范）

作为架构师和前端专家，我完全同意您的建议：在不深入具体实现（如Rust代码细节或前端组件逻辑）的前提下，先定义前后端接口是最佳实践。这有助于解耦开发、确保一致性和便于并行工作。鉴于CZI是Tauri桌面应用，前后端通信采用IPC（Inter-Process Communication）机制，而非HTTP API。Tauri IPC基于Rust命令（Command）和前端invoke调用，参数/返回值为JSON序列化，支持异步处理。

接口设计遵循RESTful-like原则（尽管是IPC）：使用动词命名命令（如`get_xx`、`query_xx`），参数简单（String或JSON字符串），返回标准化JSON。所有接口需处理错误（e.g., { "error": "描述" }），并支持项目级上下文（Project ID）。我将接口分为核心类别，基于需求文档（req.md/design.md）和项目扩展，使用表格呈现。每个接口包括：

- **命令名**：Tauri invoke的命令字符串。
- **参数**：输入类型（Rust视角：String或JSON解析）。
- **返回**：输出JSON结构示例。
- **描述**：功能和前端使用场景。
- **错误处理**：常见异常。

这些接口覆盖FR-A~D和NFR，确保性能（响应<50ms for 查询，<2min for 分析）。未来可扩展（如添加认证）。

#### 1. 配置相关接口（FR-A：Ingestion & 配置管理）
这些接口处理项目/仓库配置输入和同步。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `create_project` | `{ "project_id": String, "name": String, "description": Option<String> }` (JSON String) | `{ "success": true, "project_id": "ProjectX" }` | 前端创建新项目（聚合仓库）。用于配置视图新建按钮。 | 项目ID已存在：`{ "error": "Project ID duplicate" }` |
| `update_config` | `{ "project_id": String, "repos": [{ "url": String, "path": String, "branch": String, "auth": Option<String> }], "whitelist": [String], "blacklist": [String], "associations": [{ "from": String, "to": String, "type": "CALLS" }] }` (JSON String) | `{ "success": true }` | 更新项目配置，包括仓库、白/黑名单、关联规则。用于配置视图保存。 | 无效URL：`{ "error": "Invalid repo URL" }` |
| `get_config` | `{ "project_id": Option<String> }` (JSON String，null为所有项目) | `{ "projects": [{ "id": "ProjectX", "repos": [...], "whitelist": [...], ... }] }` | 获取当前配置。用于配置视图加载或报告视图过滤。 | 项目不存在：`{ "error": "Project not found" }` |
| `sync_repos` | `{ "project_id": String }` (JSON String) | `{ "success": true, "updated_repos": [String] }` | 异步同步Git仓库（fetch/clone）。用于配置后刷新。 | Git失败：`{ "error": "Git sync error: details" }` |

#### 2. 分析与查询接口（FR-B/C：图构建 & 分析）
这些接口触发图构建和依赖查询，支持异步（返回Promise in 前端）。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `run_analysis` | `{ "project_id": String, "config": Option<JSON String> }` (可选覆盖配置) | `{ "success": true, "analysis_id": String, "status": "completed" }` (异步轮询status) | 启动全量图构建、解析和可达性分析（BFS）。用于配置视图"运行分析"按钮，返回后前端轮询状态。 | 超时：`{ "error": "Analysis timeout" }` |
| `get_analysis_status` | `{ "analysis_id": String }` | `{ "status": "running/completed/failed", "progress": 75 }` | 查询分析进度。用于前端进度条。 | ID无效：`{ "error": "Invalid analysis ID" }` |
| `query_dependencies` | `{ "project_id": String, "symbol_id": String, "depth": Option<Integer> }` (深度限间接调用) | `{ "dependencies": [{ "id": String, "type": "Function", "edge_type": "CALLS" }], "dependents": [...] }` | 查询符号的直接/间接依赖和调用者。用于路径分析视图可视化。 | 符号不存在：`{ "error": "Symbol not found" }` |
| `get_symbol_info` | `{ "project_id": String, "symbol_id": String }` | `{ "id": String, "kind": "Function", "repo_name": String, "metadata": { "last_modified": "2018-03-15", "contributor": "John" } }` | 获取节点详情（包括Git历史）。用于报告视图侧栏。 | 同上 |

#### 3. 报告与输出接口（FR-D：Reporting）
这些接口获取分析结果，支持过滤。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `get_zombie_report` | `{ "project_id": String, "filters": { "repo": Option<String>, "language": Option<String>, "min_confidence": Option<Integer>, "status": Option<String> } }` (JSON String) | `[{ "symbol_id": "RepoA::unusedMethod()", "status": "DEAD_UNREACHABLE", "reason": "...", "kind": "Function", "repo_name": "RepoA", "last_modified": "2018-03-15", "contributor": "John", "confidence": 85 }]` (数组JSON) | 获取僵尸代码报告（标准化JSON）。用于报告视图表格。 | 无分析：`{ "error": "No analysis run" }` |
| `confirm_delete` | `{ "project_id": String, "symbols": [String], "commit_msg": String }` (JSON String) | `{ "success": true, "deleted": [String] }` | 确认删除僵尸代码（Git rm + commit）。用于报告视图批量操作。 | Git冲突：`{ "error": "Git delete failed: conflict" }` |
| `export_report` | `{ "project_id": String, "format": "JSON/CSV" }` | `{ "data": String (base64编码或直接JSON) }` | 导出报告。用于报告视图导出按钮。 | 格式无效：`{ "error": "Unsupported format" }` |

#### 4. 通用与维护接口
这些接口处理全局操作。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `get_projects` | 无 | `{ "projects": [{ "id": "ProjectX", "name": "Main App", "repo_count": 5 }] }` | 列出所有项目。用于顶部项目选择器。 | 无 |
| `delete_project` | `{ "project_id": String }` | `{ "success": true }` | 删除项目（包括缓存）。用于项目管理。 | 项目锁定：`{ "error": "Project in use" }` |
| `get_logs` | `{ "level": "error/info" }` | `{ "logs": [String] }` | 获取后端日志。用于设置视图调试。 | 无日志：`{ "logs": [] }` |

#### 实施注意（架构视角）
- **安全性**：IPC参数需Rust侧验证（e.g., sanitize路径），避免注入。黑/白名单限长度。
- **异步与状态**：分析类接口异步，前端用Tauri `invoke` + `listen`事件监听进度（e.g., emit "analysis_progress"）。
- **版本化**：接口用semver（v1.0），未来添加header `{ "api_version": "1.0" }`。
- **前端集成**：React/Vue侧用`window.__TAURI__.invoke('command', payload)`调用，返回Promise。错误统一处理（e.g., Toast通知）。
- **扩展性**：若需HTTP模式（未来云版），可映射为REST API（e.g., POST /run-analysis）。



### CZI 前后端高级接口定义（IPC规范扩展）

作为架构师和前端专家，我认为在MVP接口基础上添加高级接口是明智的扩展策略。这能提升系统的可扩展性、可观测性和用户体验，例如支持增量分析（避免全量重跑）、批量查询、历史版本管理，以及与外部工具集成（如导出到Graphviz可视化）。这些高级接口构建在原有基础上，保持JSON序列化和异步IPC机制，确保解耦（Rust后端处理复杂逻辑，前端仅消费数据）。我将新接口分类呈现，优先考虑性能（e.g., 增量操作<30s）和安全性（e.g., 限流查询）。

接口设计原则保持一致：命令动词化、参数简洁、返回标准化JSON、错误统一格式。新增接口支持项目级和全局操作，适用于大型遗留系统场景（如多版本分析）。前端集成建议：使用Redux管理状态，invoke调用后处理Promise.reject for 错误。

#### 1. 高级配置与管理接口
这些接口扩展FR-A，支持项目历史和批量管理，便于架构师追踪配置演变。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `list_project_history` | `{ "project_id": String, "limit": Option<Integer> }` (JSON String，limit默认10) | `{ "history": [{ "version_id": String, "timestamp": "2025-12-09T10:00:00Z", "changes": "Added RepoC" }] }` | 列出项目配置历史版本（基于Git commit或内部日志）。用于配置视图审计日志。 | 无历史：`{ "error": "No history available" }` |
| `revert_project_config` | `{ "project_id": String, "version_id": String }` | `{ "success": true, "new_config": { ... } }` | 回滚到指定配置版本，并触发重新分析。用于误操作恢复。 | 版本无效：`{ "error": "Invalid version ID" }` |
| `batch_add_repos` | `{ "project_id": String, "repos": [{ "url": String, ... }] }` (JSON String) | `{ "success": true, "added": Integer }` | 批量添加仓库（并行sync）。用于大规模项目导入。 | 部分失败：`{ "success": false, "failed_repos": [String] }` |

#### 2. 高级分析与优化接口（扩展FR-B/C）
这些接口引入增量模式和自定义算法参数，提升分析效率（e.g., 只处理变更文件）。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `run_incremental_analysis` | `{ "project_id": String, "since_commit": Option<String> }` (JSON String，since_commit为Git hash) | `{ "success": true, "updated_nodes": Integer, "status": "completed" }` | 增量分析（基于Git diff，只重新解析变更）。用于频繁更新场景，前端轮询状态。 | 无变更：`{ "error": "No changes detected" }` |
| `customize_analysis_params` | `{ "project_id": String, "params": { "max_depth": Integer, "ignore_patterns": [String], "confidence_threshold": Integer } }` | `{ "success": true }` | 设置分析参数（如BFS深度限、忽略模式、置信度阈值）。用于报告视图高级筛选。 | 参数无效：`{ "error": "Invalid param: max_depth > 100" }` |
| `query_path_to_root` | `{ "project_id": String, "symbol_id": String }` | `{ "path": [{ "id": String, "edge_type": "CALLS" }], "distance": Integer }` | 查询符号到最近活跃根的路径（最短路径算法，如Dijkstra）。用于路径分析视图"最接近调用者"。 | 无路径：`{ "error": "Isolated node" }` |

#### 3. 高级报告与可视化接口（扩展FR-D）
这些接口支持导出图数据和聚合统计，便于前端D3.js/Vis.js渲染或外部工具集成。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `get_aggregate_stats` | `{ "project_id": String, "group_by": "repo/language" }` (JSON String) | `{ "stats": { "total_nodes": Integer, "zombie_ratio": Float, "by_repo": { "RepoA": { "zombies": Integer } } } }` | 获取聚合统计（e.g., 僵尸占比，按仓库/语言分组）。用于报告视图汇总卡片。 | 无数据：`{ "error": "No stats available" }` |
| `export_graph` | `{ "project_id": String, "format": "GraphML/JSON/DOT", "subgraph": Option<{ "symbol_id": String }> }` | `{ "data": String (base64编码) }` | 导出全图或子图数据（e.g., DOT for Graphviz）。用于路径分析视图外部可视化。 | 格式不支持：`{ "error": "Unsupported format" }` |
| `batch_confirm_delete` | `{ "project_id": String, "symbols": [String], "commit_msg": String, "dry_run": Boolean }` | `{ "success": true, "affected_files": [String], "preview": Option<String> }` (dry_run返回预览) | 批量删除（支持dry run预览）。用于报告视图高级批量操作。 | 依赖冲突：`{ "error": "Symbols have dependents" }` |

#### 4. 高级维护与监控接口
这些接口提供系统级洞察，支持调试和性能优化，便于前端设置视图集成。

| 命令名 | 参数 | 返回 | 描述 | 错误处理 |
|--------|------|------|------|----------|
| `get_performance_metrics` | `{ "project_id": Option<String> }` | `{ "metrics": { "memory_usage": "1.2GB", "last_analysis_time": "120s", "node_count": Integer } }` | 获取性能指标（内存、时间、图规模）。用于设置视图监控。 | 无指标：`{ "error": "Metrics not collected" }` |
| `clear_cache` | `{ "project_id": Option<String>, "scope": "all/repos/graph" }` | `{ "success": true, "cleared_size": "500MB" }` | 清除缓存（仓库/图序列化）。用于内存优化。 | 锁定中：`{ "error": "Cache in use" }` |
| `import_from_external` | `{ "project_id": String, "source": "SonarQube/JSON", "data": String (base64) }` | `{ "success": true, "imported_nodes": Integer }` | 从外部工具导入分析数据（e.g., SonarQube报告）。用于集成开源工具。 | 数据无效：`{ "error": "Invalid import format" }` |

#### 实施与架构注意
- **异步扩展**：高级分析接口（如增量）使用Tauri事件emit进度（e.g., "incremental_progress"），前端listen订阅，避免轮询开销。
- **限流与安全**：Rust侧添加rate limit（e.g., 查询/分钟<100），参数验证防溢出。黑名单符号自动忽略查询。
- **兼容性**：这些接口向后兼容MVP；前端可渐进采用（e.g., 先实现run_incremental_analysis优化性能瓶颈）。
- **测试策略**：单元测试每个命令（mock Petgraph），集成测试模拟大图（100k节点）。
- **前端视角**：在React/Vue中，封装IPC hook（e.g., useIPC('run_incremental_analysis')），处理loading/error states统一。



