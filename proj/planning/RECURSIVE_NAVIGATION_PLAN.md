# Recursive Navigation Planning Document

**Date:** 2026-02-06  
**Context:** Phase 3 Planning - Advanced Features  
**Status:** PLANNING  
**Priority:** HIGH (enables scaling to multi-file mindmaps)

---

## Executive Summary

Implement recursive navigation and cross-file support to extend mindmap-cli from single-file to multi-file knowledge graphs. This enables scaling beyond ~100 nodes by splitting domains while maintaining unified navigation and relationship queries.

**Key Metric:** Align with Node [14] Core Priorities
- **Security** (prevent path traversal, infinite loops)
- **Correctness** (validate all external references)
- **Robustness** (handle missing files gracefully)
- **Maintainability** (clean API, testable)
- **Speed** (lazy loading, caching)
- **Visuals** (clear output formatting)

---

## Current State Analysis

### Existing Implementation (Already In Place)
✅ **Reference Enum with External variant**
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Reference {
    Internal(u32),
    External(u32, String),  // <-- Already defined!
}
```

✅ **Parsing for external refs in `[id](./path.md)` format**
- Located in `extract_refs_from_str()` function
- Already extracts path and ID separately
- Detects `](` pattern and parses path until `)`

✅ **Partial Integration**
- Node struct includes references field
- lint command partially validates external refs

### Current Limitations
❌ **No recursive navigation**
- `show`, `refs`, `links`, `relationships`, `graph` don't follow external files
- No file loading/caching mechanism
- No `--follow` or `--recursive` flags

❌ **No external ref validation in commands**
- External refs parsed but not resolved
- Missing files not detected at command level
- Graph generation ignores external refs

❌ **No depth/cycle protection**
- Potential for infinite loops if files reference each other
- No visited file tracking
- No recursion depth limits

---

## Proposed Architecture

### 1. MindmapCache - File Loading & Caching

```rust
/// Manages loading and caching of external mindmap files
pub struct MindmapCache {
    /// Cache of loaded mindmaps: path -> Mindmap
    cache: HashMap<PathBuf, Mindmap>,
  /// Canonicalized workspace root for safety checks
  workspace_root: PathBuf,
    /// Max file size to load (default: 10MB)
    max_file_size: u64,
    /// Max recursion depth (default: 50)
    max_depth: usize,
}

impl MindmapCache {
    pub fn new(workspace_root: PathBuf) -> Self { ... }
    
    /// Load a mindmap, caching result
    pub fn load(&mut self, base_file: &Path, relative: &str) -> Result<&Mindmap> { ... }
    
    /// Resolve relative path to canonical path, rooted at the current file
    fn resolve_path(&self, base_file: &Path, relative: &str) -> Result<PathBuf> { ... }
    
    /// Optional: clear cache between runs
    pub fn clear_cache(&mut self) { ... }
}
```

### 2. Navigation Context - Track Depth & Visited Files

```rust
/// Context for recursive navigation operations
pub struct NavigationContext {
    /// Current recursion depth
    depth: usize,
    /// Max recursion depth allowed
    max_depth: usize,
    /// Visited files to detect cycles
    visited: HashSet<PathBuf>,
}

impl NavigationContext {
    pub fn new(max_depth: usize) -> Self { ... }
  pub fn descend(&mut self) -> Result<DepthGuard> { ... }  // Check depth, increment
  pub fn is_visited(&self, path: &Path) -> bool { ... }
    pub fn mark_visited(&mut self, path: PathBuf) { ... }
}

