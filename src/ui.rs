use anyhow::Result;
use pretty_console::Console;
use std::io::{self, Write};

use mindmap_cli;

pub trait Printer {
    fn show(&self, node: &mindmap_cli::Node) -> Result<()>;
    fn list(&self, lines: &[String]) -> Result<()>;
    fn refs(&self, lines: &[String]) -> Result<()>;
    fn links(&self, id: u32, links: &[u32]) -> Result<()>;
    fn search(&self, lines: &[String]) -> Result<()>;
    fn orphans(&self, orphans: &[String]) -> Result<()>;
}

pub struct PrettyPrinter {}

impl PrettyPrinter {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Printer for PrettyPrinter {
    fn show(&self, node: &mindmap_cli::Node) -> Result<()> {
        let heading = format!(
            "[{}] **{}** - {}",
            node.id, node.raw_title, node.description
        );
        // Use bold for heading
        Console::new(&heading).bold().println();
        Ok(())
    }

    fn list(&self, lines: &[String]) -> Result<()> {
        let mut i = 0usize;
        for line in lines {
            i += 1;
            Console::new(line).println();
        }
        Ok(())
    }

    fn refs(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            Console::new(line).println();
        }
        Ok(())
    }

    fn links(&self, id: u32, links: &[u32]) -> Result<()> {
        let s = format!("Node [{}] references: {:?}", id, links);
        Console::new(&s).println();
        Ok(())
    }

    fn search(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            Console::new(line).println();
        }
        Ok(())
    }

    fn orphans(&self, orphans: &[String]) -> Result<()> {
        if orphans.is_empty() {
            Console::new("No orphans").green().println();
        } else {
            Console::new("Orphans:").yellow().bold().println();
            for o in orphans {
                Console::new(o).println();
            }
        }
        Ok(())
    }
}

pub struct PlainPrinter {}

impl PlainPrinter {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Printer for PlainPrinter {
    fn show(&self, node: &mindmap_cli::Node) -> Result<()> {
        println!(
            "[{}] **{}** - {}",
            node.id, node.raw_title, node.description
        );
        Ok(())
    }

    fn list(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            println!("{}", line);
        }
        Ok(())
    }

    fn refs(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            println!("{}", line);
        }
        Ok(())
    }

    fn links(&self, id: u32, links: &[u32]) -> Result<()> {
        println!("Node [{}] references: {:?}", id, links);
        Ok(())
    }

    fn search(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            println!("{}", line);
        }
        Ok(())
    }

    fn orphans(&self, orphans: &[String]) -> Result<()> {
        if orphans.is_empty() {
            println!("No orphans");
        } else {
            println!("Orphans:");
            for o in orphans {
                println!("{}", o);
            }
        }
        Ok(())
    }
}
