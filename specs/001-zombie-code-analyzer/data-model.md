# Data Model: CodeZombiesInvestigator (CZI)

**Feature**: CodeZombiesInvestigator (CZI) - Zombie Code Analysis System
**Branch**: `001-zombie-code-analyzer`
**Date**: 2025-12-06

## Entity Definitions

### RepositoryConfiguration
Represents a Git repository to be analyzed.

**Fields**:
- `id: String` - Unique identifier for the repository (format: `repo_<name>`)
- `name: String` - Human-readable repository name
- `url: String` - Git repository URL (HTTPS or SSH)
- `local_path: String` - Local filesystem path for cached repository
- `branch: String` - Branch to analyze (default: "main")
- `auth_type: AuthType` - Authentication method (None, SSHKey, Token, Basic)
- `auth_config: AuthConfig` - Authentication configuration details
- `last_sync: DateTime` - Timestamp of last successful sync
- `status: RepositoryStatus` - Current repository status (Active, Syncing, Error, Disabled)

**Validation Rules**:
- `name` must be unique across all configured repositories
- `url` must be a valid Git URL format
- `local_path` must be writable and have sufficient disk space
- `branch` must exist in the remote repository

**State Transitions**:
- Disabled → Active: Repository enabled for analysis
- Active → Syncing: Repository sync initiated
- Syncing → Active: Sync completed successfully
- Syncing → Error: Sync failed
- Any → Disabled: Repository disabled by user

---

### ActiveRootNode
Represents system entry points that define the analysis starting points.

**Fields**:
- `id: String` - Unique identifier (format: `root_<repo>_<type>_<hash>`)
- `repository_id: String` - Foreign key to RepositoryConfiguration
- `node_type: RootNodeType` - Type of root node (Controller, Scheduler, Listener)
- `symbol_path: String` - Full path to the symbol (e.g., "com.example.UserController.createUser")
- `file_path: String` - Relative file path within repository
- `line_number: Integer` - Line number where symbol is defined
- `metadata: Map<String, String>` - Additional metadata (annotations, signatures)
- `created_at: DateTime` - When this root node was configured
- `created_by: String` - User who configured this root node

**Validation Rules**:
- `symbol_path` must reference an existing symbol in the repository
- `node_type` must be valid for the programming language
- `file_path` must exist in the configured repository
- Each repository can have multiple root nodes

**Root Node Types**:
- `Controller`: HTTP/API endpoint methods (e.g., @RequestMapping in Spring)
- `Scheduler`: Scheduled job entry points (e.g., cron jobs, Quartz)
- `Listener`: Message queue listeners (e.g., JMS, RabbitMQ)
- `Main`: Application entry points (main methods, startup scripts)

---

### CodeSymbol
Represents an identifiable code element (function, class, file, etc.).

**Fields**:
- `id: String` - Unique identifier (format: `[RepoName]::[FilePath]::[SymbolName]`)
- `repository_id: String` - Foreign key to RepositoryConfiguration
- `symbol_type: SymbolType` - Type of symbol (Function, Class, Struct, File, etc.)
- `name: String` - Symbol name
- `qualified_name: String` - Fully qualified name including namespace/package
- `file_path: String` - Relative path within repository
- `line_number: Integer` - Line number where symbol starts
- `line_end: Integer` - Line number where symbol ends
- `language: String` - Programming language (Java, JavaScript, Python, Shell)
- `visibility: Visibility` - Symbol visibility (Public, Private, Protected, Internal)
- `signature: String` - Method signature or type definition
- `documentation: String` - Extracted documentation comments
- `metadata: Map<String, String>` - Language-specific metadata

**Validation Rules**:
- `id` must be globally unique across all repositories
- `symbol_type` must be consistent with the programming language
- `file_path` must reference an existing file in the repository
- `line_number` and `line_end` must be valid within the file

**Symbol Types**:
- `File`: Source code file
- `Function`: Function or method
- `Class`: Class definition
- `Struct`: Structure definition
- `Interface`: Interface definition
- `Enum`: Enumeration
- `Variable`: Global or class-level variable
- `Constant`: Constant definition

---

### DependencyEdge
Represents a relationship between two code symbols.

**Fields**:
- `id: String` - Unique identifier for the edge
- `source_symbol_id: String` - Foreign key to source CodeSymbol
- `target_symbol_id: String` - Foreign key to target CodeSymbol
- `edge_type: EdgeType` - Type of dependency relationship
- `strength: EdgeStrength` - Strength of dependency (Strong, Weak, Dynamic)
- `context: String` - Context where dependency occurs (file, line number)
- `metadata: Map<String, String>` - Additional relationship metadata

**Validation Rules**:
- Both `source_symbol_id` and `target_symbol_id` must reference existing symbols
- No self-references (source ≠ target)
- Edge type must be valid for the programming language context

**Edge Types**:
- `Calls`: Method/function invocation
- `Imports`: Module/package import
- `Implements`: Interface implementation (Java interfaces → MyBatis XML)
- `Invokes`: Stored procedure invocation
- `Triggers`: Scheduler script triggering stored procedures
- `Accesses`: Database table access (read/write)
- `Inherits`: Class inheritance
- `References`: Variable or constant reference

---

### DependencyGraph
Container for the complete dependency graph across all repositories.

