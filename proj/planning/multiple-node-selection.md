# Planning: Multiple Node Selection in mindmap-cli

## Problem Statement
Currently, `mindmap-cli` operates on single nodes per command (e.g., `show <id>`, `patch <id>`, `delete <id>`). For agent workflows, there's a need to:
- **Select multiple nodes** by various criteria (ID ranges, type filters, query matches)
- **Operate on that selection** atomically (batch delete, mass-patch, show-all-matching, etc.)
- **Provide clear, agent-friendly syntax** that's composable and scriptable

## Current State
- **Batch mode** exists (`mindmap batch`) but requires **explicit operation syntax** per node:
  ```bash
  mindmap batch --input - <<EOF
  patch 12 --title "New"
  patch 15 --title "New"
  delete 19
  EOF
  ```
- **List/search** commands return node IDs but don't directly feed into mutation operations
- No native "select then operate" pipeline

## Design Goals
1. **Agent-friendly UX**: Simple, composable, no complex shell scripting needed
2. **Safety**: Maintain atomic semantics and validation guarantees
3. **Backward compatible**: Don't break existing commands
4. **Scalable**: Works for 10 nodes or 100+ nodes
5. **Predictable**: Agents can predict output and construct further commands deterministically

## Proposed Solutions

### Solution A: Multi-ID Arguments (Simplest)
Allow commands that operate on multiple IDs to accept multiple arguments:
```bash
# Show multiple nodes
mindmap show 12 15 19

# Delete multiple nodes
mindmap delete 12 15 19 --force

# Patch multiple nodes (same update)
mindmap patch 12 15 19 --title "Updated Title"
```

**Pros:**
- Minimal syntax change
- Natural shell expansion
- Works well for small selections
- Agents can easily generate: `mindmap delete $(jq -r '.ids[]' selection.json)`

**Cons:**
- Doesn't work for *different* patches per node (e.g., patch 12 with one title, 15 with another)
- Command line length limits for very large selections (bash/shell ARG_MAX)
- Less composable with filters

**Implementation:**
```rust
Show { ids: Vec<u32> },
Delete { ids: Vec<u32>, force: bool },
Patch { ids: Vec<u32>, ... },
Verify { ids: Vec<u32> },
Deprecate { ids: Vec<u32>, to: u32 },  // redirect all to single target
```

### Solution B: Filtered Selection Commands (Most Flexible)
Extend `list`/`search` to emit a "selection file" that can be piped into batch mode:

```bash
# Step 1: Select via filter
mindmap list --type WF --grep "workflow" --select-output selection.json

# Step 2: Operate on selection
mindmap batch --select-file selection.json --op patch --title "Updated"
```

**Pros:**
- Highly composable
- Agents can save intermediate selections
- Powerful filtering without code changes
- Scales to any number of nodes

**Cons:**
- Two-step process (more commands)
- Requires new `--select-output` and `--select-file` flags
- Slightly more complex

**Implementation:**
- Add `--select-output <FILE>` to `list`/`search`/`refs`/`links`
- Emits JSON: `{ "ids": [12, 15, 19], "source": "list", "filter": {...} }`
- Add `--select-file` to `batch` or new `apply` command
- Apply the same operation to all selected IDs

### Solution C: Pipe-Based Selection (Unix Philosophy)
Leverage existing JSON output + new `apply` command:

```bash
# Select nodes
mindmap list --type WF --output json | \
  jq -r '.nodes[].id' | \
  mindmap apply --op patch --title "Updated" --from-stdin

# Or with xargs
mindmap search "critical" --output json | \
  jq -r '.nodes[].id' | \
  xargs mindmap verify
```

**Pros:**
- Fully Unix-native and composable
- Works with existing tools (jq, xargs, awk, etc.)
- Agents already familiar with pipes
- Zero CLI change for single-op reads

**Cons:**
- Requires agents to know jq syntax
- Not atomic at CLI level (multiple invocations)
- Can hit ARG_MAX limits with xargs

**Implementation:**
- Add `--from-stdin` to mutation commands (patch, delete, verify, deprecate)
- Read newline-separated IDs from stdin
- Apply operation to each ID
- Batch mode under the hood for atomicity

### Solution D: Interactive Selection UI (Least Agent-Friendly)
Use TUI for selection, then confirm operation. Not recommended for agents.

---

## Recommendation: **Hybrid Approach (A + C)**

