# Multi-Node Selection: Planning Summary

## The Question
> How can we support selecting multiple specific nodes? What would be the best UX for that (agents)?

## The Answer (Recommended)

### Phase 1: Multi-ID Arguments ⭐ RECOMMENDED
```bash
mindmap show 12 15 19                          # Show multiple
mindmap delete 12 15 19 --force                # Delete multiple  
mindmap patch 12 15 19 --title "Updated"      # Patch all same way
mindmap verify 5 8 12 15                       # Verify multiple
mindmap deprecate 10 11 12 --to 50             # Deprecate to same target
```

**Why?** 
- ✅ Simple syntax, natural for shells
- ✅ Works with $(command) substitution 
- ✅ Fast to implement (3-4 hours)
- ✅ Perfect for agent integration
- ✅ Backward compatible

**Limitation:** Shell ARG_MAX limits ~100K chars (roughly 1000+ numeric IDs)

---

### Phase 2: Stdin-Based Selection (Future, optional)
```bash
# Pipe selections through stdin
mindmap list --type WF --output json | \
  jq -r '.nodes[].id' | \
  mindmap patch --from-stdin --title "Updated"
```

**Why?** Unlimited scale, Unix-native, elegant compositions

---

### Phase 3: Selection Files (Future, advanced)
```bash
mindmap list --type WF --select-output wf-nodes.json
mindmap batch --select-file wf-nodes.json --op patch --title "Updated"
```

**Why?** Persistent selections, complex workflows

---

## What Problem Does This Solve?

**Before:** Agent had to generate batch mode syntax
```bash
mindmap batch --input - <<EOF
delete 12 --force
delete 15 --force
delete 19 --force
EOF
```

**After (Phase 1):** Agent simply generates multi-ID commands
```bash
mindmap delete 12 15 19 --force
```

Much simpler! Works naturally with pipes:
```bash
ORPHANS=$(mindmap orphans --output json | jq -r '.nodes[].id')
mindmap delete $ORPHANS --force
```

---

## How It Works (Architecture)

### Current (v0)
```
CLI Input: show 12
    down arrow
Command::Show { id: u32 }
    down arrow
cmd_show(&mm, id) returns String
```

### Phase 1
```
CLI Input: show 12 15 19
    down arrow
Command::Show { ids: Vec<u32> }  # Changed from id to ids
    down arrow
cmd_multi_show(&mm, ids) returns Vec<String>
    down arrow
for each id in ids:
    returns cmd_show(&mm, id)  # Reuse existing
```

**Key insight:** Wrap single-node functions, maintain backward compatibility!

---

## Implementation Checklist

### Code Changes
- [ ] Update Commands enum (Show/Delete/Patch/Verify/Deprecate)
- [ ] Add cmd_multi_show(), cmd_multi_delete(), etc.
- [ ] Update run() function to handle Vec<u32>
- [ ] Update CLI help text

### Tests
- [ ] Unit: multi-show with various ID counts
- [ ] Unit: multi-delete with force flag
- [ ] Unit: multi-patch with type/title/body
- [ ] Unit: error cases (missing IDs, etc.)
- [ ] Integration: full CLI workflows
- [ ] Shell: test with $(command) substitution

### Docs
- [ ] Update MINDMAP.md (add node for v0.1 feature)
- [ ] Update DESIGN.md (add multi-ID examples)
- [ ] Update README (show multi-ID patterns)
- [ ] Update PROTOCOL_MINDMAP.md (agent examples)

### Validation
- [ ] All existing tests pass
- [ ] Lint clean
- [ ] No performance regression
- [ ] Atomic guarantees maintained

---

## Safety Guarantees

✅ **Atomicity:** All IDs updated or none (single file write)
✅ **Validation:** All IDs checked before any mutation
✅ **ID immutability:** Node IDs never change
✅ **Backward compatible:** Old scripts still work

---

## Common Agent Patterns (Phase 1)

### Delete Orphans
```bash
mindmap delete $(mindmap orphans --output json | jq -r '.nodes[].id') --force
```

### Update All of Type
```bash
mindmap patch $(mindmap list --type TODO --output json | jq -r '.nodes[].id') \
  --title "Done"
```

### Show Dependency Chain
```bash
mindmap show 15 \
  $(mindmap links 15 --output json | jq -r '.references[].id')
```

### Mark for Review
```bash
mindmap verify $(mindmap search "critical" --output json | jq -r '.nodes[].id')
```

---

## Files in This Planning

1. **multiple-node-selection.md** — Full design analysis (4 solutions, tradeoffs)
2. **agent-multi-selection-patterns.md** — Practical examples for agents
3. **implementation-multi-id.md** — Detailed code implementation guide
4. **SUMMARY.md** — This file

---

## Roadmap

| Phase | Version | Work | Time | Status |
|-------|---------|------|------|--------|
| 1 | v0.1 | Multi-ID arguments | 3-4h | Planned |
| 2 | v0.2 | Stdin selection | 2-3h | Planned |
| 3 | v1.0 | Selection files | TBD | Future |

---

## Decision: APPROVED

**Recommended approach:** Implement Phase 1 (Multi-ID Arguments)

**Rationale:**
1. Solves 80% of agent use cases
2. Minimal implementation (3-4 hours)
3. Backward compatible
4. Natural UX for Rust CLI
5. Leaves room for Phase 2 without disruption

**Next step:** Open implementation issue and create draft PR

