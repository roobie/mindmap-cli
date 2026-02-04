# MINDMAP.md - Aisha AI DevOps and SRE Assistant

[0] **META: 🎯 PRIME DIRECTIVE FOR AI AGENTS:** - This MINDMAP documents the Aisha codebase. Read nodes [1-9] for format rules, then [10-14] for project overview. Follow `[N]` references to navigate. **Always update this file as you work.**

[1] **META: Mind Map Format** - This is a graph-based documentation format where each node is exactly one line: `[N] **Title** - content with any number of [N] references`. The format is markdown-compatible, grep-able, and git-friendly [2][3][4]. Nodes enable atomic updates, instant search, and LLM-native citation syntax [5][6].

[2] **META: Node Syntax** - Format is `[N] **Title** - description with [N] references`. Each node is exactly one line. Titles use markdown bold `**...**`. References use citation syntax `[N]` which LLMs recognize from academic papers [1][3].

[3] **META: Node Types** - Nodes are prefixed by type: `**AE: X**` (Architecture Element), `**WF: X**` (Workflow), `**DR: X**` (Decision Record), `**BUG: X**` (Bug Record), `**TODO: X**` (Planned Work), `**DOING: X**` (Work under way), `**DONE: X**` (Work completed), `**META: X**` (Documentation about this mindmap) [1][2][4].

[4] **META: Quick Start for Agents** - First time? (1) Read [1-9] for format, (2) Read [10-14] for project overview, (3) Grep for your task: `grep -i "llm"` then read matching nodes, (4) Follow `[N]` links, (5) Update nodes as you work per protocol [6][7][8].

[5] **META: Why This Format Works** - Line-oriented = atomic updates (replace line N to update node N), instant grep (`grep "^\[42\]"`), diff-friendly (only edited lines change), zero parsing overhead [1][2]. The `[N]` citation syntax leverages LLM training on academic papers [3].

[6] **META: Update Protocol** - **MANDATORY:** (1) Before work, grep for related nodes and read them [4], (2) After changes, update affected nodes immediately, (3) Add new nodes if concept is referenced 3+ times OR non-obvious from code, (4) For bugs create `**BUG:**` node with root cause + solution [3][7].

[7] **META: Node Lifecycle Example** - Initial: `[15] **AE: LLM Client** - Uses OpenAI SDK [20][25]`. After change: `[15] **AE: LLM Client** - Uses OpenAI SDK, supports local models [20][25][27] (updated 2026-02-02)` [6][3].

[8] **META: Reality vs Mindmap** - **Critical:** If MINDMAP contradicts code, code is truth—update MINDMAP immediately [6]. This MINDMAP is an index, not a spec. Stale nodes are worse than missing nodes.

[9] **META: Scaling Strategy** - Current project: <100 nodes. If exceeds 100, split into domain files like `MINDMAP.llm.md`, `MINDMAP.execution.md` [10]. Link from main: `[15] **AE: LLM System** - See MINDMAP.llm.md for details` [1][3].

---

[11] **WF: MINDMAP interactions** - VERY IMPORTANT💯: it is **mandatory** to use the program `mindmap-cli` to interact with MINDMAP files. I.e. querying, reading and updating shall be done by invoking `mindmap-cli`. It is forbidden to update METADATA.md directly. Learn how to use this tool by invoking `mindmap-cli help`. Required reading: [PROTOCOL_MINDMAP.md](./PROTOCOL_MINDMAP.md)

[12] **WF: Development basics** - Make sure to run `mise run fmt` after each edit, so that the rust source code is ensured to be canonically formatted

[13] **META: Development environment** - managed by `mise` (see mise.toml for versions and available tasks); common tasks: `mise test`, `mise run fmt`, `mise run lint`. Platform is Cargo/rust.

[14] **Core priorities** - Security > Correctness > Robustness > Maintainability > Speed > Visuals.

---

[10] **Project purpose** - Provide a robust and useful CLI interface for interacting with MINDMAP files (just like this one.) - See design document at [DESIGN](./DESIGN.md). Make sure to keep both this MINDMAP and the DESIGN document updated as implementation goes along.

[15] **AE: mindmap-cli** - v0 implementation: added CHECKLIST.md; implemented Rust skeleton (parser + CLI); commands: show,list,search,refs,links,add,deprecate,verify,lint,edit,put,patch,delete; added atomic save, edit command, put/patch, delete, --output json (updated 2026-02-03)

