//! DAG diff functionality
//!
//! Provides functions to compare two DAGs and identify added/removed leaves.

use crate::error::{Result, ScionicError};
use crate::types::{Dag, DagLeaf};
use std::collections::{HashMap, HashSet};

/// Type of difference detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    Added,
    Removed,
}

/// A single leaf difference
#[derive(Debug, Clone)]
pub struct LeafDiff {
    pub diff_type: DiffType,
    pub hash: String,
    pub leaf: DagLeaf,
}

/// Summary of differences between two DAGs
#[derive(Debug, Clone, Default)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub total: usize,
}

/// Complete diff between two DAGs
#[derive(Debug, Clone)]
pub struct DagDiff {
    pub diffs: HashMap<String, LeafDiff>,
    pub summary: DiffSummary,
}

impl DagDiff {
    /// Get all added leaves
    pub fn get_added_leaves(&self) -> HashMap<String, &DagLeaf> {
        self.diffs
            .iter()
            .filter(|(_, d)| d.diff_type == DiffType::Added)
            .map(|(h, d)| (h.clone(), &d.leaf))
            .collect()
    }

    /// Get all removed leaves
    pub fn get_removed_leaves(&self) -> HashMap<String, &DagLeaf> {
        self.diffs
            .iter()
            .filter(|(_, d)| d.diff_type == DiffType::Removed)
            .map(|(h, d)| (h.clone(), &d.leaf))
            .collect()
    }

    /// Apply this diff to an old DAG to produce a new DAG
    pub fn apply_to_dag(&self, old_dag: &Dag) -> Result<Dag> {
        // If no additions, return copy of old DAG
        if self.summary.added == 0 {
            return Ok(Dag {
                root: old_dag.root.clone(),
                leaves: old_dag.leaves.clone(),
                labels: None,
            });
        }

        // Build pool of all available leaves
        let mut leaf_pool: HashMap<String, DagLeaf> = old_dag.leaves.clone();

        // Add new leaves from diff
        for (hash, leaf_diff) in &self.diffs {
            if leaf_diff.diff_type == DiffType::Added {
                leaf_pool.insert(hash.clone(), leaf_diff.leaf.clone());
            }
        }

        // Find all child hashes referenced by any leaf
        let mut child_hashes: HashSet<String> = HashSet::new();
        for leaf in leaf_pool.values() {
            for link in &leaf.links {
                child_hashes.insert(link.clone());
            }
        }

        // Find new root among added leaves (not referenced by any other leaf, has leaf_count)
        let added_leaves = self.get_added_leaves();
        let mut new_root_hash: Option<String> = None;

        for (hash, leaf) in &added_leaves {
            if !child_hashes.contains(hash) {
                if leaf.leaf_count.is_some() && leaf.leaf_count.unwrap() > 0 {
                    new_root_hash = Some(hash.clone());
                    break;
                }
            }
        }

        let new_root_hash =
            new_root_hash.ok_or_else(|| ScionicError::InvalidDag("Cannot find new root among added leaves".into()))?;

        // Traverse from new root to collect referenced leaves
        let mut new_leaves: HashMap<String, DagLeaf> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();

        fn traverse(
            hash: &str,
            leaf_pool: &HashMap<String, DagLeaf>,
            new_leaves: &mut HashMap<String, DagLeaf>,
            visited: &mut HashSet<String>,
        ) -> Result<()> {
            if visited.contains(hash) {
                return Ok(());
            }
            visited.insert(hash.to_string());

            let leaf = leaf_pool
                .get(hash)
                .ok_or_else(|| ScionicError::InvalidDag(format!("Missing leaf in pool: {}", hash)))?;

            new_leaves.insert(hash.to_string(), leaf.clone());

            for child_hash in &leaf.links {
                traverse(child_hash, leaf_pool, new_leaves, visited)?;
            }

            Ok(())
        }

        traverse(&new_root_hash, &leaf_pool, &mut new_leaves, &mut visited)?;

        Ok(Dag {
            root: new_root_hash,
            leaves: new_leaves,
            labels: None,
        })
    }

