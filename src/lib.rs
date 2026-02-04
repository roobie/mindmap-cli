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
#[command(name = "mindmap")]
#[command(about = "CLI tool for working with MINDMAP files")]
#[command(
    long_about = r#"mindmap-cli — small CLI for inspecting and safely editing one-line MINDMAP files (default: ./MINDMAP.md).
One-node-per-line format: [N] **Title** - description with [N] references. IDs must be stable numeric values.

EXAMPLES:
  mindmap show 10
  mindmap list --type AE --grep auth
  mindmap add --type AE --title "AuthService" --desc "Handles auth [12]"
  mindmap edit 12               # opens $EDITOR for an atomic, validated edit
  mindmap patch 12 --title "AuthSvc" --desc "Updated desc"   # partial update (PATCH)
  mindmap put 12 --line "[31] **WF: Example** - Full line text [12]"   # full-line replace (PUT)
  mindmap lint

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
    /// Show a node by ID
    Show { id: u32 },

    /// List nodes (optionally filtered)
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        grep: Option<String>,
    },

    /// Show nodes that reference the given ID
    Refs { id: u32 },

    /// Show nodes that the given ID references
    Links { id: u32 },

    /// Search nodes by substring
    Search { query: String },

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

    /// Lint the mindmap for basic issues
    Lint,

    /// Show graph neighborhood for a node (DOT format for Graphviz)
    Graph { id: u32 },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub raw_title: String,
    pub description: String,
    pub references: Vec<u32>,
    pub line_index: usize,
}

#[derive(Debug)]
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

    pub fn save(&self) -> Result<()> {
        // prevent persisting when loaded from stdin (path == "-")
        if self.path.as_os_str() == "-" {
            return Err(anyhow::anyhow!(
                "Cannot save: mindmap was loaded from stdin ('-'); use --file <path> to save changes"
            ));
        }

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

// Extract references of the form [123] from a description string.
// If skip_self is Some(id) then occurrences equal to that id are ignored.
fn extract_refs_from_str(s: &str, skip_self: Option<u32>) -> Vec<u32> {
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
                    refs.push(rid);
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
            if n.references.contains(&id) {
                inbound.push(n.id);
            }
        }
        if !inbound.is_empty() {
            out.push_str(&format!("\nReferred to by: {:?}", inbound));
        }
        out
    } else {
        format!("Node {} not found", id)
    }
}

pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    let mut res = Vec::new();
    for n in &mm.nodes {
        if let Some(tf) = type_filter
            && !n.raw_title.starts_with(&format!("{}:", tf))
        {
            continue;
        }
        if let Some(q) = grep {
            let qlc = q.to_lowercase();
            if !n.raw_title.to_lowercase().contains(&qlc)
                && !n.description.to_lowercase().contains(&qlc)
            {
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
        if n.references.contains(&id) {
            out.push(format!(
                "[{}] **{}** - {}",
                n.id, n.raw_title, n.description
            ));
        }
    }
    out
}

pub fn cmd_links(mm: &Mindmap, id: u32) -> Option<Vec<u32>> {
    mm.get_node(id).map(|n| n.references.clone())
}

pub fn cmd_search(mm: &Mindmap, query: &str) -> Vec<String> {
    let qlc = query.to_lowercase();
    let mut out = Vec::new();
    for n in &mm.nodes {
        if n.raw_title.to_lowercase().contains(&qlc) || n.description.to_lowercase().contains(&qlc)
        {
            out.push(format!(
                "[{}] **{}** - {}",
                n.id, n.raw_title, n.description
            ));
        }
    }
    out
}

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
        for rid in &parsed.references {
            if !mm.by_id.contains_key(rid) {
                return Err(anyhow::anyhow!(format!(
                    "ADD strict: reference to missing node {}",
                    rid
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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;

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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;
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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;
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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;

    let parsed = parse_node_line(line, mm.nodes[idx].line_index)?;
    if parsed.id != id {
        return Err(anyhow::anyhow!("PUT line id does not match target id"));
    }

    // strict check for references
    if strict {
        for rid in &parsed.references {
            if !mm.by_id.contains_key(rid) {
                return Err(anyhow::anyhow!(format!(
                    "PUT strict: reference to missing node {}",
                    rid
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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;
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
        for rid in &parsed.references {
            if !mm.by_id.contains_key(rid) {
                return Err(anyhow::anyhow!(format!(
                    "PATCH strict: reference to missing node {}",
                    rid
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
        .ok_or_else(|| anyhow::anyhow!(format!("Node {} not found", id)))?;

    // check incoming references
    let mut incoming_from = Vec::new();
    for n in &mm.nodes {
        if n.references.contains(&id) {
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
        for rid in &n.references {
            if !mm.by_id.contains_key(rid) {
                warnings.push(format!(
                    "Missing ref: node {} references missing node {}",
                    n.id, rid
                ));
            }
        }
    }

    if warnings.is_empty() {
        Ok(vec!["Lint OK".to_string()])
    } else {
        Ok(warnings)
    }
}

pub fn cmd_orphans(mm: &Mindmap) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Orphans: nodes with no in and no out, excluding META:*
    let mut incoming: HashMap<u32, usize> = HashMap::new();
    for n in &mm.nodes {
        incoming.entry(n.id).or_insert(0);
    }
    for n in &mm.nodes {
        for rid in &n.references {
            if incoming.contains_key(rid) {
                *incoming.entry(*rid).or_insert(0) += 1;
            }
        }
    }
    for n in &mm.nodes {
        let inc = incoming.get(&n.id).copied().unwrap_or(0);
        let out = n.references.len();
        let title_up = n.raw_title.to_uppercase();
        if inc == 0 && out == 0 && !title_up.starts_with("META") {
            warnings.push(format!("{}", n.id));
        }
    }

    if warnings.is_empty() {
        Ok(vec!["No orphans".to_string()])
    } else {
        Ok(warnings)
    }
}

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
                        if n.references.contains(&id) {
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
                            eprintln!("Referred to by: {:?}", inbound);
                        }
                    }
                }
            }
            None => return Err(anyhow::anyhow!(format!("Node {} not found", id))),
        },
        Commands::List { r#type, grep } => {
            let items = cmd_list(&mm, r#type.as_deref(), grep.as_deref());
            if matches!(cli.output, OutputFormat::Json) {
                let arr: Vec<_> = items
                    .into_iter()
                    .map(|line| serde_json::json!({"line": line}))
                    .collect();
                let obj = serde_json::json!({"command": "list", "items": arr});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else if let Some(p) = &printer {
                p.list(&items)?;
            } else {
                for it in items {
                    println!("{}", it);
                }
            }
        }
        Commands::Refs { id } => {
            let items = cmd_refs(&mm, id);
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "refs", "items": items});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else if let Some(p) = &printer {
                p.refs(&items)?;
            } else {
                for it in items {
                    println!("{}", it);
                }
            }
        }
        Commands::Links { id } => match cmd_links(&mm, id) {
            Some(v) => {
                if matches!(cli.output, OutputFormat::Json) {
                    let obj = serde_json::json!({"command": "links", "id": id, "links": v});
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else if let Some(p) = &printer {
                    p.links(id, &v)?;
                } else {
                    println!("Node [{}] references: {:?}", id, v);
                }
            }
            None => return Err(anyhow::anyhow!(format!("Node [{}] not found", id))),
        },
        Commands::Search { query } => {
            let items = cmd_search(&mm, &query);
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "search", "query": query, "items": items});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else if let Some(p) = &printer {
                p.search(&items)?;
            } else {
                for it in items {
                    println!("{}", it);
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
        Commands::Lint => {
            let res = cmd_lint(&mm)?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "lint", "warnings": res});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                for r in res {
                    eprintln!("{}", r);
                }
            }
        }
        Commands::Orphans => {
            let res = cmd_orphans(&mm)?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "orphans", "orphans": res});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else if let Some(p) = &printer {
                p.orphans(&res)?;
            } else {
                for r in res {
                    eprintln!("{}", r);
                }
            }
        }
    }

    Ok(())
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
        assert_eq!(n1.references, vec![2]);
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
        let orphans = cmd_orphans(&mm)?;
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
        assert_eq!(mm.get_node(2).unwrap().references, vec![1]);

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
        assert_eq!(links, Some(vec![1]));
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_cmd_search() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n[2] **AE: Two** - second\n")?;
        let mm = Mindmap::load(file.path().to_path_buf())?;
        let results = cmd_search(&mm, "first");
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("[1] **AE: One**"));
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
        assert_eq!(out, "Node 99 not found");
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
        assert!(format!("{}", err).contains("Node 99 not found"));
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
        assert!(format!("{}", err).contains("Node 99 not found"));
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
        let orphans = cmd_orphans(&mm)?;
        assert_eq!(orphans, vec!["1".to_string(), "2".to_string()]);
        temp.close()?;
        Ok(())
    }

    #[test]
    fn test_save_stdin_path() -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let file = temp.child("MINDMAP.md");
        file.write_str("[1] **AE: One** - first\n")?;
        let mm = Mindmap::load_from_reader(
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
        assert_eq!(extract_refs_from_str("no refs", None), vec![] as Vec<u32>);
        assert_eq!(extract_refs_from_str("[1] and [2]", None), vec![1, 2]);
        assert_eq!(
            extract_refs_from_str("[1] and [1]", Some(1)),
            vec![] as Vec<u32>
        ); // skip self
        assert_eq!(
            extract_refs_from_str("[abc] invalid [123]", None),
            vec![123]
        );
    }
}
