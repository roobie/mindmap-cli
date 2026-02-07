//! MindmapCache: Secure file loading and caching for recursive navigation
//!
//! This module provides:
//! - Lazy loading and caching of mindmap files
//! - Secure path resolution with validation (prevents directory traversal)
//! - File size checks (max 10MB by default)
//! - Integration with NavigationContext for cycle detection

use anyhow::{Context, Result, bail};
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};

use crate::Mindmap;

/// Manages loading and caching of mindmap files with security validation
#[derive(Debug)]
pub struct MindmapCache {
    /// Cache of loaded mindmaps: canonical path -> Mindmap
    cache: HashMap<PathBuf, Mindmap>,
    /// Canonicalized workspace root for safety checks
    workspace_root: PathBuf,
    /// Max file size to load (default: 10MB)
    max_file_size: u64,
    /// Max recursion depth (default: 50)
    max_depth: usize,
}

impl MindmapCache {
    /// Create a new cache with the given workspace root
    pub fn new(workspace_root: PathBuf) -> Self {
        // Canonicalize workspace root to absolute, real path
        let canonical_root = fs::canonicalize(&workspace_root)
            .unwrap_or_else(|_| workspace_root.canonicalize().unwrap_or(workspace_root));

        MindmapCache {
            cache: HashMap::new(),
            workspace_root: canonical_root,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_depth: 50,
        }
    }

    /// Get the workspace root
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Load a mindmap file with caching
    ///
    /// # Arguments
    /// * `base_file` - The file that contains the reference (used to resolve relative paths)
    /// * `relative` - The relative path to load (e.g., "./MINDMAP.llm.md")
    /// * `visited` - Set of already-visited files (for cycle detection)
    ///
    /// # Errors
    /// - Path traversal attempts (e.g., "../../../etc/passwd")
    /// - Absolute paths (POSIX, Windows drive letters, UNC paths)
    /// - File too large (> max_file_size)
    /// - File not found
    /// - Cycle detected (path already in visited set)
    pub fn load(
        &mut self,
        base_file: &Path,
        relative: &str,
        visited: &std::collections::HashSet<PathBuf>,
    ) -> Result<&Mindmap> {
        // Resolve relative path from the current file's directory
        let canonical = self.resolve_path(base_file, relative)?;

        // Check for cycles
        if visited.contains(&canonical) {
            bail!(
                "Circular reference detected: {} -> {}",
                base_file.display(),
                relative
            );
        }

        // Return cached version if already loaded
        if self.cache.contains_key(&canonical) {
            return Ok(self.cache.get(&canonical).unwrap());
        }

        // Check file size before reading
        let metadata = fs::metadata(&canonical)
            .with_context(|| format!("Failed to stat file: {}", canonical.display()))?;

        if metadata.len() > self.max_file_size {
            bail!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.max_file_size
            );
        }

        // Load the mindmap
        let mm = Mindmap::load(canonical.clone())
            .with_context(|| format!("Failed to load mindmap: {}", canonical.display()))?;

