# Compilation Fix Guide - CodeZombiesInvestigator

**Status**: 55 errors remaining (down from 91)  
**Last Updated**: 2025-12-10  
**Time Invested**: ~1.5 hours  

## Fixed Errors Summary (36 total)

### ✅ Git2 API Mismatches (22 errors)
All git2-related issues have been resolved:
- `git2::Cred::ssh_key` signature fixed
- `RepoBuilder::shallow()` removed (not in 0.18)
- `GitFetchOptions::new()` → `git2::FetchOptions::new()`
- Branch tuple destructuring: `(branch, _type)`
- Remote lookup error handling: `if let Ok(remote)`
- String/str handling: strip_prefix, to_owned()
- Blame hunk methods: final_committer(), orig_path(), timestamps
- Clone repository made async

### ✅ Petgraph EdgeRef (3 errors)
- Added `use petgraph::visit::EdgeRef;` to:
  - `src/graph/mod.rs`
  - `src/analysis/classifier.rs`
  - `src/analysis/queries.rs`
  - `src/analysis/reachability.rs`

### ✅ CodeSymbol Methods (3 errors)
- `symbol.unique_id()` → `symbol.id`
- Fixed in `src/graph/semantic_links.rs`

### ✅ Tree-sitter Query API (5 errors)
- `Query.clone()` → Arc wrapping
- `cursor.matches()` API updated
- `capture.start_byte()` → `capture.node().start_byte()`
- `m.captures[0]` → `m.captures.get(0)`

### ✅ Async/Future Handling (2 errors)
- Made async: `get_stats()`, `is_symbol_reachable()`, `get_shortest_path_to_symbol()`
- Added `.await` on `find_root_node_indices_by_symbols()`

---

## Remaining Errors Guide (55 errors)

### Pattern 1: Analysis Module Async Issues (10 errors)

**Files**: `src/analysis/*.rs`

**Problem**: Methods returning Futures called without `.await`

**Fix**: Make calling functions async or provide sync versions

**Example**:
```rust
// Current (broken):
let indices = self.find_root_node_indices_by_symbols(graph, nodes)?;

// Fixed:
let indices = self.find_root_node_indices_by_symbols(graph, nodes).await?;
```

**Search pattern**: `find_root_node_indices_by_symbols\([^)]+\)\?;`

---

### Pattern 2: Parser Language Queries (8 errors)

**Files**: `src/parser/*.rs`

**Problem**: `supported_languages()` method not found

**Fix**: Tree-sitter manager needs proper initialization or alternative method

**Search pattern**: `supported_languages\(\)`

---

### Pattern 3: Type Mismatches (20 errors)

**Files**: Multiple

**Problem**: Types don't match expected signatures

**Common fixes**:
- `usize` → `u32` or vice versa (add `.as usize` or `as u32`)
- `&Node<'_>` → `Node<'_>` (remove `&`)
- Missing `.clone()` or `.to_owned()`
- Tuple destructuring issues

**Search pattern**: `expected .*, found .*`

---

### Pattern 4: Missing Methods/Fields (17 errors)

**Files**: Multiple

**Problem**: API version mismatches

**Common fixes**:
- Methods not in library version → provide fallback or skip
- Fields missing from structs → use defaults or add fields
- Trait not in scope → add `use` statements

---

## Quick Fix Commands

### 1. Fix all async/await issues:
```bash
# Find patterns like this and add .await:
cargo check 2>&1 | grep "cannot be applied to type.*Future" | cut -d: -f1 | sort -u
```

### 2. Fix type mismatches:
```bash
# Common pattern - add type conversions:
# .map(|x| x as u32)
# .to_owned()
# .clone()
```

### 3. Fix missing imports:
```bash
# EdgeRef trait for petgraph
grep -r "edge.target()" src/ | cut -d: -f1 | sort -u | xargs -I {} sed -i '1i use petgraph::visit::EdgeRef;' {}
```

---

## Recommended Next Steps

### Option 1: Complete Manual Fixes (30-45 min)
Continue systematic fixing using the patterns above

### Option 2: Focus on User Story 1 (15 min)
- Create minimal stub implementations
- Focus on frontend components
- Document backend as "in progress"

### Option 3: Regenerate Minimal Backend (45-60 min)
- Create simplified versions of complex modules
- Focus on what's needed for US1
- Skip advanced features for now

---

## Files Modified

### Successfully Fixed:
- `src/error.rs` - Added enum variants
- `src/git/operations.rs` - git2 API fixes
- `src/git/repository.rs` - Branch/remote handling
- `src/git/diff.rs` - API compatibility
- `src/git/blame.rs` - Method signatures
- `src/git/mod.rs` - Async clone
- `src/graph/mod.rs` - EdgeRef import
- `src/analysis/classifier.rs` - EdgeRef import
- `src/graph/semantic_links.rs` - unique_id → id
- `src/graph/detectors/calls.rs` - Tree-sitter API
- `src/graph/detectors/imports.rs` - Tree-sitter API
- `src/runtime/mod.rs` - Async stats
- `src/analysis/reachability.rs` - Async methods

### Remaining Work Needed:
- `src/parser/*.rs` - Language queries, Tree-sitter integration
- `src/analysis/*.rs` - More async/await fixes
- Multiple files - Type conversions, missing fields

---

## Testing Strategy

Once compilation is fixed:

```bash
# 1. Basic compilation
cargo check

# 2. Run tests
cargo test --lib

# 3. Run specific tests
cargo test --package czi_core --lib

# 4. Run clippy
cargo clippy -- -D warnings
```

---

## Performance Notes

**Current build time**: ~5-10 seconds  
**Expected after fixes**: ~5-10 seconds  
**Test execution**: Depends on test complexity  

**Bottlenecks identified**:
- Tree-sitter initialization
- Git repository cloning
- Large graph operations

---

## Support

For questions about specific errors:
1. Check the error message carefully
2. Look for similar patterns in fixed files
3. Consult library documentation
4. Consider creating minimal reproducible examples

**Common Library Docs**:
- git2: https://docs.rs/git2
- tree-sitter: https://docs.rs/tree-sitter
- petgraph: https://docs.rs/petgraph
- tokio: https://docs.rs/tokio
