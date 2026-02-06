use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{collections::HashMap, fs, io::Read, path::PathBuf};

mod ui;

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    Default,
    Json,
}

#[derive(Parser)]
#[command(name = "mindmap-cli")]
#[command(about = "CLI tool for working with MINDMAP files")]
#[command(
    long_about = r#"mindmap-cli - small CLI for inspecting and safely editing one-line MINDMAP files (default: ./MINDMAP.md).
One-node-per-line format: [N] **Title** - description with [N] references. IDs must be stable numeric values.

EXAMPLES:
  mindmap-cli show 10
  mindmap-cli list --type AE --grep auth
  mindmap-cli add --type AE --title "AuthService" --desc "Handles auth [12]"
  mindmap-cli edit 12               # opens $EDITOR for an atomic, validated edit
  mindmap-cli patch 12 --title "AuthSvc" --desc "Updated desc"   # partial update (PATCH)
  mindmap-cli put 12 --line "[31] **WF: Example** - Full line text [12]"   # full-line replace (PUT)
  mindmap-cli graph 10 | dot -Tpng > graph.png   # generate neighborhood graph
  mindmap-cli lint
  mindmap-cli batch --input - --dry-run <<EOF  # atomic batch from stdin
  add --type WF --title "New Workflow" --desc "Steps here"
  patch 15 --title "Updated Workflow"
  delete 19
  EOF

Notes:
  - Default file: ./MINDMAP.md (override with --file)
  - Use `--file -` to read a mindmap from stdin for read-only commands (list/show/refs/links/search/lint/orphans). Mutating commands will error when source is `-`.
  - Use the EDITOR env var to control the editor used by 'edit'
"#
)]
pub struct Cli {
    /// Path to MINDMAP file (defaults to ./MINDMAP.md)
    #[arg(global = true, short, long)]
    pub file: Option<PathBuf>,

    /// Output format: default (human) or json
    #[arg(global = true, long, value_enum, default_value_t = OutputFormat::Default)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show a node by ID (displays incoming and outgoing references)
    #[command(alias = "get", alias = "inspect")]
    Show {
        /// Node ID
        id: u32,
    },

    /// List nodes (optionally filtered by --type or --grep with search flags)
    List {
        /// Filter by node type prefix (case-sensitive, e.g., AE, WF, DOC)
        #[arg(long)]
        r#type: Option<String>,
        /// Filter by substring (default: case-insensitive substring match)
        #[arg(long)]
        grep: Option<String>,
        /// Match case exactly (default: case-insensitive)
        #[arg(long)]
        case_sensitive: bool,
        /// Match entire words/phrases exactly (default: substring match)
        #[arg(long)]
        exact_match: bool,
        /// Use regex pattern instead of plain text
        #[arg(long)]
        regex_mode: bool,
    },

    /// Show nodes that REFERENCE (← INCOMING) the given ID
    #[command(alias = "incoming")]
    Refs {
        /// Node ID to find incoming references for
        id: u32,
    },

    /// Show nodes that the given ID REFERENCES (→ OUTGOING)
    #[command(alias = "outgoing")]
    Links {
        /// Node ID to find outgoing references from
        id: u32,
    },

    /// Search nodes by substring (case-insensitive, alias: mindmap-cli search = mindmap-cli list --grep)
    /// Search nodes by substring (case-insensitive by default, use flags for advanced search)
    #[command(alias = "query")]
    Search {
        /// Search query (searches title and description)
        query: String,
        /// Match case exactly (default: case-insensitive)
        #[arg(long)]
        case_sensitive: bool,
        /// Match entire words/phrases exactly (default: substring match)
        #[arg(long)]
        exact_match: bool,
        /// Use regex pattern instead of plain text
        #[arg(long)]
        regex_mode: bool,
    },

    /// Add a new node
    Add {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        desc: Option<String>,
        /// When using editor flow, perform strict reference validation
        #[arg(long)]
        strict: bool,
    },

    /// Deprecate a node, redirecting to another
    Deprecate {
        id: u32,
        #[arg(long)]
        to: u32,
    },

    /// Edit a node with $EDITOR
    Edit { id: u32 },

    /// Patch (partial update) a node: --type, --title, --desc
    Patch {
        id: u32,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        desc: Option<String>,
        #[arg(long)]
        strict: bool,
    },

    /// Put (full-line replace) a node: --line
    #[command(alias = "update")]
    Put {
        id: u32,
        #[arg(long)]
        line: String,
        #[arg(long)]
        strict: bool,
    },

    /// Mark a node as needing verification (append verify tag)
    Verify { id: u32 },

    /// Delete a node by ID; use --force to remove even if referenced
    Delete {
        id: u32,
        #[arg(long)]
        force: bool,
    },

    /// Lint the mindmap for basic issues (use --fix to auto-fix spacing and type prefixes)
    Lint {
        /// Auto-fix spacing and duplicated type prefixes
        #[arg(long)]
        fix: bool,
    },

    /// Show orphan nodes (no in & no out, excluding META)
    Orphans {
        /// Include node descriptions in output
        #[arg(long)]
        with_descriptions: bool,
    },

    /// Show all node types in use with statistics and frequency
    #[command(alias = "types")]
    Type {
        /// Show details for a specific type (e.g., AE, WF, DR)
        #[arg(long)]
        of: Option<String>,
    },

    /// Show incoming and outgoing references for a node in one view
    #[command(alias = "rel")]
    Relationships {
        /// Node ID to show relationships for
        id: u32,
    },

    /// Show graph neighborhood for a node (DOT format for Graphviz)
    Graph { id: u32 },

    /// Prime: print help and list to prime an AI agent's context
    Prime,

    /// Batch mode: apply multiple non-interactive commands atomically
    Batch {
        /// Input file with commands (one per line) or '-' for stdin
        #[arg(long)]
        input: Option<PathBuf>,
        /// Input format: 'lines' or 'json'
        #[arg(long, default_value = "lines")]
        format: String,
        /// Do not write changes; just show what would happen
        #[arg(long)]
        dry_run: bool,
        /// Apply auto-fixes (spacing / duplicated type prefixes) before saving
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub raw_title: String,
    pub description: String,
    pub references: Vec<Reference>,
    pub line_index: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Reference {
    Internal(u32),
    External(u32, String),
}

pub struct Mindmap {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub nodes: Vec<Node>,
    pub by_id: HashMap<u32, usize>,
}

impl Mindmap {
    pub fn load(path: PathBuf) -> Result<Self> {
        // load from file path
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.display()))?;
        Self::from_string(content, path)
    }

    /// Load mindmap content from any reader (e.g., stdin). Provide a path placeholder (e.g. "-")
    /// so that callers can detect that the source was non-writable (stdin).
    pub fn load_from_reader<R: Read>(mut reader: R, path: PathBuf) -> Result<Self> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Self::from_string(content, path)
    }

    fn from_string(content: String, path: PathBuf) -> Result<Self> {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let mut nodes = Vec::new();
        let mut by_id = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if let Ok(node) = parse_node_line(line, i) {
                if by_id.contains_key(&node.id) {
                    eprintln!("Warning: duplicate node id {} at line {}", node.id, i + 1);
                }
                by_id.insert(node.id, nodes.len());
                nodes.push(node);
            }
        }

        Ok(Mindmap {
            path,
            lines,
            nodes,
            by_id,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        // prevent persisting when loaded from stdin (path == "-")
        if self.path.as_os_str() == "-" {
            return Err(anyhow::anyhow!(
                "Cannot save: mindmap was loaded from stdin ('-'); use --file <path> to save changes"
            ));
        }

        // Normalize spacing in-place so node lines are separated by at least one blank
        // line before writing. This updates self.lines and internal node indices.
        self.normalize_spacing()?;

        // atomic write: write to a temp file in the same dir then persist
        let dir = self
            .path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut tmp = tempfile::NamedTempFile::new_in(&dir)
            .with_context(|| format!("Failed to create temp file in {}", dir.display()))?;
        let content = self.lines.join("\n") + "\n";
        use std::io::Write;
        tmp.write_all(content.as_bytes())?;
        tmp.flush()?;
        tmp.persist(&self.path)
            .with_context(|| format!("Failed to persist temp file to {}", self.path.display()))?;
        Ok(())
    }

    pub fn next_id(&self) -> u32 {
        self.by_id.keys().max().copied().unwrap_or(0) + 1
    }

    pub fn get_node(&self, id: u32) -> Option<&Node> {
        self.by_id.get(&id).map(|&idx| &self.nodes[idx])
    }

    /// Ensure there is at least one empty line between any two adjacent node lines.
    /// This inserts a blank line when two node lines are directly adjacent, and
    /// rebuilds internal node indices accordingly. The operation is idempotent.
    pub fn normalize_spacing(&mut self) -> Result<()> {
        // Quick exit
        if self.lines.is_empty() {
            return Ok(());
        }

        let orig = self.lines.clone();
        let mut new_lines: Vec<String> = Vec::new();

        for i in 0..orig.len() {
            let line = orig[i].clone();
            new_lines.push(line.clone());

            // If this line is a node and the immediate next line is also a node,
            // insert a single empty line between them. We only insert when nodes
            // are adjacent (no blank or non-node line in between).
            if parse_node_line(&line, i).is_ok()
                && i + 1 < orig.len()
                && parse_node_line(&orig[i + 1], i + 1).is_ok()
            {
                new_lines.push(String::new());
            }
        }

        // No change
        if new_lines == orig {
            return Ok(());
        }

        // Rebuild internal state from normalized content so line_index/by_id are correct
        let content = new_lines.join("\n") + "\n";
        let normalized_mm = Mindmap::from_string(content, self.path.clone())?;
        self.lines = normalized_mm.lines;
        self.nodes = normalized_mm.nodes;
        self.by_id = normalized_mm.by_id;

        Ok(())
    }

    /// Apply automatic fixes: normalize spacing (ensuring exactly one blank between nodes)
    /// and remove duplicated leading type prefixes in node titles (e.g., "AE: AE: Foo" -> "AE: Foo").
    pub fn apply_fixes(&mut self) -> Result<FixReport> {
        let mut report = FixReport::default();

        // 1) normalize spacing (ensure exactly one blank line between nodes, collapse multiples)
        if self.lines.is_empty() {
            return Ok(report);
        }

        let orig = self.lines.clone();
        let mut new_lines: Vec<String> = Vec::new();
        let mut i = 0usize;
        while i < orig.len() {
            let line = orig[i].clone();
            new_lines.push(line.clone());

            // If this line is a node, look ahead to find next node
            if parse_node_line(&line, i).is_ok() {
                let mut j = i + 1;
                // Count blank lines following this node
                while j < orig.len() && orig[j].trim().is_empty() {
                    j += 1;
                }

                // If there's a next node at j, ensure exactly one blank line between
                if j < orig.len() && parse_node_line(&orig[j], j).is_ok() {
                    if j == i + 1 {
                        // adjacent nodes -> insert one blank
                        new_lines.push(String::new());
                        report.spacing.push(i + 1);
                    } else if j > i + 2 {
                        // multiple blanks -> collapse to one
                        new_lines.push(String::new());
                        report.spacing.push(i + 1);
                    }
                    i = j;
                    continue;
                }
            }
            i += 1;
        }

        // If spacing changed, update lines and reparse
        if !report.spacing.is_empty() {
            let content = new_lines.join("\n") + "\n";
            let normalized_mm = Mindmap::from_string(content, self.path.clone())?;
            self.lines = normalized_mm.lines;
            self.nodes = normalized_mm.nodes;
            self.by_id = normalized_mm.by_id;
        }

        // 2) fix duplicated type prefixes in node titles (e.g., "AE: AE: X" -> "AE: X")
        let mut changed = false;
        let mut new_lines = self.lines.clone();
        for node in &self.nodes {
            if let Some(colon_pos) = node.raw_title.find(':') {
                let leading_type = node.raw_title[..colon_pos].trim();
                let after_colon = node.raw_title[colon_pos + 1..].trim_start();

                // Check if after_colon also starts with the same type + ':'
                if after_colon.starts_with(&format!("{}:", leading_type)) {
                    // Remove the duplicated type prefix
                    let after_dup = after_colon[leading_type.len() + 1..].trim_start();
                    let new_raw = if after_dup.is_empty() {
                        leading_type.to_string()
                    } else {
                        format!("{}: {}", leading_type, after_dup)
                    };

                    report.title_fixes.push(TitleFix {
                        id: node.id,
                        old: node.raw_title.clone(),
                        new: new_raw.clone(),
                    });

                    // Update the corresponding line in new_lines
                    new_lines[node.line_index] =
                        format!("[{}] **{}** - {}", node.id, new_raw, node.description);
                    changed = true;
                }
            }
        }

        if changed {
            let content = new_lines.join("\n") + "\n";
            let normalized_mm = Mindmap::from_string(content, self.path.clone())?;
            self.lines = normalized_mm.lines;
            self.nodes = normalized_mm.nodes;
            self.by_id = normalized_mm.by_id;
        }

        Ok(report)
    }
}

