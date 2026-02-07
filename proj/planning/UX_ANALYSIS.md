# UX Analysis: mindmap-cli

**Date:** 2026-02-06  
**Status:** Comprehensive analysis with improvement recommendations

## Executive Summary

`mindmap-cli` is a well-designed, purpose-built CLI for managing one-line MINDMAP files. The tool demonstrates strong fundamentals (atomic writes, safety, validation) but has several UX opportunities to improve clarity, discoverability, and efficiency. This analysis identifies 15+ actionable improvements across documentation, output formatting, command semantics, and workflow optimization.

---

## Part 1: Strengths

### 1.1 Safety & Reliability ✅
- **Atomic writes**: Prevents partial file corruption
- **Reference validation**: Detects missing links
- **Type safety**: Rust compilation catches many errors
- **Edit safety**: Editor flow validates node format strictly
- **Batch atomicity**: All-or-nothing multi-operation transactions

### 1.2 Clear Mental Model ✅
- One-node-per-line format is simple and grep-friendly
- Numeric IDs are stable and immutable
- Type prefixes (AE:, WF:, DR:, etc.) are intuitive
- Reference syntax `[N]` matches academic/research patterns

### 1.3 Good Default Behaviors ✅
- Sensible defaults (MINDMAP.md, no --file needed)
- Help text includes examples and explains stdin support
- `--output json` for scripting
- stdin support for read-only operations

---

## Part 2: UX Challenges & Opportunities

### 2.1 Discoverability: What Command Should I Use?

**Problem:** Users face decision paralysis between similar commands:

| Task | Commands | Confusion |
|------|----------|-----------|
| Find nodes with "auth" | `search`, `list --grep` | "Which should I use?" |
| Find referencing nodes | `refs <id>` | Unclear name (direction?) |
| Find referenced nodes | `links <id>` | Unclear name (direction?) |
| Filter + search | `list --type X --grep Y` | "Can I combine these?" |
| Remove formatting issues | `lint --fix` | "What gets fixed?" |

**Impact:** Friction in new users, slower workflow, tendency to use grep instead of built-in commands

**Root Causes:**
- Help text doesn't clearly explain when to use what
- Similar commands lack explicit differentiation
- Naming is terse and assumes domain knowledge (refs/links)
- No "cheat sheet" mode for quick reference

**Improvement Opportunities:**
1. Add a `help --examples` mode showing use cases
2. Rename or clarify `refs` (incoming) vs `links` (outgoing)
3. Add explicit warning when `grep` is used but `mindmap search` could work
4. Create a quick-reference guide (man page / --guide)

### 2.2 Output Clarity: Hard to Extract Information

**Problem:** Output doesn't always make it obvious what you're looking at

```bash
$ mindmap-cli refs 42
[7] **META: Node Lifecycle Example** - Initial: ...
[9] **META: Scaling Strategy** - ...
[42] **DONE: Consolidate...** - ...
```

**Issues:**
- No clear header identifying this as "Nodes referring to [42]"
- Order appears random (not sorted by ID)
- Direction of relationship is implicit, not explicit
- JSON output mixes different data types awkwardly

**Example improvements:**

**Before:**
```
[7] **META: Node Lifecycle Example** - ...
[9] **META: Scaling Strategy** - ...
```

**After:**
```
Nodes referring to [42] (3 total):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[7]  META: Node Lifecycle Example
[9]  META: Scaling Strategy
[42] DONE: Consolidate search and list --grep
```

**Improvement Opportunities:**
1. Add context headers for output ("Nodes referring to [X]:")
2. Sort results by ID for consistency
3. Add result counts ("3 matches", "No matches")
4. Use visual separators (━) for clarity in default output
5. Distinguish between incoming/outgoing with arrow notation or labels

### 2.3 Empty Results: Silent Failures

**Problem:** When no results are found, there's no output:

```bash
$ mindmap-cli list --grep "XYZABC"
<no output>  # Did it work? Is there a result?
```

**Impact:** User uncertainty about whether command succeeded

**Improvement Opportunities:**
1. Output "No matches" or "0 results" message
2. Add `--quiet` flag to suppress messages (for scripting)
3. Use stderr for informational messages, keep stdout clean

### 2.4 Orphans Output: Cryptic

