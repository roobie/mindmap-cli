# Phase 3.2 Implementation Plan: Command Integration

**Date:** 2026-02-06  
**Status:** STARTING  
**Estimated Duration:** 7 hours  
**Scope:** Add `--follow` flag and integrate recursive navigation into 5 commands  

---

## Overview

Phase 3.2 integrates the MindmapCache and NavigationContext (built in Phase 3.1) into the mindmap-cli commands to enable recursive navigation across multiple files.

### Commands to Update (5 total)

1. **show** - Show a node with external refs resolved
2. **refs** - Show incoming references (including cross-file)
3. **links** - Show outgoing references (including cross-file)
4. **relationships** - Show both incoming and outgoing (including cross-file)
5. **graph** - Generate graph with external nodes included

### Backward Compatibility

All commands maintain backward compatibility:
- `--follow` flag is **optional** (defaults to false)
- Single-file behavior **unchanged** when flag not used
- All existing tests **must pass**

---

## Implementation Strategy

### Phase 3.2.1: Add `--follow` Flag to Commands (1 hour)

Update Commands enum to add `follow: bool` field:

```rust
Commands::Show {
    id: u32,
    #[arg(long)]
    follow: bool,  // NEW
}

Commands::Refs {
    id: u32,
    #[arg(long)]
    follow: bool,  // NEW
}

Commands::Links {
    id: u32,
    #[arg(long)]
    follow: bool,  // NEW
}

Commands::Relationships {
    id: u32,
    #[arg(long)]
    follow: bool,  // NEW
}

Commands::Graph {
    id: u32,
    #[arg(long)]
    follow: bool,  // NEW
}
```

### Phase 3.2.2: Create Helper Functions (2 hours)

Add recursive navigation helpers to lib.rs:

```rust
/// Resolve a single reference (internal or external)
fn resolve_reference(
    cache: &mut MindmapCache,
    mm: &Mindmap,
    current_file: &Path,
    reference: &Reference,
    ctx: &mut NavigationContext,
) -> Result<Option<(u32, PathBuf, Node)>> { ... }

/// Get all incoming references recursively
fn get_incoming_recursive(
    cache: &mut MindmapCache,
    mm: &Mindmap,
    current_file: &Path,
    id: u32,
    ctx: &mut NavigationContext,
) -> Result<Vec<(u32, PathBuf, Node)>> { ... }

/// Get all outgoing references recursively
fn get_outgoing_recursive(
    cache: &mut MindmapCache,
    mm: &Mindmap,
    current_file: &Path,
    id: u32,
    ctx: &mut NavigationContext,
) -> Result<Vec<(u32, PathBuf, Node)>> { ... }
```

### Phase 3.2.3: Update show Command (1 hour)

```rust
Commands::Show { id, follow } => {
    if follow {
        // NEW: recursive implementation
        let workspace = mm.path.parent().unwrap_or(Path::new("."));
        let mut cache = MindmapCache::new(workspace.to_path_buf());
        let mut ctx = NavigationContext::new();
        
        // Get incoming from all files
        let inbound = get_incoming_recursive(&mut cache, &mm, &mm.path, id, &mut ctx)?;
        
        // Get outgoing from all files
        let outbound = get_outgoing_recursive(&mut cache, &mm, &mm.path, id, &mut ctx)?;
        
        // Format output with file indicators
        // ...
    } else {
        // EXISTING: single-file implementation (unchanged)
        // ...
    }
}
```

### Phase 3.2.4: Update refs, links, relationships Commands (2 hours)

Similar pattern to show, but filter for specific direction:

```rust
Commands::Refs { id, follow } => {
    if follow {
        let inbound = get_incoming_recursive(&mut cache, &mm, &mm.path, id, &mut ctx)?;
        // Format and display with file paths
    } else {
        // existing code
    }
}
```

### Phase 3.2.5: Update graph Command (1 hour)

Enhance DOT generation to include external nodes:

```rust
Commands::Graph { id, follow } => {
    if follow {
        // Create subgraph cluster for each file
        // Include cross-file edges
    } else {
        // existing single-file graph
    }
}
```

### Phase 3.2.6: Add Output Formatting (1.5 hours)

Update output to show file paths for cross-file refs:

```
[15] AE: mindmap-cli
→ References (4 nodes):
  [43] **DONE: Phase 1 UX Improvements** (./MINDMAP.md)
  [44] **DOC: Phase 1 Implementation Summary** (./MINDMAP.md)
  [10] **AE: External: LLM Architecture** (./MINDMAP.llm.md) [2 hops]
  [200] **Unknown** (./MINDMAP.arch.md) [ERROR: not found]

← Incoming (3 nodes):
  [7] **META: Node Lifecycle Example** (./MINDMAP.md)
  [9] **META: Scaling Strategy** (./MINDMAP.md)
  [100] **External ref** (./MINDMAP.auth.md)
```

### Phase 3.2.7: Integration Tests (1 hour)

Add integration tests for multi-file scenarios:

```rust
#[test]
fn test_show_with_follow() { ... }

#[test]
fn test_refs_with_follow() { ... }

#[test]
fn test_links_with_follow() { ... }

#[test]
fn test_relationships_with_follow() { ... }

#[test]
fn test_graph_with_follow() { ... }

#[test]
fn test_cycle_detection_in_follow() { ... }
```

