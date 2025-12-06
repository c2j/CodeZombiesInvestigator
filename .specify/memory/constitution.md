<!--
Sync Impact Report:
- Version change: Template → 1.0.0 (Initial version)
- Modified principles: All principles newly defined from template placeholders
- Added sections: Quality Standards, Development Workflow, Governance
- Removed sections: None (template was placeholder-only)
- Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ Constitution Check section aligns
  - .specify/templates/spec-template.md: ✅ Requirements section supports constitutional principles
  - .specify/templates/tasks-template.md: ✅ Task organization supports workflow principles
  - .claude/commands/speckit.constitution.md: ✅ Command references updated
- Follow-up TODOs: None
-->

# CodeZombiesInvestigator Constitution

## Core Principles

### I. Architecture-First Design
CZI MUST maintain strict architectural boundaries between Rust core analysis engine and Tauri/Web presentation layer. The core analysis engine MUST be completely decoupled from UI concerns and exposed only through well-defined IPC interfaces.

### II. Performance-Centric Development (NON-NEGOTIABLE)
All code changes MUST prioritize performance as the primary constraint. Analysis operations on 500K LOC projects MUST complete within 2 minutes, IPC queries MUST respond within 50ms, and memory usage MUST stay below 2GB for million-node graphs.

### III. Test-First Implementation
Every feature MUST follow TDD methodology: write failing tests first, implement minimal code to pass, then refactor. All public APIs MUST have comprehensive test coverage before implementation.

### IV. Modular Rust Core
Rust codebase MUST be strictly organized into functional modules: `czi_core::config` for configuration, `czi_core::io` for file system operations, `czi_core::parser` for AST processing, `czi_core::graph` for graph algorithms (MUST remain pure), and `czi_ipc` for Tauri command wrappers.

### V. Cross-Platform Compatibility
All core functionality MUST support macOS and Windows platforms. Platform-specific code MUST be isolated and clearly documented with fallback mechanisms.

## Quality Standards

### Code Quality
- All Rust code MUST pass `cargo fmt` and `cargo clippy -- -D warnings` without exceptions
- Public APIs MUST include comprehensive documentation comments
- `unsafe` code blocks MUST include detailed safety justifications
- Memory allocations MUST be minimized, preferring references and `Arc<T>` over copying

### Interface Standards
- All IPC data structures MUST implement `serde::Serialize` and `serde::Deserialize`
- Graph query APIs MUST use standardized `SymbolID: String` inputs and return `Result<T, E>` types
- Large data structures MUST NOT be transmitted via IPC - only query results and reports

## Development Workflow

### Collaboration Protocol
- Core Team (Rust) owns FR-A, FR-B, FR-C implementation and performance requirements
- UI Team (Web/Tauri) owns FR-D implementation and user experience
- Weekly IPC interface definition meetings REQUIRED to prevent rework
- Cross-team code review REQUIRED for any IPC interface modifications

### Git Workflow
- Feature branches MUST be used for all new development
- PR templates MUST include performance impact assessment for Rust Core changes
- All PRs MUST pass automated quality checks before merge

## Governance

This constitution supersedes all other development practices. Amendments require:
1. Documentation of proposed changes with technical justification
2. Impact analysis on existing principles and workflows
3. Approval from both Core and UI team leads
4. Migration plan for updating dependent templates and processes

All PRs and code reviews MUST verify constitutional compliance. Complexity MUST be justified against performance and simplicity principles. Technical decisions that conflict with these principles REQUIRE explicit constitutional amendment.

**Version**: 1.0.0 | **Ratified**: 2025-12-06 | **Last Amended**: 2025-12-06