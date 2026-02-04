# File Navigation Feature: Planning

## Goal
Extend mindmap-cli to support references to external MINDMAP files using markdown link syntax (e.g., `[234](./MINDMAP.llm.md)`), allowing cross-file navigation and graph visualization. This enables scaling beyond single-file mindmaps by linking related domains while keeping existing `[123]` format for internal IDs.

## Current State
- References are `[123]` for internal numeric IDs.
- Commands like `show`, `refs`, `links`, `graph` operate within a single file.
- Example: Node [9] already hints at splitting: "split into domain files like `MINDMAP.llm.md`".

## Design Decisions
1. **Reference Format**:
   - `[123]` → Internal ID (u32) in current file.
   - `[234](./path.md)` → External: ID 234 in file at `./path.md`.
   - Distinguish: Parse for `[digits](path)` as external, `[digits]` as internal.
   - Paths are relative to the current mindmap file; resolve accordingly.

2. **Data Structures**:
   - `enum Reference { Internal(u32), External { id: u32, file: String } }`
   - Update `Node.references: Vec<Reference>`.
   - Update `Mindmap` to hold a cache of loaded external files (lazy load).

3. **Parsing**:
   - Extend `extract_refs_from_str` to parse `[content]` → if followed by `(path)`, treat as `External { id: content.parse()?, file: path }`; else `Internal(content.parse()?)`.
   - Validate: Ensure `content` is digits; `path` is valid relative path.
   - Update `parse_node_line` to handle new refs.

4. **File Loading**:
   - When encountering `External { id, file }`, load external file via `Mindmap::load(resolved_path)`.
   - Cache loaded mindmaps in a `HashMap<PathBuf, Mindmap>` to avoid re-loading.
   - Handle cycles (e.g., file A refs file B, B refs A).

5. **Command Updates**:
   - `show id`: If node has external refs, show "External refs: ./file.md -> [234]".
   - `refs id`: Follow into external files; show nodes in other files that ref this ID (cross-file incoming).
   - `links id`: Show outgoing refs, including external links.
   - `graph id`: Include cross-file nodes/edges in DOT output.
   - Add `--follow-files` flag to enable file navigation (default off for safety).

6. **Error Handling**:
   - Missing files: Warn but don't fail.
   - Invalid refs: Lint detects broken file refs or non-existent IDs in external files.
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
- Security: Path traversal (sanitize and canonicalize paths).
- Complexity: Parsing more complex; ensure backward compatibility with existing `[123]` refs.
- Scope: Start with read-only navigation; add editing later.

## Safety Nets
To prevent issues with infinite loops, resource exhaustion, and security vulnerabilities during file navigation:

1. **Visited Files Tracking**:
   - Maintain a `HashSet<PathBuf>` of canonicalized paths for visited files.
   - Before loading an external file, check if its canonical path is already in the set. If yes, skip with a warning (cycle detected).
   - This prevents circular references (e.g., A refs B, B refs A).

2. **Max Depth Cap**:
   - Enforce a recursion depth limit (e.g., 1000 levels) when following references across files.
   - Track depth in recursive calls; if exceeded, warn and stop following.
   - Prevents infinite chains of references.

3. **Other Safety Nets**:
   - **Path Canonicalization**: Always canonicalize paths to handle `..`, symlinks, and duplicates. Use `std::fs::canonicalize` or similar.
   - **File Size Limit**: Check file size before loading (e.g., max 10MB) to avoid loading huge files.
   - **Timeout**: If loading takes too long (e.g., network paths), implement a timeout (though local files are fast).
   - **Path Validation**: Restrict paths to relative or within a trusted directory; block absolute paths or `/` starts.
   - **Error Recovery**: On file load failure, log warning but continue with available data.
   - **Caching Limits**: Limit cache size (e.g., max 100 files) to prevent memory exhaustion.

## Acceptance Criteria
- `mindmap show 10 --follow-files` shows external refs.
- `mindmap graph 10 --follow-files | dot -Tpng` includes cross-file nodes.
- Lint warns on missing files or invalid external IDs.