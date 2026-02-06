# Recursive Navigation: Current State vs. Planned Implementation

**Date:** 2026-02-06  
**Status:** Analysis & Baseline Planning  
**Context:** Preparing Phase 3 implementation

---

## Current Implementation Status

### ✅ ALREADY IMPLEMENTED (Foundation)

#### 1. Reference Enum with External Support
**File:** `src/lib.rs:232-233`
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Reference {
    Internal(u32),
    External(u32, String),  // ← External variant exists!
}
```
**Status:** Ready to use  
**Validation:** Used in serialization, parsing complete

---

#### 2. External Reference Parsing
**File:** `src/lib.rs:540-550`
**Functionality:** Parses markdown link syntax `[id](./path.md)`
```rust
// Detects ]( pattern
if after.chars().next() == Some(']') {
    if after.chars().nth(1) == Some('(') {
        // Parse ID and path
        refs.push(Reference::External(rid, path.to_string()));
        continue;
    }
}
```

**Test Case (already exists):**
```rust
// Line 2871 - Unit test confirms External ref parsing
vec![Reference::External(234, "./file.md".to_string())]
```

**Status:** Working correctly  
**Coverage:** Full regex-free manual parser (optimal for perf)

---

#### 3. Lint Validation (Partial)
**File:** `src/lib.rs:1093-1095`
**Current:** Detects External refs in node lines
```rust
Reference::External(eid, file) => {
    // Lint check for missing external refs
}
```

**Status:** Handles External refs but doesn't resolve them  
**Gap:** No file loading or validation

---

### ❌ NOT YET IMPLEMENTED (Needed for Phase 3)

#### 1. File Loading & Caching
**Required:** MindmapCache struct
- Lazy loading of external files
- Caching to avoid re-loading
- Path resolution with security checks
- File size limits
- Cycle detection (visited set)

**Impact:** Without this, can't resolve External refs to actual nodes

---

#### 2. Recursive Navigation Functions
**Required:** 
- `get_incoming_recursive()` - Follow refs across files
- `get_outgoing_recursive()` - Follow links across files
- `resolve_reference()` - Convert any Reference to (id, file, node) tuple

**Impact:** Commands like `show --follow` won't work

---

#### 3. NavigationContext for Safety
**Required:**
- Depth tracking (prevent infinite loops)
- Visited file set (prevent cycles)
- Error propagation with context

**Impact:** Without this, circular references A→B→A cause hangs

---

#### 4. CLI Flag Support
**Required:**
- Add `--follow` flag to show/refs/links/relationships/graph
- Add `--recursive` flag to search/list
- Add `--max-depth` flag for control
- Add `--check-external` flag to lint

**Impact:** Users can't opt-in to recursive behavior

---

#### 5. Output Formatting Updates
**Required:**
- Show file paths for external refs: `(./MINDMAP.llm.md)`
- Update JSON schema with file info
- DOT graph with subgraph clustering by file

**Impact:** Users can't see which file refs come from

---

## Comparison: Before vs. After Phase 3

### Scenario: Navigate Node [15] with External References

#### BEFORE Phase 3 (Current)
```bash
$ mindmap-cli show 15
[15] AE: mindmap-cli
...See [43][44] for details...

← Referring nodes: [7, 9, 42, 43]
→ References: [43, 44]

# Can't see external refs even if they exist!
$ mindmap-cli show 15 --follow
error: unknown option '--follow'

# Graph ignores external files
$ mindmap-cli graph 15 | dot -Tpng
# Only shows nodes in current file
```

#### AFTER Phase 3 (Planned)
```bash
$ mindmap-cli show 15
[15] AE: mindmap-cli
...See [43][44] and [100](./MINDMAP.llm.md)...

← Referring nodes:
  [7, 9, 42, 43] (./MINDMAP.md)
  [200] (./MINDMAP.llm.md)  ← Cross-file ref!

→ References:
  [43, 44] (./MINDMAP.md)
  [100] (./MINDMAP.llm.md)

# New --follow flag available!
$ mindmap-cli show 15 --follow
[15] AE: mindmap-cli
...See [43][44] and [100](./MINDMAP.llm.md)...

← Incoming (7 nodes across 2 files):
  [7] **META: Node Lifecycle Example** (./MINDMAP.md)
  [9] **META: Scaling Strategy** (./MINDMAP.md)
  [42] **DONE: Consolidate search...** (./MINDMAP.md)
  [43] **DONE: Phase 1 UX Improvements** (./MINDMAP.md)
  [200] **AE: LLM Integration** (./MINDMAP.llm.md)  ← Shows loaded node!
  [300] **AE: Auth Handler** (./MINDMAP.auth.md)   ← Loaded from external file

→ Outgoing (4 nodes across 2 files):
  [43] **DONE: Phase 1 UX Improvements** (./MINDMAP.md)
  [44] **DOC: Phase 1 Implementation Summary** (./MINDMAP.md)
  [100] **AE: LLM System** (./MINDMAP.llm.md)
  [101] **WF: Token Management** (./MINDMAP.llm.md)