**Problem:** `orphans` command lists just IDs:

```bash
$ mindmap-cli orphans
Orphans:
11
12
14
16
...
```

**Issues:**
- No title/context shown
- No way to quickly understand what they are
- Users must `mindmap show` each one individually
- Output doesn't explain _why_ they're orphans (no in/out refs)

**Better approaches:**
1. Show titles: `[11] **TODO: Something** (no incoming/outgoing references)`
2. Add summary: "25 orphan nodes found (58% of total)"
3. Add `--with-context` flag to show descriptions
4. Suggest filtering with `--type` to reduce noise

### 2.5 Batch Mode: Hard to Debug

**Problem:** Batch mode error messages don't provide context:

```bash
$ mindmap-cli batch --input ops.txt
Error: Op 5: patch failed: Node 99 not found
```

**Issues:**
- No line number reference back to input file
- Operation context is lost (what was being patched?)
- Dry-run output is verbose but not organized

**Improvement Opportunities:**
1. Include input line context: `Error: ops.txt:5: patch 99 failed: Node not found`
2. Show more context in error: which field was being patched
3. Organize dry-run output by operation type
4. Add `--verbose` flag to show before/after for each operation

### 2.6 Search Ambiguity: Case Sensitivity & Partial Matching

**Problem:** Search behavior is not obvious:

```bash
$ mindmap-cli search "auth"     # Matches?
$ mindmap-cli search "Auth"     # Different result?
$ mindmap-cli search "uthenti"  # Substring matching only?
```

**Current behavior:** Case-insensitive substring matching (correct but undocumented)

**Improvement Opportunities:**
1. Document behavior in help: "Case-insensitive substring match"
2. Add flags: `--case-sensitive`, `--exact-match`, `--regex`
3. Show match count with results: "(3 matches)"
4. Highlight matching text in output (if terminal supports it)

### 2.7 JSON Output: Structure Inconsistency

**Problem:** JSON output has inconsistent structure across commands:

```bash
mindmap-cli --output json show 42    # {"command": "show", "node": {...}}
mindmap-cli --output json list       # {"command": "list", "items": [...]}
mindmap-cli --output json refs 42    # {"command": "refs", "items": [...]}
```

**Issues:**
- Some use "node", some use "items"
- No consistent timestamp or version info
- External reference format is unclear in JSON

**Improvement Opportunities:**
1. Standardize structure:
   ```json
   {
     "command": "show",
     "status": "ok|error",
     "timestamp": "2026-02-06T11:47:12Z",
     "data": {...}
   }
   ```
2. Add `--pretty` flag (currently implicit)
3. Document JSON schema or add `--schema` command
4. Add API version for forward compatibility

### 2.8 Navigation: N+1 Problem

**Problem:** Finding related nodes requires multiple commands:

```bash
$ mindmap-cli show 42           # Shows [15]
# Now I want to see [15]
$ mindmap-cli show 15           # Shows [1], [3]
# Now I want to understand the full dependency tree?
# No single command answers this
```

**Improvement Opportunities:**
1. Add `--follow` flag: `mindmap-cli show --follow 42` (recursive depth)
2. Improve `graph` command (currently requires Graphviz)
3. Add `trail` command to show path between two nodes
4. Add `suggest` command: given a node, suggest related ones

### 2.9 Type System: Weak Discovery

**Problem:** Available node types are not discoverable:

```bash
$ mindmap-cli list --type ???   # How do I know valid types?
```

**Current situation:** Types are documented in MINDMAP.md, not discoverable from CLI

**Improvement Opportunities:**
1. Add `types` or `list --types` command to show all types in use
2. Add completion: `mindmap-cli list --type AE<TAB>`
3. Suggest types when invalid one provided: "Unknown type 'AEX'. Did you mean 'AE'?"

### 2.10 Verb Semantics: Inconsistent Naming

**Problem:** Command names have inconsistent semantics:

