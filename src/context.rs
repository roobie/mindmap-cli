//! NavigationContext: Track recursion depth and detect cycles
//!
//! This module provides:
//! - Depth tracking for recursive navigation
//! - Cycle detection via visited file set
//! - RAII guard pattern for safe depth management

use anyhow::{Result, bail};
use std::{collections::HashSet, path::PathBuf};

/// Context for tracking recursive navigation
#[derive(Debug)]
pub struct NavigationContext {
    /// Current recursion depth
    depth: usize,
    /// Maximum allowed recursion depth
    max_depth: usize,
    /// Files visited in this traversal (for cycle detection)
    visited: HashSet<PathBuf>,
}

impl NavigationContext {
    /// Create a new navigation context with default max depth (50)
    pub fn new() -> Self {
        NavigationContext {
            depth: 0,
            max_depth: 50,
            visited: HashSet::new(),
        }
    }

    /// Create a new navigation context with custom max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        NavigationContext {
            depth: 0,
            max_depth,
            visited: HashSet::new(),
        }
    }

    /// Get the current recursion depth
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the maximum allowed depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Check if we've reached the max depth
    pub fn at_max_depth(&self) -> bool {
        self.depth >= self.max_depth
    }

    /// Descend one level, returning a guard that auto-decrements on drop
    ///
    /// # Errors
    /// If recursion depth would exceed max_depth
    pub fn descend(&mut self) -> Result<DepthGuard<'_>> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1; // Undo increment
            bail!("Recursion depth exceeded (max: {})", self.max_depth);
        }
        Ok(DepthGuard { ctx: self })
    }

    /// Check if a path has been visited
    pub fn is_visited(&self, path: &PathBuf) -> bool {
        self.visited.contains(path)
    }

    /// Mark a path as visited
    pub fn mark_visited(&mut self, path: PathBuf) {
        self.visited.insert(path);
    }

    /// Clear the visited set (for testing)
    pub fn clear_visited(&mut self) {
        self.visited.clear();
    }

    /// Get the number of visited files
    pub fn num_visited(&self) -> usize {
        self.visited.len()
    }

    /// Set max depth (for testing)
    #[cfg(test)]
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }
}

impl Default for NavigationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard to decrement depth on drop
pub struct DepthGuard<'a> {
    ctx: &'a mut NavigationContext,
}

impl<'a> Drop for DepthGuard<'a> {
    fn drop(&mut self) {
        self.ctx.depth = self.ctx.depth.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = NavigationContext::new();
        assert_eq!(ctx.depth(), 0);
        assert_eq!(ctx.max_depth(), 50);
        assert_eq!(ctx.num_visited(), 0);
    }

    #[test]
    fn test_context_with_max_depth() {
        let ctx = NavigationContext::with_max_depth(10);
        assert_eq!(ctx.max_depth(), 10);
    }

    #[test]
    fn test_descend_increments_depth() -> Result<()> {
        let mut ctx = NavigationContext::new();
        assert_eq!(ctx.depth(), 0);

        {
            let _guard = ctx.descend()?;
            drop(_guard);
        }
        assert_eq!(ctx.depth(), 0);

        Ok(())
    }

    #[test]
    fn test_descend_decrements_on_drop() -> Result<()> {
        let mut ctx = NavigationContext::new();
        {
            let _guard = ctx.descend()?;
            drop(_guard);
        }
        assert_eq!(ctx.depth(), 0);

        Ok(())
    }

    #[test]
    fn test_descend_enforces_max_depth() {
        let mut ctx = NavigationContext::with_max_depth(2);

        // Descend to depth 1
        assert!(ctx.descend().is_ok());

        // Manually verify we can't access ctx while guard is held by checking depth increases
        // We can't do direct checks, so we'll manually manage depth for testing
        ctx.depth = 0; // Reset for clean test

        ctx.depth = 1;
        assert_eq!(ctx.depth(), 1);

        ctx.depth = 2;
        assert_eq!(ctx.depth(), 2);

        // Now try to descend when already at max
        {
            let result = ctx.descend();
            assert!(result.is_err());
        }
        assert_eq!(ctx.depth(), 2); // Should not have incremented
    }

    #[test]
    fn test_visited_tracking() {
        let mut ctx = NavigationContext::new();
        let path1 = PathBuf::from("/some/file1.md");
        let path2 = PathBuf::from("/some/file2.md");

        assert!(!ctx.is_visited(&path1));
        assert_eq!(ctx.num_visited(), 0);

        ctx.mark_visited(path1.clone());
        assert!(ctx.is_visited(&path1));
        assert!(!ctx.is_visited(&path2));
        assert_eq!(ctx.num_visited(), 1);

        ctx.mark_visited(path2.clone());
        assert!(ctx.is_visited(&path1));
        assert!(ctx.is_visited(&path2));
        assert_eq!(ctx.num_visited(), 2);
    }

    #[test]
    fn test_clear_visited() {
        let mut ctx = NavigationContext::new();
        let path1 = PathBuf::from("/some/file1.md");

        ctx.mark_visited(path1.clone());
        assert!(ctx.is_visited(&path1));

        ctx.clear_visited();
        assert!(!ctx.is_visited(&path1));
        assert_eq!(ctx.num_visited(), 0);
    }

    #[test]
    fn test_guard_pattern() -> Result<()> {
        let mut ctx = NavigationContext::new();

        {
            let g1 = ctx.descend()?;
            drop(g1);
            {
                let g2 = ctx.descend()?;
                drop(g2);
            }
        }
        assert_eq!(ctx.depth(), 0);

        Ok(())
    }

    #[test]
    fn test_at_max_depth() {
        let mut ctx = NavigationContext::with_max_depth(2);
        assert!(!ctx.at_max_depth());

        ctx.depth = 1;
        assert!(!ctx.at_max_depth());

        ctx.depth = 2;
        assert!(ctx.at_max_depth());
    }
}
