# Phase 3.3 Plan: Enhanced Features for Multi-File Support

**Phase Start:** 2026-02-06  
**Estimated Duration:** 3 hours  
**Current Status:** PLANNING  

---

## Overview

Phase 3.3 enhances the multi-file mindmap support with better error handling, validation, and user feedback. Builds on Phase 3.1 (infrastructure) and Phase 3.2 (command integration).

---

## Phase 3.3 Objectives

### Primary Goals
1. **Recursive Search** - Search across all referenced files
2. **External Reference Validation** - Detect missing files and invalid IDs
3. **Enhanced Error Messages** - Better feedback when things go wrong
4. **Cycle Detection Warnings** - Warn users about circular references
5. **Depth Limit Warnings** - Inform when traversal hits depth limit

### Success Criteria
- ✅ Search command supports `--follow` flag
- ✅ Lint command validates external references
- ✅ Missing files detected with clear error messages
- ✅ Circular references warned (not blocked)
- ✅ Depth limit warnings when reached
- ✅ All tests passing (65+ tests expected)
- ✅ Zero breaking changes
- ✅ Full backward compatibility

---

## Phase 3.3 Sub-Phases (1-5)

### Phase 3.3.1: Recursive Search (0.5 hours)
**Goal:** Add `--follow` support to `search` command

**Implementation:**
```rust
// Add to Commands enum
Search {
    query: String,
    case_sensitive: bool,
    exact_match: bool,
    regex_mode: bool,
    follow: bool,  // NEW
}
```

**Features:**
- Search across all referenced files
- Same search parameters (case, exact, regex)
- Return results with file paths
- Maintain backward compatibility

**Output Example:**
```
[1] **Node** - matches search query (MAIN.md)
[10] **External** - also matches (external.md)
```

---

### Phase 3.3.2: External Reference Validation (1 hour)
**Goal:** Enhance lint command to validate external refs

**Implementation:**
```rust
// New lint check: validate_external_refs()
// Detects:
// - Missing files
// - Invalid node IDs in referenced files
// - Circular references (informational)
```

**Checks to Add:**
1. File exists check
   - Error if file referenced but not found
   - Show which nodes reference missing file

2. Node ID existence check
   - Error if [id](file.md) but id doesn't exist in file
   - Show which references are broken

