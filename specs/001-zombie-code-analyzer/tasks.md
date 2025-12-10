---

description: "Task list for feature implementation"
---

# Tasks: CodeZombiesInvestigator (CZI) - Zombie Code Analysis System

**Input**: Design documents from `/specs/001-zombie-code-analyzer/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Following TDD methodology from constitutional requirements. All public APIs require comprehensive test coverage.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description with file path`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Multi-crate workspace**: czi_core/, czi_ipc/, czi_desktop/ at repository root
- **Core engine**: czi_core/src/ with modular organization
- **IPC layer**: czi_ipc/src/ for command definitions
- **Desktop app**: czi_desktop/src-tauri/ for Rust code, czi_desktop/src/ for frontend
- **Tests**: tests/ directory at repository root with unit/, integration/, contract/ subdirectories

<!--
  ============================================================================
  IMPORTANT: The tasks below are organized by user story to enable
  independent implementation, testing, and delivery of each MVP increment.

  Each user story provides value independently and can be demoed separately.
  ============================================================================-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic Rust workspace structure

- [x] T001 Create multi-crate Rust workspace structure with czi_core, czi_ipc, and czi_desktop crates
- [x] T002 Initialize Cargo.toml files with dependencies from plan.md (rust 1.75+, tokio, tree-sitter, petgraph, serde, tauri)
- [x] T003 [P] Configure development environment with rustfmt, clippy, and pre-commit hooks per constitutional requirements
- [x] T004 Create directory structure following plan.md architecture (src/{config,io,parser,graph,analysis}, tests/{unit,integration,contract})
- [x] T005 [P] Setup Tauri desktop application boilerplate with basic IPC configuration
- [x] T006 Configure project for cross-platform builds (macOS, Windows) as specified in technical context

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**Status**: Implementation exists but blocked by compilation errors

- [~] T007 Create base error handling system with Result<T, E> types for consistent error management across all modules (implemented, needs compilation fixes)
- [~] T008 [P] Implement logging infrastructure with structured logging for performance monitoring and debugging (implemented, needs compilation fixes)
- [x] T009 [P] Create configuration management system for JSON/YAML config files and environment variables (completed)
- [~] T010 [P] Setup Tree-sitter integration with language grammars for Java, JavaScript, Python, and Shell scripts (implemented, needs compilation fixes)
- [~] T011 [P] Implement petgraph-based dependency graph structure with node and edge types for code analysis (implemented, needs compilation fixes)
- [x] T012 [P] Create Git operations wrapper for repository cloning, fetching, and file system abstraction using libgit2 (completed)
- [~] T013 [P] Setup async task runtime with Tokio for concurrent file parsing and analysis operations (implemented, needs compilation fixes)
- [x] T014 Implement basic IPC command framework with Tauri for communication between frontend and backend (completed)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Configure Multi-Repository Analysis (Priority: P1) 🎯 MVP

**Goal**: Enable users to configure and validate Git repositories for analysis

**Independent Test**: Can be tested by creating a configuration with 2-3 Git repositories and verifying system access without errors

### Tests for User Story 1 (TDD First) ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL before implementation

- [x] T015 [P] [US1] Contract test for repository validation endpoint in tests/contract/test_repository_validation.rs
- [x] T016 [P] [US1] Integration test for repository configuration workflow in tests/integration/test_repository_config.rs
- [x] T017 [P] [US1] Unit test for RepositoryConfiguration entity validation in czi_core/src/config/tests.rs

### Implementation for User Story 1

**Status**: Backend implementation complete, frontend pending

- [x] T018 [P] [US1] Create RepositoryConfiguration entity model in czi_core/src/config/repository.rs with validation rules and state transitions
- [x] T019 [P] [US1] Create authentication types (AuthType, AuthConfig) in czi_core/src/config/auth.rs with support for None, SSHKey, Token, Basic methods
- [x] T020 [P] [US1] Implement Git repository validation service in czi_core/src/io/validator.rs with URL parsing and access testing
- [x] T021 [P] [US1] Create repository synchronization service in czi_core/src/io/sync.rs with shallow clone and fetch operations
- [x] T022 [P] [US1] Implement configuration persistence layer in czi_core/src/config/storage.rs for JSON/YAML config files
- [x] T023 [US1] Create repository management commands in czi_ipc/src/commands/repository.rs (list_repositories, add_repository, remove_repository, sync_repository)
- [x] T024 [US1] Implement Tauri IPC command handlers in czi_ipc/src/handlers/repository.rs that call core services and handle serialization
- [ ] T025 [US1] Create frontend repository configuration UI components in czi_desktop/src/components/RepositoryConfig.vue with forms for adding/editing repositories
- [ ] T026 [US1] Implement frontend repository list component in czi_desktop/src/components/RepositoryList.vue with status indicators and sync controls
- [ ] T027 [US1] Add authentication configuration UI in czi_desktop/src/components/AuthConfig.vue with secure credential handling
- [ ] T028 [US1] Implement repository validation UI in czi_desktop/src/components/RepositoryValidator.vue with real-time access testing

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Run Dead Code Analysis (Priority: P2)

