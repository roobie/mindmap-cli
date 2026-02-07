# Implementation Guide: Multi-ID Selection (Phase 1)

## Summary
Enable mutation commands (`show`, `delete`, `patch`, `verify`, `deprecate`) to accept multiple node IDs at once. This is a **backward-compatible enhancement** where `show 12` and `show 12 15 19` both work naturally.

---

## Changes Required

### 1. Update `Commands` Enum in lib.rs

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... other commands ...

    /// Show node(s) by ID
    Show { 
        /// Node IDs to show (one or more)
        ids: Vec<u32> 
    },

    /// Delete node(s) by ID; use --force to remove even if referenced
    Delete {
        /// Node IDs to delete (one or more)
        ids: Vec<u32>,
        #[arg(long)]
        force: bool,
    },

    /// Patch (partial update) node(s): --type, --title, --body
    Patch {
        /// Node IDs to patch (one or more)
        ids: Vec<u32>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        strict: bool,
    },

    /// Mark node(s) as needing verification (append verify tag)
    Verify { 
        /// Node IDs to verify (one or more)
        ids: Vec<u32> 
    },

    /// Deprecate node(s), redirecting to another
    Deprecate {
        /// Node IDs to deprecate (one or more)
        ids: Vec<u32>,
        #[arg(long)]
        to: u32,
    },
}
```

### 2. Update Command Handlers in `run()` Function

```rust
pub fn run(cli: Cli) -> Result<()> {
    // ... load mindmap ...
    
    match &cli.command {
        // ...
        
        Commands::Show { ids } => {
            let results = cmd_multi_show(&mm, ids);
            for line in results {
                println!("{}", line);
            }
        }

        Commands::Delete { ids, force } => {
            cmd_multi_delete(&mut mm, ids, *force)?;
            let count = ids.len();
            let msg = format!("Deleted {} node(s)", count);
            match cli.output {
                OutputFormat::Json => println!("{}", serde_json::json!({
                    "deleted": count,
                    "ids": ids
                })),
                OutputFormat::Default => eprintln!("{}", msg),
            }
        }

        Commands::Patch {
            ids,
            r#type,
            title,
            body,
            strict,
        } => {
            cmd_multi_patch(&mut mm, ids, type_.as_deref(), title.as_deref(), body.as_deref(), *strict)?;
            eprintln!("Patched {} node(s)", ids.len());
        }

        Commands::Verify { ids } => {
            cmd_multi_verify(&mut mm, ids)?;
            eprintln!("Verified {} node(s)", ids.len());
        }

        Commands::Deprecate { ids, to } => {
            cmd_multi_deprecate(&mut mm, ids, *to)?;
            eprintln!("Deprecated {} node(s) → {}", ids.len(), to);
        }
        
        // ... rest of commands ...
    }
}
```

### 3. Implement Multi-Node Functions

```rust
/// Show multiple nodes
pub fn cmd_multi_show(mm: &Mindmap, ids: &[u32]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| {
            mm.get_node(*id).map(|_| cmd_show(mm, *id))
        })
        .collect()
}

/// Delete multiple nodes
pub fn cmd_multi_delete(mm: &mut Mindmap, ids: &[u32], force: bool) -> Result<()> {
    for id in ids {
        cmd_delete(mm, *id, force)?;
    }
    mm.save()
}

/// Patch multiple nodes (same update to all)
pub fn cmd_multi_patch(
    mm: &mut Mindmap,
    ids: &[u32],
    type_: Option<&str>,
    title: Option<&str>,
    body: Option<&str>,
    strict: bool,
) -> Result<()> {
    // Validate all IDs exist first
    for id in ids {
        if mm.get_node(*id).is_none() {
            anyhow::bail!("Node {} not found", id);
        }
    }

    // Apply patch to each node
    for id in ids {
        cmd_patch(mm, *id, type_, title, body, strict)?;
    }
    
    mm.save()
}

/// Verify multiple nodes
pub fn cmd_multi_verify(mm: &mut Mindmap, ids: &[u32]) -> Result<()> {
    for id in ids {
        cmd_verify(mm, *id)?;
    }
    mm.save()
}

