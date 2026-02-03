Actionable checklist (v0) for implementing the mindmap CLI

Progress (updated 2026-02-03):
- Phase 0 (project bootstrap) completed: project layout created and initial Cargo.toml/src/main.rs added.
- Rust skeleton implemented and build verified (commands: show/list/search/refs/links/add/deprecate/verify/lint).
- Refactor into src/lib.rs and unit tests added; integration tests added; atomic save and edit command implemented; lint & validation implemented (syntax, duplicate IDs, missing refs, orphan detection) (updated 2026-02-03).

Assumptions
- Default mindmap path: ./MINDMAP.md (override with --file).
- One-node-per-line format: /^\[(\d+)\]\s+\*\*(.+?)\*\*\s*-\s*(.*)$/
- Target language: Rust (Cargo project).

Phase 0 — Project bootstrap
1. Create project layout
   - Files: Cargo.toml, src/main.rs, README.md, tests/
   - Recommended deps (Cargo.toml):
     - clap = { version = "4", features = ["derive"] }
     - regex = "1"
     - anyhow = "1"
     - chrono = "0.4" (for verify timestamps)
     - tempfile = "3" (for edit temp files)
     - fs2 = "0.4" (optional file locking)
     - (tests) assert_cmd, predicates
2. Add CI skeleton (GitHub Actions) to run cargo fmt, clippy, build, test.

Phase 1 — Data model & parser
3. Implement Node and Mindmap structs
   - Node { id: u32, raw_title: String, description: String, references: Vec<u32>, line_index: usize }
   - Mindmap { path: PathBuf, lines: Vec<String>, nodes: Vec<Node>, by_id: HashMap<u32, usize> }
4. Implement loader Mindmap::load(path: PathBuf) -> Result<Mindmap>
   - Read file into lines Vec<String>.
   - Regex parse each line for node entries; preserve non-node lines in lines vector.
   - Parse references from description using ref regex (\[(\d+)\]).
   - Build by_id map and ensure duplicate-id warning.
   - Acceptance: load a sample file (MINDMAP.md) and assert nodes parsed & lines preserved.
5. Implement save() with atomic write
   - Write to temp file in same dir, fs::rename to replace original, optionally create .bak before rename.
   - Acceptance: save round-trip keeps non-node lines intact; write permissions preserved.

Phase 2 — Basic read/inspect commands
6. Implement CLI skeleton using clap with subcommands (see DESIGN)
   - Global --file argument defaulting to MINDMAP.md.
7. Implement cmd_show(id)
   - Print node line and inbound refs.
   - Acceptance: cargo run -- show 42 prints node and "Referred to by" list.
8. Implement cmd_list --type / --grep
   - Type parsing: parse type as text before ':' in raw_title (case-insensitive).
   - Acceptance: list with type AE returns only AE nodes; grep filters title/description.

Phase 3 — Navigation & query
9. Implement cmd_refs(id)
   - Return nodes that reference id.
   - Acceptance: cargo run -- refs 42 shows expected nodes.
10. Implement cmd_links(id)
    - Show outgoing references for node id.
11. Implement cmd_search(query)
    - Case-insensitive substring match across title and description.
12. (Optional) Graph neighbor: cmd_graph(id)
    - Output 1-hop neighborhood as either textual list or DOT format for Graphviz.
    - Acceptance: dot output can be piped to dot -Tpng.

Phase 4 — Edit & maintain
13. Implement next_id() logic
    - next_id = max(existing ids) + 1
    - Acceptance: adding a new node after deleting others picks max+1.
14. Implement cmd_add --type --title --desc
    - Compose full_title = "{type}: {title}"
    - Append formatted line to mm.lines, create Node, update nodes+by_id.
    - Acceptance: after add, saved file contains single appended node line and by_id contains new id.
15. Implement cmd_edit id (opens $EDITOR)
    - Workflow:
      a. Create a temp file containing the single node line (not whole document).
      b. Spawn $EDITOR (env var fallback to vi/nano) to edit.
      c. After edit: read temp file; validate exactly one line matches node regex, and id is unchanged.
      d. If valid, replace mm.lines[node.line_index] with new line and save atomically.
      e. If invalid, abort with clear error and do not change original file. Provide option to force/preview.
    - Acceptance: editing title/description succeeds; changing ID is rejected.