**Fields**:
- `id: String` - Unique graph identifier (format: `graph_<timestamp>`)
- `repository_ids: List<String>` - List of repository IDs included in graph
- `created_at: DateTime` - When graph was built
- `node_count: Integer` - Total number of nodes in graph
- `edge_count: Integer` - Total number of edges in graph
- `build_duration_ms: Integer` - Time taken to build the graph
- `metadata: Map<String, String>` - Build metadata (versions, configuration)

**Validation Rules**:
- Must contain at least one repository
- Node and edge counts must be consistent with actual data
- Build duration must be positive

---

### AnalysisResult
Results from reachability analysis identifying zombie code.

**Fields**:
- `id: String` - Unique result identifier
- `graph_id: String` - Foreign key to DependencyGraph
- `analysis_type: String` - Type of analysis (Reachability, Impact, etc.)
- `started_at: DateTime` - Analysis start time
- `completed_at: DateTime` - Analysis completion time
- `duration_ms: Integer` - Analysis duration in milliseconds
- `total_symbols: Integer` - Total symbols analyzed
- `reachable_symbols: Integer` - Number of reachable symbols
- `zombie_symbols: Integer` - Number of unreachable (zombie) symbols
- `status: AnalysisStatus` - Analysis status (Running, Completed, Failed)
- `error_message: String` - Error details if analysis failed

**Validation Rules**:
- `completed_at` must be after `started_at`
- Symbol counts must be consistent (total = reachable + zombie)
- Duration must match time difference

---

### ZombieCodeItem
Individual unreachable code item identified by analysis.

**Fields**:
- `id: String` - Unique identifier
- `result_id: String` - Foreign key to AnalysisResult
- `symbol_id: String` - Foreign key to CodeSymbol
- `zombie_type: ZombieType` - Classification of zombie code
- `isolation_distance: Integer` - Minimum steps to nearest active node
- `last_modified: DateTime` - Last Git commit date for this symbol
- `primary_contributor: String` - Main contributor to this code
- `removal_confidence: Confidence` - Confidence level for safe removal
- `context_path: List<String>` - Path to nearest active root node
- `notes: String` - Additional analysis notes

**Zombie Types**:
- `DeadCode`: Code with no incoming dependencies
- `Orphaned`: Code only referenced by other zombie code
- `Unreachable`: Code not reachable from any active root
- `Deprecated`: Code marked as obsolete but not removed

**Confidence Levels**:
- `High`: Safe to remove, no active dependencies
- `Medium`: Likely safe, minimal indirect dependencies
- `Low`: Uncertain, requires manual review
- `None`: Do not remove, may have hidden dependencies

---

### IPCCommand
Available commands for frontend-backend communication.

**Commands**:

#### Repository Management
- `list_repositories() -> List<RepositoryConfiguration>`
- `add_repository(config: RepositoryConfiguration) -> Result<String>`
- `remove_repository(id: String) -> Result<()>`
- `sync_repository(id: String) -> Result<()>`
- `validate_repository(url: String) -> Result<RepositoryInfo>`

#### Root Node Management
- `list_root_nodes(repository_id: String) -> List<ActiveRootNode>`
- `add_root_node(node: ActiveRootNode) -> Result<String>`
- `remove_root_node(id: String) -> Result<()>`
- `validate_root_node(path: String) -> Result<SymbolInfo>`

#### Analysis Operations
- `run_analysis(config: AnalysisConfig) -> Result<String>` # Returns analysis_id
- `get_analysis_status(id: String) -> AnalysisStatus`
- `get_analysis_results(id: String) -> Result<AnalysisResult>`
- `list_analyses(repository_ids: List<String>) -> List<AnalysisResult>`

#### Query Operations
- `query_dependencies(symbol_id: String) -> List<DependencyPath>`
- `query_dependents(symbol_id: String) -> List<DependencyPath>`
- `get_symbol_info(symbol_id: String) -> Result<CodeSymbol>`
- `get_zombie_report(analysis_id: String) -> Result<ZombieCodeReport>`

#### Report Generation
- `export_json_report(analysis_id: String) -> Result<String>` # Returns file path
- `filter_zombie_items(filters: ZombieFilters) -> List<ZombieCodeItem>`

## Data Flow

### Configuration Flow
1. User provides repository URLs and authentication
2. System validates repository access and creates RepositoryConfiguration
3. User defines ActiveRootNodes for each repository
4. System validates root nodes exist in the codebase

### Analysis Flow
1. System clones/fetches repositories to local cache
2. Parser analyzes source files and creates CodeSymbols
3. Dependency analyzer creates DependencyEdges between symbols
4. Graph builder constructs DependencyGraph
5. Analysis engine runs reachability algorithm from ActiveRootNodes
6. Results stored as AnalysisResult with ZombieCodeItems

### Query Flow
1. Frontend requests symbol information via IPC
2. Backend queries dependency graph for related symbols
3. Results serialized and returned to frontend
4. Frontend displays dependency relationships and zombie code details

## Validation Rules

### Cross-Entity Validation
- All symbol IDs in edges must exist in the symbols collection
- Repository IDs must be consistent across all entities
- Analysis results must reference valid graphs
- Zombie items must reference valid symbols and analysis results

### Data Integrity
- No circular dependencies in the graph (detected and reported)
- Symbol paths must be unique within each repository
- Edge relationships must be valid for the programming language
- Timestamps must be consistent (created before modified, etc.)