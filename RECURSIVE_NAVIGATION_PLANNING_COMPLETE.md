# Recursive Navigation Planning: Complete & Ready

**Date:** 2026-02-06  
**Status:** ✅ PLANNING PHASE COMPLETE  
**Duration:** ~2 hours of analysis & planning  
**Deliverables:** 3 comprehensive planning documents  

---

## What Was Delivered

### 1. **planning/RECURSIVE_NAVIGATION_PLAN.md** (20 KB)
**Comprehensive Implementation Blueprint**

Contains:
- **Executive Summary** - Overview of Phase 3 scope
- **Proposed Architecture**
  - MindmapCache: File loading, caching, security validation
  - NavigationContext: Depth tracking, cycle detection
  - Recursive helper functions: reference resolution
- **Implementation Phases** (4 phases, 19 hours total)
  - Phase 3.1: Core data structures (5h)
  - Phase 3.2: Command integration (7h)
  - Phase 3.3: File validation (3h)
  - Phase 3.4: Polish & optimization (4h)
- **Safety Net Implementation Details**
  - Path canonicalization (prevents `..` escapes)
  - Cycle detection (HashSet<PathBuf> visited)
  - Depth limiting (max 50 levels)
  - File size checks (max 10MB)
- **CLI Interface Design** with examples
- **Node [14] Priority Alignment Matrix**
- **Work Breakdown & Estimation** (hour-level tasks)
- **Risk Analysis & Mitigation**
- **Example Workflows** (before/after)
- **Success Criteria & Test Strategy**

**Perfect For:** Implementation team, detailed specification

---

### 2. **planning/RECURSIVE_NAVIGATION_ANALYSIS.md** (14 KB)
**Current State Analysis & Baseline**

Contains:
- **Current Implementation Status**
  - What's ALREADY IMPLEMENTED (foundation ready!)
    - Reference::External enum
    - Parsing for [id](./path.md)
    - Unit tests for parsing
    - Partial lint support
  - What's NOT YET IMPLEMENTED (19h work)
    - File loading & caching
    - Recursive navigation functions
    - CLI flag support
    - Output formatting
- **Before/After Comparison** with concrete examples
- **Implementation Dependencies** (build order)
- **Code Inventory** (what exists, what to reuse)
- **Key Design Decisions** (cache scope, depth limit, error handling)
- **Testing Strategy** with multi-file fixtures
- **Risk Mitigation Matrix**
- **Quick Reference** sections for fast lookup

**Perfect For:** Developers, code review, planning discussions

---

### 3. **PHASE3_PLANNING_SUMMARY.md** (9 KB)
**Executive Summary for Decision Makers**

Contains:
- **Overview** - What Phase 3 does & why
- **What's Already Built** - Strong foundation
- **What Needs Building** - Work breakdown
- **Implementation Dependencies** - Sequential order
- **Testing Strategy** - Unit, integration, performance
- **Node [14] Alignment** - Security-first approach
- **Work Breakdown** - Hours by task
- **Key Design Decisions** - Rationale
- **Success Criteria (MVP)** - Clear acceptance tests
- **Risk Mitigation** - Quick reference
- **File Modifications** - What changes
- **What Exists Already** - Reuse opportunities
- **Next Steps** - Clear action items

**Perfect For:** Managers, stakeholders, quick reference

---

## Key Findings

### Strong Foundation Already In Place
✅ **Reference::External enum** - Exists, ready to use  
✅ **External ref parsing** - Works, has unit tests  
✅ **Lint support** - Partial, ready to extend  
✅ **Test infrastructure** - Ready for new tests  

**Implication:** Phase 3.1 can focus on safety & caching, building on proven parsing

### Clear Implementation Path
- **Phase 3.1:** Core safety-first (5h) → MindmapCache + validation
- **Phase 3.2:** Commands (7h) → Add --follow flag to 5 commands  
- **Phase 3.3:** Validation (3h) → Enhanced lint checks
- **Phase 3.4:** Polish (4h) → Recursive search + docs

**Total: 19 hours** for complete multi-file support

### Node [14] Alignment (Verified)
**Core Priority:** Security > Correctness > Robustness > Speed > Visuals

