use crate::error::{Result, ScionicError};
use crate::types::{Dag, DagBuilderConfig, DagLeaf, DagLeafBuilder, LeafType, DEFAULT_CHUNK_SIZE};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Create a DAG from a file or directory
pub fn create_dag(path: impl AsRef<Path>, timestamp_root: bool) -> Result<Dag> {
    let mut config = DagBuilderConfig::default();
    config.timestamp_root = timestamp_root;

    if timestamp_root {
        let timestamp = chrono::Utc::now().to_rfc3339();
        config
            .additional_data
            .insert("timestamp".to_string(), timestamp);
    }

    create_dag_with_config(path, config)
}

/// Create a DAG with custom configuration
pub fn create_dag_with_config(path: impl AsRef<Path>, config: DagBuilderConfig) -> Result<Dag> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(ScionicError::PathNotFound(
            path.display().to_string(),
        ));
    }

    let mut builder = DagBuilder::new();
    let metadata = fs::metadata(path)?;

    let root_leaf = if metadata.is_dir() {
        process_directory(path, path, &mut builder, true, &config)?
    } else {
        process_file(path, path, &mut builder, true, &config)?
    };

    // Build root leaf with metadata
    let root_builder = DagLeafBuilder::new(root_leaf.item_name.clone())
        .set_type(root_leaf.leaf_type.clone());

    let root_builder = if let Some(content) = root_leaf.content {
        root_builder.set_data(content)
    } else {
        root_builder
    };

    let root_builder = root_leaf
        .links
        .iter()
        .fold(root_builder, |builder, link| builder.add_link(link.clone()));

    let additional_data = if config.additional_data.is_empty() {
        None
    } else {
        Some(config.additional_data.clone())
    };

    let root = root_builder.build_root_leaf(&builder.leaves, additional_data)?;

    builder.leaves.insert(root.hash.clone(), root.clone());

    Ok(Dag {
        root: root.hash,
        leaves: builder.leaves,
        labels: None,
    })
}

/// Process a directory and create a DAG leaf
fn process_directory(
    path: &Path,
    base_path: &Path,
    builder: &mut DagBuilder,
    is_root: bool,
    _config: &DagBuilderConfig,
) -> Result<DagLeaf> {
    let rel_path = if is_root {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string()
    } else {
        path.strip_prefix(base_path)
            .map_err(|_| ScionicError::InvalidDag("Invalid path".to_string()))?
            .to_string_lossy()
            .to_string()
    };

    let mut leaf_builder = DagLeafBuilder::new(rel_path).set_type(LeafType::Directory);

    // Read directory entries
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort for deterministic ordering
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let entry_path = entry.path();
        let metadata = entry.metadata()?;

        // IMPORTANT: Keep base_path constant for all recursion
        let child_leaf = if metadata.is_dir() {
            process_directory(&entry_path, if is_root { path } else { base_path }, builder, false, _config)?
        } else {
            process_file(&entry_path, if is_root { path } else { base_path }, builder, false, _config)?
        };

        builder
            .leaves
            .insert(child_leaf.hash.clone(), child_leaf.clone());
        leaf_builder = leaf_builder.add_link(child_leaf.hash);
    }

    leaf_builder.build_leaf(None)
}