# Graph now includes external files!
$ mindmap-cli graph 15 --follow | dot -Tpng
# Shows nodes from MINDMAP.md, MINDMAP.llm.md, MINDMAP.auth.md
# Grouped by file, cross-file edges highlighted

# Recursive search across all files
$ mindmap-cli search "LLM" --recursive
Search results for 'LLM' (8 results across 3 files):
(./MINDMAP.md):
  [15] **AE: mindmap-cli** - ... See [100](./MINDMAP.llm.md) ...

(./MINDMAP.llm.md):
  [100] **AE: LLM System** - ...
  [101] **WF: Token Management** - ...
  ...
```

---

## Implementation Dependencies

### Building Blocks (Required Order)
```
Phase 3.1: Core Data Structures
├── MindmapCache (file loading, caching, security)
├── NavigationContext (depth, visited tracking)
└── Helper: resolve_path, resolve_reference

         ↓ (depends on)

Phase 3.2: Command Integration
├── Update cmd_show, cmd_refs, cmd_links
├── Update cmd_relationships, cmd_graph
├── Add --follow flag support
└── Output formatting

         ↓ (depends on)

Phase 3.3: Validation
├── Enhanced lint checks
└── --check-external flag

         ↓ (depends on)

Phase 3.4: Polish & Optimization
├── Recursive search
├── Performance tuning
└── Documentation
```

---

## Code Inventory: What Exists

### Files to Modify
| File | Lines | Changes Needed |
|------|-------|-----------------|
| `src/lib.rs` | ~2900 | Add MindmapCache, update 5 command handlers, add --follow flag |
| `src/ui.rs` | ~200 | Update output formatting for file paths |
| `Cargo.toml` | ~20 | No changes (no new deps) |
| `tests/cli.rs` | ~200 | Add multi-file test scenarios |
| `README.md` | ~150 | Add multi-file workflow section |

### Reuse Opportunities
- ✅ Reference enum (already has External variant)
- ✅ Parsing logic (already extracts External refs)
- ✅ Lint infrastructure (extend existing validation)
- ✅ Output formatting (extend PrettyPrinter, PlainPrinter)
- ✅ Test framework (use existing assert_fs, assert_cmd)

---

## Key Design Decisions

### 1. Cache Scope: Per-Request vs. Global
**Option A: Per-Request Cache** (CHOSEN)
- Fresh cache created for each command
- Reset visited set between commands
- Safe: No state pollution
- Cost: Re-load files per command
- Good for: CLI tool where commands are one-off

**Option B: Global Cache**
- Shared across entire program run
- Efficient: Load file once, reuse
- Risk: Visited set must be reset per top-level command
- Better for: REPL or server mode (not applicable here)

**Decision:** Option A - simpler, safer for CLI context

---

### 2. Depth Limit: Fixed vs. Configurable
**Option A: Fixed (default 50)** (CHOSEN)
- `--follow` uses 50 levels
- Simple, safe default
- Prevents almost all real cycles

**Option B: User-Configurable**
- `--follow --max-depth 100`
- Flexibility for power users
- Risk: Users shoot themselves in the foot

**Decision:** Option A for MVP, Option B in Phase 3.4

---

### 3. Circular Ref Handling: Error vs. Warning
**Option A: Error - Stop processing** ❌
- `refs 15 --follow` fails if cycle found
- User frustrated, can't get any results
- Too harsh

**Option B: Warning - Continue with visited** ✅ (CHOSEN)
- `refs 15 --follow` completes, warns about cycle
- User gets results, understands limitation
- Graceful degradation

**Decision:** Option B - user-friendly, follows Node [14] robustness

---

### 4. File Resolution: Relative to Main vs. Current File
**Option A: Relative to main MINDMAP.md**
- `./MINDMAP.llm.md` from any file resolves same way
- Simple
- Problem: Can't organize with subdirectories

**Option B: Relative to current file** ✅ (CHOSEN)
- Each file knows its own directory
- Supports: `./MINDMAP.llm.md` and `./auth/MINDMAP.auth.md`
- More flexible

**Decision:** Option B - better for large, organized projects

---

## Testing Strategy

### Unit Tests (Phase 3.1)
```rust
#[test]
fn test_cache_loads_file() { ... }

#[test]
fn test_cache_prevents_reload() { ... }

#[test]
fn test_cycle_detection() { ... }

#[test]
fn test_depth_limit() { ... }

#[test]
fn test_path_escapes_blocked() { ... }

#[test]
fn test_large_file_rejected() { ... }
```

### Integration Tests (Phase 3.2)
```rust
#[test]
fn test_show_with_follow() { ... }

#[test]
fn test_refs_cross_file() { ... }

#[test]
fn test_graph_multi_file() { ... }