// Helper: lightweight manual parser for the strict node format
// Format: ^\[(\d+)\] \*\*(.+?)\*\* - (.*)$
pub fn parse_node_line(line: &str, line_index: usize) -> Result<Node> {
    // Fast path sanity checks
    let trimmed = line.trim_start();
    if !trimmed.starts_with('[') {
        return Err(anyhow::anyhow!("Line does not match node format"));
    }

    // Find closing bracket for ID
    let end_bracket = match trimmed.find(']') {
        Some(pos) => pos,
        None => return Err(anyhow::anyhow!("Line does not match node format")),
    };

    let id_str = &trimmed[1..end_bracket];
    let id: u32 = id_str.parse()?;

    // Expect a space after ']'
    let mut pos = end_bracket + 1;
    let chars = trimmed.as_bytes();
    if chars.get(pos).map(|b| *b as char) == Some(' ') {
        pos += 1;
    } else {
        return Err(anyhow::anyhow!("Line does not match node format"));
    }

    // Expect opening '**'
    if trimmed.get(pos..pos + 2) != Some("**") {
        return Err(anyhow::anyhow!("Line does not match node format"));
    }
    pos += 2;

    // Find closing '**' for title
    let rem = &trimmed[pos..];
    let title_rel_end = match rem.find("**") {
        Some(p) => p,
        None => return Err(anyhow::anyhow!("Line does not match node format")),
    };
    let title = rem[..title_rel_end].to_string();
    pos += title_rel_end + 2; // skip closing '**'

    // Expect ' - ' (space dash space)
    if trimmed.get(pos..pos + 3) != Some(" - ") {
        return Err(anyhow::anyhow!("Line does not match node format"));
    }
    pos += 3;

    let description = trimmed[pos..].to_string();

    // Extract references
    let references = extract_refs_from_str(&description, Some(id));

    Ok(Node {
        id,
        raw_title: title,
        description,
        references,
        line_index,
    })
}

// Extract references of the form [123] or [234](./file.md) from a description string.
// If skip_self is Some(id) then occurrences equal to that id are ignored.
fn extract_refs_from_str(s: &str, skip_self: Option<u32>) -> Vec<Reference> {
    let mut refs = Vec::new();
    let mut i = 0usize;
    while i < s.len() {
        // find next '['
        if let Some(rel) = s[i..].find('[') {
            let start = i + rel;
            if let Some(rel_end) = s[start..].find(']') {
                let end = start + rel_end;
                let idslice = &s[start + 1..end];
                if !idslice.is_empty()
                    && idslice.chars().all(|c| c.is_ascii_digit())
                    && let Ok(rid) = idslice.parse::<u32>()
                    && Some(rid) != skip_self
                {
                    // check if followed by (path)
                    let after = &s[end..];
                    if after.starts_with("](") {
                        // find closing )
                        if let Some(paren_end) = after.find(')') {
                            let path_start = end + 2; // after ](
                            let path_end = end + paren_end;
                            let path = &s[path_start..path_end];
                            refs.push(Reference::External(rid, path.to_string()));
                            i = path_end + 1;
                            continue;
                        }
                    }
                    // internal ref
                    refs.push(Reference::Internal(rid));
                }
                i = end + 1;
                continue;
            } else {
                break; // unmatched '['
            }
        } else {
            break;
        }
    }
    refs
}

// Command helpers

pub fn cmd_show(mm: &Mindmap, id: u32) -> String {
    if let Some(node) = mm.get_node(id) {
        let mut out = format!(
            "[{}] **{}** - {}",
            node.id, node.raw_title, node.description
        );

        // inbound refs
        let mut inbound = Vec::new();
        for n in &mm.nodes {
            if n.references
                .iter()
                .any(|r| matches!(r, Reference::Internal(iid) if *iid == id))
            {
                inbound.push(n.id);
            }
        }
        if !inbound.is_empty() {
            out.push_str(&format!("\nReferred to by: {:?}", inbound));
        }
        out
    } else {
        format!("Node [{}] not found", id)
    }
}

pub fn cmd_list(
    mm: &Mindmap,
    type_filter: Option<&str>,
    grep: Option<&str>,
    case_sensitive: bool,
    exact_match: bool,
    regex_mode: bool,
) -> Vec<String> {
    let mut res = Vec::new();

    // Compile regex if needed
    let regex_pattern: Option<regex::Regex> = if regex_mode && let Some(grep) = grep {
        match regex::Regex::new(grep) {
            Ok(r) => Some(r),
            Err(_) => return vec!["Invalid regex pattern".to_string()],
        }
    } else {
        None
    };

    for n in &mm.nodes {
        // Type filter
        if let Some(tf) = type_filter
            && !n.raw_title.starts_with(&format!("{}:", tf))
        {
            continue;
        }

        // Text filter
        if let Some(q) = grep {
            let matches = if let Some(re) = &regex_pattern {
                // Regex search
                re.is_match(&n.raw_title) || re.is_match(&n.description)
            } else if exact_match {
                // Exact phrase match
                let query = if case_sensitive {
                    q.to_string()
                } else {
                    q.to_lowercase()
                };
                let title = if case_sensitive {
                    n.raw_title.clone()
                } else {
                    n.raw_title.to_lowercase()
                };
                let desc = if case_sensitive {
                    n.description.clone()
                } else {
                    n.description.to_lowercase()
                };
                title == query
                    || desc == query
                    || title.contains(&format!(" {} ", query))
                    || desc.contains(&format!(" {} ", query))
            } else {
                // Substring match
                let query = if case_sensitive {
                    q.to_string()
                } else {
                    q.to_lowercase()
                };
                let title = if case_sensitive {
                    n.raw_title.clone()
                } else {
                    n.raw_title.to_lowercase()
                };
                let desc = if case_sensitive {
                    n.description.clone()
                } else {
                    n.description.to_lowercase()
                };
                title.contains(&query) || desc.contains(&query)
            };

            if !matches {
                continue;
            }
        }

        res.push(format!(
            "[{}] **{}** - {}",
            n.id, n.raw_title, n.description
        ));
    }
    res
}

pub fn cmd_refs(mm: &Mindmap, id: u32) -> Vec<String> {
    let mut out = Vec::new();
    for n in &mm.nodes {
        if n.references
            .iter()
            .any(|r| matches!(r, Reference::Internal(iid) if *iid == id))
        {
            out.push(format!(
                "[{}] **{}** - {}",
                n.id, n.raw_title, n.description
            ));
        }
    }
    out
}

pub fn cmd_links(mm: &Mindmap, id: u32) -> Option<Vec<Reference>> {
    mm.get_node(id).map(|n| n.references.clone())
}

// NOTE: cmd_search was consolidated into cmd_list to eliminate code duplication.
// See `Commands::Search` handler below which delegates to `cmd_list(mm, None, Some(query))`.

pub fn cmd_add(mm: &mut Mindmap, type_prefix: &str, title: &str, desc: &str) -> Result<u32> {
    let id = mm.next_id();
    let full_title = format!("{}: {}", type_prefix, title);
    let line = format!("[{}] **{}** - {}", id, full_title, desc);

    mm.lines.push(line.clone());

    let line_index = mm.lines.len() - 1;
    let references = extract_refs_from_str(desc, Some(id));

    let node = Node {
        id,
        raw_title: full_title,
        description: desc.to_string(),
        references,
        line_index,
    };
    mm.by_id.insert(id, mm.nodes.len());
    mm.nodes.push(node);

    Ok(id)
}

pub fn cmd_add_editor(mm: &mut Mindmap, editor: &str, strict: bool) -> Result<u32> {
    // require interactive terminal for editor
    if !atty::is(atty::Stream::Stdin) {
        return Err(anyhow::anyhow!(
            "add via editor requires an interactive terminal"
        ));
    }

    let id = mm.next_id();
    let template = format!("[{}] **TYPE: Title** - description", id);

    // create temp file and write template
    let mut tmp = tempfile::NamedTempFile::new()
        .with_context(|| "Failed to create temp file for add editor")?;
    use std::io::Write;
    writeln!(tmp, "{}", template)?;
    tmp.flush()?;

    // launch editor
    let status = std::process::Command::new(editor)
        .arg(tmp.path())
        .status()
        .with_context(|| "Failed to launch editor")?;
    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // read edited content and pick first non-empty line
    let edited = std::fs::read_to_string(tmp.path())?;
    let nonempty: Vec<&str> = edited
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if nonempty.is_empty() {
        return Err(anyhow::anyhow!("No content written in editor"));
    }
    if nonempty.len() > 1 {
        return Err(anyhow::anyhow!(
            "Expected exactly one node line in editor; found multiple lines"
        ));
    }
    let line = nonempty[0];

    // parse and validate
    let parsed = parse_node_line(line, mm.lines.len())?;
    if parsed.id != id {
        return Err(anyhow::anyhow!(format!(
            "Added line id changed; expected [{}]",
            id
        )));
    }

    if strict {
        for r in &parsed.references {
            if let Reference::Internal(iid) = r
                && !mm.by_id.contains_key(iid)
            {
                return Err(anyhow::anyhow!(format!(
                    "ADD strict: reference to missing node {}",
                    iid
                )));
            }
        }
    }

    // apply: append line and node
    mm.lines.push(line.to_string());
    let line_index = mm.lines.len() - 1;
    let node = Node {
        id: parsed.id,
        raw_title: parsed.raw_title,
        description: parsed.description,
        references: parsed.references,
        line_index,
    };
    mm.by_id.insert(id, mm.nodes.len());
    mm.nodes.push(node);

    Ok(id)
}

pub fn cmd_deprecate(mm: &mut Mindmap, id: u32, to: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;

    if !mm.by_id.contains_key(&to) {
        eprintln!(
            "Warning: target node {} does not exist (still updating title)",
            to
        );
    }

    let node = &mut mm.nodes[idx];
    if !node.raw_title.starts_with("[DEPRECATED") {
        node.raw_title = format!("[DEPRECATED → {}] {}", to, node.raw_title);
        mm.lines[node.line_index] = format!(
            "[{}] **{}** - {}",
            node.id, node.raw_title, node.description
        );
    }

    Ok(())
}

pub fn cmd_verify(mm: &mut Mindmap, id: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;
    let node = &mut mm.nodes[idx];

    let tag = format!("(verify {})", chrono::Local::now().format("%Y-%m-%d"));
    if !node.description.contains("(verify ") {
        if node.description.is_empty() {
            node.description = tag.clone();
        } else {
            node.description = format!("{} {}", node.description, tag);
        }
        mm.lines[node.line_index] = format!(
            "[{}] **{}** - {}",
            node.id, node.raw_title, node.description
        );
    }
    Ok(())
}

pub fn cmd_edit(mm: &mut Mindmap, id: u32, editor: &str) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;
    let node = &mm.nodes[idx];

    // create temp file with the single node line
    let mut tmp =
        tempfile::NamedTempFile::new().with_context(|| "Failed to create temp file for editing")?;
    use std::io::Write;
    writeln!(
        tmp,
        "[{}] **{}** - {}",
        node.id, node.raw_title, node.description
    )?;
    tmp.flush()?;

    // launch editor
    let status = std::process::Command::new(editor)
        .arg(tmp.path())
        .status()
        .with_context(|| "Failed to launch editor")?;
    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // read edited content
    let edited = std::fs::read_to_string(tmp.path())?;
    let edited_line = edited.lines().next().unwrap_or("").trim();

    // parse and validate using manual parser
    let parsed = parse_node_line(edited_line, node.line_index)?;
    if parsed.id != id {
        return Err(anyhow::anyhow!("Cannot change node ID"));
    }

    // all good: replace line in mm.lines and update node fields
    mm.lines[node.line_index] = edited_line.to_string();
    let new_title = parsed.raw_title;
    let new_desc = parsed.description;
    let new_refs = parsed.references;

    // update node in-place
    let node_mut = &mut mm.nodes[idx];
    node_mut.raw_title = new_title;
    node_mut.description = new_desc;
    node_mut.references = new_refs;

    Ok(())
}

pub fn cmd_put(mm: &mut Mindmap, id: u32, line: &str, strict: bool) -> Result<()> {
    // full-line replace: parse provided line and enforce same id
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;

    let parsed = parse_node_line(line, mm.nodes[idx].line_index)?;
    if parsed.id != id {
        return Err(anyhow::anyhow!("PUT line id does not match target id"));
    }

    // strict check for references
    if strict {
        for r in &parsed.references {
            if let Reference::Internal(iid) = r
                && !mm.by_id.contains_key(iid)
            {
                return Err(anyhow::anyhow!(format!(
                    "PUT strict: reference to missing node {}",
                    iid
                )));
            }
        }
    }

    // apply
    mm.lines[mm.nodes[idx].line_index] = line.to_string();
    let node_mut = &mut mm.nodes[idx];
    node_mut.raw_title = parsed.raw_title;
    node_mut.description = parsed.description;
    node_mut.references = parsed.references;

    Ok(())
}

pub fn cmd_patch(
    mm: &mut Mindmap,
    id: u32,
    typ: Option<&str>,
    title: Option<&str>,
    desc: Option<&str>,
    strict: bool,
) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;
    let node = &mm.nodes[idx];

    // split existing raw_title into optional type and title
    let mut existing_type: Option<&str> = None;
    let mut existing_title = node.raw_title.as_str();
    if let Some(pos) = node.raw_title.find(':') {
        existing_type = Some(node.raw_title[..pos].trim());
        existing_title = node.raw_title[pos + 1..].trim();
    }

    let new_type = typ.unwrap_or(existing_type.unwrap_or(""));
    let new_title = title.unwrap_or(existing_title);
    let new_desc = desc.unwrap_or(&node.description);

    // build raw title: if type is empty, omit prefix
    let new_raw_title = if new_type.is_empty() {
        new_title.to_string()
    } else {
        format!("{}: {}", new_type, new_title)
    };

    let new_line = format!("[{}] **{}** - {}", id, new_raw_title, new_desc);

    // validate
    let parsed = parse_node_line(&new_line, node.line_index)?;
    if parsed.id != id {
        return Err(anyhow::anyhow!("Patch resulted in different id"));
    }

    if strict {
        for r in &parsed.references {
            if let Reference::Internal(iid) = r
                && !mm.by_id.contains_key(iid)
            {
                return Err(anyhow::anyhow!(format!(
                    "PATCH strict: reference to missing node {}",
                    iid
                )));
            }
        }
    }

    // apply
    mm.lines[node.line_index] = new_line;
    let node_mut = &mut mm.nodes[idx];
    node_mut.raw_title = parsed.raw_title;
    node_mut.description = parsed.description;
    node_mut.references = parsed.references;

    Ok(())
}

pub fn cmd_delete(mm: &mut Mindmap, id: u32, force: bool) -> Result<()> {
    // find node index
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;

    // check incoming references
    let mut incoming_from = Vec::new();
    for n in &mm.nodes {
        if n.references
            .iter()
            .any(|r| matches!(r, Reference::Internal(iid) if *iid == id))
        {
            incoming_from.push(n.id);
        }
    }
    if !incoming_from.is_empty() && !force {
        return Err(anyhow::anyhow!(format!(
            "Node {} is referenced by {:?}; use --force to delete",
            id, incoming_from
        )));
    }

    // remove the line from lines
    let line_idx = mm.nodes[idx].line_index;
    mm.lines.remove(line_idx);

    // remove node from nodes vector
    mm.nodes.remove(idx);

    // rebuild by_id and fix line_index for nodes after removed line
    mm.by_id.clear();
    for (i, node) in mm.nodes.iter_mut().enumerate() {
        // if node was after removed line, decrement its line_index
        if node.line_index > line_idx {
            node.line_index -= 1;
        }
        mm.by_id.insert(node.id, i);
    }

    Ok(())
}