/// RAII guard to ensure depth is decremented on unwind
pub struct DepthGuard<'a> { ctx: &'a mut NavigationContext }
impl Drop for DepthGuard<'_> { /* decrement depth */ }
```

### 3. Extended Command Structs

```rust
// Add --follow flag to navigation commands
Commands::Show { id: u32, #[arg(long)] follow: bool }
Commands::Refs { id: u32, #[arg(long)] follow: bool }
Commands::Links { id: u32, #[arg(long)] follow: bool }
Commands::Relationships { id: u32, #[arg(long)] follow: bool }
Commands::Graph { id: u32, #[arg(long)] follow: bool }
```

### 4. Recursive Helper Functions

```rust
/// Resolve a reference (internal or external) to a node
fn resolve_reference(
    cache: &mut MindmapCache,
  mm: &Mindmap,
  current_file: &Path,
    reference: &Reference,
    ctx: &mut NavigationContext,
) -> Result<Option<(u32, PathBuf, Node)>> { ... }

/// Get incoming references (refs) with optional file following
fn get_incoming_recursive(
    cache: &mut MindmapCache,
    mm: &Mindmap,
  current_file: &Path,
    id: u32,
    follow: bool,
    ctx: &mut NavigationContext,
) -> Result<Vec<(u32, PathBuf, Node)>> { ... }

/// Get outgoing references (links) with optional file following
fn get_outgoing_recursive(
    cache: &mut MindmapCache,
    mm: &Mindmap,
  current_file: &Path,
    id: u32,
    follow: bool,
    ctx: &mut NavigationContext,
) -> Result<Vec<(u32, PathBuf, Node)>> { ... }
```

---

## Implementation Phases

### Phase 3.1: Core Data Structures & Safety Nets (3-4 hours)

**Objectives:**
- Implement MindmapCache with lazy loading
- Add NavigationContext for depth/cycle tracking
- Implement path resolution and validation
- Add safety nets per Node [14] security priority

**Deliverables:**
1. MindmapCache struct with:
   - `load()` - Load and cache external files
   - `resolve_path()` - Resolve relative paths securely
   - `is_visited()` - Cycle detection
   - Path canonicalization

2. NavigationContext for:
   - Depth tracking (max 50 levels default)
   - Visited file set (HashSet<PathBuf>)
   - Error on cycle detection

3. Safety validations:
  - ✅ Path canonicalization (prevents `..` escapes)
  - ✅ Resolve relative paths from the current file
   - ✅ File size checks (max 10MB)
   - ✅ Visited tracking (cycle detection)
   - ✅ Max depth enforcement (prevent infinite loops)
  - ✅ Relative path validation (no absolute paths, Windows-safe)

**Test Cases:**
- Loading single external file
- Cycle detection (A → B → A)
- Deep nesting (50+ levels)
- Invalid paths (not relative, Windows drive/UNC)
- Large files (>10MB rejected)
- Missing files (graceful handling)

**Success Criteria:**
- MindmapCache loads files and caches them
- Cycles detected and logged, execution continues
- Depth exceeded properly reported
- All safety nets documented and tested

---

### Phase 3.2: Command Updates - Read-Only Navigation (4-5 hours)

**Objectives:**
- Add `--follow` flag to show/refs/links/relationships/graph
- Implement recursive resolution
- Update output formatting for cross-file results
- Maintain backward compatibility (follow=false by default)

**Deliverables:**
1. Update command handlers:
   - `cmd_show()` - Show node with external refs indicated
   - `cmd_refs()` - Follow incoming refs across files
   - `cmd_links()` - Follow outgoing refs across files
   - `cmd_relationships()` - Show both with file-crossing
   - `cmd_graph()` - Include external nodes in DOT

2. Output formatting:
   ```
   Relationships for [15]:
   ← Incoming (4 nodes):
     [7] **META: Node Lifecycle Example** (./MINDMAP.md)
     [15] **Another ref** (./MINDMAP.llm.md)  ← external file indicator
   ```

3. JSON enhancement:
   ```json
   {
     "command": "relationships",
     "recursive": true,
     "depth": 2,
     "incoming": [
       { "id": 7, "file": "./MINDMAP.md", "title": "..." },
       { "id": 15, "file": "./MINDMAP.llm.md", "title": "..." }
     ]
   }
   ```

**Test Cases:**
- Single file (backward compatibility)
- Two-file reference chain
- Bidirectional refs across files
- Missing external file (warning)
- Invalid external ID (error)
- Graph generation with external nodes

**Success Criteria:**
- `show <id> --follow` shows external refs with file paths
- `refs <id> --follow` includes refs from all loaded files
- `relationships <id> --follow` shows complete picture
- Graph includes cross-file edges
- All tests green, backward compatible

---

### Phase 3.3: File Reference Validation (2-3 hours)

**Objectives:**
- Enhance lint to validate external references
- Detect missing files, invalid IDs
- Warn on unreachable external nodes
- Validate path safety

**Deliverables:**
1. Extended lint checks:
   ```
   Lint found 3 warnings:
     - Missing external file: ./MINDMAP.llm.md referenced by [15]
     - Invalid external ID: [999] in ./MINDMAP.md (max ID: 50)
     - Unreachable ref: [7] in ./MINDMAP.auth.md (not linked from main)
   ```

2. New lint flags:
   - `--check-external` - Validate all external refs (default off)
   - `--fix-external` - (Future) Remove broken external refs

**Test Cases:**
- Valid external refs pass lint
- Missing files detected
- Invalid IDs detected
- Unreachable files warned (not linked from main)

**Success Criteria:**
- Lint reports all external ref issues
- No false positives
- Clear actionable warnings

---

### Phase 3.4: Advanced Features & Polish (2-3 hours)

**Objectives:**
- Add recursive search across files
- Add depth limit options
- Performance optimizations
- Documentation and examples

**Deliverables:**
1. `search --recursive` - Search across all files
2. `list --recursive --max-depth N` - Control recursion
3. Performance optimizations:
   - Parallel file loading (async/rayon)
   - Incremental indexing for large graphs
4. Documentation:
   - README section on multi-file workflows
   - Examples: how to organize large mindmaps
   - FAQ: common issues and solutions

**Test Cases:**
- Recursive search finds cross-file matches
- Max depth limits applied
- Performance acceptable (1000+ files?)

**Success Criteria:**
- Recursive features working
- Performance satisfactory
- Documentation complete

---

## Safety Net Implementation Details

### 1. Path Canonicalization
```rust
fn resolve_path(&self, base_file: &Path, relative: &str) -> Result<PathBuf> {
  let rel_path = Path::new(relative);

  // ✅ Reject absolute paths (POSIX, Windows drive letters, UNC)
  if rel_path.is_absolute() || rel_path.components().any(|c| matches!(c, Component::Prefix(_))) {
    bail!("Absolute paths not allowed");
  }

  // ✅ Resolve relative paths from the current file's directory
  let base_dir = base_file.parent().unwrap_or(&self.workspace_root);
  let full_path = base_dir.join(rel_path);

  // ✅ Canonicalize the parent to validate traversal without requiring the target to exist
  let canonical_base = fs::canonicalize(base_dir)
    .context("Failed to resolve base dir")?;
  let canonical = canonical_base.join(rel_path);

  // ✅ Ensure result is still under workspace_root
  if !canonical.starts_with(&self.workspace_root) {
    bail!("Path escape attempt detected");
  }

  Ok(canonical)
}
```

### 2. Cycle Detection
```rust
pub fn load(
  &mut self,
  base_file: &Path,
  relative_path: &str,
  ctx: &mut NavigationContext,
) -> Result<&Mindmap> {
  let canonical = self.resolve_path(base_file, relative_path)?;

  // ✅ Cycle detection per traversal
  if ctx.is_visited(&canonical) {
    eprintln!("⚠ Circular reference detected: {}", relative_path);
    return Err("Cycle detected".into());
  }

  // ✅ Check cache
  if let Some(mm) = self.cache.get(&canonical) {
    ctx.mark_visited(canonical);
    return Ok(mm);
  }

  // ✅ Check file size before reading
  let metadata = fs::metadata(&canonical)?;
  if metadata.len() > self.max_file_size {
    bail!("File too large: {} > {}", metadata.len(), self.max_file_size);
  }

  // ✅ Load new file
  let mm = Mindmap::load(canonical.clone())?;
  self.cache.insert(canonical.clone(), mm);
  ctx.mark_visited(canonical);

  Ok(self.cache.get(&canonical).unwrap())
}
```

### 3. Depth Limiting
```rust
impl NavigationContext {
  pub fn descend(&mut self) -> Result<DepthGuard<'_>> {
    self.depth += 1;
    if self.depth > self.max_depth {
      bail!("Recursion depth exceeded (max: {})", self.max_depth);
    }
    Ok(DepthGuard { ctx: self })
  }
}
```

### 4. File Size Limits
```rust
pub fn load(
  &mut self,
  base_file: &Path,
  relative_path: &str,
  ctx: &mut NavigationContext,
) -> Result<&Mindmap> {
    // ... resolve path ...
    
    // ✅ Check file size before reading
    let metadata = fs::metadata(&canonical)?;
    if metadata.len() > self.max_file_size {
        bail!("File too large: {} > {}", metadata.len(), self.max_file_size);
    }
    
    // ... rest of loading ...
}
```

---

## CLI Interface Design

**Path Resolution Rule:** All relative paths are resolved from the current file (the file being parsed), not a global root, across all commands (including `--follow`).

Example (terse): if `docs/MINDMAP.md` contains `[10](./auth.md)` then resolution is `docs/auth.md`. Inverse (global root) would resolve to `./auth.md` at the workspace root.

### New Flags
```bash
# Enable recursive navigation
$ mindmap-cli show 15 --follow
$ mindmap-cli refs 15 --follow
$ mindmap-cli relationships 15 --follow
$ mindmap-cli graph 15 --follow
$ mindmap-cli search "pattern" --recursive

# Control recursion depth
$ mindmap-cli show 15 --follow --max-depth 3

# Validate external references
$ mindmap-cli lint --check-external

# Search across all files
$ mindmap-cli search "TODO" --recursive --output json
```

### Output Examples

#### show with --follow
```
[15] AE: mindmap-cli
v0 implementation complete...

→ References:
  [43] **DONE: Phase 1 UX Improvements** (./MINDMAP.md)
  [44] **DOC: Phase 1 Implementation Summary** (./MINDMAP.md)
  [10] **External: LLM Architecture** (./MINDMAP.llm.md) [2 hops away]

← Incoming:
  [7] **META: Node Lifecycle Example** (./MINDMAP.md)
  [9] **META: Scaling Strategy** (./MINDMAP.md)
  [100] **External ref** (./MINDMAP.arch.md)
```

#### graph with --follow
```
graph {
  subgraph cluster_main {
    label="./MINDMAP.md"
    15 [label="AE: mindmap-cli"]
    43 [label="DONE: Phase 1..."]
  }
  subgraph cluster_llm {
    label="./MINDMAP.llm.md"
    100 [label="AE: LLM System"]
  }
  15 -> 43 [label="ref"]
  15 -> 100 [label="external ref"]
}
```

#### JSON output with --recursive
```json
{
  "command": "relationships",
  "node": 15,
  "recursive": true,
  "max_depth": 50,
  "depth_reached": 2,
  "incoming": [
    { "id": 7, "file": "./MINDMAP.md", "title": "META: Node Lifecycle Example" },
    { "id": 100, "file": "./MINDMAP.llm.md", "title": "AE: LLM System" }
  ],
  "outgoing": [
    { "id": 43, "file": "./MINDMAP.md", "title": "DONE: Phase 1..." },
    { "id": 200, "file": "./MINDMAP.arch.md", "title": "DR: Architecture Decision" }
  ],
  "warnings": [
    "File ./MINDMAP.future.md referenced but not found"
  ]
}
```

---

## Node [14] Core Priorities Alignment

| Priority | Implementation | How It's Addressed |
|----------|---------------|--------------------|
| **Security** | Path traversal prevention | Canonicalize paths, validate no escapes from base_dir |
| | Infinite loops | Visited file tracking, cycle detection |
| | Resource exhaustion | File size limits, recursion depth cap |
| | Symlink attacks | Use fs::canonicalize (resolves all symlinks) |
| **Correctness** | External ref validation | Lint checks, proper error messages |
| | ID validation | Verify IDs exist in external files |
| | Path resolution | Relative to current file, not global |
| **Robustness** | Missing files | Graceful handling, warnings, continue |
| | Circular references | Detect early, warn, skip edge |
| | Large files | Size check before loading |
| | Deep nesting | Depth limit with clear error |
| **Maintainability** | Clean API | MindmapCache, NavigationContext types |
| | Testable design | Dependency injection of cache/context |
| | Documentation | Clear examples and safety guarantees |
| **Speed** | Lazy loading | Load files only when needed |
| | Caching | Avoid re-loading same file |
| | Parallel loading | (Phase 3.4) Async file loading |
| **Visuals** | File indicator | Show `(./file.md)` next to cross-file refs |
| | Subgraph clustering | DOT output with file grouping |
| | Clear warnings | Explain issues and how to fix |

---

## Work Breakdown & Estimation

| Phase | Task | Estimated | Category |
|-------|------|-----------|----------|
| 3.1 | MindmapCache implementation | 2h | Core |
| 3.1 | NavigationContext & depth tracking | 1h | Core |
| 3.1 | Safety nets (path validation, cycles) | 1h | Security |
| 3.1 | Unit tests for cache & context | 1h | Testing |
| **3.1 Total** | | **5h** | |
| 3.2 | Update show/refs/links handlers | 1.5h | Commands |
| 3.2 | Update relationships command | 1h | Commands |
| 3.2 | Update graph command for DOT | 1h | Commands |
| 3.2 | Add --follow flag & CLI integration | 0.5h | CLI |
| 3.2 | Output formatting for external refs | 1h | UI |
| 3.2 | Integration tests (multi-file) | 2h | Testing |
| **3.2 Total** | | **7h** | |
| 3.3 | Enhance lint for external refs | 1h | Validation |
| 3.3 | Add --check-external flag | 0.5h | CLI |
| 3.3 | Tests for lint validation | 1.5h | Testing |
| **3.3 Total** | | **3h** | |
| 3.4 | Recursive search implementation | 1h | Features |
| 3.4 | Max-depth flag support | 0.5h | CLI |
| 3.4 | Documentation & examples | 1.5h | Docs |
| 3.4 | Performance testing | 1h | Testing |
| **3.4 Total** | | **4h** | |
| **GRAND TOTAL** | | **19h** | |

---

## Risk Analysis

### High Risk
- **Circular references causing infinite loops**
  - Mitigation: Visited file tracking, depth limits
  - Test: Create A → B → A scenario

- **Path traversal attacks**
  - Mitigation: Canonicalize, validate within workspace_root, reject absolute (POSIX, Windows drive/UNC)
  - Test: Attempt `../../etc/passwd`, `C:\windows\system32`, `\\server\share` references

### Medium Risk
- **Performance degradation with many files**
  - Mitigation: Caching, lazy loading, limits
  - Test: 100+ file benchmark

- **Backward compatibility**
  - Mitigation: --follow defaults to false, Single-file works unchanged
  - Test: All existing tests pass

### Low Risk
- **Documentation gaps**
  - Mitigation: Clear examples, README section
  - Test: User feedback review

---

## Success Criteria (MVP)

✅ **Phase 3.1:**
- [ ] MindmapCache loads and caches files
- [ ] Path traversal attacks prevented
- [ ] Cycles detected and handled safely
- [ ] All safety nets working (depth, size, visited)

✅ **Phase 3.2:**
- [ ] `show <id> --follow` shows external refs
- [ ] `refs/links <id> --follow` cross-file navigation
- [ ] `graph <id> --follow` includes external nodes
- [ ] Backward compatible (--follow defaults to false)
- [ ] JSON output includes file paths

✅ **Phase 3.3:**
- [ ] Lint detects missing external files
- [ ] Lint detects invalid external IDs
- [ ] Clear actionable warning messages

✅ **Phase 3.4:**
- [ ] Recursive search working
- [ ] Documentation complete
- [ ] All 50+ tests passing

---

## Example Workflow

### Setting up multi-file mindmap
```bash
# Main file with architecture overview
$ cat MINDMAP.md
[1] **AE: System Architecture** - See [10](./MINDMAP.llm.md) and [20](./MINDMAP.auth.md)
[2] **META: Structure** - Split into domains

# LLM domain file
$ cat MINDMAP.llm.md
[10] **AE: LLM System** - Uses [15] from main
[11] **WF: Prompt Engineering** - [10] refs

# Auth domain file
$ cat MINDMAP.auth.md
[20] **AE: Authentication** - OAuth via [1]
[21] **DR: Session Handling** - [20] ref
```

### Using recursive navigation
```bash
# View single file (existing behavior)
$ mindmap-cli show 1
[1] **AE: System Architecture** - See [10](./MINDMAP.llm.md) and [20](./MINDMAP.auth.md)
→ References: [10], [20]  # Shows external ref format

# Navigate across files
$ mindmap-cli show 1 --follow
[1] **AE: System Architecture** - See [10](./MINDMAP.llm.md) and [20](./MINDMAP.auth.md)
→ References:
  [10] **AE: LLM System** (./MINDMAP.llm.md)
  [20] **AE: Authentication** (./MINDMAP.auth.md)

# Check all references to node 1
$ mindmap-cli refs 1 --follow
← Incoming:
  [20] **AE: Authentication** (./MINDMAP.auth.md) - "OAuth via [1]"
  [2] **META: Structure** (./MINDMAP.md)

# Build full relationship view
$ mindmap-cli relationships 10 --follow
← Incoming:
  [1] **AE: System Architecture** (./MINDMAP.md)
→ Outgoing:
  [11] **WF: Prompt Engineering** (./MINDMAP.llm.md)
  [15] **Unknown node** (./MINDMAP.md) [error: not found in main]

# Generate graph for visualization
$ mindmap-cli graph 1 --follow | dot -Tpng > graph.png
```

---

## Next Steps

1. **Approval:** Review this plan against Node [14] priorities
2. **Refinement:** Adjust phases based on feedback
3. **Implementation:** Start Phase 3.1 (Core structures)
4. **Testing:** Build test suite as we go
5. **Documentation:** Update README with multi-file examples

---

## References

- `planning/multiple-files.md` - Original design document
- `planning/UX_ANALYSIS_SUMMARY.md` - Phase roadmap
- `PHASE1_IMPLEMENTATION.md` - Prior phase details
- `PHASE2_IMPLEMENTATION.md` - Current phase work
- Node [14]: Core priorities (Security > Correctness > Robustness > Speed)
- Node [9]: Hints at scaling strategy (split into domain files)
- Node [15]: mindmap-cli current implementation status

---

**Status:** Ready for Phase 3.1 kickoff  
**Last Updated:** 2026-02-06  
**Alignment:** Prioritizes Node [14] Security as primary concern