**Goal**: Analyze configured repositories to build dependency graphs and identify unreachable code

**Independent Test**: Can be tested by running analysis on a known codebase with both active and dead code, then verifying accuracy

### Tests for User Story 2 (TDD First) ⚠️

- [ ] T029 [P] [US2] Contract test for analysis execution endpoint in tests/contract/test_analysis.rs
- [ ] T030 [P] [US2] Integration test for full analysis pipeline in tests/integration/test_analysis_pipeline.rs
- [ ] T031 [P] [US2] Unit test for dependency graph building in czi_core/src/graph/tests.rs

### Implementation for User Story 2

- [ ] T032 [P] [US2] Create ActiveRootNode entity model in czi_core/src/config/root_node.rs with RootNodeType enum and validation
- [ ] T033 [P] [US2] Implement CodeSymbol entity model in czi_core/src/parser/symbol.rs with SymbolType enum and language support
- [ ] T034 [P] [US2] Create DependencyEdge entity model in czi_core/src/graph/edge.rs with EdgeType enum and relationship types
- [ ] T035 [P] [US2] Implement Tree-sitter-based parser pipeline in czi_core/src/parser/pipeline.rs with concurrent parsing of multiple files
- [ ] T036 [P] [US2] Create multi-language symbol extraction in czi_core/src/parser/extractors/ with language-specific extractors for Java, JS, Python, Shell
- [ ] T037 [P] [US2] Implement dependency detection algorithms in czi_core/src/graph/detectors/ for calls, imports, implements, invokes, accesses
- [ ] T038 [P] [US2] Create semantic link detection in czi_core/src/graph/semantic_links.rs for MyBatis, stored procedures, and scheduler scripts
- [ ] T039 [P] [US2] Implement dependency graph builder in czi_core/src/graph/builder.rs with petgraph integration and node/edge creation
- [ ] T040 [P] [US2] Create reachability analysis engine in czi_core/src/analysis/reachability.rs with BFS algorithm starting from active root nodes
- [ ] T041 [P] [US2] Implement analysis results storage in czi_core/src/analysis/results.rs with ZombieCodeItem identification and metadata extraction
- [ ] T042 [P] [US2] Create graph serialization with bincode in czi_core/src/graph/serialization.rs for fast load/save operations
- [ ] T043 [P] [US2] Implement analysis orchestration service in czi_core/src/analysis/orchestrator.rs that coordinates parsing, graph building, and analysis
- [ ] T044 [US2] Create analysis commands in czi_ipc/src/commands/analysis.rs (run_analysis, get_analysis_status, get_analysis_results)
- [ ] T045 [US2] Implement Tauri analysis UI components in czi_desktop/src/components/AnalysisPanel.vue with progress tracking and status display
- [ ] T046 [US2] Create root node configuration UI in czi_desktop/src/components/RootNodeConfig.vue with symbol validation and type selection
- [ ] T047 [US2] Implement analysis progress visualization in czi_desktop/src/components/AnalysisProgress.vue with real-time updates and performance metrics

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Review and Act on Analysis Results (Priority: P3)

**Goal**: Enable users to review zombie code analysis results, explore dependencies, and make informed decisions

**Independent Test**: Can be tested by analyzing results for a zombie code item and verifying dependency path visualization

### Tests for User Story 3 (TDD First) ⚠️

- [ ] T048 [P] [US3] Contract test for dependency query endpoints in tests/contract/test_queries.rs
- [ ] T049 [P] [US3] Integration test for result visualization workflow in tests/integration/test_visualization.rs
- [ ] T050 [P] [US3] Unit test for dependency path algorithms in czi_core/src/analysis/path_finder.rs

### Implementation for User Story 3

