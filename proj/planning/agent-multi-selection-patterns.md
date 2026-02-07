# Agent Quick Reference: Multi-Node Selection Patterns

## Current (v0) — Single Node Only
```bash
mindmap show 12
mindmap delete 12 --force
mindmap patch 12 --title "Updated"
```

---

## Phase 1: Multi-ID Arguments (Coming in v0.1)

### Show Multiple Nodes
```bash
mindmap show 12 15 19
mindmap show 5 10 15 20 25
```

### Delete Multiple Nodes
```bash
mindmap delete 12 15 19 --force
mindmap delete $(jq -r '.ids_to_remove[]' plan.json) --force
```

### Patch Multiple Nodes (same update)
```bash
mindmap patch 12 15 19 --title "WF: Updated Title"
mindmap patch 10 11 12 --body "Handles auth and security [50]"
mindmap patch 20 21 22 --type WF
```

### Verify Multiple Nodes
```bash
mindmap verify 5 8 12 15
```

### Deprecate Multiple Nodes (redirect all to same target)
```bash
mindmap deprecate 10 11 12 --to 50
# All old node titles become: [DEPRECATED → 50] Original Title
```

---

## Common Agent Patterns (Phase 1)

### Find & Update by Type
```bash
# Step 1: Get IDs of type WF
IDS=$(mindmap list --type WF --output json | jq -r '.nodes[].id')

# Step 2: Update all (PHASE 1 needed)
mindmap patch $IDS --title "WF: Fixed" --body "Updated body"
```

### Delete Orphans
```bash
# Get all orphans
ORPHANS=$(mindmap orphans --output json | jq -r '.nodes[].id')

# Delete them all (PHASE 1 needed)
mindmap delete $ORPHANS --force
```

### Show Dependency Chain
```bash
# Get node and all nodes it references
mindmap show 15 $(mindmap links 15 --output json | jq -r '.references[].id')
```

### Mark Multiple for Review
```bash
# Verify all TODO items
TODOS=$(mindmap list --type TODO --output json | jq -r '.nodes[].id')
mindmap verify $TODOS  # (PHASE 1 needed)
```

---

## Phase 2: Stdin-Based Selection (Future, v0.2)

### Pipe Selection to Operation
```bash
# Show all WF nodes that reference node 50
mindmap refs 50 --output json | \
  jq 'select(.type == "WF") | .id' | \
  mindmap show --from-stdin

# Patch all matching nodes
mindmap list --type AE --grep "auth" --output json | \
  jq -r '.nodes[].id' | \
  mindmap patch --from-stdin --title "AE: Updated Auth"

# Verify all critical items
mindmap search "critical" --output json | \
  jq -r '.nodes[].id' | \
  mindmap verify --from-stdin
```

---

## Error Handling

### Partial Failures
```bash
# If one ID doesn't exist, operation fails with error
$ mindmap patch 12 15 99 --title "New"
Error: Node 99 not found
# Entire operation aborts; file unchanged
```

### Too Many IDs (ARG_MAX Limit)
```bash
# If you have 1000+ IDs, use Phase 2 stdin method instead
# Or use batch mode:
mindmap batch --input - <<EOF
patch 12 --title "New"
patch 15 --title "New"
...
EOF
```

---

## JSON Output for Agents

### Get IDs from list (for piping)
```bash
$ mindmap list --type WF --output json
{
  "nodes": [
    {"id": 10, "type": "WF", "title": "Example", "body": "..."},
    {"id": 15, "type": "WF", "title": "Another", "body": "..."}
  ]
}

# Extract IDs:
jq -r '.nodes[].id'
# Output: 10 15
```

### Feed into variable
```bash
NODES=$(mindmap list --type TODO --output json | jq -r '.nodes[] | @csv')
echo "Found: $NODES"
```

---

## Safety Notes for Agents

1. **Always `--force` for delete**: Default blocks deletes if referenced
   ```bash
   mindmap delete 12 15  # FAILS if either is referenced
   mindmap delete 12 15 --force  # OK, dangling refs become "lint warnings"
   ```

2. **Validate before large operations**:
   ```bash
   IDS=$(mindmap list --type WF --output json | jq -r '.nodes[].id')
   echo "Will patch: $IDS"  # Verify before running
   mindmap patch $IDS --title "New"
   ```

3. **Run lint after mutations**:
   ```bash
   mindmap patch 12 15 19 --title "Updated"
   mindmap lint  # Check for issues
   ```

4. **Use batch mode for heterogeneous updates**:
   ```bash
   # If each node needs different update, use batch:
   mindmap batch --input - <<EOF
   patch 12 --title "First Title"
   patch 15 --title "Different Title"
   patch 19 --title "Yet Another Title"
   EOF
   ```

---

## Planning Code Generation

### Agent generates multi-ID command
```python
ids = [12, 15, 19]
cmd = f"mindmap delete {' '.join(map(str, ids))} --force"
print(cmd)
# Output: mindmap delete 12 15 19 --force
```

### Agent generates batch mode (for heterogeneous updates)
```python
updates = [
    (12, "First Title"),
    (15, "Second Title"),
    (19, "Third Title"),
]

batch_ops = "\n".join(
    f'patch {id} --title "{title}"'
    for id, title in updates
)
cmd = f"mindmap batch --input - <<EOF\n{batch_ops}\nEOF"
print(cmd)
```

---

## Roadmap Summary

| Phase | Version | Feature | ETA |
|-------|---------|---------|-----|
| Current | v0 | Single-node operations | ✅ Done |
| 1 | v0.1 | Multi-ID arguments | 2-4 hours |
| 2 | v0.2 | Stdin-based selection | 2-3 hours |
| 3 | v1.0 | Selection files + advanced filters | TBD |

