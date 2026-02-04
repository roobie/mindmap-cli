# File Navigation Feature: Planning

## Goal
Extend mindmap-cli to support references to external MINDMAP files (e.g., `[MINDMAP.llm.md]` or `[./sub/map.md]`), allowing cross-file navigation and graph visualization. This enables scaling beyond single-file mindmaps by linking related domains.

## Current State
- References are `[123]` for internal numeric IDs.
- Commands like `show`, `refs`, `links`, `graph` operate within a single file.
- Example: Node [9] already hints at splitting: "split into domain files like `MINDMAP.llm.md`".

## Design Decisions
1. **Reference Format**:
   - `[123]` → Internal ID (u32).
   - `[file.md]`, `[./path.md]`, `[MINDMAP.llm.md]` → File reference (String path).
   - Distinguish: If content is all digits, ID; else, file path.
   - Support relative paths; resolve relative to current mindmap file.

2. **Data Structures**:
   - `enum Reference { Id(u32), File(String) }`
   - Update `Node.references: Vec<Reference>`.
   - Update `Mindmap` to hold a cache of loaded external files (lazy load).

3. **Parsing**:
   - Extend `extract_refs_from_str` to parse `[content]` → if numeric, `Id`; else `File`.
   - Validate file refs: Check if path exists (optional, warn if not).
   - Update `parse_node_line` to handle new refs.

4. **File Loading**:
   - When encountering `File(path)`, load external file via `Mindmap::load(path)`.
   - Cache loaded mindmaps in a `HashMap<PathBuf, Mindmap>` to avoid re-loading.
   - Handle cycles (e.g., file A refs file B, B refs A).

5. **Command Updates**:
   - `show id`: If node has file refs, show "External refs: file.md -> [123]".
   - `refs id`: Follow into external files; show nodes in other files that ref this ID (cross-file incoming).
   - `links id`: Show outgoing refs, including file links.
   - `graph id`: Include cross-file nodes/edges in DOT output.
   - Add `--follow-files` flag to enable file navigation (default off for safety).

6. **Error Handling**:
   - Missing files: Warn but don't fail.
   - Invalid refs: Lint detects broken file refs.
   - Circular deps: Detect and warn.

## Implementation Plan (Incremental)
1. **Phase 1: Core Data & Parsing**
   - Add `enum Reference` and update structs.
   - Modify `extract_refs_from_str` and `parse_node_line`.
   - Add unit tests for new ref parsing.

2. **Phase 2: File Loading & Caching**
   - Add `MindmapCache` struct with lazy loading.
   - Update `Mindmap` to use cache for file refs.

3. **Phase 3: Update Commands**
   - Modify `cmd_show`, `cmd_refs`, `cmd_links`, `cmd_graph` to handle file refs.
   - Add `--follow-files` flag to CLI.

4. **Phase 4: Testing & Validation**
   - Add integration tests with multi-file mindmaps.
   - Update lint for file ref validation.

## Risks & Considerations
- Performance: Loading many files on demand.
- Security: Path traversal (sanitize paths).
- Complexity: Increases codebase size; ensure backward compatibility.
- Scope: Start with read-only navigation; add editing later.

## Acceptance Criteria
- `mindmap show 10 --follow-files` shows external refs.
- `mindmap graph 10 --follow-files | dot -Tpng` includes cross-file nodes.
- Lint warns on missing files.