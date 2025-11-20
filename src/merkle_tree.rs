use crate::error::{Result, ScionicError};
use crate::types::MerkleProof;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Classic Merkle Tree implementation
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Root hash of the tree
    pub root: Vec<u8>,

    /// Proofs for each leaf
    pub proofs: Vec<MerkleProof>,

    /// Mapping of keys to indices
    key_to_index: HashMap<String, usize>,

    /// Original leaves
    leaves: Vec<Vec<u8>>,
}

impl MerkleTree {
    /// Create a new Merkle tree from data blocks
    pub fn new(data: Vec<(String, Vec<u8>)>) -> Result<Self> {
        if data.is_empty() {
            return Err(ScionicError::InvalidLeaf(
                "Cannot create tree with no data".to_string(),
            ));
        }

        let mut key_to_index = HashMap::new();
        let mut leaves = Vec::new();

        // Hash each data block to create leaves
        for (i, (key, value)) in data.iter().enumerate() {
            let mut hasher = Sha256::new();
            hasher.update(value);
            let leaf_hash = hasher.finalize().to_vec();

            leaves.push(leaf_hash);
            key_to_index.insert(key.clone(), i);
        }

        // Build the tree
        let (root, proofs) = build_tree(&leaves);

        Ok(Self {
            root,
            proofs,
            key_to_index,
            leaves,
        })
    }

    /// Get the index for a given key
    pub fn get_index_for_key(&self, key: &str) -> Option<usize> {
        self.key_to_index.get(key).copied()
    }

    /// Verify a proof against the root
    pub fn verify(&self, data: &[u8], proof: &MerkleProof) -> Result<()> {
        verify_proof(data, proof, &self.root)
    }
}

/// Build a Merkle tree and generate proofs
fn build_tree(leaves: &[Vec<u8>]) -> (Vec<u8>, Vec<MerkleProof>) {
    if leaves.is_empty() {
        return (vec![], vec![]);
    }

    if leaves.len() == 1 {
        let proof = MerkleProof {
            siblings: vec![],
            path: 0,
        };
        return (leaves[0].clone(), vec![proof]);
    }

    // Build levels from bottom up
    let mut current_level = leaves.to_vec();
    let mut all_levels = vec![current_level.clone()];

    while current_level.len() > 1 {
        let mut next_level = Vec::new();

        for chunk in current_level.chunks(2) {
            let hash = if chunk.len() == 2 {
                hash_pair(&chunk[0], &chunk[1])
            } else {
                // Odd number, promote the single node
                chunk[0].clone()
            };
            next_level.push(hash);
        }

        current_level = next_level;
        all_levels.push(current_level.clone());
    }

    let root = current_level[0].clone();

    // Generate proofs for each leaf
    let mut proofs = Vec::new();
    for i in 0..leaves.len() {
        let proof = generate_proof(i, &all_levels);
        proofs.push(proof);
    }

    (root, proofs)
}

/// Generate a Merkle proof for a specific leaf index
fn generate_proof(leaf_index: usize, levels: &[Vec<Vec<u8>>]) -> MerkleProof {
    let mut siblings = Vec::new();
    let mut path: u32 = 0;
    let mut index = leaf_index;

    for (depth, level) in levels.iter().take(levels.len() - 1).enumerate() {
        let is_right = index % 2 == 1;

        // Set bit in path if sibling is on right (we're on left)
        if !is_right {
            path |= 1 << depth;
        }

        let sibling_index = if is_right { index - 1 } else { index + 1 };

        if sibling_index < level.len() {
            siblings.push(level[sibling_index].clone());
        }

        index /= 2;
    }

    MerkleProof { siblings, path }
}

/// Hash a pair of nodes
fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().to_vec()
}

/// Verify a Merkle proof
pub fn verify_proof(data: &[u8], proof: &MerkleProof, root: &[u8]) -> Result<()> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut current_hash = hasher.finalize().to_vec();

    for (depth, sibling) in proof.siblings.iter().enumerate() {
        // Check bit in path - if set, sibling is on right (we're on left)
        let sibling_on_right = (proof.path & (1 << depth)) != 0;

        current_hash = if sibling_on_right {
            hash_pair(&current_hash, sibling)
        } else {
            hash_pair(sibling, &current_hash)
        };
    }

    if current_hash == root {
        Ok(())
    } else {
        Err(ScionicError::InvalidProof)
    }
}

/// Builder for creating Merkle trees
pub struct MerkleTreeBuilder {
    data: Vec<(String, Vec<u8>)>,
}

impl MerkleTreeBuilder {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_leaf(&mut self, key: String, value: Vec<u8>) {
        self.data.push((key, value));
    }

    pub fn build(self) -> Result<MerkleTree> {
        MerkleTree::new(self.data)
    }
}

impl Default for MerkleTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_creation() {
        let data = vec![
            ("key1".to_string(), b"data1".to_vec()),
            ("key2".to_string(), b"data2".to_vec()),
            ("key3".to_string(), b"data3".to_vec()),
        ];

        let tree = MerkleTree::new(data).unwrap();
        assert!(!tree.root.is_empty());
        assert_eq!(tree.proofs.len(), 3);
    }

    #[test]
    fn test_merkle_proof_verification() {
        let data = vec![
            ("key1".to_string(), b"data1".to_vec()),
            ("key2".to_string(), b"data2".to_vec()),
        ];

        let tree = MerkleTree::new(data).unwrap();

        // Verify first leaf
        let result = tree.verify(b"data1", &tree.proofs[0]);
        assert!(result.is_ok());

        // Verify second leaf
        let result = tree.verify(b"data2", &tree.proofs[1]);
        assert!(result.is_ok());

        // Verify with wrong data should fail
        let result = tree.verify(b"wrong", &tree.proofs[0]);
        assert!(result.is_err());
    }
}
