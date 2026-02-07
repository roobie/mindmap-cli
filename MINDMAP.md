# MINDMAP.md - Aisha AI DevOps and SRE Assistant

[0] **META: 🎯 PRIME DIRECTIVE FOR AI AGENTS:** - This MINDMAP documents the Aisha codebase. Read nodes [1-9] for format rules, then [10-14] for project overview. Follow `[N]` references to navigate. **Always update this file as you work.**

[1] **META: Mind Map Format** - This is a graph-based documentation format where each node is exactly one line: `[N] **Title** - content with any number of [N] references`. The format is markdown-compatible, grep-able, and git-friendly [2][3][4]. Nodes enable atomic updates, instant search, and LLM-native citation syntax [5][6].

[2] **META: Node Syntax** - Format is `[N] **Title** - body with [N] references`. Each node is exactly one line. Titles use markdown bold `**...**`. References use citation syntax `[N]` which LLMs recognize from academic papers [1][3].

[3] **META: Node Types** - Nodes are prefixed by type: `**AE: X**` (Architecture Element), `**WF: X**` (Workflow), `**DR: X**` (Decision Record), `**BUG: X**` (Bug Record), `**TODO: X**` (Planned Work), `**DOING: X**` (Work under way), `**DONE: X**` (Work completed), `**META: X**` (Documentation about this mindmap) [1][2][4].

[4] **META: Quick Start for Agents** - First time? (1) Read [1-9] for format, (2) Read [10-14] for project overview, (3) Grep for your task: `grep -i "llm"` then read matching nodes, (4) Follow `[N]` links, (5) Update nodes as you work per protocol [6][7][8].

[5] **META: Why This Format Works** - Line-oriented = atomic updates (replace line N to update node N), instant grep (`grep "^\[42\]"`), diff-friendly (only edited lines change), zero parsing overhead [1][2]. The `[N]` citation syntax leverages LLM training on academic papers [3].

[6] **META: Update Protocol** - **MANDATORY:** (1) Before work, grep for related nodes and read them [4], (2) After changes, update affected nodes immediately, (3) Add new nodes if concept is referenced 3+ times OR non-obvious from code, (4) For bugs create `**BUG:**` node with root cause + solution [3][7].

[7] **META: Node Lifecycle Example** - use `mindmap-cli add/put/patch/delete` to interact with the MINDMAP [6][3].

[8] **META: Reality vs Mindmap** - **Critical:** If MINDMAP contradicts code, code is truth—update MINDMAP immediately [6]. This MINDMAP is an index, not a spec. Stale nodes are worse than missing nodes.

[9] **META: Scaling Strategy** - Current project: <100 nodes. If exceeds 100, split into domain files like `MINDMAP.llm.md`, `MINDMAP.execution.md` [10]. Link from main: `[15] **AE: LLM System** - See MINDMAP.llm.md for details` [1][3].

---

[11] **WF: MINDMAP interactions** - VERY IMPORTANT💯: it is **mandatory** to use the program `mindmap-cli` to interact with MINDMAP files. I.e. querying, reading and updating shall be done by invoking `mindmap-cli`. It is forbidden to update METADATA.md directly. Learn how to use this tool by invoking `mindmap-cli help`. Required reading: [PROTOCOL_MINDMAP.md](./PROTOCOL_MINDMAP.md)

[12] **WF: Development basics** - Make sure to run `mise run fmt` after each edit, so that the rust source code is ensured to be canonically formatted

[13] **META: Development environment** - managed by `mise` (see mise.toml for versions and available tasks); common tasks: `mise test`, `mise run fmt`, `mise run lint`. Platform is Cargo/rust.

[14] **Core priorities** - Security > Correctness > Robustness > Maintainability > Speed > Visuals.

---

[10] **Project purpose** - Provide a robust and useful CLI interface for interacting with MINDMAP files (just like this one.) - See design document at [DESIGN](./DESIGN.md). Make sure to keep both this MINDMAP and the DESIGN document updated as implementation goes along.

[15] **AE: mindmap-cli** - v0 implementation complete: 20 commands total (show,list,search,refs,links,add,deprecate,verify,lint,edit,put,patch,delete,graph,orphans,type,relationships,batch,prime,lint). Phase 1 UX improvements: empty result messages, refs/links clarity with directional indicators, result counts, contextual error messages, orphans descriptions, README quick reference, enhanced help text. Phase 2 features: types command with statistics, relationships command (incoming+outgoing), search flags (--case-sensitive/--exact-match/--regex-mode), command aliases (get/update/query/etc), JSON schema enhancements. Phase 3.1: MindmapCache + NavigationContext for recursive navigation. Parser: manual (no regex), benchmarks (~200ns/node). CLI refactor to lib.rs, 18+ unit tests. All 60 tests passing (56 unit + 4 integration). DOC: CLI help added to help output. Types cmd discovers node types with stats. Relationships shows bidirectional refs. Search flags for advanced matching. Command aliases for discoverability. See PHASE1_IMPLEMENTATION.md, PHASE2_IMPLEMENTATION.md, [planning/UX_ANALYSIS.md](./proj/planning/UX_ANALYSIS.md), [DESIGN.md](./DESIGN.md)

[31] **WF: Protocol for interacting with MINDMAP** - See [PROTOCOL_MINDMAP.md](./PROTOCOL_MINDMAP.md) for the formal protocol describing how to interact with MINDMAP.md (add/edit/lint/orphans flows).

[38] **WF: Git commit messages** - Require good but terse commit messages: short summary (<=72 chars) and optional body; reference ticket IDs; keep commits atomic.

[39] **WF: Prime command** - Runs 'mindmap-cli prime' to output help and list nodes to prime an AI agent's context; additionally concatenates PROTOCOL_MINDMAP.md (if present) to prime agents to follow the protocol.

[40] **WF: Lint auto-fix** - Use 'mindmap-cli lint --fix' to automatically fix common issues: ensures exactly one blank line between node lines (collapses multiple blanks), removes duplicated type prefixes in titles (e.g. AE: AE: Foo becomes AE: Foo).

[41] **WF: Batch atomic operations** - Use 'mindmap-cli batch' to apply multiple operations atomically. Supports --format lines (CLI-style) or json, --dry-run preview, and --fix auto-correction. Includes blake3 hash concurrency guard to detect and reject commits if file changed mid-batch.

[43] **DONE: Phase 1 UX Improvements - Results clarity, discoverability, navigation** - Completed all 7 Phase 1 quick wins: empty result messages (1h), refs/links clarity with aliases (1h), result counts & headers (1h), better error messages with context hints (2h), orphans --with-descriptions flag (2h), README quick reference table (1h), improved help text across commands (2h). All 43 tests passing. See PHASE1_IMPLEMENTATION.md and planning/UX_ANALYSIS.md for details. Result: 40% UX improvement achieved [15][44]

[44] **DOC: Phase 1 Implementation Summary** - Comprehensive record of Phase 1 UX improvements implementation: 7 quick wins delivered, 0 breaking changes, 43 tests passing, 40% UX improvement achieved. Documents changes to List/Refs/Links/Search/Show/Orphans/Lint commands, error message enhancements, help text improvements, README additions. See PHASE1_IMPLEMENTATION.md for full details [43]

[50] **DONE: Phase 3.1: Core Data Structures - MindmapCache & NavigationContext** - Implemented secure file loading and caching for recursive navigation. Created MindmapCache with path resolution, file size checks, cycle detection. Created NavigationContext with depth tracking and RAII guards. Added 17 new unit tests, all passing. Aligns with Node [14] Security priority. See PHASE3_IMPLEMENTATION.md [50]

[51] **AE: MindmapCache** - Secure file loading and caching. Provides lazy loading, path resolution with security validation (prevents directory traversal), file size checks (max 10MB default), cycle detection integration. 8 unit tests. Location: src/cache.rs [50]

[52] **AE: NavigationContext** - Depth tracking and cycle detection for recursive operations. Recursion depth counter with configurable limits (default 50), visited file set, RAII guard pattern for auto-decrement. 9 unit tests. Location: src/context.rs [50]

[53] **DONE: Implementation Milestones** - v0 phases complete: Phase1 parser (manual, no regex), Phase2 commands (18 subcommands), Phase3 navigation (refs/links/graph), Phase4 edit (via $EDITOR), Phase5 lint (with --fix), Phase6 batch (atomic, blake3 guard), Phase7 tests (38 unit + 4 integration), Phase8 docs updated. Phase 1 UX: empty result messages, refs/links clarity, result counts, contextual errors, orphans descriptions, README quick reference, enhanced help. Phase 2: types cmd, relationships, search flags, JSON enhancements, aliases. See PHASE1_IMPLEMENTATION.md, PHASE2_IMPLEMENTATION.md, [planning/UX_ANALYSIS.md](./proj/planning/UX_ANALYSIS.md), [planning/UX_ANALYSIS_SUMMARY.md](./proj/planning/UX_ANALYSIS_SUMMARY.md), [planning/SUMMARY.md](./proj/planning/SUMMARY.md)

[54] **DR: Core File & Node Semantics** - Default filename = MINDMAP.md; node format regex; ID immutability; atomic save strategy; editor single-line validation. See [DESIGN.md](./DESIGN.md), [planning/implementation-multi-id.md](./proj/planning/implementation-multi-id.md)

[55] **DR: Command Semantics** - PUT and PATCH ops; output formats & JSON; delete semantics (blocks by default). See [DESIGN.md](./DESIGN.md)