- [ ] T051 [P] [US3] Create dependency query service in czi_core/src/analysis/queries.rs for query_dependencies and query_dependents operations
- [ ] T052 [P] [US3] Implement dependency path finding algorithm in czi_core/src/analysis/path_finder.rs with shortest path and isolation boundary detection
- [ ] T053 [P] [US3] Create Git history extraction service in czi_core/src/io/git_history.rs for last modification dates and contributor information
- [ ] T054 [P] [US3] Implement zombie code classification in czi_core/src/analysis/classifier.rs with ZombieType enum and confidence scoring
- [ ] T055 [P] [US3] Create report generation service in czi_core/src/reports/generator.rs with JSON export and filtering capabilities
- [ ] T056 [P] [US3] Implement filtering and search in czi_core/src/reports/filter.rs with support for repository, language, modification date filters
- [ ] T057 [P] [US3] Create query commands in czi_ipc/src/commands/queries.rs (query_dependencies, query_dependents, get_symbol_info)
- [ ] T058 [P] [US3] Implement report commands in czi_ipc/src/commands/reports.rs (get_zombie_report, export_json_report, filter_zombie_items)
- [ ] T059 [P] [US3] Create zombie code results table component in czi_desktop/src/components/ZombieCodeTable.vue with sorting, filtering, and selection
- [ ] T060 [P] [US3] Implement dependency graph visualization in czi_desktop/src/components/DependencyGraph.vue using D3.js or vis.js for interactive exploration
- [ ] T061 [P] [US3] Create symbol details panel in czi_desktop/src/components/SymbolDetails.vue with metadata, history, and dependency information
- [ ] T062 [P] [US3] Implement path analysis visualization in czi_desktop/src/components/PathAnalysis.vue showing dependency chains and isolation boundaries
- [ ] T063 [P] [US3] Add export functionality in czi_desktop/src/components/ReportExport.vue with JSON download and report generation

**Checkpoint**: All user stories should now be independently functional and testable

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Performance optimization, error handling, and quality improvements across all stories

- [ ] T064 [P] Performance optimization for large codebase analysis - implement incremental parsing and graph updates
- [ ] T065 [P] Memory optimization for million-node graphs - use memory-efficient data structures and lazy loading
- [ ] T066 [P] Implement comprehensive error handling with user-friendly error messages and recovery strategies
- [ ] T067 [P] Add comprehensive logging and monitoring for performance metrics (analysis time, memory usage, query response times)
- [ ] T068 [P] Implement configuration hot-reloading for development efficiency
- [ ] T069 [P] Create comprehensive test suite with integration tests covering cross-repository dependencies
- [ ] T070 [P] Add security audit for authentication credential storage and IPC communication
- [ ] T071 [P] Implement cross-platform packaging for macOS and Windows deployment
- [ ] T072 [P] Create user documentation and help system within the desktop application
- [ ] T073 Code quality review - ensure all code passes cargo clippy -- -D warnings and follows Rust best practices
- [ ] T074 Performance benchmarking against constitutional requirements (2min analysis, 50ms queries, 2GB memory limit)
- [ ] T075 Final integration testing with real multi-repository codebases to validate analysis accuracy and performance

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel or sequentially in priority order (US1 → US2 → US3)
  - Each story provides independent value without breaking previous stories
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational phase - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational phase - May use repository configuration from US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational phase - May integrate with US1/US2 but should work independently

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models/Entities before services
- Services before commands/endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together (TDD first):
Task: "Contract test for repository validation endpoint in tests/contract/test_repository_validation.rs"
Task: "Integration test for repository configuration workflow in tests/integration/test_repository_config.rs"
Task: "Unit test for RepositoryConfiguration entity validation in czi_core/src/config/tests.rs"

# Launch all models for User Story 1 together:
Task: "Create RepositoryConfiguration entity model in czi_core/src/config/repository.rs"
Task: "Create authentication types (AuthType, AuthConfig) in czi_core/src/config/auth.rs"

# Launch all services for User Story 1 together:
Task: "Implement Git repository validation service in czi_core/src/io/validator.rs"
Task: "Create repository synchronization service in czi_core/src/io/sync.rs"
Task: "Implement configuration persistence layer in czi_core/src/config/storage.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo repository configuration functionality

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (Repository Config MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo (Analysis Capability)
4. Add User Story 3 → Test independently → Deploy/Demo (Results Review)
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers (Core Team for Rust, UI Team for Web/Tauri):

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - **Core Team (Rust)**: User Story 2 backend (analysis engine)
   - **UI Team (Web/Tauri)**: User Story 1 frontend (repository configuration)
3. Stories complete and integrate independently
4. Final integration and polish phase

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Constitutional requirements (TDD, performance, modularity) must be followed throughout
- Performance requirements (2min analysis, 50ms queries, 2GB memory) must be validated during Phase 6
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence