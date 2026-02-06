# Planning Documentation

This folder contains planning and design documents for mindmap-cli feature development.

## Current Planning: Multi-Node Selection

**Question:** How can we support selecting multiple specific nodes? What would be the best UX for agents?

### Documents

1. **[SUMMARY.md](./SUMMARY.md)** ← START HERE
   - Overview of the problem and recommended solution
   - Phase 1-3 roadmap
   - Quick implementation checklist
   - Common agent patterns

2. **[multiple-node-selection.md](./multiple-node-selection.md)**
   - Comprehensive design analysis
   - 4 solution approaches compared
   - Tradeoffs analysis (complexity, safety, UX, composability, scale)
   - Recommended hybrid Phase 1 + Phase 2 + Phase 3 approach
   - Risks and mitigations

3. **[implementation-multi-id.md](./implementation-multi-id.md)** 
   - Detailed code implementation guide for Phase 1
   - Exact changes to Commands enum
   - Function implementations with Rust code
   - Complete test suite (unit + integration)
   - Backward compatibility notes
   - Timeline: ~3-4 hours

4. **[agent-multi-selection-patterns.md](./agent-multi-selection-patterns.md)**
   - Practical usage patterns for AI agents
   - Current (v0) vs Phase 1 vs Phase 2 examples
   - Common workflows: delete orphans, update by type, mark for review
   - Error handling guide
   - JSON output patterns
   - Code generation examples for agents

---

## Quick Summary

### Problem
Currently `mindmap-cli` only operates on single nodes. Agents need to:
- Select multiple nodes (by ID, type, search results)
- Perform bulk operations atomically
- Use natural shell composition (`$(command)`)

### Solution (Recommended)

**Phase 1:** Multi-ID arguments
```bash
mindmap show 12 15 19
mindmap delete 12 15 19 --force
mindmap patch 12 15 19 --title "Updated"
```

**Phase 2:** Stdin-based selection (optional)
```bash
mindmap list --type WF --output json | jq -r '.nodes[].id' | \
  mindmap patch --from-stdin --title "Updated"
```

**Phase 3:** Selection files (future, optional)
```bash
mindmap list --type WF --select-output wf-nodes.json
mindmap batch --select-file wf-nodes.json --op patch
```

### Why Phase 1?
- ✅ Solves 80% of use cases
- ✅ 3-4 hours to implement
- ✅ Backward compatible
- ✅ Natural shell syntax
- ✅ Leverages existing code

---

## How to Use This Planning

### For Implementers
1. Read SUMMARY.md (5 min)
2. Read implementation-multi-id.md (20 min)
3. Follow the checklist and code examples
4. Refer back to multiple-node-selection.md for design rationale

### For Reviewers
1. Read SUMMARY.md
2. Check implementation-multi-id.md for test coverage
3. Review against multiple-node-selection.md tradeoffs

### For Agent Developers
1. Read agent-multi-selection-patterns.md
2. Refer to SUMMARY.md for command syntax
3. Test patterns in implementation-multi-id.md

---

## Related Issues / Documents

- MINDMAP.md: See node [15] for current CLI architecture
- DESIGN.md: Section 1 for command capabilities
- PROTOCOL_MINDMAP.md: Section 2 for mandatory workflow
- tests/cli.rs: Integration test examples
- src/lib.rs: Command parsing and execution

---

## Status

**Decision:** APPROVED ✅

**Next step:** Implement Phase 1 (Multi-ID arguments) following implementation-multi-id.md

