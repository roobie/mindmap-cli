# Quick Reference: search vs list --grep

## Current Behavior

### `search` Command
```bash
mindmap-cli search "auth"
```
Output: All nodes containing "auth" in title or description

### `list` Command  
```bash
mindmap-cli list                     # all nodes
mindmap-cli list --type AE           # only AE: nodes
mindmap-cli list --grep "auth"       # only nodes with "auth"
mindmap-cli list --type AE --grep "auth"  # AE: nodes with "auth"
```

---

## Implementation Comparison

### Side-by-Side Execution Flow

```
┌─ User Input ──────────────────────────────────────┐
│                                                    │
├─ search "auth"       ──────────────────────────┐  │
│  ↓                                              │  │
│  Commands::Search { query: "auth" }             │  │
│  ↓                                              │  │
│  cmd_search(mm, "auth")                        │  │
│  → loop: n.title.contains("auth") ||            │  │
│          n.description.contains("auth")         │  │
│  ↓                                              │  │
│  Output: matching nodes                         │  │
│                                                 │  │
├─ list --grep "auth"  ──────────────────────────┤  │
│  ↓                                              │  │
│  Commands::List { type_filter: None,            │  │
│                   grep: Some("auth") }          │  │
│  ↓                                              │  │
│  cmd_list(mm, None, Some("auth"))              │  │
│  → loop: n.title.contains("auth") ||            │  │
│          n.description.contains("auth")         │  │
│  ↓                                              │  │
│  Output: matching nodes                         │  │
│                                                 │  │
└─ IDENTICAL RESULTS ──────────────────────────────┘
```

### Code Diff

**cmd_search** (lines ~555–568):
```rust
pub fn cmd_search(mm: &Mindmap, query: &str) -> Vec<String> {
    let qlc = query.to_lowercase();
    let mut out = Vec::new();
    for n in &mm.nodes {
        if n.raw_title.to_lowercase().contains(&qlc) || 
           n.description.to_lowercase().contains(&qlc) {
            out.push(format!("[{}] **{}** - {}", n.id, n.raw_title, n.description));
        }
    }
    out
}
```

**cmd_list** (lines ~530–547):
```rust
pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    let mut res = Vec::new();
    for n in &mm.nodes {
        if let Some(tf) = type_filter && !n.raw_title.starts_with(&format!("{}:", tf)) {
            continue;
        }
        if let Some(q) = grep {
            let qlc = q.to_lowercase();
            if !n.raw_title.to_lowercase().contains(&qlc) &&
               !n.description.to_lowercase().contains(&qlc) {
                continue;
            }
        }
        res.push(format!("[{}] **{}** - {}", n.id, n.raw_title, n.description));
    }
    res
}
```

**Observation:** The grep logic (highlighted) is **identical** between both functions.

---

## Proposed Consolidation

### Before (Status Quo)

```rust
pub fn cmd_search(mm: &Mindmap, query: &str) -> Vec<String> {
    // [~13 lines of search-specific code]
}

pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    // [~17 lines of list-specific code]
    // [DUPLICATES the grep logic from cmd_search]
}

// In Commands::Search handler:
let items = cmd_search(&mm, &query);

// In Commands::List handler:
let items = cmd_list(&mm, type_filter.as_deref(), grep.as_deref());
```

### After (Proposed)

```rust
pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    // [~17 lines, single implementation]
}

// In Commands::Search handler:
let items = cmd_list(&mm, None, Some(&query));  // ← delegates to list

// In Commands::List handler:
let items = cmd_list(&mm, type_filter.as_deref(), grep.as_deref());
```

**Result:**
- ✅ Eliminate ~13 lines of duplicated code
- ✅ Single source of truth for grep logic
- ✅ Bug fixes apply to both commands
- ✅ No breaking changes (both commands still exist)
- ✅ Minimal diff (remove cmd_search function, add delegation in Commands::Search)

---

## Testing the Consolidation

### Test Case: Equivalence

After consolidation, ensure:

```rust
#[test]
fn test_search_list_grep_equivalence() -> Result<()> {
    let mm = load_test_mindmap()?;
    
    let search_results = cmd_search(&mm, "auth");
    let list_results = cmd_list(&mm, None, Some("auth"));
    
    assert_eq!(search_results, list_results);
    Ok(())
}
```

### Test Case: Type Filtering Still Works

```rust
#[test]
fn test_list_with_type_filter() -> Result<()> {
    let mm = load_test_mindmap()?;
    
    let list_type = cmd_list(&mm, Some("AE"), None);
    let search = cmd_search(&mm, "");
    
    // list with type should be subset of search
    assert!(list_type.len() <= search.len());
    Ok(())
}
```

---

## CLI Documentation Impact

After consolidation, help text should clarify:

```
  search <QUERY>
          Search nodes by substring
          (Equivalent to: list --grep <QUERY>)

  list [OPTIONS]
          List nodes (optionally filtered by type and/or grep pattern)
          
          Options:
            --type <TYPE>    Filter by node type prefix (e.g., AE, WF, DR)
            --grep <PATTERN> Filter by substring in title or description
```

---

## Decision Checklist

Before implementing consolidation:

- [ ] Confirm both commands produce identical output for same query
- [ ] Verify no edge cases differ (empty query, special chars, etc.)
- [ ] Review test coverage for both commands
- [ ] Plan help text updates
- [ ] Verify backward compatibility
- [ ] Update DESIGN.md if needed
- [ ] Plan MINDMAP.md node updates

---

## Summary

| Metric | Value |
|--------|-------|
| Lines of duplicated code | ~13 |
| Implementation effort | Low (~5 lines) |
| Breaking changes | None |
| Test coverage impact | None (both commands still tested) |
| User experience impact | None (both commands still exist) |
| Maintenance benefit | High (single source of truth) |

**Recommendation:** Implement consolidation. Low effort, high maintainability gain.
