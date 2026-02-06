use anyhow::Result;
use pretty_console::Console;

pub trait Printer {
    fn show(
        &self,
        node: &crate::Node,
        inbound: &[u32],
        outbound: &[crate::Reference],
    ) -> Result<()>;
    fn list(&self, lines: &[String]) -> Result<()>;
    fn refs(&self, lines: &[String]) -> Result<()>;
    fn links(&self, id: u32, links: &[crate::Reference]) -> Result<()>;
    fn orphans(&self, orphans: &[String]) -> Result<()>;
}

pub struct PrettyPrinter {}

impl PrettyPrinter {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Printer for PrettyPrinter {
    fn show(
        &self,
        node: &crate::Node,
        inbound: &[u32],
        outbound: &[crate::Reference],
    ) -> Result<()> {
        // ID in green (no newline)
        Console::new(format!("[{}] ", node.id)).green().print();
        // Title bold (uncolored) on same line
        Console::new(&node.raw_title).bold().println();

        // Description on new line
        Console::new(&node.description).println();

        // Incoming references in blue
        if !inbound.is_empty() {
            Console::new("← Referring nodes:").blue().print();
            Console::new(format!(" {:?}", inbound)).blue().println();
        }

        // Outgoing references in magenta
        let mut outbound_ids = Vec::new();
        for r in outbound {
            if let crate::Reference::Internal(rid) = r {
                outbound_ids.push(*rid);
            }
        }
        if !outbound_ids.is_empty() {
            Console::new("→ References:").magenta().print();
            Console::new(format!(" {:?}", outbound_ids))
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

    fn links(&self, id: u32, links: &[crate::Reference]) -> Result<()> {
        let mut internal = Vec::new();
        let mut external = Vec::new();
        for r in links {
            match r {
                crate::Reference::Internal(rid) => internal.push(*rid),
                crate::Reference::External(eid, file) => {
                    external.push(format!("[{}] in {}", eid, file))
                }
            }
        }
        if !internal.is_empty() {
            Console::new(format!("→ [{}] refers to: {:?}", id, internal)).println();
        }
        if !external.is_empty() {
            Console::new(format!("[{}] external refs: {:?}", id, external)).println();
        }
        Ok(())
    }

    fn orphans(&self, orphans: &[String]) -> Result<()> {
        // Orphans are data for the orphans command — print to stdout
        for o in orphans {
            if o != "No orphans" {
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
    fn show(
        &self,
        node: &crate::Node,
        inbound: &[u32],
        outbound: &[crate::Reference],
    ) -> Result<()> {
        println!("[{}] {}", node.id, node.raw_title);
        println!("{}", node.description);
        if !inbound.is_empty() {
            println!("← Referring nodes: {:?}", inbound);
        }
        let mut outbound_ids = Vec::new();
        for r in outbound {
            if let crate::Reference::Internal(rid) = r {
                outbound_ids.push(*rid);
            }
        }
        if !outbound_ids.is_empty() {
            println!("→ References: {:?}", outbound_ids);
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

    fn links(&self, id: u32, links: &[crate::Reference]) -> Result<()> {
        let mut internal = Vec::new();
        let mut external = Vec::new();
        for r in links {
            match r {
                crate::Reference::Internal(rid) => internal.push(*rid),
                crate::Reference::External(eid, file) => {
                    external.push(format!("[{}] in {}", eid, file))
                }
            }
        }
        if !internal.is_empty() {
            println!("→ [{}] refers to: {:?}", id, internal);
        }
        if !external.is_empty() {
            println!("[{}] external refs: {:?}", id, external);
        }
        Ok(())
    }

    fn orphans(&self, orphans: &[String]) -> Result<()> {
        for o in orphans {
            if o != "No orphans" {
                println!("{}", o);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_printer_smoke() -> Result<()> {
        let p = PrettyPrinter::new()?;
        let node = crate::Node {
            id: 1,
            raw_title: "AE: Test".to_string(),
            description: "desc".to_string(),
            references: vec![crate::Reference::Internal(2)],
            line_index: 0,
        };
        p.show(&node, &vec![3], &node.references)?;
        p.list(&vec!["one".to_string(), "two".to_string()])?;
        p.refs(&vec!["ref".to_string()])?;
        p.links(1, &vec![crate::Reference::Internal(2)])?;
        p.orphans(&Vec::<String>::new())?;
        p.orphans(&vec!["4".to_string()])?;
        Ok(())
    }

    #[test]
    fn plain_printer_smoke() -> Result<()> {
        let p = PlainPrinter::new()?;
        let node = crate::Node {
            id: 1,
            raw_title: "AE: Test".to_string(),
            description: "desc".to_string(),
            references: vec![crate::Reference::Internal(2)],
            line_index: 0,
        };
        p.show(&node, &vec![3], &node.references)?;
        p.list(&vec!["one".to_string(), "two".to_string()])?;
        p.refs(&vec!["ref".to_string()])?;
        p.links(1, &vec![crate::Reference::Internal(2)])?;
        p.orphans(&Vec::<String>::new())?;
        p.orphans(&vec!["4".to_string()])?;
        Ok(())
    }
}
