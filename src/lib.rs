use anyhow::{Context, Result};
use regex::Regex;
use std::{collections::HashMap, fs, path::PathBuf};

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
                    eprintln!("Warning: duplicate node [{}] at line {}", id, i + 1);
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

    pub fn save(&self) -> Result<()> {
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
        format!("Node [{}] not found", id)
    }
}

pub fn cmd_list(mm: &Mindmap, type_filter: Option<&str>, grep: Option<&str>) -> Vec<String> {
    let mut res = Vec::new();
    for n in &mm.nodes {
        if let Some(tf) = type_filter {
            if !n.raw_title.starts_with(&format!("{}:", tf)) {
                continue;
            }
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

    Ok(id)
}

pub fn cmd_deprecate(mm: &mut Mindmap, id: u32, to: u32) -> Result<()> {
    let idx = *mm
        .by_id
        .get(&id)
        .ok_or_else(|| anyhow::anyhow!("Node [{}] not found", id))?;

    if !mm.by_id.contains_key(&to) {
        eprintln!(
            "Warning: target node [{}] does not exist (still updating title)",
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
        .ok_or_else(|| anyhow::anyhow!("Node [{}] not found", id))?;
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
        .ok_or_else(|| anyhow::anyhow!("Node [{}] not found", id))?;
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

    Ok(())
}

pub fn parse_node_line(line: &str, line_index: usize) -> Result<Node> {
    let re = Regex::new(r#"^\[(\d+)\] \*\*(.+?)\*\* - (.*)$"#)?;
    let ref_re = Regex::new(r#"\[(\d+)\]"#)?;
    let caps = re
        .captures(line)
        .ok_or_else(|| anyhow::anyhow!("Line does not match node format"))?;
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
    Ok(Node {
        id,
        raw_title,
        description,
        references,
        line_index,
    })
}

pub fn cmd_lint(mm: &Mindmap) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    let node_re = Regex::new(r#"^\[(\d+)\] \*\*(.+?)\*\* - (.*)$"#)?;

    // 1) Syntax: lines starting with '[' but not matching node regex
    for (i, line) in mm.lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            if !node_re.is_match(line) {
                warnings.push(format!(
                    "Syntax: line {} starts with '[' but does not match node format",
                    i + 1
                ));
            }
        }
    }

    // 2) Duplicate IDs: scan lines for node ids
    let mut id_map: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, line) in mm.lines.iter().enumerate() {
        if let Some(caps) = node_re.captures(line) {
            if let Ok(id) = caps[1].parse::<u32>() {
                id_map.entry(id).or_insert_with(Vec::new).push(i + 1);
            }
        }
    }
    for (id, locations) in &id_map {
        if locations.len() > 1 {
            warnings.push(format!(
                "Duplicate ID: node [{}] appears on lines {:?}",
                id, locations
            ));
        }
    }

    // 3) Missing references
    for n in &mm.nodes {
        for rid in &n.references {
            if !mm.by_id.contains_key(rid) {
                warnings.push(format!(
                    "Missing ref: node [{}] references missing node [{}]",
                    n.id, rid
                ));
            }
        }
    }

    // 4) Orphans: nodes with no in and no out, excluding META:*
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
            warnings.push(format!("Orphan: node [{}] appears to be orphan", n.id));
        }
    }

    if warnings.is_empty() {
        Ok(vec!["Lint OK".to_string()])
    } else {
        Ok(warnings)
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
        // Expect at least syntax, duplicate, and orphan warnings
        let joined = warnings.join("\n");
        assert!(joined.contains("Syntax"));
        assert!(joined.contains("Duplicate ID"));
        assert!(joined.contains("Orphan"));

        temp.close()?;
        Ok(())
    }
}