pub fn cmd_lint(mm: &Mindmap) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // 1) Syntax: lines starting with '[' but not matching node format
    for (i, line) in mm.lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') && parse_node_line(trimmed, i).is_err() {
            warnings.push(format!(
                "Syntax: line {} starts with '[' but does not match node format",
                i + 1
            ));
        }
    }

    // 2) Duplicate IDs: scan lines for node ids
    let mut id_map: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, line) in mm.lines.iter().enumerate() {
        if let Ok(node) = parse_node_line(line, i) {
            id_map.entry(node.id).or_default().push(i + 1);
        }
    }
    for (id, locations) in &id_map {
        if locations.len() > 1 {
            warnings.push(format!(
                "Duplicate ID: node {} appears on lines {:?}",
                id, locations
            ));
        }
    }

    // 3) Missing references
    for n in &mm.nodes {
        for r in &n.references {
            match r {
                Reference::Internal(iid) => {
                    if !mm.by_id.contains_key(iid) {
                        warnings.push(format!(
                            "Missing ref: node {} references missing node {}",
                            n.id, iid
                        ));
                    }
                }
                Reference::External(eid, file) => {
                    if !std::path::Path::new(file).exists() {
                        warnings.push(format!(
                            "Missing file: node {} references {} in missing file {}",
                            n.id, eid, file
                        ));
                    }
                }
            }
        }
    }

    if warnings.is_empty() {
        Ok(vec!["Lint OK".to_string()])
    } else {
        Ok(warnings)
    }
}

pub fn cmd_orphans(mm: &Mindmap, with_descriptions: bool) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Orphans: nodes with no in and no out, excluding META:*
    let mut incoming: HashMap<u32, usize> = HashMap::new();
    for n in &mm.nodes {
        incoming.entry(n.id).or_insert(0);
    }
    for n in &mm.nodes {
        for r in &n.references {
            if let Reference::Internal(iid) = r
                && incoming.contains_key(iid)
            {
                *incoming.entry(*iid).or_insert(0) += 1;
            }
        }
    }

    let mut orphan_nodes = Vec::new();
    for n in &mm.nodes {
        let inc = incoming.get(&n.id).copied().unwrap_or(0);
        let out = n.references.len();
        let title_up = n.raw_title.to_uppercase();
        if inc == 0 && out == 0 && !title_up.starts_with("META") {
            orphan_nodes.push(n.clone());
        }
    }

    if orphan_nodes.is_empty() {
        Ok(vec!["No orphans".to_string()])
    } else {
        for n in orphan_nodes {
            if with_descriptions {
                warnings.push(format!(
                    "[{}] **{}** - {}",
                    n.id, n.raw_title, n.description
                ));
            } else {
                warnings.push(format!("{}", n.id));
            }
        }
        Ok(warnings)
    }
}

pub fn cmd_graph(mm: &Mindmap, id: u32) -> Result<String> {
    if !mm.by_id.contains_key(&id) {
        return Err(anyhow::anyhow!(format!("Node {} not found", id)));
    }

    // Collect 1-hop neighborhood: self, direct references (out), and nodes that reference self (in)
    let mut nodes = std::collections::HashSet::new();
    nodes.insert(id);

    // Outgoing: references from self
    if let Some(node) = mm.get_node(id) {
        for r in &node.references {
            if let Reference::Internal(rid) = r {
                nodes.insert(*rid);
            }
        }
    }

    // Incoming: nodes that reference self
    for n in &mm.nodes {
        for r in &n.references {
            if let Reference::Internal(rid) = r
                && *rid == id
            {
                nodes.insert(n.id);
            }
        }
    }

    // Generate DOT
    let mut dot = String::new();
    dot.push_str("digraph {\n");
    dot.push_str("  rankdir=LR;\n");

    // Add nodes
    for &nid in &nodes {
        if let Some(node) = mm.get_node(nid) {
            let label = format!("{}: {}", node.id, node.raw_title.replace("\"", "\\\""));
            dot.push_str(&format!("  {} [label=\"{}\"];\n", nid, label));
        }
    }

    // Add edges: from each node to its references, if both in neighborhood
    for &nid in &nodes {
        if let Some(node) = mm.get_node(nid) {
            for r in &node.references {
                if let Reference::Internal(rid) = r
                    && nodes.contains(rid)
                {
                    dot.push_str(&format!("  {} -> {};\n", nid, rid));
                }
            }
        }
    }

    dot.push_str("}\n");
    Ok(dot)
}

pub fn cmd_types(mm: &Mindmap, type_of: Option<&str>) -> Result<Vec<String>> {
    // Collect all types with their counts
    let mut type_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut type_examples: std::collections::HashMap<String, Vec<u32>> =
        std::collections::HashMap::new();

    for n in &mm.nodes {
        if let Some(colon_pos) = n.raw_title.find(':') {
            let node_type = n.raw_title[..colon_pos].to_string();
            *type_counts.entry(node_type.clone()).or_insert(0) += 1;
            type_examples.entry(node_type).or_default().push(n.id);
        }
    }

    let mut results = Vec::new();

    if let Some(specific_type) = type_of {
        // Show details for specific type
        if let Some(count) = type_counts.get(specific_type) {
            results.push(format!("Type '{}': {} nodes", specific_type, count));
            if let Some(examples) = type_examples.get(specific_type) {
                results.push(format!(
                    "  Examples: {}",
                    examples
                        .iter()
                        .take(5)
                        .map(|id| format!("[{}]", id))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        } else {
            results.push(format!("Type '{}' not found in use", specific_type));
        }
    } else {
        // Show summary of all types
        results.push(format!("Node types in use ({} types):", type_counts.len()));
        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        for (node_type, count) in sorted_types {
            results.push(format!("  {:<10} ({:>3} nodes)", node_type, count));
        }
    }

    Ok(results)
}

pub fn cmd_relationships(mm: &Mindmap, id: u32) -> Result<(Vec<u32>, Vec<Reference>)> {
    // Get node
    mm.get_node(id)
        .ok_or_else(|| anyhow::anyhow!(format!("Node [{}] not found", id)))?;

    // Get incoming references
    let mut incoming = Vec::new();
    for n in &mm.nodes {
        if n.references
            .iter()
            .any(|r| matches!(r, Reference::Internal(iid) if *iid == id))
        {
            incoming.push(n.id);
        }
    }

    // Get outgoing references
    let outgoing = mm
        .get_node(id)
        .map(|n| n.references.clone())
        .unwrap_or_default();

    Ok((incoming, outgoing))
}

/// Compute blake3 hash of content (hex encoded)
fn blake3_hash(content: &[u8]) -> String {
    blake3::hash(content).to_hex().to_string()
}

#[derive(Debug, Clone)]
enum BatchOp {
    Add {
        type_prefix: String,
        title: String,
        desc: String,
    },
    Patch {
        id: u32,
        type_prefix: Option<String>,
        title: Option<String>,
        desc: Option<String>,
    },
    Put {
        id: u32,
        line: String,
    },
    Delete {
        id: u32,
        force: bool,
    },
    Deprecate {
        id: u32,
        to: u32,
    },
    Verify {
        id: u32,
    },
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchResult {
    pub total_ops: usize,
    pub applied: usize,
    pub added_ids: Vec<u32>,
    pub patched_ids: Vec<u32>,
    pub deleted_ids: Vec<u32>,
    pub warnings: Vec<String>,
}

/// Parse a batch operation from a JSON value
fn parse_batch_op_json(val: &serde_json::Value) -> Result<BatchOp> {
    let obj = val
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Op must be a JSON object"))?;
    let op_type = obj
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'op' field"))?;

    match op_type {
        "add" => {
            let type_prefix = obj
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("add: missing 'type' field"))?
                .to_string();
            let title = obj
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("add: missing 'title' field"))?
                .to_string();
            let desc = obj
                .get("desc")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("add: missing 'desc' field"))?
                .to_string();
            Ok(BatchOp::Add {
                type_prefix,
                title,
                desc,
            })
        }
        "patch" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("patch: missing 'id' field"))?
                as u32;
            let type_prefix = obj.get("type").and_then(|v| v.as_str()).map(String::from);
            let title = obj.get("title").and_then(|v| v.as_str()).map(String::from);
            let desc = obj.get("desc").and_then(|v| v.as_str()).map(String::from);
            Ok(BatchOp::Patch {
                id,
                type_prefix,
                title,
                desc,
            })
        }
        "put" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("put: missing 'id' field"))?
                as u32;
            let line = obj
                .get("line")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("put: missing 'line' field"))?
                .to_string();
            Ok(BatchOp::Put { id, line })
        }
        "delete" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("delete: missing 'id' field"))?
                as u32;
            let force = obj.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(BatchOp::Delete { id, force })
        }
        "deprecate" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("deprecate: missing 'id' field"))?
                as u32;
            let to = obj
                .get("to")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("deprecate: missing 'to' field"))?
                as u32;
            Ok(BatchOp::Deprecate { id, to })
        }
        "verify" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("verify: missing 'id' field"))?
                as u32;
            Ok(BatchOp::Verify { id })
        }
        other => Err(anyhow::anyhow!("Unknown op type: {}", other)),
    }
}

