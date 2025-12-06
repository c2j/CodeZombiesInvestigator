## 📐 1. 架构与模块化规范 (Architecture & Modularity)

核心原则是**“清晰的分层和技术边界”**。

### 1.1 核心分层 (Rust Core)
Rust 代码库必须严格按功能分为以下模块（Crates 或 Modules）：

* **`czi_core::config`:** 仅处理项目配置、多仓库清单、入口点定义。
* **`czi_core::io`:** 负责所有文件系统和 Git 操作（如 `shallow clone`、文件读取）。
* **`czi_core::parser`:** 封装 Tree-sitter 逻辑，负责 AST 生成和符号提取。
* **`czi_core::graph`:** 包含 `DependencyGraph` 结构和所有图算法（如 `find_zombies`、BFS）。**这是项目的核心逻辑，必须保持纯净，不包含任何 I/O 或 IPC 代码。**
* **`czi_ipc` (Wrapper Crate):** 仅包含 Tauri IPC Command 的函数签名和胶水代码，负责调用 `czi_core::graph` 的 API，并处理数据序列化/反序列化。

### 1.2 前端分层 (Tauri/Web)
Web 应用层必须采用主流框架的模块化规范：

* **View/Page Components:** 负责路由和整体布局。
* **Functional Components:** 可复用的 UI 元素（如表格、按钮、筛选器）。
* **State Management:** 必须使用统一的状态管理模式（如 Redux/Vuex/Zustand），避免状态分散。
* **Service Layer:** 封装所有对 Rust Core 的 IPC 调用，前端组件不得直接调用 IPC 命令。

---

## 🛡️ 2. Rust 代码质量与性能规范 (Rust Quality & Performance)

为了保证 CZI 的核心价值（极致性能），必须执行严格的 Rust 规范。

### 2.1 性能与安全
* **Minimal `unsafe`:** 严格限制 `unsafe` 代码的使用。所有 `unsafe` 块必须有详细的注释解释其安全性保障（Safety Justification）。
* **并发模型:** 所有耗时的 I/O 和计算任务（如文件解析、Git 操作）必须使用 **Tokio Runtime** 启动为异步任务，确保 UI 线程不被阻塞。
* **内存优化:** 尽量避免不必要的内存分配和复制，尤其是在图构建和遍历过程中。优先使用引用 (`&`) 或 `Arc<T>`，而非值拷贝。

### 2.2 格式化与静态分析
* **`cargo fmt`:** 所有代码提交前必须执行 `cargo fmt`。
* **`clippy`:** 持续集成 (CI) 流程中必须包含 **`cargo clippy -- -D warnings`**，强制修复所有 lint 警告。
* **文档:** 所有公共 API（Public Functions, Structs, Enums）必须包含清晰的 **文档注释**（`///`），解释其用途、参数和返回值。

---

## 🎨 3. 接口与数据标准 (Interface & Data Standards)

确保 Rust 核心与 Web 前端之间的数据传输准确无误。

### 3.1 IPC 数据模型
所有通过 Tauri IPC 传输的数据结构（Structs）必须：
* **可序列化/反序列化:** 必须实现 **`serde::Serialize`** 和 **`serde::Deserialize`** Traits。
* **简洁性:** 避免在 IPC 中传输大型原始数据结构（如整个 `petgraph` 对象）。只传输查询结果或报告数据（如 `CodeZombiesReport` JSON）。

### 3.2 图结构 API 标准
`DependencyGraph` 上的所有查询方法必须通过 **Trait** 或 **公共方法** 暴露，且其输入输出必须是标准化的。

* **输入:** 统一使用 `SymbolID: String`。
* **输出:** 统一返回结构体或 `Result<T, E>`（处理 I/O 或解析错误）。

---

## 🤝 4. 协作与工作流规范 (Workflow & Collaboration)

针对跨越 Rust 和 Web 的团队协作，建立明确的沟通流程。

### 4.1 分工与责任
* **Core Team (Rust):** 负责 **FR-A, FR-B, FR-C** 的实现，确保分析结果的准确性和性能 (NFR-P)。
* **UI Team (Web/Tauri):** 负责 **FR-D** 的实现，确保用户体验和界面响应。
* **接口会议:** 每周至少一次 **“IPC 接口定义会议”**，确认前端所需数据格式和 Rust 核心的 API 签名，避免返工。

### 4.2 Git 工作流 (Git Workflow)
* **分支策略:** 采用 Git Flow 或 Trunk-Based Development 策略。所有新功能必须在独立的功能分支上开发。
* **Pull Request (PR) 规范:**
    * **模板强制:** 强制使用 PR 模板，包含：目的、关联 Issue、技术实现细节、以及**性能影响评估**（仅限 Rust Core）。
    * **交叉评审:** 任何修改了 IPC 接口的代码，必须由 **Core Team** 和 **UI Team** 各一名成员进行评审和批准。

### 4.3 Issue 追踪
* **标签分类:** 所有 Issue 必须使用标签分类，区分技术栈（如 `rust::parser`, `web::ui`, `bug::ipc`），方便团队成员快速定位。
* **性能预算:** 针对 Rust Core 的关键功能（如图构建时间），设立性能预算，并将其记录在对应的 Issue 中。
