# Implementation Plan: CodeZombiesInvestigator (CZI) - Zombie Code Analysis System

**Branch**: `001-zombie-code-analyzer` | **Date**: 2025-12-06 | **Spec**: `/specs/001-zombie-code-analyzer/spec.md`
**Input**: Feature specification from `/specs/001-zombie-code-analyzer/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build CodeZombiesInvestigator (CZI) - A high-performance static analysis tool for identifying dead code and isolated modules in large legacy systems through dependency graph analysis and reachability algorithms. The system will analyze multi-repository codebases, build dependency graphs, and identify unreachable code to enable safe cleanup of technical debt.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75+ (Tokio async runtime) with Tauri 1.5+ for desktop UI
**Primary Dependencies**: Tree-sitter (multi-language parsing), petgraph (graph algorithms), serde (serialization), tokio (async runtime)
**Storage**: File-based graph serialization with bincode, Git repository caching, JSON configuration files
**Testing**: cargo test with unit, integration, and contract testing following TDD methodology
**Target Platform**: Cross-platform desktop application (macOS and Windows primary, Linux secondary)
**Project Type**: Desktop application with Rust core + Web frontend via Tauri
**Performance Goals**: Analyze 500K LOC within 2 minutes, IPC queries respond within 50ms, memory usage under 2GB for million-node graphs
**Constraints**: <2 minute analysis time for large codebases, <50ms query response time, <2GB memory usage, must support offline analysis
**Scale/Scope**: Enterprise codebases (100K-1M LOC), multi-repository analysis (2-10 repos), million-node dependency graphs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ✅ PASSED (Re-evaluated after Phase 1 design)

### Constitutional Compliance Analysis

**Architecture-First Design**: ✅ COMPLIANT - Plan specifies strict separation between Rust core analysis engine and Tauri/Web presentation layer through IPC interfaces.

**Performance-Centric Development**: ✅ COMPLIANT - Technical context explicitly defines performance requirements: 2-minute analysis for 500K LOC, 50ms query response, 2GB memory limit.

**Test-First Implementation**: ✅ COMPLIANT - Testing approach specifies cargo test with unit, integration, and contract testing, aligned with TDD methodology requirement.

**Modular Rust Core**: ✅ COMPLIANT - Project structure will follow modular organization with separate crates for config, io, parser, graph, and IPC as specified.

**Cross-Platform Compatibility**: ✅ COMPLIANT - Target platform specification includes macOS and Windows as primary platforms with Linux secondary.

**Quality Standards**: ✅ COMPLIANT - Rust toolchain requirements (cargo fmt, cargo clippy) and interface standards (serde, SymbolID format) align with constitutional requirements.

**Development Workflow**: ✅ COMPLIANT - Team collaboration structure aligns with Core Team (Rust) and UI Team (Web/Tauri) responsibilities defined in constitution.

**CONCLUSION**: All constitutional principles satisfied. No violations detected.

### Post-Design Compliance Verification

After completing Phase 0 (Research) and Phase 1 (Design), the implementation plan has been re-evaluated against the constitution:

**Data Model Review**: The entity definitions (RepositoryConfiguration, ActiveRootNode, CodeSymbol, DependencyEdge, etc.) align with constitutional requirements:
- Clear separation of concerns between configuration, analysis, and presentation layers
- No UI concerns in the core data model
- Performance considerations embedded in entity design (e.g., optimization for million-node graphs)

**API Contract Review**: The OpenAPI specification confirms:
- Strict IPC interface boundaries with typed data structures
- All interfaces follow serde serialization standards
- Query APIs use standardized SymbolID: String format
- No large data structures transmitted via IPC

**Architecture Review**: The multi-crate workspace structure (czi_core, czi_ipc, czi_desktop) enforces:
- Clear module boundaries as specified in constitution
- Pure logic in czi_core::graph module (no I/O concerns)
- IPC layer acts as translation boundary only
- Performance requirements embedded in design decisions

**Final Verification**: All design artifacts (data-model.md, contracts/openapi.yaml, quickstart.md) are consistent with constitutional principles and performance requirements.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# Desktop Application with Rust Core + Web Frontend (CZI Architecture)
czi_core/
├── Cargo.toml
├── src/
│   ├── config/          # Repository configuration management
│   ├── io/              # Git operations and file system abstraction
│   ├── parser/          # Tree-sitter integration for multi-language parsing
│   ├── graph/           # Dependency graph building and algorithms (pure logic)
│   ├── analysis/        # Reachability analysis and zombie code detection
│   └── lib.rs           # Public API exports
└── tests/
    ├── unit/            # Unit tests for individual modules
    ├── integration/     # Integration tests for module interactions
    └── contract/        # Contract tests for public APIs

czi_ipc/
├── Cargo.toml
├── src/
│   ├── commands/        # Tauri command definitions
│   ├── types/           # IPC data structures
│   └── lib.rs           # IPC interface exports
└── tests/
    └── contract/        # IPC contract tests

czi_desktop/
├── src-tauri/
│   ├── Cargo.toml       # Tauri application manifest
│   ├── src/
│   │   └── main.rs      # Tauri application entry point
│   └── tauri.conf.json  # Tauri configuration
├── src/                 # Frontend source (React/Vue)
│   ├── components/      # UI components
│   ├── services/        # API service layer
│   ├── views/           # Page components
│   └── main.js          # Frontend entry point
├── public/              # Static assets
└── tests/
    ├── unit/            # Frontend unit tests
    └── e2e/             # End-to-end tests

# Root level configuration
cargo_workspace.toml     # Rust workspace configuration
package.json             # Frontend dependencies and scripts
```

**Structure Decision**: Multi-crate Rust workspace with Tauri desktop application. This structure enforces architectural boundaries between core analysis engine (czi_core), IPC interface (czi_ipc), and presentation layer (czi_desktop), aligning with constitutional requirements for modular design and performance isolation.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