#[test]
fn test_backward_compat_no_follow() { ... }
```

### Multi-File Test Fixture
```
temp_dir/
├── MINDMAP.md
│   [1] Main: See [10](./llm/MINDMAP.md)
│   [2] Ref to [11](./llm/MINDMAP.md)
├── llm/
│   └── MINDMAP.md
│       [10] LLM: Ref [1]
│       [11] WF: Ref [2]
└── auth/
    └── MINDMAP.md
        [20] Auth: Ref [1]
```

---

## Alignment with Node [14]: Core Priorities

### Security ✅
- **Path traversal:** Canonicalize paths, validate no escapes
- **Resource limits:** File size check, depth limit
- **Cycles:** Visited tracking, cycle detection
- **Symlinks:** fs::canonicalize resolves all

### Correctness ✅
- **Validation:** Lint checks for missing files/IDs
- **Resolution:** Proper relative path handling
- **Errors:** Clear messages for issues

### Robustness ✅
- **Missing files:** Warn but continue
- **Cycles:** Detect early, skip edge, continue
- **Depth:** Cap enforced, user warned
- **Large files:** Rejected with explanation

### Maintainability ✅
- **Clean API:** MindmapCache, NavigationContext types
- **Testable:** DI pattern for cache/context
- **Documented:** Examples, inline comments

### Speed ✅
- **Caching:** Avoid re-loading files
- **Lazy loading:** Load only what's needed
- **Efficient:** HashMap lookups, no regex in hot path

### Visuals ✅
- **File indicators:** Show `(./MINDMAP.llm.md)` next to refs
- **Subgraphs:** DOT output groups by file
- **Clear warnings:** Actionable error messages

---

## Quick Reference: What's Ready

### Ready to Use
```rust
// Reference enum with External
enum Reference { Internal(u32), External(u32, String) }

// Parsing already works
[100](./MINDMAP.llm.md) → Reference::External(100, "./MINDMAP.llm.md")

// Serialization ready
serde_json::to_string(&Reference::External(...))
```

### Ready to Extend
```rust
// Existing command handlers
pub fn cmd_show(mm: &Mindmap, id: u32) -> Result<String>
// Easy to add: follow: bool parameter

// Existing output formatters
impl PrettyPrinter { fn node_refs(...) }
// Easy to extend: add file path column

// Existing lint logic
fn validate_refs(mm: &Mindmap) -> Vec<String>
// Easy to extend: validate external refs
```

---

## Risk Mitigation Summary

| Risk | Severity | Mitigation | Testing |
|------|----------|-----------|---------|
| Path traversal | HIGH | Canonicalize, validate base_dir | `test_path_escapes_blocked` |
| Infinite loops | HIGH | Visited set, depth limit | `test_cycle_detection`, `test_depth_limit` |
| File exhaustion | MEDIUM | File size limit (10MB) | `test_large_file_rejected` |
| Backward compat | MEDIUM | --follow defaults false | All existing tests pass |
| Performance | MEDIUM | Caching, limits | Benchmark 100+ files |
| Circular refs | MEDIUM | Continue with warning | `test_circular_reference_warns` |

---

## Success Definition

**Phase 3.1 Complete When:**
- ✅ MindmapCache loads/caches files
- ✅ Path validation prevents traversal
- ✅ Cycles detected
- ✅ All unit tests green

**Phase 3.2 Complete When:**
- ✅ `show <id> --follow` works
- ✅ `refs <id> --follow` follows files
- ✅ `graph <id> --follow` includes external
- ✅ All integration tests green
- ✅ Backward compatible

**Phase 3.3 Complete When:**
- ✅ Lint detects missing files
- ✅ Lint detects invalid IDs
- ✅ Clear warning messages

**Phase 3.4 Complete When:**
- ✅ Recursive search works
- ✅ Documentation updated
- ✅ Performance acceptable

**OVERALL MVP Complete When:**
- 🎯 **All 50+ tests passing**
- 🎯 **Zero security vulnerabilities**
- 🎯 **100% backward compatible**
- 🎯 **Ready for production use**

---

## Estimated Effort Breakdown

```
Total Phase 3: ~19 hours

Phase 3.1 (Core):     5h - MindmapCache, validation, tests
Phase 3.2 (Commands): 7h - Update 5 handlers, flags, output
Phase 3.3 (Lint):     3h - External validation, tests
Phase 3.4 (Polish):   4h - Search, docs, perf testing

Buffer (contingency): ~5h (for unknowns)
```

---

## Document Cross-References

- **Original Design:** `planning/multiple-files.md`
- **UX Roadmap:** `planning/UX_ANALYSIS_SUMMARY.md`
- **Phase 1 Work:** `PHASE1_IMPLEMENTATION.md`
- **Phase 2 Work:** `PHASE2_IMPLEMENTATION.md`
- **Full Plan:** `planning/RECURSIVE_NAVIGATION_PLAN.md` (this directory)
- **Node [14]:** Core priorities (Security > Correctness > Robustness)
- **Node [9]:** Scaling strategy hint
- **Node [15]:** mindmap-cli feature status

---

**Status:** Ready for Phase 3.1 implementation  
**Last Updated:** 2026-02-06  
**Confidence:** HIGH - Strong foundation already in place