/// Parse a batch operation from a CLI line (e.g., "add --type WF --title X --desc Y")
fn parse_batch_op_line(line: &str) -> Result<BatchOp> {
    use shell_words;

    let parts = shell_words::split(line)?;
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty operation line"));
    }

    match parts[0].as_str() {
        "add" => {
            let mut type_prefix = String::new();
            let mut title = String::new();
            let mut desc = String::new();
            let mut i = 1;
            while i < parts.len() {
                match parts[i].as_str() {
                    "--type" => {
                        i += 1;
                        type_prefix = parts
                            .get(i)
                            .ok_or_else(|| anyhow::anyhow!("add: --type requires value"))?
                            .clone();
                    }
                    "--title" => {
                        i += 1;
                        title = parts
                            .get(i)
                            .ok_or_else(|| anyhow::anyhow!("add: --title requires value"))?
                            .clone();
                    }
                    "--desc" => {
                        i += 1;
                        desc = parts
                            .get(i)
                            .ok_or_else(|| anyhow::anyhow!("add: --desc requires value"))?
                            .clone();
                    }
                    _ => {}
                }
                i += 1;
            }
            if type_prefix.is_empty() || title.is_empty() || desc.is_empty() {
                return Err(anyhow::anyhow!("add: requires --type, --title, --desc"));
            }
            Ok(BatchOp::Add {
                type_prefix,
                title,
                desc,
            })
        }
        "patch" => {
            let id: u32 = parts
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("patch: missing id"))?
                .parse()?;
            let mut type_prefix: Option<String> = None;
            let mut title: Option<String> = None;
            let mut desc: Option<String> = None;
            let mut i = 2;
            while i < parts.len() {
                match parts[i].as_str() {
                    "--type" => {
                        i += 1;
                        type_prefix = Some(
                            parts
                                .get(i)
                                .ok_or_else(|| anyhow::anyhow!("patch: --type requires value"))?
                                .clone(),
                        );
                    }
                    "--title" => {
                        i += 1;
                        title = Some(
                            parts
                                .get(i)
                                .ok_or_else(|| anyhow::anyhow!("patch: --title requires value"))?
                                .clone(),
                        );
                    }
                    "--desc" => {
                        i += 1;
                        desc = Some(
                            parts
                                .get(i)
                                .ok_or_else(|| anyhow::anyhow!("patch: --desc requires value"))?
                                .clone(),
                        );
                    }
                    _ => {}
                }
                i += 1;
            }
            Ok(BatchOp::Patch {
                id,
                type_prefix,
                title,
                desc,
            })
        }
        "put" => {
            let id: u32 = parts
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("put: missing id"))?
                .parse()?;
            let mut line = String::new();
            let mut i = 2;
            while i < parts.len() {
                if parts[i] == "--line" {
                    i += 1;
                    line = parts
                        .get(i)
                        .ok_or_else(|| anyhow::anyhow!("put: --line requires value"))?
                        .clone();
                    break;
                }
                i += 1;
            }
            if line.is_empty() {
                return Err(anyhow::anyhow!("put: requires --line"));
            }
            Ok(BatchOp::Put { id, line })
        }
        "delete" => {
            let id: u32 = parts
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("delete: missing id"))?
                .parse()?;
            let force = parts.contains(&"--force".to_string());
            Ok(BatchOp::Delete { id, force })
        }
        "deprecate" => {
            let id: u32 = parts
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("deprecate: missing id"))?
                .parse()?;
            let mut to: Option<u32> = None;
            let mut i = 2;
            while i < parts.len() {
                if parts[i] == "--to" {
                    i += 1;
                    to = Some(
                        parts
                            .get(i)
                            .ok_or_else(|| anyhow::anyhow!("deprecate: --to requires value"))?
                            .parse()?,
                    );
                    break;
                }
                i += 1;
            }
            let to = to.ok_or_else(|| anyhow::anyhow!("deprecate: requires --to"))?;
            Ok(BatchOp::Deprecate { id, to })
        }
        "verify" => {
            let id: u32 = parts
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("verify: missing id"))?
                .parse()?;
            Ok(BatchOp::Verify { id })
        }
        other => Err(anyhow::anyhow!("Unknown batch command: {}", other)),
    }
}

// mod ui;