### Phase 1: Multi-ID Arguments (Quick Win)
Implement **Solution A** for immediate agent support:
- `mindmap show 12 15 19`
- `mindmap delete 12 15 19 --force`
- `mindmap patch 12 15 19 --title "New"`
- `mindmap verify 12 15 19`
- `mindmap deprecate 12 15 19 --to 100`

**Benefits:** Easy to implement, covers 80% of use cases, no breaking changes

### Phase 2: Stdin-Based Selection (Composability)
Add `--from-stdin` flag to mutation commands:
```bash
mindmap list --type WF --output json | \
  jq -r '.nodes[].id' | \
  mindmap patch --from-stdin --title "Updated WFs"
```

**Benefits:** Unlocks powerful compositions, works with existing filters, Unix-native

### Phase 3: Selection Files (Optional, Future)
If needed for complex workflows:
```bash
mindmap list --type AE --select-file ae-nodes.json
mindmap batch --select-file ae-nodes.json --op delete --force
```

---

## Implementation Plan for Phase 1 (Multi-ID Arguments)

### Changes to lib.rs enum Commands:
```rust
Show { ids: Vec<u32> },
Delete { ids: Vec<u32>, force: bool },
Patch { ids: Vec<u32>, ... },
Verify { ids: Vec<u32> },
Deprecate { ids: Vec<u32>, to: u32 },
```

### New internal function: `cmd_multi_<op>`
```rust
fn cmd_multi_show(mm: &Mindmap, ids: &[u32]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| mm.get_node(*id).map(|n| format_node(n)))
        .collect()
}

fn cmd_multi_delete(mm: &mut Mindmap, ids: &[u32], force: bool) -> Result<()> {
    for id in ids {
        cmd_delete(mm, *id, force)?;
    }
    mm.save()
}

fn cmd_multi_patch(
    mm: &mut Mindmap,
    ids: &[u32],
    type_: Option<&str>,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<()> {
    for id in ids {
        cmd_patch(mm, *id, type_, title, body, false)?;
    }
    mm.save()
}
```

### CLI changes (clap):
```rust
#[derive(Parser)]
pub struct ShowArgs {
    /// Node IDs to show (can be multiple)
    ids: Vec<u32>,
}

#[derive(Parser)]
pub struct DeleteArgs {
    /// Node IDs to delete (can be multiple)
    ids: Vec<u32>,
    #[arg(long)]
    force: bool,
}
```

### Tests:
- Multi-show with valid IDs → output all
- Multi-show with missing ID → warn, show others
- Multi-delete → all deleted, refs updated
- Multi-patch → all patched atomically

---

## UX Examples for Agents

### Batch delete a list of IDs:
```bash
mindmap delete 12 15 19 --force
# Or from agent-generated file:
NODES=$(jq -r '.to_delete[]' plan.json)
mindmap delete $NODES --force
```

### Verify multiple nodes:
```bash
mindmap verify 5 10 15 20
```

### Patch all nodes of a type (two-step for now):
```bash
IDS=$(mindmap list --type WF --output json | jq -r '.nodes[].id')
mindmap patch $IDS --title "WF: Updated"
```

### Show a dependency chain:
```bash
mindmap show 15
mindmap show $(mindmap links 15 --output json | jq -r '.references[].id')
```

---

## Backward Compatibility
- Single-ID commands continue to work: `mindmap show 12` (ids=[12])
- All existing tests pass without modification
- No breaking changes to existing scripts

---

## Risks & Mitigations
| Risk | Mitigation |
|------|-----------|
| ARG_MAX limits (shell) | Document; agents should use stdin mode for 100+ IDs |
| Different updates per node | Use batch mode explicitly for heterogeneous updates |
| Partial failure in multi-op | Fail early, abort transaction, error message lists failed IDs |
| Atomicity across multiple calls | Rely on existing mindmap-cli atomicity per call; batch for stronger guarantee |

---

## Next Steps
1. **Agree on Phase 1 scope**: Multi-ID show/delete/patch/verify/deprecate?
2. **Create GitHub issue** for tracking
3. **Open draft PR** with multi-ID args
4. **Collect feedback** from agent usage
5. **Plan Phase 2** if stdin mode is needed

---

## Related Issues
- #15: Batch mode (already implemented ✓)
- #25: Stdin support for read-only ops (already implemented ✓)
- Future: Pipe composition with jq/xargs (needs Phase 2)

