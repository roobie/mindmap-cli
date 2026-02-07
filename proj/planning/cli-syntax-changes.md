# Phase 1: CLI Syntax Changes (Before/After)

## Read-Only Commands (No Changes)
These remain single-node only; use with piping if needed:

```bash
# Show - no change needed (can already compose)
mindmap show 12
mindmap links 12
mindmap refs 12
mindmap list --type WF
mindmap search "critical"
```

---

## Mutation Commands (MULTI-ID SUPPORT ADDED)

### 1. Show Command

**Before (v0):**
```bash
mindmap show 12
# Output:
# [12] **AE: AuthService** - Handles auth [15][20]
```

**After (v0.1):**
```bash
# Single ID (backward compatible)
mindmap show 12
# Output:
# [12] **AE: AuthService** - Handles auth [15][20]

# Multiple IDs (new capability)
mindmap show 12 15 20
# Output:
# [12] **AE: AuthService** - Handles auth [15][20]
# [15] **WF: Login** - Login workflow [12]
# [20] **WF: Logout** - Logout workflow [12]

# Via command substitution
mindmap show $(mindmap links 12 --output json | jq -r '.references[].id')
# Shows all nodes that 12 references
```

---

### 2. Delete Command

**Before (v0):**
```bash
mindmap delete 12 --force
# Deleted node [12]
```

**After (v0.1):**
```bash
# Single ID (backward compatible)
mindmap delete 12 --force
# Deleted 1 node(s)

# Multiple IDs (new capability)
mindmap delete 12 15 20 --force
# Deleted 3 node(s)

# Via orphans
ORPHANS=$(mindmap orphans --output json | jq -r '.nodes[].id')
mindmap delete $ORPHANS --force
# Deleted all orphaned nodes atomically
```

---

### 3. Patch Command

**Before (v0):**
```bash
mindmap patch 12 --title "New Title" --body "New body"
# Patched node [12]
```

**After (v0.1):**
```bash
# Single ID (backward compatible)
mindmap patch 12 --title "New Title"
# Patched 1 node(s)

# Multiple IDs (new capability) - same patch to all
mindmap patch 12 15 20 --title "Updated Title"
# Patched 3 node(s)

# Patch all of a type
IDS=$(mindmap list --type TODO --output json | jq -r '.nodes[].id')
mindmap patch $IDS --title "TODO: Fixed"
# Patched 5 node(s)
```

**Note:** All nodes get the SAME patch. For different patches, use batch mode:
```bash
mindmap batch --input - <<EOF
patch 12 --title "Title for 12"
patch 15 --title "Title for 15"
patch 20 --title "Title for 20"
EOF
```

---

### 4. Verify Command

**Before (v0):**
```bash
mindmap verify 12
# Verified node [12]
```

**After (v0.1):**
```bash
# Single ID (backward compatible)
mindmap verify 12
# Verified 1 node(s)

# Multiple IDs (new capability)
mindmap verify 12 15 20 25
# Verified 4 node(s)

# Verify all critical items
CRITICAL=$(mindmap search "critical" --output json | jq -r '.nodes[].id')
mindmap verify $CRITICAL
# Verified X node(s)
```

---

### 5. Deprecate Command

**Before (v0):**
```bash
mindmap deprecate 12 --to 99
# Deprecated node [12] -> [99]
```

**After (v0.1):**
```bash
# Single ID (backward compatible)
mindmap deprecate 12 --to 99
# Deprecated 1 node(s) -> 99

# Multiple IDs (new capability) - all deprecated to same target
mindmap deprecate 12 15 20 --to 99
# Deprecated 3 node(s) -> 99

# All nodes change to:
# [12] **[DEPRECATED -> 99] AE: AuthService** - ...
# [15] **[DEPRECATED -> 99] WF: Login** - ...
# [20] **[DEPRECATED -> 99] WF: Logout** - ...

# Deprecate all old versions
OLD=$(mindmap list --type AE --grep "v1" --output json | jq -r '.nodes[].id')
mindmap deprecate $OLD --to 50
# All old versions redirected to new v2 node
```