pub fn run(cli: Cli) -> Result<()> {
    let path = cli.file.unwrap_or_else(|| PathBuf::from("MINDMAP.md"));

    // If user passed '-' use stdin as source
    let mut mm = if path.as_os_str() == "-" {
        Mindmap::load_from_reader(std::io::stdin(), path.clone())?
    } else {
        Mindmap::load(path.clone())?
    };

    // determine whether to use pretty output (interactive + default format)
    let interactive = atty::is(atty::Stream::Stdout);
    let env_override = std::env::var("MINDMAP_PRETTY").ok();
    let pretty_enabled = match env_override.as_deref() {
        Some("0") => false,
        Some("1") => true,
        _ => interactive,
    } && matches!(cli.output, OutputFormat::Default);

    let printer: Option<Box<dyn ui::Printer>> = if matches!(cli.output, OutputFormat::Default) {
        if pretty_enabled {
            Some(Box::new(crate::ui::PrettyPrinter::new()?))
        } else {
            Some(Box::new(crate::ui::PlainPrinter::new()?))
        }
    } else {
        None
    };

    // helper to reject mutating commands when mm.path == '-'
    let cannot_write_err = |cmd_name: &str| -> anyhow::Error {
        anyhow::anyhow!(format!(
            "Cannot {}: mindmap was loaded from stdin ('-'); use --file <path> to save changes",
            cmd_name
        ))
    };

    match cli.command {
        Commands::Show { id } => match mm.get_node(id) {
            Some(node) => {
                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({
                        "command": "show",
                        "node": {
                            "id": node.id,
                            "raw_title": node.raw_title,
                            "description": node.description,
                            "references": node.references,
                            "line_index": node.line_index,
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    // compute inbound refs
                    let mut inbound = Vec::new();
                    for n in &mm.nodes {
                        if n.references
                            .iter()
                            .any(|r| matches!(r, Reference::Internal(iid) if *iid == id))
                        {
                            inbound.push(n.id);
                        }
                    }

                    if let Some(p) = &printer {
                        p.show(node, &inbound, &node.references)?;
                    } else {
                        println!(
                            "[{}] **{}** - {}",
                            node.id, node.raw_title, node.description
                        );
                        if !inbound.is_empty() {
                            eprintln!("← Nodes referring to [{}]: {:?}", id, inbound);
                        }
                        let outbound: Vec<u32> = node
                            .references
                            .iter()
                            .filter_map(|r| match r {
                                Reference::Internal(rid) => Some(*rid),
                                _ => None,
                            })
                            .collect();
                        if !outbound.is_empty() {
                            eprintln!("→ [{}] refers to: {:?}", id, outbound);
                        }
                    }
                }
            }
            None => {
                let min_id = mm.nodes.iter().map(|n| n.id).min();
                let max_id = mm.nodes.iter().map(|n| n.id).max();
                let hint = if let (Some(min), Some(max)) = (min_id, max_id) {
                    format!(
                        " (Valid node IDs: {} to {}). Use `mindmap-cli list` to see all nodes.",
                        min, max
                    )
                } else {
                    " No nodes exist yet. Use `mindmap-cli add` to create one.".to_string()
                };
                return Err(anyhow::anyhow!(format!("Node [{}] not found{}", id, hint)));
            }
        },
        Commands::List {
            r#type,
            grep,
            case_sensitive,
            exact_match,
            regex_mode,
        } => {
            let items = cmd_list(
                &mm,
                r#type.as_deref(),
                grep.as_deref(),
                case_sensitive,
                exact_match,
                regex_mode,
            );
            let count = items.len();

            if matches!(cli.output, OutputFormat::Json) {
                let arr: Vec<_> = items
                    .into_iter()
                    .map(|line| serde_json::json!({"line": line}))
                    .collect();
                let obj = serde_json::json!({"command": "list", "count": count, "items": arr});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                if count == 0 {
                    eprintln!("No matching nodes found (0 results)");
                } else {
                    eprintln!(
                        "Matching nodes ({} result{}:)",
                        count,
                        if count == 1 { "" } else { "s" },
                    );
                }
                if let Some(p) = &printer {
                    p.list(&items)?;
                } else {
                    for it in items {
                        println!("{}", it);
                    }
                }
            }
        }
        Commands::Refs { id } => {
            let items = cmd_refs(&mm, id);
            let count = items.len();

            // First check if the node exists
            if mm.get_node(id).is_none() {
                let min_id = mm.nodes.iter().map(|n| n.id).min();
                let max_id = mm.nodes.iter().map(|n| n.id).max();
                let hint = if let (Some(min), Some(max)) = (min_id, max_id) {
                    format!(" (Valid node IDs: {} to {})", min, max)
                } else {
                    " No nodes exist.".to_string()
                };
                return Err(anyhow::anyhow!(format!("Node [{}] not found{}", id, hint)));
            }

            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "refs", "target": id, "count": count, "items": items});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                if count == 0 {
                    eprintln!("No nodes refer to [{}] (0 results)", id);
                } else {
                    eprintln!(
                        "← Nodes referring to [{}] ({} result{})",
                        id,
                        count,
                        if count == 1 { "" } else { "s" }
                    );
                }
                if let Some(p) = &printer {
                    p.refs(&items)?;
                } else {
                    for it in items {
                        println!("{}", it);
                    }
                }
            }
        }
        Commands::Links { id } => match cmd_links(&mm, id) {
            Some(v) => {
                let count = v
                    .iter()
                    .filter(|r| matches!(r, Reference::Internal(_)))
                    .count();
                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({"command": "links", "source": id, "count": count, "links": v});
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    if count == 0 {
                        eprintln!("→ [{}] refers to no nodes (0 results)", id);
                    } else {
                        eprintln!(
                            "→ [{}] refers to ({} result{})",
                            id,
                            count,
                            if count == 1 { "" } else { "s" }
                        );
                    }
                    if let Some(p) = &printer {
                        p.links(id, &v)?;
                    } else {
                        println!("Node [{}] references: {:?}", id, v);
                    }
                }
            }
            None => {
                let min_id = mm.nodes.iter().map(|n| n.id).min();
                let max_id = mm.nodes.iter().map(|n| n.id).max();
                let hint = if let (Some(min), Some(max)) = (min_id, max_id) {
                    format!(" (Valid node IDs: {} to {})", min, max)
                } else {
                    " No nodes exist.".to_string()
                };
                return Err(anyhow::anyhow!(format!("Node [{}] not found{}", id, hint)));
            }
        },
        Commands::Search {
            query,
            case_sensitive,
            exact_match,
            regex_mode,
        } => {
            // Delegate to cmd_list with grep filter (no type filter)
            let items = cmd_list(
                &mm,
                None,
                Some(&query),
                case_sensitive,
                exact_match,
                regex_mode,
            );
            let count = items.len();

            if matches!(cli.output, OutputFormat::Json) {
                let arr: Vec<_> = items
                    .into_iter()
                    .map(|line| serde_json::json!({"line": line}))
                    .collect();
                let obj = serde_json::json!({"command": "search", "query": query, "count": count, "items": arr});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                if count == 0 {
                    eprintln!("No matches for '{}' (0 results)", query);
                } else {
                    eprintln!(
                        "Search results for '{}' ({} result{})",
                        query,
                        count,
                        if count == 1 { "" } else { "s" }
                    );
                }
                if let Some(p) = &printer {
                    p.list(&items)?;
                } else {
                    for it in items {
                        println!("{}", it);
                    }
                }
            }
        }
        Commands::Add {
            r#type,
            title,
            desc,
            strict,
        } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("add"));
            }
            match (r#type.as_deref(), title.as_deref(), desc.as_deref()) {
                (Some(tp), Some(tt), Some(dd)) => {
                    let id = cmd_add(&mut mm, tp, tt, dd)?;
                    mm.save()?;
                    if matches!(cli.output, OutputFormat::Json)
                        && let Some(node) = mm.get_node(id)
                    {
                        let obj = serde_json::json!({"command": "add", "node": {"id": node.id, "raw_title": node.raw_title, "description": node.description, "references": node.references}});
                        println!("{}", serde_json::to_string_pretty(&obj)?);
                    }
                    eprintln!("Added node [{}]", id);
                }
                (None, None, None) => {
                    // editor flow
                    if !atty::is(atty::Stream::Stdin) {
                        return Err(anyhow::anyhow!(
                            "add via editor requires an interactive terminal"
                        ));
                    }
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                    let id = cmd_add_editor(&mut mm, &editor, strict)?;
                    mm.save()?;
                    if matches!(cli.output, OutputFormat::Json)
                        && let Some(node) = mm.get_node(id)
                    {
                        let obj = serde_json::json!({"command": "add", "node": {"id": node.id, "raw_title": node.raw_title, "description": node.description, "references": node.references}});
                        println!("{}", serde_json::to_string_pretty(&obj)?);
                    }
                    eprintln!("Added node [{}]", id);
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "add requires either all of --type,--title,--desc or none (editor)"
                    ));
                }
            }
        }
        Commands::Deprecate { id, to } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("deprecate"));
            }
            cmd_deprecate(&mut mm, id, to)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json)
                && let Some(node) = mm.get_node(id)
            {
                let obj = serde_json::json!({"command": "deprecate", "node": {"id": node.id, "raw_title": node.raw_title}});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Deprecated node [{}] → [{}]", id, to);
        }
        Commands::Edit { id } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("edit"));
            }
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            cmd_edit(&mut mm, id, &editor)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json)
                && let Some(node) = mm.get_node(id)
            {
                let obj = serde_json::json!({"command": "edit", "node": {"id": node.id, "raw_title": node.raw_title, "description": node.description, "references": node.references}});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Edited node [{}]", id);
        }
        Commands::Patch {
            id,
            r#type,
            title,
            desc,
            strict,
        } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("patch"));
            }
            cmd_patch(
                &mut mm,
                id,
                r#type.as_deref(),
                title.as_deref(),
                desc.as_deref(),
                strict,
            )?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json)
                && let Some(node) = mm.get_node(id)
            {
                let obj = serde_json::json!({"command": "patch", "node": {"id": node.id, "raw_title": node.raw_title, "description": node.description, "references": node.references}});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Patched node [{}]", id);
        }
        Commands::Put { id, line, strict } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("put"));
            }
            cmd_put(&mut mm, id, &line, strict)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json)
                && let Some(node) = mm.get_node(id)
            {
                let obj = serde_json::json!({"command": "put", "node": {"id": node.id, "raw_title": node.raw_title, "description": node.description, "references": node.references}});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Put node [{}]", id);
        }
        Commands::Verify { id } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("verify"));
            }
            cmd_verify(&mut mm, id)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json)
                && let Some(node) = mm.get_node(id)
            {
                let obj = serde_json::json!({"command": "verify", "node": {"id": node.id, "description": node.description}});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Marked node [{}] for verification", id);
        }
        Commands::Delete { id, force } => {
            if mm.path.as_os_str() == "-" {
                return Err(cannot_write_err("delete"));
            }
            cmd_delete(&mut mm, id, force)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "delete", "deleted": id});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Deleted node [{}]", id);
        }
        Commands::Lint { fix } => {
            if fix {
                if mm.path.as_os_str() == "-" {
                    return Err(cannot_write_err("lint --fix"));
                }

                // apply fixes
                let report = mm.apply_fixes()?;
                if report.any_changes() {
                    mm.save()?;
                }

                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({"command": "lint", "fixed": report.any_changes(), "fixes": report});
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    if !report.spacing.is_empty() {
                        eprintln!(
                            "Fixed spacing: inserted {} blank lines",
                            report.spacing.len()
                        );
                    }
                    for tf in &report.title_fixes {
                        eprintln!(
                            "Fixed title for node {}: '{}' -> '{}'",
                            tf.id, tf.old, tf.new
                        );
                    }
                    if !report.any_changes() {
                        eprintln!("No fixes necessary");
                    }

                    // run lint after fixes and print any remaining warnings
                    let res = cmd_lint(&mm)?;
                    for r in res {
                        eprintln!("{}", r);
                    }
                }
            } else {
                let res = cmd_lint(&mm)?;
                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({"command": "lint", "warnings": res.iter().filter(|r| *r != "Lint OK").collect::<Vec<_>>()});
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else if res.len() == 1 && res[0] == "Lint OK" {
                    eprintln!("✓ Lint OK (0 warnings)");
                } else {
                    eprintln!(
                        "Lint found {} warning{}:",
                        res.len(),
                        if res.len() == 1 { "" } else { "s" }
                    );
                    for r in res {
                        eprintln!("  - {}", r);
                    }
                }
            }
        }
        Commands::Orphans { with_descriptions } => {
            let res = cmd_orphans(&mm, with_descriptions)?;
            if matches!(cli.output, OutputFormat::Json) {
                let count = if res.iter().any(|r| r == "No orphans") {
                    0
                } else {
                    res.len()
                };
                let obj = serde_json::json!({"command": "orphans", "count": count, "orphans": res});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                // Print header to stderr
                if res.iter().any(|r| r == "No orphans") {
                    eprintln!("✓ No orphans found (0 results)");
                } else {
                    eprintln!(
                        "Orphan nodes ({} result{}):",
                        res.len(),
                        if res.len() == 1 { "" } else { "s" }
                    );
                }

                // Print data to stdout via printer
                if let Some(p) = &printer {
                    p.orphans(&res)?;
                } else {
                    for r in res {
                        if r != "No orphans" {
                            println!("{}", r);
                        }
                    }
                }
            }
        }
        Commands::Type { of } => {
            let res = cmd_types(&mm, of.as_deref())?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "type", "filter": of, "results": res});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                eprintln!("Node types information:");
                for line in res {
                    if line.starts_with("  ") {
                        println!("{}", line);
                    } else {
                        eprintln!("{}", line);
                    }
                }
            }
        }
        Commands::Relationships { id } => {
            let (incoming, outgoing) = cmd_relationships(&mm, id)?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({
                    "command": "relationships",
                    "node": id,
                    "incoming": incoming,
                    "outgoing": outgoing,
                    "incoming_count": incoming.len(),
                    "outgoing_count": outgoing.len(),
                });
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                eprintln!("Relationships for [{}]:", id);
                eprintln!("← Incoming ({} nodes):", incoming.len());
                for incoming_id in &incoming {
                    if let Some(node) = mm.get_node(*incoming_id) {
                        eprintln!("  [{}] **{}**", incoming_id, node.raw_title);
                    }
                }
                eprintln!("→ Outgoing ({} nodes):", outgoing.len());
                for outgoing_ref in &outgoing {
                    if let Reference::Internal(outgoing_id) = outgoing_ref
                        && let Some(node) = mm.get_node(*outgoing_id)
                    {
                        println!("  [{}] **{}**", outgoing_id, node.raw_title);
                    }
                }
            }
        }
        Commands::Graph { id } => {
            let dot = cmd_graph(&mm, id)?;
            println!("{}", dot);
        }
        Commands::Prime => {
            // Produce help text and then list nodes to prime an agent's context.
            use clap::CommandFactory;
            use std::path::Path;

            let mut cmd = Cli::command();
            // capture help into string
            let mut buf: Vec<u8> = Vec::new();
            cmd.write_long_help(&mut buf)?;
            let help_str = String::from_utf8(buf)?;

            // try to read PROTOCOL_MINDMAP.md next to the mindmap file
            let protocol_path = mm
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
                .join("PROTOCOL_MINDMAP.md");

            let protocol = if Path::new(&protocol_path).exists() {
                match fs::read_to_string(&protocol_path) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        eprintln!("Warning: failed to read {}: {}", protocol_path.display(), e);
                        None
                    }
                }
            } else {
                None
            };

            let items = cmd_list(&mm, None, None, false, false, false);

            if matches!(cli.output, OutputFormat::Json) {
                let arr: Vec<_> = items
                    .into_iter()
                    .map(|line| serde_json::json!({"line": line}))
                    .collect();
                let mut obj =
                    serde_json::json!({"command": "prime", "help": help_str, "items": arr});
                if let Some(proto) = protocol {
                    obj["protocol"] = serde_json::json!(proto);
                }
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                // print help
                println!("{}", help_str);

                // print protocol if found
                if let Some(proto) = protocol {
                    eprintln!("--- PROTOCOL_MINDMAP.md ---");
                    println!("{}", proto);
                    eprintln!("--- end protocol ---");
                }

                // print list
                if let Some(p) = &printer {
                    p.list(&items)?;
                } else {
                    for it in items {
                        println!("{}", it);
                    }
                }
            }
        }
        Commands::Batch {
            input,
            format,
            dry_run,
            fix,
        } => {
            // Reject if writing to stdin source
            if path.as_os_str() == "-" {
                return Err(anyhow::anyhow!(
                    "Cannot batch: mindmap was loaded from stdin ('-'); use --file <path> to save changes"
                ));
            }

            // Compute base file hash before starting
            let base_content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read base file {}", path.display()))?;
            let base_hash = blake3_hash(base_content.as_bytes());

            // Read batch input
            let mut buf = String::new();
            match input {
                Some(p) if p.as_os_str() == "-" => {
                    std::io::stdin().read_to_string(&mut buf)?;
                }
                Some(p) => {
                    buf = std::fs::read_to_string(p)?;
                }
                None => {
                    std::io::stdin().read_to_string(&mut buf)?;
                }
            }

            // Parse ops
            let mut ops: Vec<BatchOp> = Vec::new();
            if format == "json" {
                // Parse JSON array of op objects
                let arr = serde_json::from_str::<Vec<serde_json::Value>>(&buf)?;
                for (i, val) in arr.iter().enumerate() {
                    match parse_batch_op_json(val) {
                        Ok(op) => ops.push(op),
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to parse batch op {}: {}", i, e));
                        }
                    }
                }
            } else {
                // Parse lines format (space-separated, respecting double-quotes)
                for (i, line) in buf.lines().enumerate() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    match parse_batch_op_line(line) {
                        Ok(op) => ops.push(op),
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Failed to parse batch line {}: {}",
                                i + 1,
                                e
                            ));
                        }
                    }
                }
            }

            // Clone mm and work on clone (do not persist until all ops succeed)
            let mut mm_clone = Mindmap::from_string(base_content.clone(), path.clone())?;

            // Replay ops
            let mut result = BatchResult {
                total_ops: ops.len(),
                applied: 0,
                added_ids: Vec::new(),
                patched_ids: Vec::new(),
                deleted_ids: Vec::new(),
                warnings: Vec::new(),
            };

            for (i, op) in ops.iter().enumerate() {
                match op {
                    BatchOp::Add {
                        type_prefix,
                        title,
                        desc,
                    } => match cmd_add(&mut mm_clone, type_prefix, title, desc) {
                        Ok(id) => {
                            result.added_ids.push(id);
                            result.applied += 1;
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Op {}: add failed: {}", i, e));
                        }
                    },
                    BatchOp::Patch {
                        id,
                        type_prefix,
                        title,
                        desc,
                    } => {
                        match cmd_patch(
                            &mut mm_clone,
                            *id,
                            type_prefix.as_deref(),
                            title.as_deref(),
                            desc.as_deref(),
                            false,
                        ) {
                            Ok(_) => {
                                result.patched_ids.push(*id);
                                result.applied += 1;
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Op {}: patch failed: {}", i, e));
                            }
                        }
                    }
                    BatchOp::Put { id, line } => match cmd_put(&mut mm_clone, *id, line, false) {
                        Ok(_) => {
                            result.patched_ids.push(*id);
                            result.applied += 1;
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Op {}: put failed: {}", i, e));
                        }
                    },
                    BatchOp::Delete { id, force } => match cmd_delete(&mut mm_clone, *id, *force) {
                        Ok(_) => {
                            result.deleted_ids.push(*id);
                            result.applied += 1;
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Op {}: delete failed: {}", i, e));
                        }
                    },
                    BatchOp::Deprecate { id, to } => match cmd_deprecate(&mut mm_clone, *id, *to) {
                        Ok(_) => {
                            result.patched_ids.push(*id);
                            result.applied += 1;
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Op {}: deprecate failed: {}", i, e));
                        }
                    },
                    BatchOp::Verify { id } => match cmd_verify(&mut mm_clone, *id) {
                        Ok(_) => {
                            result.patched_ids.push(*id);
                            result.applied += 1;
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Op {}: verify failed: {}", i, e));
                        }
                    },
                }
            }

            // Apply auto-fixes if requested
            if fix {
                match mm_clone.apply_fixes() {
                    Ok(report) => {
                        if !report.spacing.is_empty() {
                            result.warnings.push(format!(
                                "Auto-fixed: inserted {} spacing lines",
                                report.spacing.len()
                            ));
                        }
                        for tf in &report.title_fixes {
                            result.warnings.push(format!(
                                "Auto-fixed title for node {}: '{}' -> '{}'",
                                tf.id, tf.old, tf.new
                            ));
                        }
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to apply fixes: {}", e));
                    }
                }
            }

            // Run lint and collect warnings (non-blocking)
            match cmd_lint(&mm_clone) {
                Ok(warnings) => {
                    result.warnings.extend(warnings);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Lint check failed: {}", e));
                }
            }

            if dry_run {
                // Print what would be written
                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({
                        "command": "batch",
                        "dry_run": true,
                        "result": result,
                        "content": mm_clone.lines.join("\n") + "\n"
                    });
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    eprintln!("--- DRY RUN: No changes written ---");
                    eprintln!(
                        "Would apply {} operations: {} added, {} patched, {} deleted",
                        result.applied,
                        result.added_ids.len(),
                        result.patched_ids.len(),
                        result.deleted_ids.len()
                    );
                    if !result.warnings.is_empty() {
                        eprintln!("Warnings:");
                        for w in &result.warnings {
                            eprintln!("  {}", w);
                        }
                    }
                    println!("{}", mm_clone.lines.join("\n"));
                }
            } else {
                // Check file hash again before writing (concurrency guard)
                let current_content = fs::read_to_string(&path).with_context(|| {
                    format!("Failed to re-read file before commit {}", path.display())
                })?;
                let current_hash = blake3_hash(current_content.as_bytes());

                if current_hash != base_hash {
                    return Err(anyhow::anyhow!(
                        "Cannot commit batch: target file changed since batch began (hash mismatch).\n\
                         Base hash: {}\n\
                         Current hash: {}\n\
                         The file was likely modified by another process. \
                         Re-run begin your batch on the current file.",
                        base_hash,
                        current_hash
                    ));
                }

                // Persist changes atomically
                mm_clone.save()?;

                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({
                        "command": "batch",
                        "dry_run": false,
                        "result": result
                    });
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    eprintln!("Batch applied successfully: {} ops applied", result.applied);
                    if !result.added_ids.is_empty() {
                        eprintln!("  Added nodes: {:?}", result.added_ids);
                    }
                    if !result.patched_ids.is_empty() {
                        eprintln!("  Patched nodes: {:?}", result.patched_ids);
                    }
                    if !result.deleted_ids.is_empty() {
                        eprintln!("  Deleted nodes: {:?}", result.deleted_ids);
                    }
                    if !result.warnings.is_empty() {
                        eprintln!("Warnings:");
                        for w in &result.warnings {
                            eprintln!("  {}", w);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct FixReport {
    pub spacing: Vec<usize>,
    pub title_fixes: Vec<TitleFix>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TitleFix {
    pub id: u32,
    pub old: String,
    pub new: String,
}

impl FixReport {
    pub fn any_changes(&self) -> bool {
        !self.spacing.is_empty() || !self.title_fixes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_parse_nodes() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str(
            "Header line\n[1] **AE: A** - refers to [2]\nSome note\n[2] **AE: B** - base\n",
        )?;

        let mm = Mindmap::load(file.path().to_path_buf())?;
        assert_eq!(mm.nodes.len(), 2);
        assert!(mm.by_id.contains_key(&1));
        assert!(mm.by_id.contains_key(&2));
        let n1 = mm.get_node(1).unwrap();
        assert_eq!(n1.references, vec![Reference::Internal(2)]);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_save_atomic() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - base\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        // append a node line
        let id = mm.next_id();
        mm.lines.push(format!("[{}] **AE: C** - new\n", id));
        // reflect node
        let node = Node {
            id,
            raw_title: "AE: C".to_string(),
            description: "new".to_string(),
            references: vec![],
            line_index: mm.lines.len() - 1,
        };
        mm.by_id.insert(id, mm.nodes.len());
        mm.nodes.push(node);

        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert!(content.contains("AE: C"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_syntax_and_duplicates_and_orphan() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[bad] not a node\n[1] **AE: A** - base\n[1] **AE: Adup** - dup\n[2] **AE: Orphan** - lonely\n")?;

        let mm = Mindmap::load(file.path().to_path_buf())?;
        let warnings = cmd_lint(&mm)?;
        // Expect at least syntax and duplicate warnings from lint
        let joined = warnings.join("\n");
        assert!(joined.contains("Syntax"));
        assert!(joined.contains("Duplicate ID"));

        // Orphan detection is now a separate command; verify orphans via cmd_orphans()
        let orphans = cmd_orphans(&mm, false)?;
        let joined_o = orphans.join("\n");
        // expect node id 2 to be reported as orphan
        assert!(joined_o.contains("2"));

        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_put_and_patch_basic() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - second\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        // patch title only for node 1
        cmd_patch(&mut mm, 1, Some("AE"), Some("OneNew"), None, false)?;
        assert_eq!(mm.get_node(1).unwrap().raw_title, "AE: OneNew");

        // put full line for node 2
        let new_line = "[2] **DR: Replaced** - replaced desc [1]";
        cmd_put(&mut mm, 2, new_line, false)?;
        assert_eq!(mm.get_node(2).unwrap().raw_title, "DR: Replaced");
        assert_eq!(
            mm.get_node(2).unwrap().references,
            vec![Reference::Internal(1)]
        );

        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_show() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let out = cmd_show(&mm, 1);
        assert!(out.contains("[1] **AE: One**"));
        assert!(out.contains("Referred to by: [2]"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_refs() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let refs = cmd_refs(&mm, 1);
        assert_eq!(refs.len(), 1);
        assert!(refs[0].contains("[2] **AE: Two**"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_links() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let links = cmd_links(&mm, 2);
        assert_eq!(links, Some(vec![Reference::Internal(1)]));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_search() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - second\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        // Search now delegates to list --grep
        let results = cmd_list(&mm, None, Some("first"), false, false, false);
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("[1] **AE: One**"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_search_list_grep_equivalence() -> Result<()> {
        // Verify that search (via cmd_list) produces identical output to list --grep
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first node\n[2] **WF: Two** - second node\n[3] **DR: Three** - third\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;

        // Both should produce the same output
        let search_results = cmd_list(&mm, None, Some("node"), false, false, false);
        let list_grep_results = cmd_list(&mm, None, Some("node"), false, false, false);
        assert_eq!(search_results, list_grep_results);
        assert_eq!(search_results.len(), 2);

        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_add() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let id = cmd_add(&mut mm, "AE", "Two", "second")?;
        assert_eq!(id, 2);
        assert_eq!(mm.nodes.len(), 2);
        let node = mm.get_node(2).unwrap();
        assert_eq!(node.raw_title, "AE: Two");
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_deprecate() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - second\n")?;
        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        cmd_deprecate(&mut mm, 1, 2)?;
        let node = mm.get_node(1).unwrap();
        assert!(node.raw_title.starts_with("[DEPRECATED → 2]"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_verify() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        cmd_verify(&mut mm, 1)?;
        let node = mm.get_node(1).unwrap();
        assert!(node.description.contains("(verify"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_show_non_existing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let out = cmd_show(&mm, 99);
        assert_eq!(out, "Node [99] not found");
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_refs_non_existing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let refs = cmd_refs(&mm, 99);
        assert_eq!(refs.len(), 0);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_links_non_existing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let links = cmd_links(&mm, 99);
        assert_eq!(links, None);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_put_non_existing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let err = cmd_put(&mut mm, 99, "[99] **AE: New** - new", false).unwrap_err();
        assert!(format!("{}", err).contains("Node [99] not found"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_patch_non_existing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let err = cmd_patch(&mut mm, 99, None, Some("New"), None, false).unwrap_err();
        assert!(format!("{}", err).contains("Node [99] not found"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_load_from_reader() -> Result<()> {
        use std::io::Cursor;
        let content = "[1] **AE: One** - first\n";
        let reader = Cursor::new(content);
        let path = PathBuf::from("-");
        let mm = Mindmap::load_from_reader(reader, path)?;
        assert_eq!(mm.nodes.len(), 1);
        assert_eq!(mm.nodes[0].id, 1);
        Ok(())
    }

    #[test]
    fn test_next_id() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[3] **AE: Three** - third\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        assert_eq!(mm.next_id(), 4);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_get_node() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let node = mm.get_node(1).unwrap();
        assert_eq!(node.id, 1);
        assert!(mm.get_node(99).is_none());
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_orphans() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Orphan** - lonely\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let orphans = cmd_orphans(&mm, false)?;
        assert_eq!(orphans, vec!["1".to_string(), "2".to_string()]);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_graph() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str(
            "[1] **AE: One** - first\n[2] **AE: Two** - refers [1]\n[3] **AE: Three** - also [1]\n",
        )?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let dot = cmd_graph(&mm, 1)?;
        assert!(dot.contains("digraph {"));
        assert!(dot.contains("1 [label=\"1: AE: One\"]"));
        assert!(dot.contains("2 [label=\"2: AE: Two\"]"));
        assert!(dot.contains("3 [label=\"3: AE: Three\"]"));
        assert!(dot.contains("2 -> 1;"));
        assert!(dot.contains("3 -> 1;"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_save_stdin_path() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mut mm = Mindmap::load_from_reader(
            std::io::Cursor::new("[1] **AE: One** - first\n"),
            PathBuf::from("-"),
        )?;
        let err = mm.save().unwrap_err();
        assert!(format!("{}", err).contains("Cannot save"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_extract_refs_from_str() {
        assert_eq!(
            extract_refs_from_str("no refs", None),
            vec![] as Vec<Reference>
        );
        assert_eq!(
            extract_refs_from_str("[1] and [2]", None),
            vec![Reference::Internal(1), Reference::Internal(2)]
        );
        assert_eq!(
            extract_refs_from_str("[1] and [1]", Some(1)),
            vec![] as Vec<Reference>
        ); // skip self
        assert_eq!(
            extract_refs_from_str("[abc] invalid [123]", None),
            vec![Reference::Internal(123)]
        );
        assert_eq!(
            extract_refs_from_str("[234](./file.md)", None),
            vec![Reference::External(234, "./file.md".to_string())]
        );
    }

    #[test]
    fn test_normalize_adjacent_nodes() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert_eq!(content, "[1] **AE: A** - a\n\n[2] **AE: B** - b\n");
        // line indices: node 1 at 0, blank at 1, node 2 at 2
        assert_eq!(mm.get_node(2).unwrap().line_index, 2);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_normalize_idempotent() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        mm.normalize_spacing()?;
        let snapshot = mm.lines.clone();
        mm.normalize_spacing()?;
        assert_eq!(mm.lines, snapshot);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_preserve_non_node_lines() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\nHeader line\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        // Should remain unchanged apart from ensuring trailing newline
        assert_eq!(
            content,
            "[1] **AE: A** - a\nHeader line\n[2] **AE: B** - b\n"
        );
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_fix_spacing() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let report = mm.apply_fixes()?;
        assert!(!report.spacing.is_empty());
        assert_eq!(report.title_fixes.len(), 0);
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert_eq!(content, "[1] **AE: A** - a\n\n[2] **AE: B** - b\n");
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_fix_duplicated_type() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: AE: Auth** - desc\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let report = mm.apply_fixes()?;
        assert_eq!(report.title_fixes.len(), 1);
        assert_eq!(report.title_fixes[0].new, "AE: Auth");
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert!(content.contains("[1] **AE: Auth** - desc"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_fix_combined() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **WF: WF: Workflow** - first\n[2] **AE: Auth** - second\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let report = mm.apply_fixes()?;
        assert!(!report.spacing.is_empty());
        assert_eq!(report.title_fixes.len(), 1);
        assert_eq!(report.title_fixes[0].id, 1);
        assert_eq!(report.title_fixes[0].new, "WF: Workflow");
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert!(content.contains("[1] **WF: Workflow** - first"));
        assert!(content.contains("\n\n[2] **AE: Auth** - second"));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_fix_idempotent() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: AE: A** - a\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let report1 = mm.apply_fixes()?;
        assert!(report1.any_changes());

        // Apply again; should have no changes
        let report2 = mm.apply_fixes()?;
        assert!(!report2.any_changes());
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_lint_fix_collapse_multiple_blanks() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\n\n\n[2] **AE: B** - b\n")?;

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        let report = mm.apply_fixes()?;
        assert!(!report.spacing.is_empty());
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        // Should have exactly one blank line between nodes
        assert_eq!(content, "[1] **AE: A** - a\n\n[2] **AE: B** - b\n");
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_batch_op_parse_line_add() -> Result<()> {
        let line = "add --type WF --title Test --desc desc";
        let op = parse_batch_op_line(line)?;
        match op {
            BatchOp::Add {
                type_prefix,
                title,
                desc,
            } => {
                assert_eq!(type_prefix, "WF");
                assert_eq!(title, "Test");
                assert_eq!(desc, "desc");
            }
            _ => panic!("Expected Add op"),
        }
        Ok(())
    }

    #[test]
    fn test_batch_op_parse_line_patch() -> Result<()> {
        let line = "patch 1 --title NewTitle";
        let op = parse_batch_op_line(line)?;
        match op {
            BatchOp::Patch {
                id,
                title,
                type_prefix,
                desc,
            } => {
                assert_eq!(id, 1);
                assert_eq!(title, Some("NewTitle".to_string()));
                assert_eq!(type_prefix, None);
                assert_eq!(desc, None);
            }
            _ => panic!("Expected Patch op"),
        }
        Ok(())
    }

    #[test]
    fn test_batch_op_parse_line_delete() -> Result<()> {
        let line = "delete 5 --force";
        let op = parse_batch_op_line(line)?;
        match op {
            BatchOp::Delete { id, force } => {
                assert_eq!(id, 5);
                assert!(force);
            }
            _ => panic!("Expected Delete op"),
        }
        Ok(())
    }

    #[test]
    fn test_batch_hash_concurrency_check() -> Result<()> {
        // Verify blake3_hash function works
        let content1 = "hello world";
        let content2 = "hello world";
        let content3 = "hello world!";

        let hash1 = blake3_hash(content1.as_bytes());
        let hash2 = blake3_hash(content2.as_bytes());
        let hash3 = blake3_hash(content3.as_bytes());

        assert_eq!(hash1, hash2); // identical content = same hash
        assert_ne!(hash1, hash3); // different content = different hash
        Ok(())
    }

    #[test]
    fn test_batch_simple_add() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: A** - a\n")?;

        // Simulate batch with one add operation (use quotes for multi-word args)
        let batch_input = r#"add --type WF --title Work --desc "do work""#;
        let ops = vec![parse_batch_op_line(batch_input)?];

        let mut mm = Mindmap::load(file.path().to_path_buf())?;
        for op in ops {
            match op {
                BatchOp::Add {
                    type_prefix,
                    title,
                    desc,
                } => {
                    cmd_add(&mut mm, &type_prefix, &title, &desc)?;
                }
                _ => {}
            }
        }
        mm.save()?;

        let content = std::fs::read_to_string(file.path())?;
        assert!(content.contains("WF: Work") && content.contains("do work"));
        temp.close()?;
        Ok(())
    }
}
