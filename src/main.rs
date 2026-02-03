use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Parser)]
#[command(name = "mindmap")]
#[command(about = "CLI tool for working with MINDMAP files", long_about = None)]
struct Cli {
    /// Path to MINDMAP file (defaults to ./MINDMAP.md)
    #[arg(global = true, short, long)]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        r#type: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        desc: String,
    },

    /// Deprecate a node, redirecting to another
    Deprecate {
        id: u32,
        #[arg(long)]
        to: u32,
    },

    /// Mark a node as needing verification (append verify tag)
    Verify { id: u32 },

    /// Edit a node with $EDITOR
    Edit { id: u32 },

    /// Lint the mindmap for basic issues
    Lint,
}

#[derive(Debug, Clone)]
struct Node {
    id: u32,
    raw_title: String,
    description: String,
    references: Vec<u32>,
    line_index: usize,
}

#[derive(Debug)]
struct Mindmap {
    path: PathBuf,
    lines: Vec<String>,
    nodes: Vec<Node>,
    by_id: HashMap<u32, usize>,
}

impl Mindmap {
    fn load(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.display()))?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let re = Regex::new(r#"^\[(\d+)\] \*\*(.+?)\*\* - (.*)$"#)?;
        let ref_re = Regex::new(r#"\[(\d+)\]"#)?;

        let mut nodes = Vec::new();
        let mut by_id = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(caps) = re.captures(line) {
                let id: u32 = caps[1].parse()?;
                let raw_title = caps[2].to_string();
                let description = caps[3].to_string();

                let mut references = Vec::new();
                for rcaps in ref_re.captures_iter(&description) {
                    if let Ok(rid) = rcaps[1].parse::<u32>() {
                        if rid != id {
                            references.push(rid);
                        }
                    }
                }

                let node = Node {
                    id,
                    raw_title,
                    description,
                    references,
                    line_index: i,
                };

                if by_id.contains_key(&id) {
                    eprintln!("Warning: duplicate node id {} at line {}", id, i + 1);
                }
                by_id.insert(id, nodes.len());
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

    fn save(&self) -> Result<()> {
        // atomic write: write to a temp file in the same dir then rename
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

    fn next_id(&self) -> u32 {
        self.by_id.keys().max().copied().unwrap_or(0) + 1
    }

    fn get_node(&self, id: u32) -> Option<&Node> {
        self.by_id.get(&id).map(|&idx| &self.nodes[idx])
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let path = cli
        .file
        .unwrap_or_else(|| PathBuf::from("MINDMAP.md"));

    let mut mm = Mindmap::load(path)?;

    match cli.command {
        Commands::Show { id } => cmd_show(&mm, id),
        Commands::List { r#type, grep } => cmd_list(&mm, r#type, grep),
        Commands::Refs { id } => cmd_refs(&mm, id),
        Commands::Links { id } => cmd_links(&mm, id),
        Commands::Search { query } => cmd_search(&mm, query),
        Commands::Add { r#type, title, desc } => {
            cmd_add(&mut mm, &r#type, &title, &desc)?;
            mm.save()?;
        }
        Commands::Deprecate { id, to } => {
            cmd_deprecate(&mut mm, id, to)?;
            mm.save()?;
        }
        Commands::Verify { id } => {
            cmd_verify(&mut mm, id)?;
            mm.save()?;
        }
            Commands::Edit { id } => {
            cmd_edit(&mut mm, id)?;
            mm.save()?;
        }
        Commands::Lint => cmd_lint(&mm)?,
    }

    Ok(())
}

fn cmd_show(mm: &Mindmap, id: u32) {
    if let Some(node) = mm.get_node(id) {
        println!("[{}] **{}** - {}", node.id, node.raw_title, node.description);

        // inbound refs
        let mut inbound = Vec::new();
        for n in &mm.nodes {
            if n.references.contains(&id) {
                inbound.push(n.id);
            }
        }
        if !inbound.is_empty() {
            println!("\nReferred to by: {:?}", inbound);
        }
    } else {
        eprintln!("Node {} not found", id);
    }
}

fn cmd_list(mm: &Mindmap, type_filter: Option<String>, grep: Option<String>) {
    for n in &mm.nodes {
        if let Some(ref tf) = type_filter {
            if !n.raw_title.starts_with(&format!("{}:", tf)) {
                continue;
            }
        }
        if let Some(ref q) = grep {
            let qlc = q.to_lowercase();
            if !n.raw_title.to_lowercase().contains(&qlc)
                && !n.description.to_lowercase().contains(&qlc)
            {
                continue;
            }
        }
        println!("[{}] **{}** - {}", n.id, n.raw_title, n.description);
    }
}

fn cmd_refs(mm: &Mindmap, id: u32) {
    let mut found = false;
    for n in &mm.nodes {
        if n.references.contains(&id) {
            println!("[{}] **{}** - {}", n.id, n.raw_title, n.description);
            found = true;
        }
    }
    if !found {
        println!("No nodes reference {}", id);
    }
}

fn cmd_links(mm: &Mindmap, id: u32) {
    if let Some(n) = mm.get_node(id) {
        if n.references.is_empty() {
            println!("Node {} has no outgoing references", id);
        } else {
            println!("Node {} references: {:?}", id, n.references);
        }
    } else {
        eprintln!("Node {} not found", id);
    }
}

fn cmd_search(mm: &Mindmap, query: String) {
    let qlc = query.to_lowercase();
    for n in &mm.nodes {
        if n.raw_title.to_lowercase().contains(&qlc)
            || n.description.to_lowercase().contains(&qlc)
        {
            println!("[{}] **{}** - {}", n.id, n.raw_title, n.description);
        }
    }
}

fn cmd_add(mm: &mut Mindmap, type_prefix: &str, title: &str, desc: &str) -> Result<()> {
    let id = mm.next_id();
    let full_title = format!("{}: {}", type_prefix, title);
    let line = format!("[{}] **{}** - {}", id, full_title, desc);

    mm.lines.push(line.clone());

    let line_index = mm.lines.len() - 1;
    let refs_re = Regex::new(r#"\[(\d+)\]"#)?;
    let mut references = Vec::new();
    for rcaps in refs_re.captures_iter(desc) {
        if let Ok(rid) = rcaps[1].parse::<u32>() {
            if rid != id {
                references.push(rid);
            }
        }
    }

    let node = Node {
        id,
        raw_title: full_title,
        description: desc.to_string(),
        references,
        line_index,
    };
    mm.by_id.insert(id, mm.nodes.len());
    mm.nodes.push(node);

    println!("Added node [{}]", id);
    Ok(())
}

fn cmd_deprecate(mm: &mut Mindmap, id: u32, to: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id))?;
    let node = &mut mm.nodes[idx];

    if !mm.by_id.contains_key(&to) {
        eprintln!(
            "Warning: target node {} does not exist (still updating title)",
            to
        );
    }

    if !node.raw_title.starts_with("[DEPRECATED") {
        node.raw_title = format!("[DEPRECATED → {}] {}", to, node.raw_title);
        mm.lines[node.line_index] =
            format!("[{}] **{}** - {}", node.id, node.raw_title, node.description);
    }

    println!("Deprecated node {} → {}", id, to);
    Ok(())
}

fn cmd_verify(mm: &mut Mindmap, id: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id))?;
    let node = &mut mm.nodes[idx];

    let tag = format!("(verify {})", chrono::Local::now().format("%Y-%m-%d"));
    if !node.description.contains("(verify ") {
        if node.description.is_empty() {
            node.description = tag.clone();
        } else {
            node.description = format!("{} {}", node.description, tag);
        }
        mm.lines[node.line_index] =
            format!("[{}] **{}** - {}", node.id, node.raw_title, node.description);
        println!("Marked node {} for verification", id);
    } else {
        println!("Node {} already has a verify tag", id);
    }
    Ok(())
}

fn cmd_edit(mm: &mut Mindmap, id: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!("Node {} not found", id))?;
    let node = &mm.nodes[idx];

    // create temp file with the single node line
    let mut tmp = tempfile::NamedTempFile::new()
        .with_context(|| "Failed to create temp file for editing")?;
    use std::io::Write;
    writeln!(tmp, "[{}] **{}** - {}", node.id, node.raw_title, node.description)?;
    tmp.flush()?;

    // launch editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
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

    // validate: must match node regex and keep same id
    let re = Regex::new(r#"^\[(\d+)\] \*\*(.+?)\*\* - (.*)$"#)?;
    let caps = re
        .captures(edited_line)
        .ok_or_else(|| anyhow::anyhow!("Edited line does not match node format"))?;
    let new_id: u32 = caps[1].parse()?;
    if new_id != id {
        return Err(anyhow::anyhow!("Cannot change node ID"));
    }

    // all good: replace line in mm.lines and update node fields
    mm.lines[node.line_index] = edited_line.to_string();
    let new_title = caps[2].to_string();
    let new_desc = caps[3].to_string();
    let mut new_refs = Vec::new();
    let ref_re = Regex::new(r#"\[(\d+)\]"#)?;
    for rcaps in ref_re.captures_iter(&new_desc) {
        if let Ok(rid) = rcaps[1].parse::<u32>() {
            if rid != id {
                new_refs.push(rid);
            }
        }
    }

    // update node in-place
    let node_mut = &mut mm.nodes[idx];
    node_mut.raw_title = new_title;
    node_mut.description = new_desc;
    node_mut.references = new_refs;

    println!("Edited node {}", id);
    Ok(())
}

fn cmd_lint(mm: &Mindmap) -> Result<()> {
    let mut ok = true;

    // Check for missing references
    for n in &mm.nodes {
        for rid in &n.references {
            if !mm.by_id.contains_key(rid) {
                eprintln!("Warning: node {} references missing node {}", n.id, rid);
                ok = false;
            }
        }
    }

    if ok {
        println!("Lint OK");
    } else {
        println!("Lint completed with warnings");
    }

    Ok(())
}