---

## Commands NOT Changing (Single-Node Only)

These commands remain single-node focused and don't need multi-ID:

```bash
mindmap add --type AE --title "New" --body "Description"
# Add one at a time; batch mode for multiple

mindmap edit 12
# Edit one node at a time with $EDITOR

mindmap put 12 --line "[12] **AE: New** - body"
# Replace one node at a time
```

---

## Error Handling

### Empty ID List
```bash
$ mindmap show
Error: expected at least one ID
```

### Missing Node
```bash
$ mindmap show 12 99 15
Error: Node 99 not found
# Entire operation fails; mindmap unchanged
```

### Delete with References (without --force)
```bash
$ mindmap delete 12 15
Error: Cannot delete node 12; referenced by [10][11]
# Add --force to override
```

### Atomicity Guaranteed
```bash
# If any ID fails, entire operation aborts
$ mindmap patch 12 99 15 --title "New"
Error: Node 99 not found
# File unchanged; 12 and 15 NOT patched
```

---

## JSON Output (--output json)

### Show
```bash
$ mindmap show 12 15 --output json
[
  {"id": 12, "type": "AE", "title": "AuthService", ...},
  {"id": 15, "type": "WF", "title": "Login", ...}
]
```

### Delete
```bash
$ mindmap delete 12 15 --force --output json
{"deleted": 2, "ids": [12, 15]}
```

### Patch
```bash
$ mindmap patch 12 15 --title "Updated" --output json
{"patched": 2, "ids": [12, 15], "fields": {"title": "Updated"}}
```

### Verify
```bash
$ mindmap verify 12 15 --output json
{"verified": 2, "ids": [12, 15]}
```

### Deprecate
```bash
$ mindmap deprecate 12 15 --to 99 --output json
{"deprecated": 2, "ids": [12, 15], "target": 99}
```

---

## Help Text (Updated)

```
USAGE:
    mindmap show [IDS]...

Show node(s) by ID

ARGS:
    <IDS>...    Node IDs to show (one or more)

EXAMPLES:
    mindmap show 10
    mindmap show 10 15 20
    mindmap show $(mindmap links 50 --output json | jq -r '.references[].id')

```

---

## Performance Notes

### Single ID (v0 behavior preserved)
```bash
mindmap show 12
# ~1-2ms: parse file, find node, format output
```

### Multiple IDs (linear scaling)
```bash
mindmap show 12 15 20 25 30
# ~2-3ms: parse file once, find 5 nodes, format output
# Roughly same as v0 due to single file parse
```

### Large File Scaling
```bash
# 100-node mindmap:
mindmap show 12 15 20              # ~3ms
mindmap show $(seq 1 100)          # ~4ms (parse all, output all)

# No performance regression expected
```

---

## Backward Compatibility Matrix

| Command | v0 | v0.1 | Compat |
|---------|-------|-------|--------|
| `show 12` | Works | Works | ✅ |
| `show 12 15` | Error | Works | - |
| `delete 12` | Works | Works | ✅ |
| `delete 12 15` | Error | Works | - |
| `patch 12 --title X` | Works | Works | ✅ |
| `patch 12 15 --title X` | Error | Works | - |
| `verify 12` | Works | Works | ✅ |
| `verify 12 15` | Error | Works | - |
| `deprecate 12 --to 99` | Works | Works | ✅ |
| `deprecate 12 15 --to 99` | Error | Works | - |
| `batch` mode | Works | Works | ✅ |
| `list` | Works | Works | ✅ |
| `search` | Works | Works | ✅ |
| `refs` | Works | Works | ✅ |
| `links` | Works | Works | ✅ |

**All v0 scripts continue to work unchanged!**

