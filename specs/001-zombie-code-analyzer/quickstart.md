# Quickstart Guide: CodeZombiesInvestigator (CZI)

**Feature**: CodeZombiesInvestigator (CZI) - Zombie Code Analysis System
**Branch**: `001-zombie-code-analyzer`
**Date**: 2025-12-06

## Overview

CodeZombiesInvestigator (CZI) is a high-performance static analysis tool that identifies dead code in multi-repository codebases. This guide will help you get started with analyzing your repositories to find unreachable code.

## Prerequisites

### System Requirements
- **Operating System**: macOS 10.15+, Windows 10+, or Linux (Ubuntu 20.04+)
- **Memory**: 8GB minimum, 16GB recommended for large codebases
- **Disk Space**: 10GB+ free space (depends on repository sizes)
- **Git**: Version 2.20 or later

### Required Software
- **Rust**: Version 1.75 or later ([Installation guide](https://www.rust-lang.org/tools/install))
- **Node.js**: Version 16 or later (for frontend development)
- **Tauri CLI**: Install via `cargo install tauri-cli`

## Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd CodeZombiesInvestigator_V0
```

### 2. Install Dependencies
```bash
# Install Rust dependencies
cargo build

# Install frontend dependencies
npm install
```

### 3. Verify Installation
```bash
# Run tests to verify everything works
cargo test

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings
```

## Configuration

### Repository Configuration

CZI uses JSON or YAML configuration files to specify repositories to analyze.

**Example configuration (config.json)**:
```json
{
  "repositories": [
    {
      "name": "backend-service",
      "url": "https://github.com/company/backend-service.git",
      "branch": "main",
      "auth_type": "token",
      "local_path": "./cache/backend-service"
    },
    {
      "name": "frontend-app",
      "url": "https://github.com/company/frontend-app.git",
      "branch": "develop",
      "auth_type": "ssh_key",
      "local_path": "./cache/frontend-app"
    }
  ]
}
```

**Authentication Methods**:
- **None**: Public repositories without authentication
- **Token**: Personal access token (GitHub, GitLab)
- **SSH Key**: SSH key-based authentication
- **Basic**: Username/password authentication (not recommended)

### Active Root Nodes

Define entry points that should be considered "active" (reachable):

```json
{
  "active_root_nodes": [
    {
      "repository_id": "backend-service",
      "node_type": "controller",
      "symbol_path": "com.example.UserController.createUser",
      "file_path": "src/main/java/com/example/UserController.java",
      "line_number": 45
    },
    {
      "repository_id": "backend-service",
      "node_type": "scheduler",
      "symbol_path": "com.example.DailyReportJob",
      "file_path": "src/main/java/com/example/DailyReportJob.java",
      "line_number": 12
    }
  ]
}
```

**Root Node Types**:
- **Controller**: HTTP/API endpoints
- **Scheduler**: Scheduled jobs/cron jobs
- **Listener**: Message queue listeners
- **Main**: Application entry points

## Usage

### 1. Configure Repositories

Option A: Using Configuration File
```bash
# Place your config.json in the project root
# CZI will automatically detect it
```

Option B: Using the CZI Desktop Application
1. Launch CZI: `cargo tauri dev`
2. Click "Add Repository"
3. Enter repository details
4. Click "Validate" to test access
5. Click "Save" to add to configuration

### 2. Define Active Root Nodes

1. Navigate to "Root Nodes" tab
2. Click "Add Root Node"
3. Select repository from dropdown
4. Enter symbol path and file location
5. Click "Validate" to ensure the symbol exists
6. Save the configuration

### 3. Run Analysis

```bash
# Using CLI
cargo run --bin czi-cli -- --config config.json analyze

# Using Desktop Application
1. Click "Analysis" tab
2. Click "New Analysis"
3. Select repositories to include
4. Click "Start Analysis"
5. Monitor progress in real-time
```

### 4. Review Results

Analysis results include:

**Summary Statistics**:
- Total symbols analyzed
- Reachable symbols
- Zombie code symbols
- Analysis duration

**Detailed Report**:
```json
{
  "zombie_items": [
    {
      "symbol_id": "backend-service::src/old/UserService.java::unusedMethod",
      "zombie_type": "dead_code",
      "last_modified": "2023-06-15",
      "primary_contributor": "john.doe@company.com",
      "removal_confidence": "high",
      "isolation_distance": 5
    }
  ]
}
```

**Filtering Options**:
- By repository
- By programming language
- By last modification date
- By zombie type
- By removal confidence level

## Common Workflows

### Workflow 1: Initial Analysis of New System

1. **Configure repositories**: Add all relevant repositories to configuration
2. **Identify active root nodes**: Work with team to identify all entry points
3. **Run full analysis**: Analyze all repositories together
4. **Review results**: Focus on high-confidence, high-impact zombie code
5. **Create cleanup plan**: Prioritize code removal based on confidence and impact

### Workflow 2: Incremental Analysis

1. **Use previous configuration**: Load existing config
2. **Run incremental analysis**: Only analyze changed files
3. **Review deltas**: Check for new zombie code since last analysis
4. **Update documentation**: Document decisions about each zombie item

### Workflow 3: Code Review Assistance

1. **Filter by recent changes**: Show zombie code modified in last X days
2. **Check pull request impact**: Verify removal won't break dependencies
3. **Review confidence levels**: Focus on high-confidence items first
4. **Export report**: Generate report for code review discussion

## Performance Tips

### For Large Codebases (500K+ LOC)

1. **Use shallow cloning**: Reduces initial download time
```bash
git clone --depth 1 <repository-url>
```

2. **Enable incremental analysis**: Only re-analyze changed files
```json
{
  "analysis_mode": "incremental",
  "cache_enabled": true
}
```

3. **Parallel processing**: CZI automatically uses all available CPU cores

4. **Memory considerations**: For million-node graphs, 16GB+ RAM recommended

### For Multiple Repositories

1. **Analyze related repositories together**: Ensures cross-repository dependencies are detected
2. **Use consistent naming**: Helps identify related symbols across repos
3. **Synchronize branches**: Ensure all repositories are on the same version/tag

## Troubleshooting

### Repository Access Issues

**Problem**: "Authentication failed"
- **Solution**: Verify credentials in auth_config, use tokens instead of passwords

**Problem**: "Repository not found"
- **Solution**: Check URL and access permissions

**Problem**: "Shallow clone failed"
- **Solution**: Ensure Git version supports shallow clones

### Analysis Issues

**Problem**: "Out of memory"
- **Solution**: Increase available RAM or run analysis on smaller subset of repos

**Problem**: "Symbol not found"
- **Solution**: Verify active root node paths are correct and code has been parsed

**Problem**: "Circular dependency detected"
- **Solution**: Normal for large codebases; CZI will detect and report these

### Performance Issues

**Problem**: "Analysis taking too long"
- **Solution**: Enable incremental mode, check disk I/O, ensure sufficient RAM

**Problem**: "High memory usage"
- **Solution**: Normal for large graphs; consider analyzing repos separately

## Best Practices

### Repository Organization
- Use consistent directory structures across repositories
- Follow language-specific naming conventions
- Keep repositories reasonably sized (ideally < 1M LOC each)

### Active Root Node Configuration
- Be comprehensive: include all entry points (controllers, schedulers, listeners)
- Keep configuration in version control alongside code
- Review and update root nodes after significant refactoring

### Analysis Frequency
- Run initial analysis before major refactoring
- Run incremental analysis after each release
- Schedule regular full analyses (monthly/quarterly)

### Zombie Code Handling
- Start with high-confidence, low-risk removals
- Review medium-confidence items with team
- Document decisions for future reference
- Use gradual rollout for large removals

## Example: Analyzing a Java Spring Boot Application

### Step 1: Configure Repository
```json
{
  "repositories": [
    {
      "name": "spring-app",
      "url": "https://github.com/company/spring-app.git",
      "branch": "main",
      "auth_type": "token"
    }
  ]
}
```

### Step 2: Define Active Root Nodes
```json
{
  "active_root_nodes": [
    {
      "repository_id": "spring-app",
      "node_type": "controller",
      "symbol_path": "com.example.controller.UserController",
      "file_path": "src/main/java/com/example/controller/UserController.java",
      "metadata": {
        "annotations": "@RestController,@RequestMapping"
      }
    },
    {
      "repository_id": "spring-app",
      "node_type": "scheduler",
      "symbol_path": "com.example.jobs.DailyCleanup",
      "file_path": "src/main/java/com/example/jobs/DailyCleanup.java",
      "metadata": {
        "cron": "0 0 2 * * ?"
      }
    }
  ]
}
```

### Step 3: Run Analysis
```bash
cargo run -- analyze --config config.json --output results.json
```

### Step 4: Review Results
```bash
# View summary
cat results.json | jq '.summary'

# View zombie items
cat results.json | jq '.zombie_items[] | select(.removal_confidence == "high")'

# Export to CSV
cat results.json | jq -r '.zombie_items[] | [.symbol_id, .zombie_type, .removal_confidence] | @csv'
```

## Next Steps

After completing your first analysis:

1. **Review zombie items**: Prioritize based on confidence and impact
2. **Plan cleanup**: Create refactoring plan with team
3. **Implement gradually**: Start with safe, high-confidence removals
4. **Re-run analysis**: Verify cleanup didn't introduce new zombie code
5. **Document findings**: Share results with development team
6. **Schedule regular analysis**: Set up periodic zombie code audits

## Support

- **Documentation**: See `/docs` directory for detailed guides
- **Issues**: Report bugs via GitHub Issues
- **Discussions**: Join GitHub Discussions for questions
- **Constitution**: Review project constitution for development guidelines

## Additional Resources

- [Feature Specification](./spec.md)
- [Implementation Plan](./plan.md)
- [Data Model](./data-model.md)
- [API Contracts](./contracts/openapi.yaml)
- [Project Constitution](../../.specify/memory/constitution.md)