[16] **TODO: v0 phases** - Follow CHECKLIST.md: Phase1 parser, Phase2 commands, Phase3 navigation, Phase4 edit, Phase5 lint, tests (integration tests added 2026-02-03); CI workflow added (updated 2026-02-03)

[20] **DOC: CLI help** - Added high-level description and core usage examples to CLI help output (mindmap-cli --help / help) (updated 2026-02-03)

[17] **DONE: CHECKLIST.md** - Created CHECKLIST.md with actionable implementation checklist for v0 (2026-02-03)

[18] **AE: mindmap-cli default** - Default mindmap file changed to MINDMAP.md (removed .core) (updated 2026-02-03)

[19] **DONE: Lint & Validation** - Implemented syntax checks, duplicate ID detection, missing-ref warnings and orphan detection; added unit and integration tests for lint (updated 2026-02-03)

[21] **DR: Default filename = MINDMAP.md** - Default mindmap filename is ./MINDMAP.md; CLI and tests rely on this default; override with --file if needed.

[22] **DR: Node format regex** - Nodes must follow ^\[(\d+)\] \*\*(.+?)\*\* - (.*)$ (one-node-per-line). Parsers, edit, and lint depend on this exact format.

[23] **DR: ID immutability** - Node numeric IDs are immutable; edits/put/patch must preserve the bracketed id. Tests enforce reject on id change.

[24] **DR: Atomic save strategy** - Writes are atomic: write to temp file in same dir and persist/rename to replace original to avoid partial writes.

[25] **DR: Editor single-line validation** - Edit flow provides a single-line temp file; editor must produce exactly one valid node line or the edit is aborted to prevent file corruption.

[26] **DR: PUT and PATCH semantics** - PUT = full-line replace (id must match); PATCH = partial update of type/title/desc; --strict fails on missing refs.

[27] **DR: Output formats & JSON** - CLI supports --output json to emit structured data on stdout; informational messages and warnings go to stderr to keep stdout machine-actionable.

[28] **DR: Delete semantics** - Delete blocks by default when referenced; use --force to delete and leave dangling refs (lint will report). No automatic cleanup by default.

[29] **DONE: Read stdin** - Implemented support for reading MINDMAP content from stdin via `--file -` for read-only commands; mutating operations are disallowed when source is '-' (use `--file <path>` to persist changes) (updated 2026-02-04)

[30] **DONE: Add from $EDITOR** - `add` editor flow implemented: calling `mindmap-cli add` with no args opens $EDITOR to author a single validated node line which is appended to MINDMAP.md (updated 2026-02-04)

[31] **WF: Protocol for interacting with MINDMAP** - See [PROTOCOL_MINDMAP.md](./PROTOCOL_MINDMAP.md) for the formal protocol describing how to interact with MINDMAP.md (add/edit/lint/orphans flows).

[32] **AE: Manual parser for node lines** - Replaced regex captures with manual parser to avoid repeated compilation and allocations; tests updated (refactor 2026-02-04)

[33] **AE: Parser consolidation** - Removed NODE_RE; cmd_edit uses manual parser as well to avoid any regex usage in hot paths (refactor 2026-02-04)

[34] **AE: CLI refactor to lib.rs and unit tests** - Moved Cli/Commands structs and command logic to lib.rs via run() function; simplified main.rs; added 18+ unit tests for cmd_* functions and error cases; test coverage improved to 50% (from ~31%) using tarpaulin

[35] **PERF: Manual parser benchmarks** - Added criterion benchmarks: parse_node_line (~200ns), mindmap_from_string (~8µs for 3 nodes); quantifies performance of regex-free manual parser

[36] **DONE: Graph subcommand** - Implemented 'mindmap graph <id>' to output 1-hop neighborhood in DOT format for Graphviz; can pipe to 'dot -Tpng > graph.png'

[37] **DONE: External reference syntax** - Parser now supports [id](./file.md) markdown links for cross-file references; Reference enum extended with External variant; groundwork for multi-file mindmaps [9]

[38] **WF: Git commit messages** - Require good but terse commit messages: short summary (<=72 chars) and optional body; reference ticket IDs; keep commits atomic.

[39] **WF: Bootstrap command** - Runs 'mindmap-cli help' and 'mindmap-cli list' to bootstrap an AI agent's context; additionally concatenates PROTOCOL_MINDMAP.md (if present) to prime agents to follow the protocol.