    /// Create a partial DAG containing only the added leaves with verification paths
    pub fn create_partial_dag(&self, full_new_dag: &Dag) -> Result<Dag> {
        let added_leaves = self.get_added_leaves();
        if added_leaves.is_empty() {
            return Err(ScionicError::InvalidDag("No added leaves to create partial DAG".into()));
        }

        let added_hashes: Vec<String> = added_leaves.keys().cloned().collect();
        full_new_dag.get_partial(&added_hashes, false)
    }
}

/// Compare two DAGs and return the differences
pub fn diff(first_dag: &Dag, second_dag: &Dag) -> Result<DagDiff> {
    let mut diffs: HashMap<String, LeafDiff> = HashMap::new();
    let mut summary = DiffSummary::default();

    let old_leaves: HashSet<&String> = first_dag.leaves.keys().collect();
    let new_leaves: HashSet<&String> = second_dag.leaves.keys().collect();

    // Find added leaves (in second but not in first)
    for hash in &new_leaves {
        if !old_leaves.contains(hash) {
            let leaf = second_dag.leaves.get(*hash).unwrap().clone();
            diffs.insert(
                (*hash).clone(),
                LeafDiff {
                    diff_type: DiffType::Added,
                    hash: (*hash).clone(),
                    leaf,
                },
            );
            summary.added += 1;
            summary.total += 1;
        }
    }

    // Find removed leaves (in first but not in second)
    for hash in &old_leaves {
        if !new_leaves.contains(hash) {
            let leaf = first_dag.leaves.get(*hash).unwrap().clone();
            diffs.insert(
                (*hash).clone(),
                LeafDiff {
                    diff_type: DiffType::Removed,
                    hash: (*hash).clone(),
                    leaf,
                },
            );
            summary.removed += 1;
            summary.total += 1;
        }
    }

    Ok(DagDiff { diffs, summary })
}

