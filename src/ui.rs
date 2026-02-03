use anyhow::Result;
use pretty_console::Console;

pub trait Printer {
    fn show(&self, node: &mindmap_cli::Node, inbound: &[u32], outbound: &[u32]) -> Result<()>;
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
    fn show(&self, node: &mindmap_cli::Node, inbound: &[u32], outbound: &[u32]) -> Result<()> {
        // ID in green (no newline)
        Console::new(format!("[{}] ", node.id)).green().print();
        // Title bold (uncolored) on same line
        Console::new(&node.raw_title).bold().println();

        // Description on new line
        Console::new(&node.description).println();

        // Incoming references in blue
        if !inbound.is_empty() {
            Console::new("Incoming:").blue().print();
            Console::new(format!(" {:?}", inbound)).blue().println();
        }

        // Outgoing references in magenta
        if !outbound.is_empty() {
            Console::new("Outgoing:").magenta().print();
            Console::new(format!(" {:?}", outbound))
                .magenta()
                .println();
        }

        Ok(())
    }

    fn list(&self, lines: &[String]) -> Result<()> {
        for line in lines {
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
        // Orphans are data for the orphans command — print to stdout
        if orphans.is_empty() {
            Console::new("No orphans").green().println();
        } else {
            Console::new("Orphans:").yellow().bold().println();
            for o in orphans {
                Console::new(format!("[{}]", o)).println();
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
    fn show(&self, node: &mindmap_cli::Node, inbound: &[u32], outbound: &[u32]) -> Result<()> {
        println!("[{}] {}", node.id, node.raw_title);
        println!("{}", node.description);
        if !inbound.is_empty() {
            println!("Incoming: {:?}", inbound);
        }
        if !outbound.is_empty() {
            println!("Outgoing: {:?}", outbound);
        }
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