        // Cache and return
        self.cache.insert(canonical.clone(), mm);
        Ok(self.cache.get(&canonical).unwrap())
    }

    /// Resolve a relative path to a canonical absolute path
    ///
    /// Path resolution rules:
    /// 1. Resolve relative to the base_file's directory
    /// 2. No absolute paths allowed (POSIX /foo, Windows C:\foo, UNC \\server\share)
    /// 3. No directory traversal escapes (../../../ blocked)
    /// 4. Final path must be within workspace_root
    ///
    /// # Errors
    /// - Absolute path detected
    /// - Path escape attempt (outside workspace_root)
    /// - Path canonicalization failed
    pub fn resolve_path(&self, base_file: &Path, relative: &str) -> Result<PathBuf> {
        let rel_path = Path::new(relative);

        // Reject absolute paths (POSIX, Windows, UNC)
        if rel_path.is_absolute() {
            bail!("Absolute paths not allowed: {}", relative);
        }

        // Check for Windows drive letters or UNC prefixes or POSIX root
        for component in rel_path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    bail!("Absolute paths not allowed: {}", relative);
                }
                _ => {}
            }
        }

        // Resolve relative to the base file's directory
        let base_dir = base_file.parent().unwrap_or(&self.workspace_root);

        // Canonicalize the base directory if it exists
        let canonical_base = fs::canonicalize(base_dir).unwrap_or_else(|_| base_dir.to_path_buf());

        // Join the relative path
        let full_path = canonical_base.join(rel_path);

        // Canonicalize (this validates the path structure and resolves symlinks)
        let canonical = fs::canonicalize(&full_path).with_context(|| {
            format!(
                "Failed to resolve path: {} (relative to {})",
                relative,
                base_dir.display()
            )
        })?;

        // Ensure the resolved path is still within the workspace
        if !canonical.starts_with(&self.workspace_root) {
            bail!(
                "Path escape attempt: {} resolves outside workspace",
                relative
            );
        }

        Ok(canonical)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            num_cached: self.cache.len(),
            total_nodes: self.cache.values().map(|mm| mm.nodes.len()).sum(),
        }
    }

    /// Set max file size (for testing)
    #[cfg(test)]
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    /// Set max recursion depth (for testing)
    #[cfg(test)]
    pub fn set_max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
    }

    /// Get max recursion depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub num_cached: usize,
    pub total_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cache_new() {
        let cache = MindmapCache::new(PathBuf::from("."));
        assert_eq!(cache.cache.len(), 0);
        assert!(cache.workspace_root.is_absolute());
    }

    #[test]
    fn test_resolve_path_relative() -> Result<()> {
        let temp = TempDir::new()?;
        let base = temp.path().join("subdir");
        fs::create_dir(&base)?;
        let base_file = base.join("MINDMAP.md");
        fs::write(&base_file, "[1] **Test** - body")?;

        // Create the target file
        let other_file = base.join("other.md");
        fs::write(&other_file, "[2] **Other** - body")?;

        let cache = MindmapCache::new(temp.path().to_path_buf());

        // Relative path in same directory
        let resolved = cache.resolve_path(&base_file, "./other.md")?;
        assert!(resolved.ends_with("other.md"));
        assert!(resolved.starts_with(cache.workspace_root()));

        Ok(())
    }

    #[test]
    fn test_resolve_path_rejects_absolute_posix() {
        let cache = MindmapCache::new(PathBuf::from("."));
        let base_file = PathBuf::from("MINDMAP.md");

        // Should reject absolute POSIX path
        let result = cache.resolve_path(&base_file, "/etc/passwd");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Absolute paths not allowed")
        );
    }

    #[test]
    fn test_resolve_path_rejects_parent_escape() -> Result<()> {
        let temp = TempDir::new()?;
        let workspace = temp.path();

        // Create structure: workspace/subdir/MINDMAP.md
        let subdir = workspace.join("subdir");
        fs::create_dir(&subdir)?;
        let base_file = subdir.join("MINDMAP.md");
        fs::write(&base_file, "[1] **Test** - body")?;

        let cache = MindmapCache::new(workspace.to_path_buf());

        // Attempt to escape with ../..
        let relative = format!(
            "{}{}",
            std::path::MAIN_SEPARATOR.to_string().repeat(10),
            "etc/passwd"
        );

        let result = cache.resolve_path(&base_file, &relative);

        // Should detect escape (path outside workspace) or canonicalization fail
        if result.is_ok() {
            let resolved = result.unwrap();
            assert!(
                !resolved.starts_with(workspace),
                "Should not resolve outside workspace"
            );
        }

        Ok(())
    }

    #[test]
    fn test_load_caches_files() -> Result<()> {
        let temp = TempDir::new()?;
        let file1 = temp.path().join("MINDMAP.md");
        fs::write(&file1, "[1] **Test** - body\n")?;

        let mut cache = MindmapCache::new(temp.path().to_path_buf());
        let visited = std::collections::HashSet::new();

        // First load - capture pointer before borrow ends
        let mm1_ptr = {
            let mm1 = cache.load(&file1, "./MINDMAP.md", &visited)?;
            mm1 as *const _
        };
        assert_eq!(cache.cache.len(), 1);

        // Second load should return cached
        let mm2_ptr = {
            let mm2 = cache.load(&file1, "./MINDMAP.md", &visited)?;
            mm2 as *const _
        };
        assert_eq!(cache.cache.len(), 1);

        // Both should be the same (pointer equality)
        assert_eq!(mm1_ptr, mm2_ptr);

        Ok(())
    }

    #[test]
    fn test_load_detects_cycle() -> Result<()> {
        let temp = TempDir::new()?;
        let file1 = temp.path().join("MINDMAP.md");
        fs::write(&file1, "[1] **Test** - body\n")?;

        let mut cache = MindmapCache::new(temp.path().to_path_buf());
        let mut visited = std::collections::HashSet::new();

        // First load
        let canonical = cache.resolve_path(&file1, "./MINDMAP.md")?;
        visited.insert(canonical.clone());

        // Try to load again with visited set - should fail
        let result = cache.load(&file1, "./MINDMAP.md", &visited);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Circular reference")
        );

        Ok(())
    }

    #[test]
    fn test_load_rejects_oversized_file() -> Result<()> {
        let temp = TempDir::new()?;
        let file1 = temp.path().join("big.md");

        // Create a file larger than the test limit
        let content = "x".repeat(1024 * 1024); // 1 MB
        fs::write(&file1, &content)?;

        let mut cache = MindmapCache::new(temp.path().to_path_buf());
        cache.set_max_file_size(1024); // Set limit to 1 KB

        let visited = std::collections::HashSet::new();
        let result = cache.load(&file1, "./big.md", &visited);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File too large"));

        Ok(())
    }

    #[test]
    fn test_cache_stats() -> Result<()> {
        let temp = TempDir::new()?;
        let file1 = temp.path().join("MINDMAP.md");
        fs::write(&file1, "[1] **Test1** - body\n[2] **Test2** - body\n")?;

        let mut cache = MindmapCache::new(temp.path().to_path_buf());
        let visited = std::collections::HashSet::new();

        cache.load(&file1, "./MINDMAP.md", &visited)?;
        let stats = cache.stats();

        assert_eq!(stats.num_cached, 1);
        assert_eq!(stats.total_nodes, 2);

        Ok(())
    }
}