✅ **Security (Priority #1)**
- Path canonicalization prevents `../../etc/passwd` escapes
- Cycle detection prevents A→B→A loops
- Depth limiting prevents infinite chains (max 50)
- File size checks prevent memory exhaustion (max 10MB)

✅ **Correctness (Priority #2)**
- All external refs validated before resolution
- Missing files detected and reported
- Invalid IDs flagged in lint

✅ **Robustness (Priority #3)**
- Graceful degradation: warn but continue
- Clear error messages for all issues
- Comprehensive test coverage

---

## How to Use These Documents

### For Starting Implementation
1. **Read:** PHASE3_PLANNING_SUMMARY.md (9 KB, 10 min)
   - Get overview and context
   
2. **Review:** planning/RECURSIVE_NAVIGATION_PLAN.md (20 KB, 30 min)
   - Understand full architecture
   - See Phase 3.1 in detail
   
3. **Reference:** planning/RECURSIVE_NAVIGATION_ANALYSIS.md (14 KB, as needed)
   - What's implemented, what's not
   - Design decisions and rationale
   - Testing strategy

### For Code Review
- Use RECURSIVE_NAVIGATION_ANALYSIS.md for "what exists"
- Use RECURSIVE_NAVIGATION_PLAN.md for "what should be built"
- Compare implementation against safety nets in Phase 3.1

### For Team Communication
- Show PHASE3_PLANNING_SUMMARY.md to stakeholders
- Use RECURSIVE_NAVIGATION_PLAN.md for technical discussions
- Reference risk mitigation table when addressing concerns

### For Testing
- Multi-file fixture design in RECURSIVE_NAVIGATION_ANALYSIS.md
- Test cases in RECURSIVE_NAVIGATION_PLAN.md
- Success criteria for each phase

---

## Implementation Checklist: Phase 3.1

Getting started with Phase 3.1 (Core Data Structures):

### Preparation
- [ ] Review planning/RECURSIVE_NAVIGATION_PLAN.md Phase 3.1
- [ ] Review planning/RECURSIVE_NAVIGATION_ANALYSIS.md Code Inventory
- [ ] Set up test fixtures (multi-file structure)
- [ ] Plan git branches/PRs

### MindmapCache Implementation
- [ ] Create MindmapCache struct
  - [ ] `new(base_dir)` constructor
  - [ ] `load(path)` with caching
  - [ ] `resolve_path(relative)` with validation
  - [ ] Internal `visited: HashSet<PathBuf>`
- [ ] Path validation
  - [ ] Canonicalize paths
  - [ ] Check no escapes from base_dir
  - [ ] Reject absolute paths
  - [ ] Validate relative paths
- [ ] File loading
  - [ ] Load Mindmap from path
  - [ ] Cache loaded mindmaps
  - [ ] Handle missing files gracefully
  - [ ] Check file size (max 10MB)

### NavigationContext Implementation
- [ ] Create NavigationContext struct
  - [ ] `depth: usize` counter
  - [ ] `max_depth: usize` limit (default 50)
  - [ ] `visited: HashSet<PathBuf>` for cycles
- [ ] Depth tracking
  - [ ] `descend()` function (increment + check limit)
  - [ ] Error on depth exceeded
- [ ] Cycle detection
  - [ ] `mark_visited(path)` function
  - [ ] `has_visited(path)` check
  - [ ] Detect cycles before loading

### Testing (Phase 3.1)
- [ ] Unit tests for MindmapCache
  - [ ] test_cache_loads_file
  - [ ] test_cache_prevents_reload (caching works)
  - [ ] test_path_escapes_blocked (security)
  - [ ] test_large_file_rejected (size limit)
  - [ ] test_path_canonicalization
- [ ] Unit tests for NavigationContext
  - [ ] test_depth_limit_enforced
  - [ ] test_cycle_detection
  - [ ] test_visited_tracking
- [ ] Integration test: multi-file setup
  - [ ] Create test fixture (3 files with cross-refs)
  - [ ] Test loading chain of files
  - [ ] Test cycle detection

### Verification
- [ ] All 8+ new tests passing
- [ ] Existing 43 tests still passing
- [ ] No compiler warnings
- [ ] Code review checklist passed

---

## Critical Design Points (Recap)

### Security-First
Every design decision prioritizes Node [14]'s #1 priority: **Security**
- Path handling: Canonicalize everything, validate bounds
- Loop prevention: Visited set + depth limit
- Resource limits: File size check before loading
- Error messages: Clear, no path disclosure

### Backward Compatible
- Existing single-file commands unchanged
- New `--follow` flag optional (defaults to false)
- All existing tests must pass
- External refs in parsing already handled

### Graceful Degradation
- Missing files: Warn, continue with available data
- Circular refs: Detect, skip edge, complete traversal
- Deep nesting: Enforce limit, report clearly
- Invalid paths: Reject early with clear message

### Performance
- Lazy loading: Only load files when needed
- Caching: Avoid re-loading same file
- HashMap: O(1) node lookup
- Limits: Prevent pathological cases (100+ files)

---

## Document Locations

All planning documents are in the project repository:

```
mindmap-cli/
├── PHASE3_PLANNING_SUMMARY.md          ← Executive summary (read this first)
├── planning/
│   ├── RECURSIVE_NAVIGATION_PLAN.md     ← Full implementation blueprint
│   ├── RECURSIVE_NAVIGATION_ANALYSIS.md ← Current state analysis
│   └── multiple-files.md                ← Original baseline planning
├── PHASE1_IMPLEMENTATION.md             ← Previous phase reference
├── PHASE2_IMPLEMENTATION.md             ← Previous phase reference
└── src/lib.rs                           ← Code with existing External refs
```

---

## Key References in Code

### Current Reference Handling
- **src/lib.rs line 232-233:** Reference enum with External variant
- **src/lib.rs line 540-550:** External ref parsing logic
- **src/lib.rs line 1093-1095:** Partial lint validation for External refs
- **tests/lib.rs line 2871:** Unit test for External ref parsing

### Test Framework Ready
- **tests/cli.rs:** Integration test structure
- **assert_fs:** Temp directory fixture support
- **assert_cmd:** Command execution testing

---

## Questions & Clarifications

### Q: Why start with MindmapCache instead of CLI flags?
**A:** Foundation-first approach. Cache is the critical piece that enables everything else. Without it, can't resolve external refs.

### Q: Why max 50 depth instead of configurable?
**A:** MVP safety. Prevents mistakes by default. Can add --max-depth in Phase 3.4 for advanced users.

### Q: Why warn-on-cycle instead of error?
**A:** User-friendly graceful degradation. User gets results with notification vs. command failure.

### Q: Why relative paths (to file) instead of absolute?
**A:** Flexibility. Supports subdirectory organization (`./auth/MINDMAP.md`). Relative paths are standard in markdown.

### Q: How does this differ from the original planning/multiple-files.md?
**A:** This plan incorporates lessons from Phase 1 & 2, aligns with Node [14] priorities, adds concrete safety nets, and provides detailed implementation roadmap vs. high-level design.

---

## Success Metrics

**Phase 3 Complete When:**
- ✅ 50+ tests passing (all existing + new)
- ✅ Zero security issues or exploits possible
- ✅ 100% backward compatible with Phase 1 & 2
- ✅ External refs resolved and navigable
- ✅ Multi-file mindmaps fully supported
- ✅ Documentation comprehensive
- ✅ Production-ready code quality

**Estimated Timeline:**
- Phase 3.1: ~1 week (5h of focused work)
- Phase 3.2: ~1.5 weeks (7h of focused work)
- Phase 3.3: ~3-4 days (3h of focused work)
- Phase 3.4: ~4-5 days (4h of focused work)
- **Total: ~3-4 weeks** casual pace, or **3-4 days** intensive

---

## Next Actions

1. **Review** this planning summary
2. **Read** planning/RECURSIVE_NAVIGATION_PLAN.md Phase 3.1 section
3. **Approve** 19-hour estimate & approach
4. **Kick off** Phase 3.1 implementation
5. **Execute** incrementally (1 phase per week recommended)
6. **Track** progress against hour estimates
7. **Ship** complete multi-file support

---

## Document Metadata

| Document | Size | Read Time | Purpose |
|----------|------|-----------|---------|
| PHASE3_PLANNING_SUMMARY.md | 9 KB | 10 min | Overview & decisions |
| RECURSIVE_NAVIGATION_PLAN.md | 20 KB | 30 min | Implementation blueprint |
| RECURSIVE_NAVIGATION_ANALYSIS.md | 14 KB | 20 min | Current state & design |

**Total Documentation:** 43 KB of detailed, actionable planning

---

## Conclusion

Phase 3 planning is **complete and comprehensive**. The foundation is strong (reference parsing already works). The path forward is clear (4 sequential phases, 19 hours). Safety is prioritized (Node [14] alignment). Implementation can start immediately.

**Status:** ✅ Ready for Phase 3.1 kickoff

---

**Generated:** 2026-02-06  
**Planning Time Investment:** ~2 hours  
**Estimated Implementation Time:** 19 hours  
**Confidence Level:** HIGH (existing foundation, proven approach)  
**Risk Level:** LOW (security-first design, extensive safety nets)

**Next milestone:** Phase 3.1 implementation (MindmapCache + NavigationContext)