| Command | Semantics | Issue |
|---------|-----------|-------|
| `show` | Read one | ✓ Clear |
| `list` | Read many | ✓ Clear |
| `search` | Query (alias for list --grep) | ~ Unclear it's an alias |
| `refs` | Incoming relationships | ✗ Name doesn't suggest direction |
| `links` | Outgoing relationships | ✗ Name doesn't suggest direction |
| `patch` | Partial update | ✓ Clear (HTTP method) |
| `put` | Full update | ✓ Clear (HTTP method) |
| `deprecate` | Mark as deprecated | ✓ Clear |
| `verify` | Mark for review | ✓ Clear |
| `lint` | Validate | ✓ Clear |
| `orphans` | Find isolated nodes | ~ Unclear |

**Improvement Opportunities:**
1. Rename `refs` → `incoming` (or add alias)
2. Rename `links` → `outgoing` (or add alias)
3. Add help: `mindmap refs --help` explains "nodes referring to [ID]"
4. Add `relationships` command that shows both in/out together

### 2.11 Error Messages: Generic and Terse

**Problem:** Error messages don't help users fix problems:

```bash
$ mindmap-cli patch 999 --title "New"
Error: Node 999 not found

# User questions:
# - Is 999 a valid ID range?
# - What IDs do exist?
# - Should I use `list` first?
```

**Better approach:**

```bash
Error: Node 999 not found

Hint: Use `mindmap-cli list` to see all nodes.
      Valid IDs range from 0 to 42.
      Use `mindmap-cli show <ID>` to inspect a node.
```

**Improvement Opportunities:**
1. Add contextual hints to error messages
2. Suggest commands for recovery
3. Show valid range of IDs when ID not found
4. Add `--verbose` to show more context

### 2.12 Stdin Mode: Hidden Feature

**Problem:** `--file -` (stdin) is powerful but barely documented

```bash
cat MINDMAP.md | mindmap-cli --file - lint   # Works!
```

**Current documentation:**
- Mentioned in help text (small note)
- Not in README examples
- Unclear which commands support it

**Improvement Opportunities:**
1. Add `--file -` to README with example
2. Add error message when mutating commands used with stdin
3. List which commands support stdin in help
4. Add `--stdin` alias for clarity

### 2.13 File Locking: Race Conditions

**Problem:** No file locking means concurrent access is unsafe:

```bash
# Terminal 1:          # Terminal 2:
mindmap-cli patch 1    mindmap-cli patch 2
# Both write at same time → race condition!
```

**Current mitigation:** blake3 hash check in batch mode only

**Improvement Opportunities:**
1. Add file locking for all write operations
2. Add `--no-lock` flag for read-only operations
3. Show warning if file modified during operation
4. Add timeout for lock acquisition

### 2.14 Undo/Rollback: Not Possible

**Problem:** No way to revert a change:

```bash
$ mindmap-cli patch 1 --title "Wrong"
# Oops! Can I undo this?
# No. Must manually fix or restore from git.
```

**Improvement Opportunities:**
1. Add `--backup` flag (creates .bak file)
2. Add `undo` command (requires journal file)
3. Show git integration hints: "Use `git diff` to see changes"
4. Add `--dry-run` for all write commands

### 2.15 Performance: No Feedback for Large Files

**Problem:** If user creates a very large MINDMAP, no feedback about parsing time:

```bash
$ mindmap-cli lint  # Takes 5 seconds... user waits silently
```

**Improvement Opportunities:**
1. Add `--verbose` to show timing
2. Add progress indicators for large files
3. Warn if file exceeds certain size
4. Suggest split into multiple files (per MINDMAP.md design)

---

## Part 3: Specific Improvement Recommendations

### Priority 1: High-Impact, Low-Effort

#### 1.1 Add Result Counts to Output

**Current:**
```bash
$ mindmap-cli list --grep "auth"
[3] **META: Node Types** - ...
[19] **DONE: Lint & Validation** - ...
```

**Proposed:**
```bash
$ mindmap-cli list --grep "auth"
Matching nodes (2 results):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[3]  META: Node Types
[19] DONE: Lint & Validation
```

**Implementation:** Add header in `Printer` trait methods

**Effort:** 1-2 hours  
**Impact:** High (clarity)

#### 1.2 Distinguish Refs vs Links in Help

**Current:**
```
refs      Show nodes that reference the given ID
links     Show nodes that the given ID references
```

**Proposed:**
```
refs      Show INCOMING references (nodes that refer to the given ID)
links     Show OUTGOING references (nodes that the given ID refers to)
```

**Implementation:** Update help strings and README