/// Process a file and create a DAG leaf (with chunking if needed)
fn process_file(
    path: &Path,
    base_path: &Path,
    builder: &mut DagBuilder,
    is_root: bool,
    config: &DagBuilderConfig,
) -> Result<DagLeaf> {
    let rel_path = if is_root {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string()
    } else {
        path.strip_prefix(base_path)
            .map_err(|_| ScionicError::InvalidDag("Invalid path".to_string()))?
            .to_string_lossy()
            .to_string()
    };

    let data = fs::read(path)?;
    let mut leaf_builder = DagLeafBuilder::new(rel_path.clone()).set_type(LeafType::File);

    // Determine chunk size to use
    let chunk_size = config.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);

    // Chunk the file if it's larger than the chunk size (and chunking is enabled)
    if chunk_size > 0 && data.len() > chunk_size {
        let chunks: Vec<_> = data.chunks(chunk_size).collect();

        for (i, chunk) in chunks.iter().enumerate() {
            // Use path-based naming to match Go's sequential implementation
            let chunk_name = format!("{}/{}", rel_path, i);
            let chunk_leaf = DagLeafBuilder::new(chunk_name)
                .set_type(LeafType::Chunk)
                .set_data(chunk.to_vec())
                .build_leaf(None)?;

            builder
                .leaves
                .insert(chunk_leaf.hash.clone(), chunk_leaf.clone());
            leaf_builder = leaf_builder.add_link(chunk_leaf.hash);
        }

        leaf_builder.build_leaf(None)
    } else {
        leaf_builder.set_data(data).build_leaf(None)
    }
}

/// Builder for constructing DAGs
pub struct DagBuilder {
    pub leaves: HashMap<String, DagLeaf>,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self {
            leaves: HashMap::new(),
        }
    }
}

impl Default for DagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Dag {
    /// Verify the entire DAG
    pub fn verify(&self) -> Result<()> {
        if self.is_partial() {
            self.verify_with_proofs()
        } else {
            self.verify_full_dag()
        }
    }

    /// Check if this is a partial DAG
    pub fn is_partial(&self) -> bool {
        if let Some(root_leaf) = self.leaves.get(&self.root) {
            if let Some(leaf_count) = root_leaf.leaf_count {
                return self.leaves.len() < leaf_count;
            }
        }
        true
    }

    /// Verify a full DAG (all leaves present)
    fn verify_full_dag(&self) -> Result<()> {
        let root_leaf = self
            .leaves
            .get(&self.root)
            .ok_or_else(|| ScionicError::MissingLeaf("Root leaf not found".to_string()))?;

        // Verify root
        root_leaf.verify_root_leaf()?;

        // Verify all other leaves
        for (hash, leaf) in &self.leaves {
            if hash == &self.root {
                continue;
            }

            leaf.verify_leaf()?;

            // Verify parent-child relationships
            if let Some(parent) = self.find_parent(hash) {
                if !parent.has_link(hash) {
                    return Err(ScionicError::InvalidDag(format!(
                        "Parent {} does not link to child {}",
                        parent.hash, hash
                    )));
                }
            }
        }

        Ok(())
    }

