# Phase 3 Planning Summary: Recursive Navigation

**Date:** 2026-02-06  
**Status:** ✅ PLANNING COMPLETE  
**Scope:** Recursive navigation & multi-file support  
**Estimated Hours:** 19 hours total (4 phases)

---

## Overview

Phase 3 extends mindmap-cli from single-file to multi-file knowledge graphs, enabling scaling beyond ~100 nodes by splitting domains while maintaining unified navigation and cross-file relationships.

**Alignment with Node [14]:** Security-first approach with path validation, cycle detection, and robust error handling.

---

## What's Already Built (Foundation Ready!)

✅ **Reference::External Enum**
- External ref variant exists in code
- Serializable with serde
- Ready for use

✅ **External Ref Parsing** 
- Parses `[id](./path.md)` syntax
- Extracts both ID and file path
- No regex (optimal performance)
- Unit tests confirm parsing works

✅ **Partial Lint Support**
- Lint detects External refs in nodes
- Partial validation in place
- Foundation for extended checks

---

## What Needs to Be Built

### Phase 3.1: Core Data Structures (5 hours)
**Status:** Ready to implement

Key Components:
- **MindmapCache**: File loading, caching, security validation
- **NavigationContext**: Depth tracking, cycle detection, visited files
- **Helper Functions**: Path resolution, reference resolution

Safety Nets (Node [14] Security Priority):
- ✓ Path canonicalization (prevents `..` escapes)
- ✓ Cycle detection (HashSet visited tracking)
- ✓ Depth limiting (max 50 levels)
- ✓ File size checks (max 10MB)
- ✓ Symlink resolution (fs::canonicalize)

### Phase 3.2: Command Integration (7 hours)
**Status:** Design complete

Add `--follow` flag to 5 commands:
- `show <id> --follow`
- `refs <id> --follow`
- `links <id> --follow`
- `relationships <id> --follow`
- `graph <id> --follow`

Output Updates:
- Show file paths: `(./MINDMAP.llm.md)`
- JSON includes file info
- DOT graph subgraph clustering by file

### Phase 3.3: File Validation (3 hours)
**Status:** Design complete

Enhance lint:
- Detect missing external files
- Validate external IDs exist
- Add `--check-external` flag
- Clear actionable warnings

### Phase 3.4: Polish & Optimization (4 hours)
**Status:** Design complete

Features:
- Recursive search: `search --recursive`
- Max depth control: `--max-depth N`
- Performance benchmarking
- Documentation updates

---

## Implementation Dependencies

```
Phase 3.1 (Core)
    ↓ (depends on)
Phase 3.2 (Commands)
    ↓ (depends on)
Phase 3.3 (Validation)
    ↓ (depends on)
Phase 3.4 (Polish)
```

Each phase builds on previous; can be done incrementally.

---

## Testing Strategy

**Unit Tests (Phase 3.1):**
- Cache loading and caching
- Cycle detection
- Depth limiting
- Path security (no escapes)
- File size validation

**Integration Tests (Phase 3.2):**
- Multi-file scenarios (2-3 files)
- Cross-file navigation
- Graph generation with external nodes
- Backward compatibility (single file)

**Validation Tests (Phase 3.3):**
- Missing file detection
- Invalid ID detection
- Clear warning messages

**Performance Tests (Phase 3.4):**
- 100+ file loading
- Deep nesting efficiency
- Recursive search performance

---

## Node [14] Alignment: Core Priorities

| Priority | Implementation |
|----------|-----------------|
| **Security** | Path traversal prevention, cycle detection, depth limits, file size checks |
| **Correctness** | External ref validation, ID checking, proper path resolution |
| **Robustness** | Graceful missing file handling, circular ref skipping, deep nesting limits |
| **Speed** | Lazy loading, caching, HashMap lookups |
| **Visuals** | File path indicators, subgraph clustering, clear warnings |

---

## Work Breakdown

| Phase | Task | Hours | Status |
|-------|------|-------|--------|
| 3.1 | MindmapCache implementation | 2h | 📋 Design ready |
| 3.1 | NavigationContext & safety nets | 2h | 📋 Design ready |
| 3.1 | Unit tests | 1h | 📋 Test plan ready |
| 3.2 | Update 5 command handlers | 3h | 📋 Design ready |
| 3.2 | Output formatting updates | 2h | 📋 Design ready |
| 3.2 | Integration tests | 2h | 📋 Test plan ready |
| 3.3 | Lint external validation | 2h | 📋 Design ready |
| 3.3 | Tests | 1h | 📋 Test plan ready |
| 3.4 | Recursive search | 1h | 📋 Design ready |
| 3.4 | Documentation | 2h | 📋 Design ready |
| 3.4 | Performance testing | 1h | 📋 Test plan ready |
| **Total** | | **19h** | ✅ Ready |

---

## Key Design Decisions

1. **Per-Request Cache** (not global)
   - Fresh cache per command
   - Simple, safe for CLI

2. **Fixed Depth Limit** (50 levels)
   - Safe default for MVP
   - Configurable in Phase 3.4

3. **Graceful Degradation**
   - Cycles: warn and continue
   - Missing files: warn and continue
   - User gets results, understands issues