**Effort:** 30 minutes  
**Impact:** Medium (discoverability)

#### 1.3 Add "No Results" Messages

**Current:**
```bash
$ mindmap-cli list --grep "XYZABC"
<silence>
```

**Proposed:**
```bash
$ mindmap-cli list --grep "XYZABC"
No matching nodes found.
```

**Implementation:** Add check before printing results

**Effort:** 1 hour  
**Impact:** High (user confidence)

#### 1.4 Improve Error Messages with Hints

**Current:**
```bash
Error: Node 999 not found
```

**Proposed:**
```bash
Error: Node 999 not found
Hint: Valid node IDs range from 0 to 42. Use `mindmap-cli list` to see all nodes.
```

**Implementation:** Enhance error messages in command handlers

**Effort:** 2-3 hours  
**Impact:** Medium (user experience)

### Priority 2: Medium-Impact, Medium-Effort

#### 2.1 Add `--with-descriptions` flag for Orphans

**Current:**
```bash
$ mindmap-cli orphans
Orphans:
11
12
14
```

**Proposed:**
```bash
$ mindmap-cli orphans --with-descriptions
Orphan nodes (25 total, 58% of nodes):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[11] TODO: Some planned work - This task is...
[12] WF: Another workflow - ...
[14] BUG: Known issue - ...
```

**Implementation:** Extend `cmd_orphans()` to optionally include descriptions

**Effort:** 2 hours  
**Impact:** Medium (usability)

#### 2.2 Add `types` Command

**Current:**
```bash
# No way to discover available types
# Must read MINDMAP.md or source code
```

**Proposed:**
```bash
$ mindmap-cli types
Available node types (in use):
  AE   Architecture Element
  WF   Workflow
  DR   Decision Record
  TODO Planned Work
  DONE Completed Work
  META Documentation
  ...

Most common:
  AE:  8 nodes
  WF:  7 nodes
  DR:  5 nodes
  TODO: 12 nodes
  DONE: 8 nodes
```

**Implementation:** New command that analyzes type prefixes and shows stats

**Effort:** 3-4 hours  
**Impact:** Medium (discoverability)

#### 2.3 Add `relationships` Command (Show In+Out)

**Current:**
```bash
$ mindmap-cli refs 42    # Incoming
$ mindmap-cli links 42   # Outgoing
# User must run both commands
```

**Proposed:**
```bash
$ mindmap-cli relationships 42
[42] DONE: Consolidate search and list --grep

Incoming (3 nodes refer to this):
  [2] META: Node Syntax
  [7] META: Node Lifecycle Example
  [9] META: Scaling Strategy

Outgoing (1 node this refers to):
  [15] AE: mindmap-cli
```

**Implementation:** New command that combines refs + links output

**Effort:** 2 hours  
**Impact:** Medium (efficiency)

### Priority 3: Lower-Priority, Larger-Effort

#### 3.1 Standardize JSON Output Structure

**Current:**
```json
{"command": "show", "node": {...}}
{"command": "list", "items": [...]}
```

**Proposed:**
```json
{
  "command": "show",
  "status": "ok",
  "timestamp": "2026-02-06T11:47:12Z",
  "data": {
    "type": "node",
    "node": {...}
  }
}
```

**Effort:** 4-5 hours  
**Impact:** High (scripting)

#### 3.2 Add `--follow` Flag for Recursive Navigation

**Current:**
```bash
$ mindmap-cli show 42
# User must manually follow to [15]
$ mindmap-cli show 15
# etc.
```

**Proposed:**
```bash
$ mindmap-cli show 42 --follow 2
[42] DONE: Consolidate...
  └─ [15] AE: mindmap-cli
      └─ [10] Project purpose
```

**Effort:** 5-6 hours  
**Impact:** Medium (navigation)

#### 3.3 Add Search Flags

**Current:**
```bash
mindmap-cli search "auth"  # Case-insensitive substring only
```

**Proposed:**
```bash
mindmap-cli search "auth"                 # Case-insensitive substring
mindmap-cli search "auth" --case-sensitive  # Case-sensitive
mindmap-cli search "auth" --exact-match   # Whole-word match
mindmap-cli search "auth" --regex "^AE: .*auth" # Regex support
```

**Effort:** 3-4 hours  
**Impact:** Medium (power users)

