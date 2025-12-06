# Feature Specification: CodeZombiesInvestigator (CZI) - Zombie Code Analysis System

**Feature Branch**: `001-zombie-code-analyzer`
**Created**: 2025-12-06
**Status**: Draft
**Input**: User description: "需求来自docs/req.md"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Configure Multi-Repository Analysis (Priority: P1)

As a technical lead, I want to configure multiple Git repositories for analysis so that I can identify dead code across our entire system architecture.

**Why this priority**: This is the foundational step that enables all other functionality. Without proper repository configuration, no analysis can begin.

**Independent Test**: Can be tested by creating a configuration with 2-3 Git repositories and verifying that the system can access and analyze them without errors.

**Acceptance Scenarios**:

1. **Given** I have Git repository URLs with authentication credentials, **When** I input them through the configuration interface, **Then** the system validates access and stores the configuration persistently.

2. **Given** I have configured multiple repositories, **When** I specify active root nodes (controller methods, scheduler scripts), **Then** the system accepts and validates these entry points for analysis.

3. **Given** repositories require authentication, **When** I provide credentials through secure configuration, **Then** the system can access private repositories without exposing credentials.

---

### User Story 2 - Run Dead Code Analysis (Priority: P2)

As a developer, I want to run automated analysis to identify unreachable code so that I can safely remove dead code during refactoring.

**Why this priority**: This delivers the core value proposition - identifying which code can be safely deleted, reducing technical debt and system complexity.

**Independent Test**: Can be tested by running analysis on a known codebase with both active and dead code, then verifying the accuracy of identified zombie code against expected results.

**Acceptance Scenarios**:

1. **Given** I have configured repositories and active root nodes, **When** I initiate analysis, **Then** the system builds a complete dependency graph and identifies unreachable code within 2 minutes for 500K LOC.

2. **Given** the analysis has completed, **When** I review the results, **Then** I can see clear identification of dead code with repository, file path, and last modification information.

3. **Given** complex cross-repository dependencies exist, **When** analysis runs, **Then** the system correctly identifies implicit links (MyBatis, stored procedures, scheduler scripts) and marks truly unreachable code.

---

### User Story 3 - Review and Act on Analysis Results (Priority: P3)

As a code reviewer, I want to visualize dependency relationships and review detailed reports so that I can make informed decisions about code removal during pull requests.

**Why this priority**: This enables safe code removal decisions and provides the detailed information needed for code review processes.

**Independent Test**: Can be tested by analyzing results for a zombie code item, viewing its dependency chain, and verifying the path analysis shows the isolation boundary clearly.

**Acceptance Scenarios**:

1. **Given** analysis has identified zombie code, **When** I click on a specific item, **Then** I can see its complete dependency chain and understand why it's considered unreachable.

2. **Given** I need to filter results, **When** I apply filters (by repository, language, modification date), **Then** the system shows only relevant zombie code items with accurate filtering.

3. **Given** I want to export findings, **When** I request the standardized JSON report, **Then** I receive complete data including symbol IDs, types, repositories, and modification history.

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

- What happens when a repository becomes temporarily unavailable during analysis?
- How does the system handle circular dependencies between modules?
- What occurs when active root nodes reference non-existent code paths?
- How are analysis results handled when the codebase changes during long-running analysis?
- What happens when multiple users initiate analysis simultaneously on the same repositories?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST support configuration of multiple Git repositories (minimum 2) with authentication credentials through JSON/YAML files or UI interface.

- **FR-002**: System MUST perform asynchronous Git operations including shallow clone and fetch to keep local code cache synchronized with remote repositories.

- **FR-003**: System MUST allow users to define and persist active root nodes including scheduler script paths, Java controller annotation methods, and message queue listener signatures.

- **FR-004**: System MUST parse multiple programming languages concurrently using Tree-sitter to generate abstract syntax trees for dependency analysis.

- **FR-005**: System MUST create unique identifiers for code symbols using format `[RepoName]::[Path]::[Symbol]` to enable cross-repository referencing.

- **FR-006**: System MUST identify and create dependency edges for method calls, imports, MyBatis implementations, stored procedure invocations, and database table accesses.

- **FR-007**: System MUST implement reachability analysis using breadth-first search (BFS) algorithm starting from active root nodes to identify unreachable code.

- **FR-008**: System MUST provide IPC APIs for dependency queries including `query_dependents(SymbolID)` and `query_dependencies(SymbolID)` operations.

- **FR-009**: System MUST extract Git history data (last modification date, primary contributor) for all identified zombie code items.

- **FR-010**: System MUST generate standardized JSON reports containing zombie code identification with symbol ID, type, repository name, and modification metadata.

- **FR-011**: System MUST present analysis results in tabular format with filtering capabilities by repository, programming language, and modification date.

- **FR-012**: System MUST support interactive path analysis visualization showing dependency chains from zombie code to nearest active nodes or isolation boundaries.

### Key Entities *(include if feature involves data)*

- **Repository Configuration**: Contains Git URL, local path, authentication credentials, and branch specification for each codebase to analyze.

- **Active Root Node**: Represents system entry points including controller methods, scheduler scripts, and message listeners that define the analysis starting points.

- **Code Symbol**: Represents identifiable code elements (functions, classes, files) with unique IDs, types, and repository associations.

- **Dependency Edge**: Represents relationships between code symbols including calls, imports, implements, invokes, and accesses relationships.

- **Zombie Code Report**: Contains analysis results with unreachable code items, their metadata, and dependency information for cleanup decisions.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: Users can configure multi-repository analysis and identify dead code within 5 minutes of starting the tool for the first time.

- **SC-002**: The system accurately identifies zombie code with less than 5% false positive rate when compared against manual code review by senior developers.

- **SC-003**: Analysis completes within 2 minutes for codebases up to 500,000 lines of code while maintaining system responsiveness.

- **SC-004**: 90% of users can successfully interpret analysis results and identify specific code sections for removal without additional training.

- **SC-005**: The system reduces time spent on dead code identification during code reviews by 75% compared to manual inspection methods.

- **SC-006**: Dependency queries respond within 50 milliseconds, enabling interactive exploration of code relationships without perceptible delays.

## Assumptions

- Users have basic understanding of their system architecture and can identify legitimate entry points
- Git repositories are accessible and contain valid source code in supported programming languages
- Analysis runs on development machines with sufficient memory (8GB+) for large codebases
- Users have appropriate permissions to access and analyze the target repositories
- The system will analyze Java, JavaScript, Python, and shell script codebases initially
- Repository sizes are typically enterprise-scale (100K-1M lines of code) rather than massive monorepos