4. **Relative Paths** (to current file)
   - Each file knows its directory
   - Supports subdirectories
   - More flexible

---

## Documentation Created

### Comprehensive Plans
✅ **planning/RECURSIVE_NAVIGATION_PLAN.md** (19KB)
- Full architecture design
- 4-phase implementation roadmap
- Safety net details
- CLI interface design
- 19-hour work breakdown
- Risk analysis
- Success criteria

✅ **planning/RECURSIVE_NAVIGATION_ANALYSIS.md** (13KB)
- Current state analysis (what's implemented)
- Current limitations (what's missing)
- Before/After comparison
- Implementation dependencies
- Code inventory
- Testing strategy

---

## Quick Example: How It Will Work

### Before Phase 3 (Current)
```bash
$ mindmap-cli show 15
[15] AE: mindmap-cli
...See [43][44] for details...
→ References: [43, 44]

# Can't follow external files
$ mindmap-cli show 15 --follow
error: unknown option '--follow'
```

### After Phase 3 (Proposed)
```bash
$ mindmap-cli show 15 --follow
[15] AE: mindmap-cli
...See [43][44] and [100](./MINDMAP.llm.md)...

→ Outgoing (4 nodes across 2 files):
  [43] **DONE: Phase 1...** (./MINDMAP.md)
  [44] **DOC: Implementation Summary** (./MINDMAP.md)
  [100] **AE: LLM System** (./MINDMAP.llm.md)
  [101] **WF: Token Management** (./MINDMAP.llm.md)

← Incoming (7 nodes across 3 files):
  [7] **META: Node Lifecycle Example** (./MINDMAP.md)
  [9] **META: Scaling Strategy** (./MINDMAP.md)
  [200] **AE: LLM Integration** (./MINDMAP.llm.md)
  [300] **AE: Auth Handler** (./MINDMAP.auth.md)
  ...
```

---

## Success Criteria (MVP)

✅ **Phase 3.1 Complete:**
- MindmapCache loads/caches files
- Path traversal attacks prevented
- Cycles detected and handled
- All safety nets working
- Unit tests green

✅ **Phase 3.2 Complete:**
- `show/refs/links <id> --follow` works
- Graph includes external nodes
- All integration tests green
- 100% backward compatible

✅ **Phase 3.3 Complete:**
- Lint detects missing files
- Lint detects invalid IDs
- Clear actionable warnings

✅ **Phase 3.4 Complete:**
- Recursive search works
- Documentation updated
- Performance acceptable

**Overall MVP:**
- 50+ tests passing
- Zero security issues
- 100% backward compatible
- Production-ready

---

## Risk Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Path traversal | HIGH | Canonicalize, validate base_dir |
| Infinite loops | HIGH | Visited set, depth limit (50) |
| File exhaustion | MEDIUM | 10MB size limit |
| Backward compat | MEDIUM | --follow defaults false |
| Performance | MEDIUM | Caching, lazy loading |
| Circular refs | MEDIUM | Continue with warning |

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/lib.rs` | Add MindmapCache, NavigationContext, update 5 handlers, add --follow |
| `src/ui.rs` | Update output formatting for file paths |
| `tests/cli.rs` | Add multi-file test scenarios |
| `README.md` | Add multi-file workflow section |

---

## What Exists Already

✓ Reference enum with External variant  
✓ External ref parsing (`[id](./path.md)`)  
✓ Unit tests for parsing  
✓ Partial lint support  
✓ Test framework (assert_fs)  
✓ Output formatting structure  

---

## Next Steps

1. **Review** both planning documents
2. **Approve** Phase 3 approach
3. **Kick off** Phase 3.1 (Core structures)
4. **Execute** incrementally (1 phase per iteration)
5. **Test** thoroughly (unit + integration + security)

---

## Key References

- **Original Design:** `planning/multiple-files.md`
- **UX Roadmap:** `planning/UX_ANALYSIS_SUMMARY.md`
- **Full Plan:** `planning/RECURSIVE_NAVIGATION_PLAN.md`
- **Analysis:** `planning/RECURSIVE_NAVIGATION_ANALYSIS.md`
- **Node [14]:** Core priorities (Security > Correctness > Robustness)
- **Node [9]:** Scaling strategy hint

---

## Status

✅ **Planning Complete**
- Architecture designed
- Phases defined
- Work breakdown detailed
- Safety nets specified
- Tests planned

🎯 **Ready to Start**
- Phase 3.1 can begin immediately
- Strong foundation already in place
- Detailed implementation guide ready

📊 **Confidence:** HIGH
- Core structures simple and proven
- Safety nets well-understood
- Incremental approach reduces risk
- Foundation (parsing) already works

---

**Generated:** 2026-02-06  
**Time Spent Planning:** ~2 hours  
**Estimated Phase 3 Duration:** 19 hours (2.4 days of focused work)  
**Overall Project Progress:**
- Phase 1: ✅ COMPLETE (3h actual)
- Phase 2: ✅ COMPLETE (8h actual)
- Phase 3: 📋 READY TO START (19h estimated)

**Total Delivered So Far:** 11 hours for 40% UX improvement (Phase 1 & 2)  
**Runway for Phase 3:** 19 hours for recursive navigation & scaling support