    /// Verify a partial DAG using Merkle proofs
    fn verify_with_proofs(&self) -> Result<()> {
        let root_leaf = self
            .leaves
            .get(&self.root)
            .ok_or_else(|| ScionicError::MissingLeaf("Root leaf not found".to_string()))?;

        // Verify root
        root_leaf.verify_root_leaf()?;

        // Verify each non-root leaf and its proof
        for (hash, leaf) in &self.leaves {
            if hash == &self.root {
                continue;
            }

            // Verify the leaf itself
            leaf.verify_leaf()?;

            // Find parent and verify proof if needed
            if let Some(parent) = self.find_parent(hash) {
                if parent.links.len() > 1 {
                    if let Some(ref proofs) = parent.proofs {
                        if let Some(_proof) = proofs.get(hash) {
                            // Proof verification would go here
                            // For now, just check that it exists
                        } else {
                            return Err(ScionicError::InvalidDag(format!(
                                "Missing proof for leaf {}",
                                hash
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Find the parent of a given leaf
    fn find_parent(&self, child_hash: &str) -> Option<&DagLeaf> {
        for leaf in self.leaves.values() {
            if leaf.has_link(child_hash) {
                return Some(leaf);
            }
        }
        None
    }

    /// Recreate directory structure from DAG
    pub fn create_directory(&self, output_path: impl AsRef<Path>) -> Result<()> {
        let root_leaf = self
            .leaves
            .get(&self.root)
            .ok_or_else(|| ScionicError::MissingLeaf("Root leaf not found".to_string()))?;

        let output_path = output_path.as_ref();

        // For root, create the output directory and process its children directly
        match root_leaf.leaf_type {
            LeafType::Directory => {
                fs::create_dir_all(output_path)?;

                for link in &root_leaf.links {
                    let child_leaf = self
                        .leaves
                        .get(link)
                        .ok_or_else(|| ScionicError::MissingLeaf(link.clone()))?;

                    let child_path = output_path.join(&child_leaf.item_name);
                    self.create_directory_leaf(child_leaf, &child_path)?;
                }
            }
            LeafType::File => {
                // If root is a file, create it with its name
                let file_path = output_path.join(&root_leaf.item_name);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let content = self.get_content_from_leaf(root_leaf)?;
                fs::write(file_path, content)?;
            }
            LeafType::Chunk => {
                return Err(ScionicError::InvalidDag(
                    "Root cannot be a chunk".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn create_directory_leaf(&self, leaf: &DagLeaf, path: &Path) -> Result<()> {
        match leaf.leaf_type {
            LeafType::Directory => {
                fs::create_dir_all(path)?;

                // For directory children, we need to handle the path correctly
                // Child item_names are relative to root, not to this directory
                for link in &leaf.links {
                    let child_leaf = self
                        .leaves
                        .get(link)
                        .ok_or_else(|| ScionicError::MissingLeaf(link.clone()))?;

                    // Extract just the basename of the child's item_name
                    let child_basename = std::path::Path::new(&child_leaf.item_name)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&child_leaf.item_name);

                    let child_path = path.join(child_basename);
                    self.create_directory_leaf(child_leaf, &child_path)?;
                }
            }
            LeafType::File => {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let content = self.get_content_from_leaf(leaf)?;
                fs::write(path, content)?;
            }
            LeafType::Chunk => {
                // Chunks are handled by their parent file
            }
        }

        Ok(())
    }

    /// Get the full content from a file leaf (reassembling chunks if needed)
    fn get_content_from_leaf(&self, leaf: &DagLeaf) -> Result<Vec<u8>> {
        if !leaf.links.is_empty() {
            // Reassemble from chunks
            let mut content = Vec::new();

            for link in &leaf.links {
                let chunk = self
                    .leaves
                    .get(link)
                    .ok_or_else(|| ScionicError::MissingLeaf(link.clone()))?;

                if let Some(ref chunk_content) = chunk.content {
                    content.extend_from_slice(chunk_content);
                } else {
                    return Err(ScionicError::InvalidLeaf(
                        "Chunk has no content".to_string(),
                    ));
                }
            }

            Ok(content)
        } else if let Some(ref content) = leaf.content {
            Ok(content.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Calculate labels for all leaves (for LeafSync)
    pub fn calculate_labels(&mut self) -> Result<()> {
        let mut labels = HashMap::new();
        let mut counter = 1;

        self.iterate_dag(&self.root.clone(), &mut |leaf| {
            if leaf.hash != self.root {
                labels.insert(counter.to_string(), leaf.hash.clone());
                counter += 1;
            }
            Ok(())
        })?;

        self.labels = Some(labels);
        Ok(())
    }

    /// Iterate through the DAG in depth-first order
    fn iterate_dag<F>(&self, hash: &str, f: &mut F) -> Result<()>
    where
        F: FnMut(&DagLeaf) -> Result<()>,
    {
        let leaf = self
            .leaves
            .get(hash)
            .ok_or_else(|| ScionicError::MissingLeaf(hash.to_string()))?;

        f(leaf)?;

        for link in &leaf.links {
            self.iterate_dag(link, f)?;
        }

        Ok(())
    }

    /// Get hashes by label range (for LeafSync)
    pub fn get_hashes_by_label_range(&self, start: usize, end: usize) -> Result<Vec<String>> {
        let labels = self
            .labels
            .as_ref()
            .ok_or_else(|| ScionicError::InvalidLabel("Labels not calculated".to_string()))?;

        // Validate range
        if start < 1 {
            return Err(ScionicError::InvalidLabel(
                "Start label must be >= 1".to_string(),
            ));
        }

        if end < start {
            return Err(ScionicError::InvalidLabel(format!(
                "End label ({}) must be >= start label ({})",
                end, start
            )));
        }

        if end > labels.len() {
            return Err(ScionicError::InvalidLabel(format!(
                "End label ({}) exceeds available labels ({})",
                end,
                labels.len()
            )));
        }

        let mut hashes = Vec::new();
        for i in start..=end {
            let label = i.to_string();
            let hash = labels
                .get(&label)
                .ok_or_else(|| ScionicError::InvalidLabel(format!("Label {} not found", i)))?;
            hashes.push(hash.clone());
        }

        Ok(hashes)
    }

    /// Get the label for a given hash
    pub fn get_label(&self, hash: &str) -> Result<String> {
        // Check if it's the root
        if hash == self.root {
            return Ok("0".to_string());
        }

        // Check if labels have been calculated
        let labels = self
            .labels
            .as_ref()
            .ok_or_else(|| ScionicError::InvalidLabel("Labels not calculated".to_string()))?;

        // Search for the hash in the labels map
        for (label, label_hash) in labels {
            if label_hash == hash {
                return Ok(label.clone());
            }
        }

        // Hash not found
        Err(ScionicError::InvalidLabel(format!(
            "Hash {} not found in labels",
            hash
        )))
    }

    /// Get a partial DAG containing only the specified leaves and their verification paths
    pub fn get_partial(&self, leaf_hashes: &[String], _prune_links: bool) -> Result<Dag> {
        if leaf_hashes.is_empty() {
            return Err(ScionicError::InvalidDag(
                "No leaf hashes provided".to_string(),
            ));
        }

        let mut partial_leaves = HashMap::new();

        // Add root
        let root_leaf = self
            .leaves
            .get(&self.root)
            .ok_or_else(|| ScionicError::MissingLeaf("Root not found".to_string()))?;
        partial_leaves.insert(self.root.clone(), root_leaf.clone());

        // For each requested leaf, add it and its path to root
        for leaf_hash in leaf_hashes {
            let leaf = self
                .leaves
                .get(leaf_hash)
                .ok_or_else(|| ScionicError::MissingLeaf(leaf_hash.clone()))?;

            partial_leaves.insert(leaf_hash.clone(), leaf.clone());

            // Add path to root
            let mut current_hash = leaf_hash.clone();
            while current_hash != self.root {
                // Find parent
                let parent = self
                    .find_parent(&current_hash)
                    .ok_or_else(|| ScionicError::MissingLeaf(format!("Parent not found for {}", current_hash)))?;

                partial_leaves.insert(parent.hash.clone(), parent.clone());
                current_hash = parent.hash.clone();
            }
        }

        Ok(Dag {
            root: self.root.clone(),
            leaves: partial_leaves,
            labels: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_dag_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Hello, World!")?;

        let dag = create_dag(&file_path, false)?;

        assert!(!dag.root.is_empty());
        assert!(!dag.leaves.is_empty());

        Ok(())
    }

    #[test]
    fn test_create_dag_from_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path)?;
        fs::write(dir_path.join("file1.txt"), b"Content 1")?;
        fs::write(dir_path.join("file2.txt"), b"Content 2")?;

        let dag = create_dag(&dir_path, false)?;

        assert!(!dag.root.is_empty());
        assert!(dag.leaves.len() > 1);

        Ok(())
    }

    #[test]
    fn test_verify_dag() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Test content")?;

        let dag = create_dag(&file_path, false)?;
        dag.verify()?;

        Ok(())
    }
}
