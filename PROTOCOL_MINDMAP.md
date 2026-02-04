MINDMAP Interaction Protocol (mindmap-cli)

Purpose
- Ensure every read or write operation against the repository MINDMAP files (default ./MINDMAP.md) is atomic, validated, and preserves the "one-node-per-line" format and numeric node-ID invariants.
- Prevent accidental or unsafe manual edits and maintain strong cross-reference integrity.

Scope
- Applies to all contributors, automation, and CI jobs that create, modify, or delete nodes in MINDMAP.md.
- Uses the repo-provided tool: `mindmap-cli` as the canonical interface.

Assumptions
- `mindmap-cli` is available on PATH and configured to operate on ./MINDMAP.md by default.
- The EDITOR environment variable controls the interactive editor used by `mindmap-cli edit`.

Summary workflow (recommended)
1) Inspect
   - Run: `mindmap-cli lint` to surface basic format issues.
   - Show node(s): `mindmap-cli show <id>` or `mindmap-cli list --type <TYPE> --grep "<term>"`.
   - Find references: `mindmap-cli refs <id>` and `mindmap-cli links <id>` before modifying or deleting a node.

2) Plan
   - Decide whether to `add`, `patch`, `put`, `deprecate`, or `delete`.
   - If removing a node with incoming refs, update/redirect those refs first.

3) Make the change (non-interactive preferred)
   - Add a node: `mindmap-cli add --type WF --title "Title" --desc "Description [link](./file.md)"`
   - Patch a node (partial): `mindmap-cli patch 31 --title "New title" --desc "Updated desc"`
   - Put a node (full replace): `mindmap-cli put 31 --line "[31] **WF: Example** - Full line text [12]"`
   - Deprecate a node: `mindmap-cli deprecate 12 --redirect 31`
   - Delete a node (after refs removed): `mindmap-cli delete 12 --force`

4) Validate & Sanity-check
   - Run: `mindmap-cli lint`
   - Re-check refs and show changed node(s): `mindmap-cli refs <id>`; `mindmap-cli show <id>`.

5) Commit
   - `git add MINDMAP.md PROTOCOL_MINDMAP.md` (or other affected files)
   - `git commit -m "mindmap: <short summary> (nodes: <ids>)"`
   - Open a PR if appropriate and reference the nodes/changes in the description.

Editor note
- Use `mindmap-cli edit <id>` when manual intervention is needed; this opens $EDITOR for an atomic, validated update.

Automation / CI recommendations
- Use non-interactive `mindmap-cli` commands (add/patch/put/deprecate/delete) with `--output json` to assert effects programmatically.
- Include `mindmap-cli lint` as part of any CI job that modifies MINDMAP.md.

Exceptions & fallback
- If `mindmap-cli` cannot express a legitimate necessary change (rare), capture the failing command and error output and request explicit approval before making any direct edits to MINDMAP.md. Direct edits are allowed only with explicit approval and must be followed by lint/refs checks.
- Orphaned items are those that neither are referenced or refers to other nodes. Having any number of orphans is **not** exceptional. Determine which nodes are orphans by `mindmap-cli orphans`.

Revision history
- v1.0 — created and adopted (automated by assistant) — use `mindmap-cli` for all future edits.
