Overview

This document describes the implemented v0 design for mindmap-cli (a small Rust CLI for maintaining one-line MINDMAP files). It records the data model, command semantics, safety invariants, and rationale for important choices. The DESIGN.md is kept in sync with the code and the MINDMAP (see DR nodes in MINDMAP.md for formalized decisions).

Goals for v0
- Provide reliable, scriptable read & query operations (show, list, refs, links, search)
- Provide safe edit operations (edit via $EDITOR, scripted patch/put) with strong validation
- Ensure atomic file writes and minimal blast radius for edits
- Ship with unit + integration tests and CI


1. Core capabilities (implemented)

Read / Inspect
- show node by id: mindmap show <id>
- list nodes: mindmap list [--type TYPE] [--grep PATTERN]

Query / Navigate
- refs <id>: list nodes that reference id
- links <id>: list outgoing references from id
- search <query>: case-insensitive substring match over title and body

Edit / Maintain (implemented)
- add: mindmap add --type <TYPE> --title <TITLE> --body <DESC>
  - Appends a properly formatted line and registers a new node id (next_id = max existing + 1).
- edit: mindmap edit <id>
  - Opens a temp file in $EDITOR containing a single node line; edited file must produce exactly one valid node line and preserve the bracketed id.
- patch: mindmap patch <id> [--type TYPE] [--title TITLE] [--body DESC] [--strict]
  - Partial update; unspecified fields are preserved. When a type prefix is present, patch splits raw_title on first ':' to determine prefix vs title.
- put: mindmap put <id> --line "[id] **TYPE: Title** - body" [--strict]
  - Full-line, idempotent replacement. Provided line is parsed and must contain the same id.
- deprecate: mindmap deprecate <id> --to <id>
  - Adds a "[DEPRECATED → X]" prefix to the title if not already present.
- verify: mindmap verify <id>
  - Appends a (verify YYYY-MM-DD) tag to the body (idempotent).

Lint / Safety (implemented)
- mindmap lint reports:
  - Lines that look like nodes but do not match the node regex (syntax errors)
  - Duplicate IDs and their line numbers
  - Missing references

Orphans (separate command)
- Orphan detection is available as a dedicated command:
  - `mindmap orphans` — lists nodes that have no incoming and no outgoing references (excluding nodes whose title starts with "META").
  - Rationale: keep `lint` focused on format and reference validity; make orphan listing explicit so it can be used selectively.


2. Data model & invariants (DRs)

Core invariants (recorded in MINDMAP.md as DR: nodes):
- Default filename: MINDMAP.md is the CLI default; override with --file
- Node format: ^\[(\d+)\] \*\*(.+?)\*\* - (.*)$ (one-node-per-line)
- ID immutability: numeric IDs are stable and cannot be changed by edits/put/patch
- Atomic writes: saves are atomic via tempfile in the same dir + persist/rename
- Editor flow: edit supplies a single-line temp file; edited result must match node regex exactly
- PUT/PATCH semantics: PUT is full-line replace (id must match); PATCH is partial update; both update parsed references; --strict fails on missing refs
- Reference parsing: references are parsed with \[(\d+)\] and self-references are ignored when building references vector
- Orphan exception: nodes starting with META are excluded from orphan warnings


3. CLI shape (current)

Global:
- --file <path> (optional, default ./MINDMAP.md)

Subcommands (implemented):
- show <id>
- list [--type TYPE] [--grep PATTERN]
- refs <id>
- links <id>
- search <query>
- add --type <TYPE> --title <TITLE> --body <DESC>
- edit <id>  (uses $EDITOR)
- patch <id> [--type] [--title] [--body] [--strict]
- put <id> --line "..." [--strict]
- deprecate <id> --to <id>
- verify <id>
- orphans
- lint

Help and examples are embedded in the CLI (long_about / after_help) so `mindmap --help` includes concise examples and notes about $EDITOR and default file.


4. Safety notes and edge-cases

- Editor behavior:
  - The edit command creates a NamedTempFile containing a single node line. The editor is invoked with that path. After the editor exits, the first non-empty line is parsed and must match the node regex and preserve id; otherwise the edit is aborted and the original file remains unchanged.
  - Tests simulate editors by making small executable scripts that write the desired content to the temp file. This is Unix-friendly; CI runs on ubuntu-latest.

- Atomic save:
  - Save writes to a tempfile in the same directory then persists/renames to the target path. This avoids partial files left by crashes.
  - Backups and file-locking are considered future improvements.

- PUT vs PATCH:
  - PUT is intended to be deterministic and idempotent (replace whole node line). Use PUT for migrations or exact replacements.
  - PATCH is intended for small scripted changes; it composes the new raw_title from existing components unless overridden.

- Strict mode:
  - Both put and patch accept --strict; when set, the operation fails if any referenced id does not exist. Default behavior is permissive (allow and warn).

- References and self-links:
  - Parser collects numeric IDs from the body via \[(\d+)\]. Self-references (node referring to itself) are ignored when populating the node.references vector.


5. Tests & CI

- Unit tests cover parsing, save atomicity, lint behavior, and put/patch helpers.
- Integration tests (tests/cli.rs) exercise add, edit (editor simulation), edit-change-id-fails, and patch/put flows.
- CI workflow (.github/workflows/ci.yml) runs fmt check, clippy, cargo build and cargo test on ubuntu-latest.


6. Next steps (short-term)

- Address outstanding lint warnings recorded in MINDMAP.md (duplicate IDs, syntax lines) — housekeeping task.
- Add README install instructions and example workflows (done: short README updated).
- Add optional file-locking and backups (fs2, or simple .bak policy) if needed for multi-user workflows.
- Expand tests to cover strict mode failures and additional negative cases (multi-line edit attempts, malformed put lines).
- Consider adding a 'doc' or 'dr' command to print Decision Records extracted from MINDMAP.md.


7. Appendix: command examples

```bash
mindmap show 10
mindmap list --type AE --grep auth
mindmap add --type AE --title "AuthService" --body "Handles auth [12]"
mindmap edit 12
mindmap patch 12 --title "AuthSvc" --body "Updated body"
mindmap put 12 --line "[12] **AE: AuthSvc** - Updated body [10]"
mindmap lint
```

This DESIGN.md should be kept in sync with the code and the DR nodes in MINDMAP.md. If you want I can also append the DR entries to DESIGN.md or add cross references to test locations.