---

## Error Handling Strategy

1. **Missing Files** - Warn and continue with available data
2. **Invalid IDs** - Mark as [ERROR: not found]
3. **Circular References** - Skip edge, mark as [CYCLE]
4. **Depth Exceeded** - Report reached max depth
5. **File Read Errors** - Report with context

---

## Output Format Design

### Single File (backward compatible)
```
[15] **Title** - body
← Incoming: [7], [9], [42]
→ References: [43], [44], [45]
```

### Multiple Files with --follow
```
[15] **Title** - body

→ References (3 nodes):
  [43] **DONE: Phase 1** (./MINDMAP.md)
  [100] **External Node** (./MINDMAP.llm.md)
  [999] **Missing** (./MINDMAP.auth.md) [ERROR: not found]

← Incoming (2 nodes):
  [7] **META: Node Lifecycle** (./MINDMAP.md)
  [200] **Cross-file ref** (./MINDMAP.llm.md)
```

### JSON Output
```json
{
  "command": "show",
  "node": 15,
  "follow": true,
  "incoming": [
    { "id": 7, "file": "./MINDMAP.md", "title": "META: Node Lifecycle" },
    { "id": 200, "file": "./MINDMAP.llm.md", "title": "Cross-file ref" }
  ],
  "outgoing": [
    { "id": 43, "file": "./MINDMAP.md", "title": "DONE: Phase 1" },
    { "id": 100, "file": "./MINDMAP.llm.md", "title": "External Node" },
    { "id": 999, "file": "./MINDMAP.auth.md", "error": "not found" }
  ],
  "warnings": [
    "File ./MINDMAP.auth.md referenced but not found"
  ]
}
```

---

## Testing Strategy

### Unit Tests (new, ~10)
- Test recursive reference resolution
- Test cache usage
- Test cycle detection
- Test depth limiting
- Test error handling

### Integration Tests (new, ~5)
- Multi-file setup with 3+ files
- Cross-file references
- Circular references between files
- Missing files
- Deep nesting

### Backward Compatibility Tests
- All existing 43 unit tests must pass
- All existing 4 integration tests must pass
- No behavior change when --follow not used

---

## Success Criteria

✅ **Phase 3.2.1** Complete:
- [ ] All 5 commands have `--follow` flag
- [ ] Flag integrated into Commands enum
- [ ] Compiles without errors

✅ **Phase 3.2.2-5** Complete:
- [ ] All 5 commands support --follow
- [ ] Recursive reference resolution works
- [ ] Output formatting correct
- [ ] Graph generation includes external nodes

✅ **Phase 3.2.6-7** Complete:
- [ ] Output formatting with file paths
- [ ] JSON schema updated
- [ ] 10+ new unit tests passing
- [ ] 5+ new integration tests passing

✅ **Final Quality**:
- [ ] 70+ total tests passing (60 existing + 10+ new)
- [ ] All existing tests still passing
- [ ] No compiler warnings
- [ ] Clippy approved
- [ ] Full backward compatibility
- [ ] Ready for Phase 3.3

---

## Time Breakdown (7 hours estimated)

| Task | Hours | Notes |
|------|-------|-------|
| Add --follow flag | 1 | Commands enum update |
| Helper functions | 2 | Recursive resolution logic |
| Command updates | 3 | show/refs/links/relationships/graph |
| Output formatting | 1 | File paths, error messages |
| Testing | 1 | Unit + integration tests |
| **Total** | **8** | Slightly over estimate for polish |

---

## Known Challenges

1. **Borrow Checker** - Cache/context borrowing in recursive calls
   - Solution: Careful lifetime management, pass references

2. **Output Formatting** - Show file paths without breaking existing output
   - Solution: Check --follow flag, use different format when true

3. **Error Handling** - Missing files, invalid IDs in external refs
   - Solution: Graceful degradation with warnings

4. **Performance** - Loading many large files
   - Solution: Caching (already implemented in Phase 3.1)

---

## Integration Points

### With Phase 3.1 Code
- Uses MindmapCache for file loading
- Uses NavigationContext for depth/cycle tracking
- Clean separation of concerns

### With Existing Commands
- No breaking changes
- --follow is optional flag
- Single-file behavior unchanged

### With Phase 3.3
- Phase 3.3 will validate external refs in lint
- Can reuse recursive resolution helpers
- Same output format for file paths

---

## Rollback Plan

If issues arise:
1. Remove --follow flag support (revert to single-file only)
2. All tests pass (backward compatible)
3. No changes to existing functionality
4. Restart Phase 3.2 with adjusted approach

---

## Next Phase Preview (Phase 3.3)

Phase 3.3 will enhance lint to validate external references:

```bash
$ mindmap-cli lint --check-external
Lint found 2 issues:
  - Missing file: ./MINDMAP.llm.md referenced by [15]
  - Invalid ID: [999] in ./MINDMAP.auth.md (max ID: 200)
```

Uses same recursive helpers from Phase 3.2.

---

**Status:** Ready to implement  
**Next:** Phase 3.2.1 - Add --follow flag to Commands enum
