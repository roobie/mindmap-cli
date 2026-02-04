use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod ui;

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
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
struct Cli {
    /// Path to MINDMAP file (defaults to ./MINDMAP.md)
    #[arg(global = true, short, long)]
    file: Option<PathBuf>,

    /// Output format: default (human) or json
    #[arg(global = true, long, value_enum, default_value_t = OutputFormat::Default)]
    output: OutputFormat,

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

    /// Show orphan nodes (no in & no out, excluding META)
    Orphans,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let path = cli.file.unwrap_or_else(|| PathBuf::from("MINDMAP.md"));

    // If user passed '-' use stdin as source
    let mut mm = if path.as_os_str() == "-" {
        mindmap_cli::Mindmap::load_from_reader(std::io::stdin(), path.clone())?
    } else {
        mindmap_cli::Mindmap::load(path.clone())?
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
            Some(Box::new(ui::PrettyPrinter::new()?))
        } else {
            Some(Box::new(ui::PlainPrinter::new()?))
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
            let items = mindmap_cli::cmd_list(&mm, r#type.as_deref(), grep.as_deref());
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
            let items = mindmap_cli::cmd_refs(&mm, id);
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
        Commands::Links { id } => match mindmap_cli::cmd_links(&mm, id) {
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
            let items = mindmap_cli::cmd_search(&mm, &query);
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
                    let id = mindmap_cli::cmd_add(&mut mm, tp, tt, dd)?;
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
                    let id = mindmap_cli::cmd_add_editor(&mut mm, &editor, strict)?;
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
            mindmap_cli::cmd_deprecate(&mut mm, id, to)?;
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
            mindmap_cli::cmd_edit(&mut mm, id, &editor)?;
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
            mindmap_cli::cmd_patch(
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
            mindmap_cli::cmd_put(&mut mm, id, &line, strict)?;
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
            mindmap_cli::cmd_verify(&mut mm, id)?;
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
            mindmap_cli::cmd_delete(&mut mm, id, force)?;
            mm.save()?;
            if matches!(cli.output, OutputFormat::Json) {
                let obj = serde_json::json!({"command": "delete", "deleted": id});
                println!("{}", serde_json::to_string_pretty(&obj)?);
            }
            eprintln!("Deleted node [{}]", id);
        }
        Commands::Lint => {
            let res = mindmap_cli::cmd_lint(&mm)?;
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
            let res = mindmap_cli::cmd_orphans(&mm)?;
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
