use std::collections::HashSet;

#[derive(Debug)]
pub struct NavigationContext {
    depth: usize,
    max_depth: usize,
    visited: HashSet<std::path::PathBuf>,
}

impl NavigationContext {
    pub fn new() -> Self {
        NavigationContext {
            depth: 0,
            max_depth: 50,
            visited: HashSet::new(),
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        NavigationContext {
            depth: 0,
            max_depth,
            visited: HashSet::new(),
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn descend(&mut self) -> Result<DepthGuard<'_>, String> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            Err(format!("Recursion depth exceeded (max: {})", self.max_depth))
        } else {
            Ok(DepthGuard { ctx: self })
        }
    }
}

pub struct DepthGuard<'a> {
    ctx: &'a mut NavigationContext,
}

impl<'a> Drop for DepthGuard<'a> {
    fn drop(&mut self) {
        println!("Dropping guard, depth was {}", self.ctx.depth);
        self.ctx.depth = self.ctx.depth.saturating_sub(1);
        println!("After drop, depth is {}", self.ctx.depth);
    }
}

fn main() {
    let mut ctx = NavigationContext::with_max_depth(2);

    // First descent should work
    {
        println!("Before first descend: depth = {}", ctx.depth());
        let _guard1 = ctx.descend();
        println!("After first descend: depth = {}", ctx.depth());
        assert!(_guard1.is_ok());
        println!("After first assert: depth = {}", ctx.depth());
    }
    println!("After first scope: depth = {}", ctx.depth());
    assert_eq!(ctx.depth(), 1);
}