16. Implement cmd_deprecate id --to X
    - Prepend title with "[DEPRECATED → X] " unless already deprecated.
    - Update mm.lines[node.line_index] and save.
    - Acceptance: repeated deprecate is idempotent.
17. Implement cmd_verify id
    - Append "(verify YYYY-MM-DD)" to description if not present (use chrono for date).
    - Update mm.lines and save.

18. Implement put & patch (HTTP semantics)
    - put <id> --line "[id] **TYPE: Title** - desc [N]"  # full-line replace, id must match
    - patch <id> [--type TYPE] [--title TITLE] [--desc DESC] [--strict]
      - partial update: only provided fields are changed
    - Validation: id cannot change, single-line format enforced, references parsed and updated
    - Strict mode: if enabled, fail when references point to missing IDs
    - Acceptance: unit & integration tests verify put and patch behaviors.

Phase 5 — Lint / Safety (implemented)
18. Implement cmd_lint (done)
    - Checks implemented:
      - Lines that start with '[' but do not match node regex (syntax errors).
      - Duplicate IDs.
      - References to non-existent IDs (warn).
      - Orphans (no in & no out) — skips nodes of type META/*.
      - Acceptance: `cargo run -- lint` returns warnings; unit & integration tests cover cases.
19. Add validation helpers (done)
    - parse_node_line(line: &str, line_index: usize) -> Result<Node> implemented and used in tests and lint.
    - Edits validated to preserve one-node-per-line invariant and prevent ID changes.

Phase 6 — Robustness & UX improvements
20. Atomic and safe writes
    - Use tempfile + rename; keep backup copy .bak or .orig-TIMESTAMP.
21. File locking (optional for concurrency)
    - Use fs2 crate to obtain exclusive lock while modifying file.
22. Logging and clear exit codes
    - Return non-zero on errors; print helpful messages on stderr.
23. Improve type filtering
    - Accept both "AE" and "AE:" as filter, case-insensitive.
24. CLI help & examples
    - Document examples in help & README.
25. Config
    - Allow config file (~/.config/mindmap/config or env var) for default path and type prefixes.

Phase 7 — Tests & CI
26. Unit tests
    - Parser tests for valid/invalid lines, reference extraction, duplicate IDs.
    - Mindmap::load/save round-trip tests using tempdir.
27. Integration tests
    - Use assert_cmd to run binary against temp files for add, edit (simulate editor by writing to temp file), deprecate, verify, lint.
28. CI
    - Run cargo fmt, clippy, build, test on PRs. (CI workflow added: .github/workflows/ci.yml)

Phase 8 — Documentation & housekeeping
29. Update repository docs
    - README.md with install & usage examples.
    - Update @MINDMAP.md nodes: add node(s) describing the CLI tool and implementation state per MINDMAP update protocol.
30. Release & packaging
    - Prepare cargo package metadata; optionally provide a small release checklist (version, changelog).

Extras / Future improvements (post-v0)
- graph subcommand to emit DOT + simple html viewer
- interactive TUI for browsing nodes
- richer lint rules (broken backlinks suggestions)
- support multiple mindmap files (namespaces), merging, cross-file refs
- richer migration tools (reassign IDs, split/merge nodes)

Suggested verification commands during development
- Build: cargo build
- Run command: cargo run -- show 42 --file ./MINDMAP.md
- Tests: cargo test
- Lint/format: cargo fmt && cargo clippy

Minimal acceptance criteria for v0
- Parser correctly loads nodes and preserves non-node lines
- show/list/search/refs/links work and produce expected output
- add appends new node with stable ID
- edit opens $EDITOR, validates, and safely writes a single-line update
- deprecate/verify update the node title/description and are idempotent-safe
- lint reports missing references and duplicate IDs
- save uses atomic write and preserves file when validation fails

If you want, I can:
- generate a prioritized task list with estimated hours per task, or
- open the repository and start implementing Phase 1–3 (create files, add tests, implement parser and show/list). Which next step should I take?