/// Compare old DAG with a set of new leaves (e.g., from partial DAG)
/// Identifies added leaves and removed leaves no longer referenced by new structure
pub fn diff_from_new_leaves(original_dag: &Dag, new_leaves: &HashMap<String, DagLeaf>) -> Result<DagDiff> {
    let mut diffs: HashMap<String, LeafDiff> = HashMap::new();
    let mut summary = DiffSummary::default();

    let old_leaves: HashSet<&String> = original_dag.leaves.keys().collect();

    // Find new root (leaf with leaf_count > 0)
    let mut new_root: Option<(&String, &DagLeaf)> = None;
    for (hash, leaf) in new_leaves {
        if leaf.leaf_count.is_some() && leaf.leaf_count.unwrap() > 0 {
            new_root = Some((hash, leaf));
            break;
        }
    }

    // Find added leaves
    for (hash, leaf) in new_leaves {
        if !old_leaves.contains(hash) {
            diffs.insert(
                hash.clone(),
                LeafDiff {
                    diff_type: DiffType::Added,
                    hash: hash.clone(),
                    leaf: leaf.clone(),
                },
            );
            summary.added += 1;
            summary.total += 1;
        }
    }

    // Find removed leaves - those not reachable from new root
    let mut reachable: HashSet<String> = HashSet::new();

    if let Some((root_hash, _)) = new_root {
        fn traverse_reachable(
            hash: &str,
            new_leaves: &HashMap<String, DagLeaf>,
            old_leaves: &HashMap<String, DagLeaf>,
            reachable: &mut HashSet<String>,
        ) {
            if reachable.contains(hash) {
                return;
            }
            reachable.insert(hash.to_string());

            // Look in both new and old leaves
            let leaf = new_leaves.get(hash).or_else(|| old_leaves.get(hash));

            if let Some(leaf) = leaf {
                for child_hash in &leaf.links {
                    traverse_reachable(child_hash, new_leaves, old_leaves, reachable);
                }
            }
        }

        traverse_reachable(root_hash, new_leaves, &original_dag.leaves, &mut reachable);
    }

    // Any old leaf not reachable is removed
    for (hash, leaf) in &original_dag.leaves {
        if !reachable.contains(hash) {
            diffs.insert(
                hash.clone(),
                LeafDiff {
                    diff_type: DiffType::Removed,
                    hash: hash.clone(),
                    leaf: leaf.clone(),
                },
            );
            summary.removed += 1;
            summary.total += 1;
        }
    }

    Ok(DagDiff { diffs, summary })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_dag;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_diff_identical_dags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().join("test");
        fs::create_dir(&dir)?;
        fs::write(dir.join("file.txt"), "content")?;

        let dag1 = create_dag(&dir, false)?;
        let dag2 = create_dag(&dir, false)?;

        let result = diff(&dag1, &dag2)?;

        assert_eq!(result.summary.total, 0);
        assert_eq!(result.summary.added, 0);
        assert_eq!(result.summary.removed, 0);

        Ok(())
    }

    #[test]
    fn test_diff_added_leaves() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create small DAG
        let dir1 = temp_dir.path().join("small");
        fs::create_dir(&dir1)?;
        fs::write(dir1.join("file1.txt"), "content1")?;
        let dag1 = create_dag(&dir1, false)?;

        // Create larger DAG
        let dir2 = temp_dir.path().join("large");
        fs::create_dir(&dir2)?;
        fs::write(dir2.join("file1.txt"), "content1")?;
        fs::write(dir2.join("file2.txt"), "content2")?;
        let dag2 = create_dag(&dir2, false)?;

        let result = diff(&dag1, &dag2)?;

        assert!(result.summary.added > 0);

        Ok(())
    }

    #[test]
    fn test_diff_removed_leaves() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create larger DAG first
        let dir1 = temp_dir.path().join("large");
        fs::create_dir(&dir1)?;
        fs::write(dir1.join("file1.txt"), "content1")?;
        fs::write(dir1.join("file2.txt"), "content2")?;
        let dag1 = create_dag(&dir1, false)?;

        // Create smaller DAG
        let dir2 = temp_dir.path().join("small");
        fs::create_dir(&dir2)?;
        fs::write(dir2.join("file1.txt"), "content1")?;
        let dag2 = create_dag(&dir2, false)?;

        let result = diff(&dag1, &dag2)?;

        assert!(result.summary.removed > 0);

        Ok(())
    }

    #[test]
    fn test_diff_modified_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir = temp_dir.path().join("test");
        fs::create_dir(&dir)?;

        // Create first DAG
        fs::write(dir.join("file.txt"), "original content")?;
        let dag1 = create_dag(&dir, false)?;

        // Modify and create second DAG
        fs::write(dir.join("file.txt"), "modified content")?;
        let dag2 = create_dag(&dir, false)?;

        let result = diff(&dag1, &dag2)?;

        // Modified = removal + addition in content-addressed systems
        assert!(result.summary.added > 0);
        assert!(result.summary.removed > 0);

        Ok(())
    }

    #[test]
    fn test_get_added_removed_leaves() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let dir1 = temp_dir.path().join("dir1");
        fs::create_dir(&dir1)?;
        fs::write(dir1.join("file1.txt"), "content1")?;
        let dag1 = create_dag(&dir1, false)?;

        let dir2 = temp_dir.path().join("dir2");
        fs::create_dir(&dir2)?;
        fs::write(dir2.join("file2.txt"), "content2")?;
        let dag2 = create_dag(&dir2, false)?;

        let result = diff(&dag1, &dag2)?;

        let added = result.get_added_leaves();
        let removed = result.get_removed_leaves();

        assert_eq!(added.len(), result.summary.added);
        assert_eq!(removed.len(), result.summary.removed);

        Ok(())
    }
}
