MINDMAP Interaction Protocol (mindmap-cli)

Prime directive:
Any and all interactions with MINDMAP files **MUST** be through `mindmap-cli`. It is **strictly forbidden** to update MINDMAP files in other ways.

Purpose
- Ensure every read or write operation against the repository MINDMAP files (default ./MINDMAP.md) is atomic, validated, and preserves the "one-node-per-line" format and numeric node-ID invariants.
- Prevent accidental or unsafe manual edits and maintain strong cross-reference integrity.

Scope
- Applies to all contributors, automation, and CI jobs that create, modify, or delete nodes in MINDMAP.md.
- Uses the tool: `mindmap-cli` as the canonical interface.

Assumptions
- `mindmap-cli` is available on PATH and configured to operate on ./MINDMAP.md by default.
- The EDITOR environment variable controls the interactive editor used by `mindmap-cli edit`.

Summary workflow (mandatory)
1) Inspect
   - Run: `mindmap-cli lint` to surface basic format issues.
   - Show node(s): `mindmap-cli show <id>` or `mindmap-cli list --type <TYPE> --grep "<term>"` or `mindmap-cli search <term>`.
   - Find references: `mindmap-cli refs <id>` and `mindmap-cli links <id>` before modifying or deleting a node.
   - Read from stdin (read-only): You can supply `--file -` to read a mindmap from stdin for read-only operations (list, show, refs, links, search, lint, orphans). Example: `cat MINDMAP.md | mindmap-cli --file - lint`. Mutating commands (add/put/patch/edit/delete/deprecate/verify) will error when the source is `-`—use a real path with `--file <path>` or operate on the default `./MINDMAP.md` to persist changes.

2) Plan
   - Decide whether to `add`, `patch`, `put`, `deprecate`, or `delete`.
   - If removing a node with incoming refs, update/redirect those refs first.
   - For single operations, use the corresponding command directly.
   - For multiple operations, consider using `batch` mode:
     ```bash
     mindmap-cli batch --input - --format lines <<EOF
     add --type 'AE' --title 'Entry Points' --desc '...'
     patch 12 --title 'WF: Project overview' --desc '...'
     delete 13
     deprecate 14 --to 31
     EOF
     ```
     Use `--dry-run` to preview changes before committing. Batch mode is atomic: all-or-nothing.
   - If several nodes need to be added, updated and/or deleted in a batch; then produce a `.sh` file that contains the changeset, e.g.
```bash
mindmap-cli patch 12 --title 'WF: Project overview & purpose' --desc '...'

mindmap-cli delete 13

mindmap-cli add --type 'AE' --title 'Entry Points' --desc '...'

mindmap-cli put 15 --line "[15] **DR: Why safety over speed** - explanation here"
```
    This ephemeral script can subsequently be executed by the agent or the user.

3) Make the change (non-interactive preferred)
   - Add a node: `mindmap-cli add --type WF --title "Title" --desc "Description [SOME_NODE_ID] or [link](./file.md)"`
   - Patch a node (partial): `mindmap-cli patch 31 --title "New title" --desc "Updated desc"`
   - Put a node (full replace - ID must match): `mindmap-cli put 31 --line "[31] **WF: Example** - Full line text [12]"`
   - Deprecate a node: `mindmap-cli deprecate 12 --redirect 31`
   - Delete a node (after refs removed): `mindmap-cli delete 12`

4) Validate & Sanity-check
   - Run: `mindmap-cli lint` to surface any issues; optionally run `mindmap-cli lint --fix` to auto-correct spacing and duplicated type prefixes.
   - Re-check refs and show changed node(s): `mindmap-cli refs <id>`; `mindmap-cli show <id>`.

5) Commit
   - `git add MINDMAP.md` (or other affected files)
   - `git commit -m "mindmap: <short summary> (nodes: <ids>)\n\n<longer description if needed>"`
   - Open a PR if appropriate and reference the nodes/changes in the description.

Editor note
- Use `mindmap-cli edit <id>` when manual intervention is needed; this opens $EDITOR for an atomic, validated update.

Automation / CI recommendations
- Use non-interactive `mindmap-cli` commands (add/patch/put/deprecate/delete) with `--output json` to assert effects programmatically. Pairs well with tools like `jq`
- Include `mindmap-cli lint` as part of any script or CI job that modifies MINDMAP.md.

Exceptions & fallback
- If `mindmap-cli` cannot express a legitimate necessary change (rare), capture the failing command and error output and request explicit approval before making any direct edits to MINDMAP.md. Direct edits are allowed only with explicit approval and must be followed by lint/refs checks.
- Orphaned items are those that neither are referenced or refers to other nodes. Having any number of orphans is **not** exceptional. Determine which nodes are orphans by `mindmap-cli orphans`.

Batch mode (atomic multi-operation edits)
- Use `mindmap-cli batch` when applying multiple non-interactive operations atomically:
  - Supported operations: `add`, `patch`, `put`, `delete`, `deprecate`, `verify`.
  - Input format options:
    - `--format lines` (default): each line is a CLI-style invocation (e.g., `add --type WF --title X --desc Y`). Use double-quotes for multi-word arguments.
    - `--format json`: each operation is a JSON object in an array (e.g., `[{"op": "add", "type": "WF", "title": "X", "desc": "Y"}, ...]`).
  - Flags:
    - `--dry-run`: preview changes without writing to file.
    - `--fix`: auto-fix spacing and duplicated type prefixes before committing.
  - Concurrency safety: batch mode computes a blake3 hash of the target file at start and verifies the hash again before writing. If the file was modified by another process between start and commit, the batch aborts with an error and no changes are written. This prevents race conditions.
  - Atomicity: if any operation fails during parsing or execution, the entire batch is rejected and no changes are persisted.
  - Example:
    ```bash
    mindmap-cli batch --input batch-ops.txt --fix
    # or from stdin
    cat ops.txt | mindmap-cli batch --input - --dry-run
    ```

Revision history
- v3.1 - adds batch mode documentation and concurrency safety guarantees
- v3.0 - adds wording about batch updating
- v2.0 - wording updated to be clearer.
- v1.0 — created and adopted (automated by assistant) — use `mindmap-cli` for all future edits.