3. Circular reference detection
   - Track traversal path
   - Warn (don't error) on cycles
   - Show cycle path to user

**Output:**
```
ERROR: Missing file ./external.md (referenced by [1])
ERROR: Node [99] not found in ./external.md (referenced by [2])
WARNING: Circular reference: [1] → [10] → [1]
```

---

### Phase 3.3.3: Better Error Messages (0.75 hours)
**Goal:** Improve error feedback for external references

**Current Issues:**
- Missing files silently skipped
- Invalid IDs just return empty
- No indication of what went wrong

**Improvements:**
1. Add optional `--strict` flag to show command
   - Error on missing files instead of skipping
   - Error on invalid node IDs
   - For production use cases

2. Enhanced error context
   - Show which file couldn't be loaded
   - Show why (not found, too big, invalid syntax)
   - Suggest fixes

3. Warning system
   - Track problems during traversal
   - Display at end of output
   - Don't break on warnings

**Output Example:**
```
[1] **Main** - ...

⚠ WARNINGS:
  - File ./typo.md referenced but not found (by [2])
  - Depth limit reached (max 50) - skipping deeper refs
```

---

### Phase 3.3.4: Cycle Detection Warnings (0.5 hours)
**Goal:** Detect and warn about circular references

**Implementation:**
```rust
// In NavigationContext
pub fn detect_cycle(&self, current_path: &Path, next_ref: &Reference) -> Option<Vec<PathBuf>>

// Returns cycle path if detected, None otherwise
```

**Features:**
- Detect cycles during traversal
- Show the cycle path to user
- Continue traversal (don't block)
- Helpful warning message

**Warning Example:**
```
⚠ Circular reference detected:
  MAIN.md [1] → external.md [10] → MAIN.md [1]
```

---

### Phase 3.3.5: Depth Limit Warnings (0.25 hours)
**Goal:** Inform users when depth limit is hit

**Implementation:**
```rust
// In NavigationContext
pub fn at_max_depth(&self) -> bool
pub fn depth_remaining(&self) -> usize
pub fn warn_depth_limit(&self) -> String
```

**When to Warn:**
- During show command: if we stopped following refs due to depth
- During search: if some results skipped
- During lint: if external files not checked

**Warning Example:**
```
⚠ Depth limit reached (max 50 levels)
  Some external references may not be shown
  Use --max-depth to increase limit (Phase 3.4)
```

---

## Detailed Implementation Plan

### Phase 3.3.1: Recursive Search Implementation

**Step 1: Update Commands enum**
```rust
Commands::Search {
    query: String,
    case_sensitive: bool,
    exact_match: bool,
    regex_mode: bool,
    follow: bool,  // ADD THIS
}
```

**Step 2: Update command-line parser**
```bash
mindmap-cli search <query> --follow [--case-sensitive] [--exact] [--regex]
```

**Step 3: Implement handler logic**
```rust
if follow {
    // Create cache and context
    // Call cmd_list for main file
    // Recursively call cmd_list for each referenced file
    // Merge results with file paths
} else {
    // Original single-file search
}
```

**Step 4: Output formatting**
```
For each result: "[id] **title** - description (file.md)"
Count by file: "5 results (3 in MAIN.md, 2 in external.md)"
```

**Step 5: Testing**
```rust
#[test]
fn test_search_recursive_basic() { }
#[test]
fn test_search_recursive_cross_file() { }
#[test]
fn test_search_backward_compat() { }
```

---

### Phase 3.3.2: External Reference Validation

**New lint check: validate_external_refs()**

```rust
pub fn validate_external_refs(
    mm: &Mindmap,
    workspace: &Path,
    cache: &mut MindmapCache,
) -> Vec<LintMessage>

// Returns list of validation errors/warnings
```

**Validations:**
1. For each node with External refs:
   - Check if file exists
   - Load file from cache
   - Check if node ID exists in file
   - Detect cycles

**Integration with lint command:**
```rust
// In cmd_lint:
let mut messages = lint_basic(mm);
messages.extend(validate_external_refs(mm, workspace, &mut cache));
```

**Lint output:**
```
✘ [1] References missing file ./external.md
✘ [2] References non-existent node [99] in ./external.md
⚠ [1] Circular reference with external.md [10]
```

---

### Phase 3.3.3: Better Error Messages

**Add --strict flag to show command:**
```rust
Commands::Show {
    id: u32,
    follow: bool,
    strict: bool,  // ADD THIS
}
```

**Strict mode behavior:**
```
Normal mode (--follow, no --strict):
  - Skip missing files silently
  - Return empty list for invalid IDs
  - Show warnings at end

Strict mode (--follow --strict):
  - Error on missing files
  - Error on invalid IDs
  - Fail if any refs couldn't be resolved
```

**Warning display:**
```rust
struct TraversalWarnings {
    missing_files: Vec<(String, u32)>,  // (path, referencing_node_id)
    invalid_nodes: Vec<(u32, String)>,  // (id, file_path)
    depth_limit_hit: bool,
    cycles: Vec<Vec<PathBuf>>,
}
```

---

## Testing Strategy

### New Test Scenarios

#### Recursive Search Tests (3 tests)
```rust
#[test]
fn integration_search_recursive_basic()
// Multi-file setup, search for term in all files

#[test]
fn integration_search_recursive_regex()
// Test regex search across files

#[test]
fn integration_search_backward_compat()
// Verify single-file search unchanged
```

#### Validation Tests (4 tests)
```rust
#[test]
fn test_validate_external_refs_missing_file()
// Detect missing external file

#[test]
fn test_validate_external_refs_invalid_node()
// Detect invalid node ID in external file

#[test]
fn test_validate_external_refs_circular()
// Detect circular reference

#[test]
fn test_validate_external_refs_valid()
// Pass when all refs valid
```

#### Cycle Detection Tests (2 tests)
```rust
#[test]
fn test_cycle_detection_basic()
// Simple A → B → A cycle

#[test]
fn test_cycle_detection_complex()
// Multi-file: A → B → C → A cycle
```

#### Depth Limit Tests (2 tests)
```rust
#[test]
fn test_depth_limit_warning_single_file()
// Verify warning shown when limit hit

#[test]
fn test_depth_limit_configurable()
// Verify --max-depth flag works
```

---

## Implementation Phases Timeline

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 3.3.1 | Recursive search | 0.5 hours | ⏳ NEXT |
| 3.3.2 | External ref validation | 1.0 hour | ⏳ AFTER 3.3.1 |
| 3.3.3 | Better error messages | 0.75 hours | ⏳ AFTER 3.3.2 |
| 3.3.4 | Cycle detection warnings | 0.5 hours | ⏳ AFTER 3.3.3 |
| 3.3.5 | Depth limit warnings | 0.25 hours | ⏳ AFTER 3.3.4 |
| Testing | Integration tests | 0.5 hours | ⏳ AS WE GO |
| **TOTAL** | | **~3.0 hours** | |

---

## Success Metrics

### Code Metrics
- ✅ 300+ LOC added (search + validation)
- ✅ 11+ new tests added
- ✅ All tests passing (71+ total)
- ✅ 0 compiler warnings
- ✅ 0 clippy violations

### Feature Metrics
- ✅ Search command supports --follow
- ✅ Lint validates external refs
- ✅ Missing files detected
- ✅ Invalid IDs detected
- ✅ Cycles warned
- ✅ Depth limits warned

### Quality Metrics
- ✅ 100% backward compatible
- ✅ All original tests pass
- ✅ New tests comprehensive
- ✅ Error handling robust
- ✅ Performance acceptable

---

## Integration Approach

### Build on Phase 3.2
- Use existing MindmapCache (Phase 3.1)
- Use existing NavigationContext (Phase 3.1)
- Leverage recursive helpers (Phase 3.2)
- Follow same patterns as show/refs/links

### No Breaking Changes
- All new features optional
- Backward compatible flags
- Default behavior unchanged
- Safe error handling

### Clean Code Principles
- DRY: Extract validation logic
- SOLID: Single responsibility per function
- Error propagation: Use Result<T>
- Testing: Add tests as we go

---

## Known Considerations

### 1. Search Performance
- Need to avoid O(n²) file scanning
- Cache results to avoid rescans
- Consider: background indexing (Phase 3.4)

### 2. Cycle Detection
- Track path, not just visited set
- Show cycle to user (helpful)
- Allow cycles (informational only)

### 3. Error Messages
- Balance verbosity with clarity
- Show actionable suggestions
- Consider: error code system (Phase 3.4)

### 4. Depth Limits
- Current default: 50 levels
- Should be configurable: Phase 3.4
- Need to warn when hitting limit

---

## Files to Modify

### src/lib.rs
- Add `follow: bool` to Search command
- Add `strict: bool` to Show command
- Implement validate_external_refs()
- Implement cycle detection
- Update lint handler
- Update search handler

### tests/cli.rs
- Add recursive search test
- Add validation tests
- Add cycle detection tests
- Add depth limit tests

### New files (optional)
- Create src/validation.rs for lint logic (Phase 3.4)
- Create src/warnings.rs for warning display (Phase 3.4)

---

## Rollout Plan

### Step 1: Implement 3.3.1-3.3.5
- Add features incrementally
- Test after each feature
- Commit after each working feature

### Step 2: Integration Testing
- Test all combinations
- Test error cases
- Test performance

### Step 3: Documentation
- Update help text
- Add examples
- Document new flags

### Step 4: Release
- Bump version (v0.5.1 or 0.6.0?)
- Update changelog
- Create release notes

---

## Future Enhancements (Phase 3.4+)

### Phase 3.4: Performance
- [ ] LRU cache for file access
- [ ] Background file indexing
- [ ] Async file loading

### Phase 3.5: Advanced Features
- [ ] Configurable depth limit (--max-depth)
- [ ] Configurable file size limit
- [ ] Configurable cycle behavior
- [ ] Multi-file graph visualization

### Phase 4.0: Analysis
- [ ] File dependency graphs
- [ ] Unused reference detection
- [ ] Dead code analysis
- [ ] Relationship metrics

---

## Ready to Begin

All prerequisites met:
- ✅ Phase 3.1 complete (infrastructure)
- ✅ Phase 3.2 complete (command integration)
- ✅ Recursive helpers available
- ✅ Test framework in place
- ✅ Security validated

**Status: Ready for Phase 3.3 implementation**

---

*Plan Created: 2026-02-06*  
*Duration Estimate: 3 hours*  
*Start when ready*