---

## Part 4: Documentation Improvements

### 4.1 Create Quick Start Guide

**Location:** Quick reference in README

```markdown
## Quick Reference

| Task | Command |
|------|---------|
| View all nodes | `mindmap-cli list` |
| Find nodes with "auth" | `mindmap-cli search "auth"` |
| Show node details | `mindmap-cli show 12` |
| View incoming refs | `mindmap-cli refs 12` |
| View outgoing refs | `mindmap-cli links 12` |
| Add new node | `mindmap-cli add --type AE --title "..." --body "..."` |
| Edit node | `mindmap-cli edit 12` |
| Validate file | `mindmap-cli lint` |
| Find isolated nodes | `mindmap-cli orphans` |
```

### 4.2 Create Tutorial for New Users

**Topics:**
1. What is a MINDMAP file?
2. Node format and types
3. Basic queries (list, search, show)
4. Navigation (refs, links)
5. Editing (add, patch, edit)
6. Batch operations
7. JSON scripting

### 4.3 Add Command Aliases

**Current:** `search` is already an alias for `list --grep`

**Proposed additions:**
- `incoming` → `refs` (with direction clarification)
- `outgoing` → `links` (with direction clarification)
- `inspect` → `show` (for discoverability)
- `validate` → `lint`
- `find` → `search`

---

## Part 5: Implementation Roadmap

### Phase 1: Quick Wins (Week 1)
- [ ] Add result counts ("2 matches", "No results")
- [ ] Improve help text for refs/links
- [ ] Add "No results" messages
- [ ] Update README with quick reference
- [ ] Enhance error messages with hints

**Effort:** 5-8 hours  
**Impact:** High visibility improvements

### Phase 2: Feature Additions (Week 2-3)
- [ ] Add `types` command
- [ ] Add `relationships` command
- [ ] Improve `orphans` output with `--with-descriptions`
- [ ] Add search flags (--case-sensitive, --exact-match)
- [ ] Add command aliases (incoming, outgoing, etc.)

**Effort:** 15-20 hours  
**Impact:** Medium to high

### Phase 3: Advanced Features (Week 4+)
- [ ] Standardize JSON output structure
- [ ] Add `--follow` flag for recursive navigation
- [ ] Implement file locking
- [ ] Add backup/undo support
- [ ] Performance optimizations for large files

**Effort:** 20+ hours  
**Impact:** High for power users

---

## Part 6: Quick Wins Summary

| Issue | Fix | Time | Impact |
|-------|-----|------|--------|
| "No results" silent failure | Output "No matches" | 1h | High |
| Refs/links unclear | Better help text + arrows in output | 1h | High |
| Orphans is cryptic | Add descriptions flag | 2h | Medium |
| Types not discoverable | New `types` command | 4h | Medium |
| Missing result counts | Add headers and counts | 1h | High |
| Error messages unhelpful | Add hints and suggestions | 2h | Medium |
| **Total** | | **11h** | **High** |

---

## Part 7: Conclusion

`mindmap-cli` is a solid, well-engineered tool with strong fundamentals. The main opportunities are:

1. **Clarity:** Better output headers, result counts, and contextual messages
2. **Discoverability:** Help text, commands for learning available types/relationships
3. **Efficiency:** Combined commands (relationships), recursive navigation (--follow)
4. **Power:** Search flags, JSON schema, better batch error messages
5. **Safety:** File locking, undo support, backup options

**Immediate recommendations:**
1. Implement Quick Wins (Priority 1) for immediate UX improvements
2. Add `types` and `relationships` commands for better discoverability
3. Standardize JSON output for scriptability
4. Create comprehensive quick-start guide

**Time to significant UX improvement:** ~15-20 hours of development

---

## Appendix: Testing Recommendations

For each improvement, add tests covering:
- Empty results (list --grep with no matches)
- Single result
- Multiple results
- Large result sets (performance)
- Error cases with hints
- JSON output structure consistency

Example test:
```rust
#[test]
fn test_empty_search_shows_message() {
    // cmd_search with no results should output "No matches"
    let items = cmd_list(&mm, None, Some("XYZABC"));
    assert!(items.is_empty());  // Currently true
    // After fix: test that empty output still works in JSON
}
```
