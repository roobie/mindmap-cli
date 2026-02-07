# Analysis: `search` vs `list --grep` Consolidation

**Date:** 2026-02-06

## Overview
The `mindmap-cli` currently exposes two similar commands for searching nodes by substring:
1. **`mindmap-cli search <query>`** - dedicated search command
2. **`mindmap-cli list --grep <pattern>`** - grep option on list command

This analysis examines whether consolidation is needed.

---

## Current Implementation

### `cmd_search(mm, query)` 
**Location:** `src/lib.rs` lines ~555–568

```rust
pub fn cmd_search(mm: &Mindmap, query: &str) -> Vec<String> {
    let qlc = query.to_lowercase();
    let mut out = Vec::new();
    for n in &mm.nodes {
        if n.raw_title.to_lowercase().contains(&qlc) || n.body.to_lowercase().contains(&qlc)
        {
            out.push(format!(
                "[{}] **{}** - {}",
                n.id, n.raw_title, n.body
            ));
        }
    }
    out
}
```

**Key characteristics:**
- Takes a single `query: &str` positional argument
- Searches both title and body (case-insensitive)
- Returns formatted node lines
- No type filtering

### `cmd_list(mm, type_filter, grep)`
**Location:** `src/lib.rs` lines ~530–547

```rust
pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    let mut res = Vec::new();
    for n in &mm.nodes {
        if let Some(tf) = type_filter
            && !n.raw_title.starts_with(&format!("{}:", tf))
        {
            continue;
        }
        if let Some(q) = grep {
            let qlc = q.to_lowercase();
            if !n.raw_title.to_lowercase().contains(&qlc)
                && !n.body.to_lowercase().contains(&qlc)
            {
                continue;
            }
        }
        res.push(format!(
            "[{}] **{}** - {}",
            n.id, n.raw_title, n.body
        ));
    }
    res
}
```

**Key characteristics:**
- Supports optional type filtering (`--type <TYPE>`)
- Supports optional grep filtering (`--grep <pattern>`)
- Can combine type + grep filters
- Same case-insensitive substring match behavior as search
- Returns formatted node lines

### CLI Definitions
**Location:** `src/lib.rs` lines ~78–100

```rust
/// List nodes (optionally filtered)
List {
    #[arg(long)]
    r#type: Option<String>,
    #[arg(long)]
    grep: Option<String>,
},

/// Search nodes by substring
Search { query: String },
```

---

## Comparison

| Aspect | `search` | `list --grep` |
|--------|----------|---------------|
| **Usage** | `mindmap-cli search "term"` | `mindmap-cli list --grep "term"` |
| **Type filter** | ❌ No | ✅ Yes (`--type AE`) |
| **Grep filter** | ✅ Yes (query) | ✅ Yes (`--grep`) |
| **Implementation** | Single-purpose | Multi-purpose (with optional filters) |
| **Cognitive load** | Simple, direct | More options, more flexible |
| **Core logic** | Identical substring matching | Identical substring matching |

---

## Analysis: Consolidation Options

### Option 1: Keep Both (Status Quo) ✓
**Pros:**
- Simple UX for common case: `search "term"` is more intuitive than `list --grep "term"`
- Type filtering only when needed
- Backward compatible

**Cons:**
- Code duplication (identical search logic)
- User confusion: when to use which?
- Maintenance: fix bugs in two places
- Inconsistent with minimalist design philosophy

**Recommendation:** ❌ Not optimal

---

### Option 2: Deprecate `search`, Make `list --grep` Primary ✓
**Approach:**
- Remove `search` subcommand entirely
- Use `list --grep "term"` for substring search
- Combine filters: `list --type AE --grep "auth"`

**Pros:**
- Single source of truth
- Reduces CLI surface area
- More powerful (type + grep together)
- No code duplication

**Cons:**
- Longer invocation for simple searches
- `list` becomes overloaded with optional filters
- Breaking change for existing scripts

**Recommendation:** ⚠️ Viable but UX impact

---

### Option 3: Consolidate, Alias `search` to `list --grep` 
**Approach:**
- Keep `search` as a convenience alias/subcommand
- Internally delegates to `list --grep`
- Both commands share identical implementation

```rust
Commands::Search { query } => {
    // Delegate to list with only grep filter
    let items = cmd_list(&mm, None, Some(&query));
    // ... output handling
}
```

**Pros:**
- No code duplication
- Backward compatible
- Simple search UX preserved
- Type filtering still available via `list`
- Single underlying logic path

**Cons:**
- Still have two command definitions (minor)
- Users must learn both exist, but that's a documentation problem

**Recommendation:** ✅ **Best option**

---

### Option 4: Consolidate with New `filter` Command
**Approach:**
- Replace both with a more explicit `filter` command
- `mindmap-cli filter [--type AE] [--grep term]`

**Pros:**
- Clear semantics
- Extensible for future filters
- Single, unified filtering interface

**Cons:**
- More breaking changes
- `list` without args should still work (what does it return?)
- Additional complexity for simple case

**Recommendation:** ❌ Overengineering

---

## Recommended Solution: Option 3

**Rationale:**
1. **DRY principle**: Single `cmd_list()` function handles all logic
2. **UX improvement**: `search` remains simple, `list` offers power
3. **Backward compatible**: No breaking changes
4. **Minimal change**: Few lines to modify
5. **Discoverability**: Help text clarifies the relationship

**Implementation Details:**

1. Keep both command definitions in `Commands` enum
2. Unify output handling: both `Search` and `List` routes through the same printer calls
3. Update help text to note relationship:
   ```
   /// Search nodes by substring (equivalent to: list --grep)
   Search { query: String },
   ```

4. Optional: add a comment in code explaining they're aliases

**Example code change:**
```rust
Commands::Search { query } => {
    // Search is an alias for list --grep
    let items = cmd_list(&mm, None, Some(&query));
    // (reuse exact same output handling as List command)
}

Commands::List { r#type, grep } => {
    let items = cmd_list(&mm, r#type.as_deref(), grep.as_deref());
    // (output handling)
}
```

---

## Migration Path (if consolidation is desired)

**Phase 1 (Current):** Both commands exist, both work
**Phase 2:** Add deprecation warning if `search` is used  
  `eprintln!("Note: 'search' is equivalent to 'list --grep'; both are supported")`
**Phase 3 (v1.0):** Decide whether to remove `search`

---

## Questions for Review

1. **Is code duplication significant enough to address now?**
   - Current: ~40 bytes of identical logic in two functions
   - Overhead is minimal; could argue for keeping as-is

2. **Do we want to encourage type-filtered searches?**
   - If yes: promote `list --type AE --grep auth` over plain `search`
   - If no: `search` is the happy path

3. **Should documentation mention this relationship?**
   - Yes: help text and README should note `search` ≈ `list --grep`

---

## Conclusion

**Recommended action:** Implement **Option 3** (Alias approach)

- ✅ Minimal code change
- ✅ Eliminates code duplication  
- ✅ Preserves UX for both simple and advanced cases
- ✅ No breaking changes
- ✅ Single implementation path for future fixes

**Implementation effort:** ~5 lines of code change (add comments, verify output routing is identical)

**Testing:** Verify both commands produce identical output for same query; add test case comparing outputs.