/// Deprecate multiple nodes to a single target
pub fn cmd_multi_deprecate(mm: &mut Mindmap, ids: &[u32], to: u32) -> Result<()> {
    // Validate target exists
    if mm.get_node(to).is_none() {
        anyhow::bail!("Target node {} not found", to);
    }

    for id in ids {
        cmd_deprecate(mm, *id, to)?;
    }
    
    mm.save()
}
```

---

## Testing Strategy

### Unit Tests (in lib.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_multi_show_single_id() {
        let mm = Mindmap::from_string(
            "[10] **AE: Test** - body [11]\n[11] **WF: Other** - test",
            PathBuf::from("test.md"),
        ).unwrap();
        let results = cmd_multi_show(&mm, &[10]);
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("[10]"));
    }

    #[test]
    fn test_cmd_multi_show_multiple_ids() {
        let mm = Mindmap::from_string(
            "[10] **AE: Test** - body\n[11] **WF: Other** - test\n[12] **DR: Why** - reason",
            PathBuf::from("test.md"),
        ).unwrap();
        let results = cmd_multi_show(&mm, &[10, 12]);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_cmd_multi_show_missing_id() {
        let mm = Mindmap::from_string(
            "[10] **AE: Test** - body",
            PathBuf::from("test.md"),
        ).unwrap();
        let results = cmd_multi_show(&mm, &[10, 99]);
        // Should skip missing ID
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_cmd_multi_delete() {
        let mut mm = Mindmap::from_string(
            "[10] **AE: Test** - body\n[11] **WF: Other** - test\n[12] **DR: Why** - reason",
            PathBuf::from("test.md"),
        ).unwrap();
        
        cmd_multi_delete(&mut mm, &[10, 12], true).unwrap();
        
        assert!(mm.get_node(10).is_none());
        assert!(mm.get_node(12).is_none());
        assert!(mm.get_node(11).is_some());
    }

    #[test]
    fn test_cmd_multi_patch() {
        let mut mm = Mindmap::from_string(
            "[10] **AE: Test** - body\n[11] **AE: Other** - test",
            PathBuf::from("test.md"),
        ).unwrap();
        
        cmd_multi_patch(&mut mm, &[10, 11], None, Some("Updated"), None, false).unwrap();
        
        let n10 = mm.get_node(10).unwrap();
        let n11 = mm.get_node(11).unwrap();
        
        assert_eq!(n10.title, "Updated");
        assert_eq!(n11.title, "Updated");
    }

    #[test]
    fn test_cmd_multi_patch_missing_id_fails() {
        let mut mm = Mindmap::from_string(
            "[10] **AE: Test** - body",
            PathBuf::from("test.md"),
        ).unwrap();
        
        let result = cmd_multi_patch(&mut mm, &[10, 99], None, Some("New"), None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_multi_verify() {
        let mut mm = Mindmap::from_string(
            "[10] **AE: Test** - body\n[11] **AE: Other** - test",
            PathBuf::from("test.md"),
        ).unwrap();
        
        cmd_multi_verify(&mut mm, &[10, 11]).unwrap();
        
        let n10 = mm.get_node(10).unwrap();
        assert!(n10.body.contains("(verify"));
    }

    #[test]
    fn test_cmd_multi_deprecate() {
        let mut mm = Mindmap::from_string(
            "[10] **AE: Old** - body\n[11] **AE: Newer** - body\n[99] **AE: Target** - target",
            PathBuf::from("test.md"),
        ).unwrap();
        
        cmd_multi_deprecate(&mut mm, &[10, 11], 99).unwrap();
        
        let n10 = mm.get_node(10).unwrap();
        let n11 = mm.get_node(11).unwrap();
        
        assert!(n10.title.contains("[DEPRECATED → 99]"));
        assert!(n11.title.contains("[DEPRECATED → 99]"));
    }
}
```

### Integration Tests (tests/cli.rs)

