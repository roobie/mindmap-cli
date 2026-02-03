use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    /// Lint the mindmap for basic issues
    Lint,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let path = cli.file.unwrap_or_else(|| PathBuf::from("MINDMAP.md"));

    let mut mm = mindmap_cli::Mindmap::load(path)?;

    match cli.command {
        Commands::Show { id } => println!("{}", mindmap_cli::cmd_show(&mm, id)),
        Commands::List { r#type, grep } => {
            let items = mindmap_cli::cmd_list(&mm, r#type.as_deref(), grep.as_deref());
            for it in items {
                println!("{}", it);
            }
        }
        Commands::Refs { id } => {
            for it in mindmap_cli::cmd_refs(&mm, id) {
                println!("{}", it);
            }
        }
        Commands::Links { id } => match mindmap_cli::cmd_links(&mm, id) {
            Some(v) => println!("Node [{}] references: {:?}", id, v),
            None => eprintln!("Node [{}] not found", id),
        },
        Commands::Search { query } => {
            for it in mindmap_cli::cmd_search(&mm, &query) {
                println!("{}", it);
            }
        }
        Commands::Add {
            r#type,
            title,
            desc,
        } => {
            let id = mindmap_cli::cmd_add(&mut mm, &r#type, &title, &desc)?;
            mm.save()?;
            println!("Added node [{}]", id);
        }
        Commands::Deprecate { id, to } => {
            mindmap_cli::cmd_deprecate(&mut mm, id, to)?;
            mm.save()?;
            println!("Deprecated node [{}] → [{}]", id, to);
        }
        Commands::Edit { id } => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            mindmap_cli::cmd_edit(&mut mm, id, &editor)?;
            mm.save()?;
            println!("Edited node [{}]", id);
        }
        Commands::Patch { id, r#type, title, desc, strict } => {
            mindmap_cli::cmd_patch(
                &mut mm,
                id,
                r#type.as_deref(),
                title.as_deref(),
                desc.as_deref(),
                strict,
            )?;
            mm.save()?;
            println!("Patched node [{}]", id);
        }
        Commands::Put { id, line, strict } => {
            mindmap_cli::cmd_put(&mut mm, id, &line, strict)?;
            mm.save()?;
            println!("Put node [{}]", id);
        }
        Commands::Verify { id } => {
            mindmap_cli::cmd_verify(&mut mm, id)?;
            mm.save()?;
            println!("Marked node [{}] for verification", id);
        }
        Commands::Lint => {
            let res = mindmap_cli::cmd_lint(&mm)?;
            for r in res {
                println!("{}", r);
            }
        }
    }

    Ok(())
}