```rust
#[test]
fn test_multi_show_via_cli() {
    let temp_file = create_temp_mindmap(
        "[10] **AE: Test** - body\n[11] **WF: Other** - test\n[12] **DR: Why** - reason"
    );
    
    let output = run_cli(&["show", "10", "12", "--file", temp_file.path().to_str().unwrap()]);
    
    assert!(output.contains("[10]"));
    assert!(output.contains("[12]"));
    assert!(!output.contains("[11]"));
}

#[test]
fn test_multi_delete_via_cli() {
    let temp_file = create_temp_mindmap(
        "[10] **AE: Test** - body\n[11] **WF: Other** - test\n[12] **DR: Why** - reason"
    );
    
    run_cli(&["delete", "10", "12", "--force", "--file", temp_file.path().to_str().unwrap()]);
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(!content.contains("[10]"));
    assert!(content.contains("[11]"));
    assert!(!content.contains("[12]"));
}

#[test]
fn test_multi_patch_via_cli() {
    let temp_file = create_temp_mindmap(
        "[10] **AE: Test** - body\n[11] **AE: Other** - test"
    );
    
    run_cli(&["patch", "10", "11", "--title", "Updated", "--file", temp_file.path().to_str().unwrap()]);
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("**Updated**"));
}
```

---

## Backward Compatibility

✅ **Fully backward compatible:**
- `mindmap show 12` becomes `Show { ids: vec![12] }`
- `mindmap delete 12 --force` becomes `Delete { ids: vec![12], force: true }`
- All existing scripts continue to work
- No breaking changes to CLI interface

---

## Error Handling

### Empty ID List
```bash
$ mindmap show
Error: expected at least one ID
```

### Missing Node
```bash
$ mindmap show 12 99
Error: Node 99 not found

# (For show, could be lenient and skip missing)
```

### Delete with References (without --force)
```bash
$ mindmap delete 12 15
Error: Cannot delete node 12; it is referenced by [10][11]

# Add --force to proceed anyway
```

### Patch with Invalid References (--strict)
```bash
$ mindmap patch 12 --body "See [99]" --strict
Error: Referenced node 99 not found
```

---

## CLI Help Text Update

```
Show node(s) by ID

USAGE:
    mindmap show [IDS]...

ARGS:
    <IDS>...    Node IDs to show (one or more)

EXAMPLES:
    mindmap show 10
    mindmap show 10 15 20
    mindmap show $(mindmap links 50 --output json | jq -r '.references[].id')
```

---

## Documentation Updates

1. **MINDMAP.md**: Add notes about multi-ID support
   ```markdown
   [X] **WF: Multi-node operations** - Implemented: show, delete, patch, verify, deprecate accept multiple IDs
   ```

2. **DESIGN.md**: Section 3, add multi-ID examples
3. **README.md**: Add examples showing multi-ID usage
4. **PROTOCOL_MINDMAP.md**: Update examples to show multi-ID patterns

---

## Performance Considerations

### Atomicity
- Each operation still goes through a single `mm.save()` call
- **Atomic guarantee maintained**: all-or-nothing

### Speed
- No significant overhead (linear in number of IDs)
- Typical usage: 2-20 IDs per operation
- No performance regression for single-ID operations

### Memory
- IDs are small (u32), minimal overhead
- `Vec<u32>` is efficient

---

## Migration Path from Current Code

1. **Minimal refactor**: Make Show/Delete/Patch/Verify/Deprecate accept `Vec<u32>` instead of `u32`
2. **Reuse existing logic**: New `cmd_multi_*` functions call existing `cmd_*` functions in loop
3. **No changes to file format**: Still one-line-per-node
4. **No changes to Mindmap struct**: Just accept multiple IDs upfront

---

## Success Criteria

- ✅ All existing tests pass
- ✅ 10+ new unit tests for multi-ID operations
- ✅ 5+ new integration tests
- ✅ Multi-ID examples work as documented
- ✅ Help text updated
- ✅ No performance regression
- ✅ Atomic guarantees maintained
- ✅ Error messages are clear

---

## Timeline Estimate

| Task | Time | Notes |
|------|------|-------|
| Update Commands enum | 15 min | Straightforward |
| Implement cmd_multi_* functions | 30 min | Reuse existing code |
| Update run() handler | 15 min | Pattern matching changes |
| Unit tests | 60 min | ~10 test cases |
| Integration tests | 30 min | ~5 test cases |
| Docs + help text | 15 min | Copy from Phase 1 plan |
| Testing & validation | 30 min | Manual testing, lint |
| **Total** | **195 min** | **~3.25 hours** |

