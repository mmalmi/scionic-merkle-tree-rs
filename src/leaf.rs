use crate::error::{Result, ScionicError};
use crate::merkle_tree::MerkleTreeBuilder;
use crate::types::{ClassicTreeBranch, DagLeaf, DagLeafBuilder, LeafType};
use cid::Cid;
use multihash::Multihash;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Sort a HashMap by keys and return as Vec of tuples
fn sort_map_for_verification(map: &Option<HashMap<String, String>>) -> Vec<(String, String)> {
    match map {
        None => Vec::new(),
        Some(m) => {
            let mut pairs: Vec<_> = m.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            pairs
        }
    }
}

impl DagLeafBuilder {
    /// Build a regular (non-root) leaf
    pub fn build_leaf(
        self,
        additional_data: Option<HashMap<String, String>>,
    ) -> Result<DagLeaf> {
        let leaf_type = self
            .leaf_type
            .ok_or_else(|| ScionicError::InvalidLeaf("Leaf must have a type".to_string()))?;

        // Build merkle root for links
        let merkle_root = if self.links.len() > 1 {
            let mut builder = MerkleTreeBuilder::new();
            for link in &self.links {
                builder.add_leaf(link.clone(), link.as_bytes().to_vec());
            }
            let tree = builder.build()?;
            Some(tree.root)
        } else if self.links.len() == 1 {
            Some(self.links[0].as_bytes().to_vec())
        } else {
            None
        };

        // Compute content hash
        let content_hash = self.data.as_ref().map(|data| {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        // Create leaf data for hashing
        #[derive(Serialize)]
        struct LeafData {
            #[serde(rename = "ItemName")]
            item_name: String,
            #[serde(rename = "Type")]
            leaf_type: String,
            #[serde(rename = "MerkleRoot")]
            merkle_root: Vec<u8>,
            #[serde(rename = "CurrentLinkCount")]
            current_link_count: usize,
            #[serde(rename = "ContentHash")]
            content_hash: Option<Vec<u8>>,
            #[serde(rename = "AdditionalData")]
            additional_data: Vec<(String, String)>,
        }

        let leaf_data = LeafData {
            item_name: self.item_name.clone(),
            leaf_type: leaf_type.to_string(),
            merkle_root: merkle_root.clone().unwrap_or_default(),
            current_link_count: self.links.len(),
            content_hash: content_hash.clone(),
            additional_data: sort_map_for_verification(&additional_data),
        };

        // Serialize with CBOR
        let serialized = serde_cbor::to_vec(&leaf_data)
            .map_err(|e| ScionicError::Serialization(e.to_string()))?;

        // Create CID with SHA2-256 hash
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash_bytes = hasher.finalize();

        // Create multihash from the hash bytes
        let mh = Multihash::<64>::wrap(0x12, &hash_bytes) // 0x12 = SHA2-256
            .map_err(|e| ScionicError::InvalidCid(e.to_string()))?;

        let cid = Cid::new_v1(0x71, mh); // 0x71 = CBOR codec

        // Sort links (for directories only, preserve order for files)
        let mut sorted_links = self.links.clone();
        if leaf_type == LeafType::Directory {
            sorted_links.sort();
        }

        Ok(DagLeaf {
            hash: cid.to_string(),
            item_name: self.item_name,
            leaf_type,
            content_hash,
            content: self.data,
            classic_merkle_root: merkle_root,
            current_link_count: sorted_links.len(),
            leaf_count: None,
            content_size: None,
            dag_size: None,
            links: sorted_links,
            parent_hash: None,
            additional_data,
            proofs: None,
        })
    }

    /// Build a root leaf (includes leaf count and sizes)
    pub fn build_root_leaf(
        self,
        leaves: &HashMap<String, DagLeaf>,
        additional_data: Option<HashMap<String, String>>,
    ) -> Result<DagLeaf> {
        let leaf_type = self
            .leaf_type
            .ok_or_else(|| ScionicError::InvalidLeaf("Leaf must have a type".to_string()))?;

        // Build merkle root for links
        let merkle_root = if self.links.len() > 1 {
            let mut builder = MerkleTreeBuilder::new();
            for link in &self.links {
                builder.add_leaf(link.clone(), link.as_bytes().to_vec());
            }
            let tree = builder.build()?;
            Some(tree.root)
        } else if self.links.len() == 1 {
            Some(self.links[0].as_bytes().to_vec())
        } else {
            None
        };

        // Calculate content size
        let mut content_size: i64 = 0;
        for leaf in leaves.values() {
            if let Some(ref content) = leaf.content {
                content_size += content.len() as i64;
            }
        }
        if let Some(ref data) = self.data {
            content_size += data.len() as i64;
        }

        // Compute content hash
        let content_hash = self.data.as_ref().map(|data| {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        });

        let leaf_count = leaves.len() + 1; // +1 for root itself

        // Calculate DAG size (approximate for now)
        let dag_size: i64 = 0; // Will be calculated properly after leaf creation

        // Create leaf data for hashing
        #[derive(Serialize)]
        struct RootLeafData {
            #[serde(rename = "ItemName")]
            item_name: String,
            #[serde(rename = "Type")]
            leaf_type: String,
            #[serde(rename = "MerkleRoot")]
            merkle_root: Vec<u8>,
            #[serde(rename = "CurrentLinkCount")]
            current_link_count: usize,
            #[serde(rename = "LeafCount")]
            leaf_count: usize,
            #[serde(rename = "ContentSize")]
            content_size: i64,
            #[serde(rename = "DagSize")]
            dag_size: i64,
            #[serde(rename = "ContentHash")]
            content_hash: Option<Vec<u8>>,
            #[serde(rename = "AdditionalData")]
            additional_data: Vec<(String, String)>,
        }

        let leaf_data = RootLeafData {
            item_name: self.item_name.clone(),
            leaf_type: leaf_type.to_string(),
            merkle_root: merkle_root.clone().unwrap_or_default(),
            current_link_count: self.links.len(),
            leaf_count,
            content_size,
            dag_size,
            content_hash: content_hash.clone(),
            additional_data: sort_map_for_verification(&additional_data),
        };

        // Serialize with CBOR
        let serialized = serde_cbor::to_vec(&leaf_data)
            .map_err(|e| ScionicError::Serialization(e.to_string()))?;

        // Create CID with SHA2-256 hash
        let mut hasher_cid = Sha256::new();
        hasher_cid.update(&serialized);
        let hash_bytes = hasher_cid.finalize();

        // Create multihash from the hash bytes
        let mh = Multihash::<64>::wrap(0x12, &hash_bytes) // 0x12 = SHA2-256
            .map_err(|e| ScionicError::InvalidCid(e.to_string()))?;

        let cid = Cid::new_v1(0x71, mh); // 0x71 = CBOR codec

        // Sort links (for directories only)
        let mut sorted_links = self.links.clone();
        if leaf_type == LeafType::Directory {
            sorted_links.sort();
        }

        Ok(DagLeaf {
            hash: cid.to_string(),
            item_name: self.item_name,
            leaf_type,
            content_hash,
            content: self.data,
            classic_merkle_root: merkle_root,
            current_link_count: sorted_links.len(),
            leaf_count: Some(leaf_count),
            content_size: Some(content_size),
            dag_size: Some(dag_size),
            links: sorted_links,
            parent_hash: None,
            additional_data,
            proofs: None,
        })
    }
}

impl DagLeaf {
    /// Verify a regular (non-root) leaf
    pub fn verify_leaf(&self) -> Result<()> {
        #[derive(Serialize)]
        struct LeafData {
            #[serde(rename = "ItemName")]
            item_name: String,
            #[serde(rename = "Type")]
            leaf_type: String,
            #[serde(rename = "MerkleRoot")]
            merkle_root: Vec<u8>,
            #[serde(rename = "CurrentLinkCount")]
            current_link_count: usize,
            #[serde(rename = "ContentHash")]
            content_hash: Option<Vec<u8>>,
            #[serde(rename = "AdditionalData")]
            additional_data: Vec<(String, String)>,
        }

        let leaf_data = LeafData {
            item_name: self.item_name.clone(),
            leaf_type: self.leaf_type.to_string(),
            merkle_root: self.classic_merkle_root.clone().unwrap_or_default(),
            current_link_count: self.current_link_count,
            content_hash: self.content_hash.clone(),
            additional_data: sort_map_for_verification(&self.additional_data),
        };

        // Serialize with CBOR
        let serialized = serde_cbor::to_vec(&leaf_data)
            .map_err(|e| ScionicError::Serialization(e.to_string()))?;

        // Create CID with SHA2-256 hash
        let mut hasher_cid = Sha256::new();
        hasher_cid.update(&serialized);
        let hash_bytes = hasher_cid.finalize();

        // Create multihash from the hash bytes
        let mh = Multihash::<64>::wrap(0x12, &hash_bytes) // 0x12 = SHA2-256
            .map_err(|e| ScionicError::InvalidCid(e.to_string()))?;

        let cid = Cid::new_v1(0x71, mh); // 0x71 = CBOR codec

        // Compare with stored hash
        if cid.to_string() != self.hash {
            return Err(ScionicError::HashMismatch {
                expected: self.hash.clone(),
                got: cid.to_string(),
            });
        }

        Ok(())
    }

    /// Verify root leaf (includes leaf count and sizes)
    pub fn verify_root_leaf(&self) -> Result<()> {
        #[derive(Serialize)]
        struct RootLeafData {
            #[serde(rename = "ItemName")]
            item_name: String,
            #[serde(rename = "Type")]
            leaf_type: String,
            #[serde(rename = "MerkleRoot")]
            merkle_root: Vec<u8>,
            #[serde(rename = "CurrentLinkCount")]
            current_link_count: usize,
            #[serde(rename = "LeafCount")]
            leaf_count: usize,
            #[serde(rename = "ContentSize")]
            content_size: i64,
            #[serde(rename = "DagSize")]
            dag_size: i64,
            #[serde(rename = "ContentHash")]
            content_hash: Option<Vec<u8>>,
            #[serde(rename = "AdditionalData")]
            additional_data: Vec<(String, String)>,
        }

        let leaf_data = RootLeafData {
            item_name: self.item_name.clone(),
            leaf_type: self.leaf_type.to_string(),
            merkle_root: self.classic_merkle_root.clone().unwrap_or_default(),
            current_link_count: self.current_link_count,
            leaf_count: self.leaf_count.unwrap_or(0),
            content_size: self.content_size.unwrap_or(0),
            dag_size: self.dag_size.unwrap_or(0),
            content_hash: self.content_hash.clone(),
            additional_data: sort_map_for_verification(&self.additional_data),
        };

        // Serialize with CBOR
        let serialized = serde_cbor::to_vec(&leaf_data)
            .map_err(|e| ScionicError::Serialization(e.to_string()))?;

        // Create CID with SHA2-256 hash
        let mut hasher_cid = Sha256::new();
        hasher_cid.update(&serialized);
        let hash_bytes = hasher_cid.finalize();

        // Create multihash from the hash bytes
        let mh = Multihash::<64>::wrap(0x12, &hash_bytes) // 0x12 = SHA2-256
            .map_err(|e| ScionicError::InvalidCid(e.to_string()))?;

        let cid = Cid::new_v1(0x71, mh); // 0x71 = CBOR codec

        // Compare with stored hash
        if cid.to_string() != self.hash {
            return Err(ScionicError::HashMismatch {
                expected: self.hash.clone(),
                got: cid.to_string(),
            });
        }

        Ok(())
    }

    /// Check if this leaf has a specific link
    pub fn has_link(&self, hash: &str) -> bool {
        self.links.iter().any(|link| link == hash)
    }

    /// Get a Merkle branch/proof for a specific child
    pub fn get_branch(&self, key: &str) -> Result<Option<ClassicTreeBranch>> {
        if self.links.len() <= 1 {
            return Ok(None);
        }

        // Build merkle tree
        let mut builder = MerkleTreeBuilder::new();
        for link in &self.links {
            builder.add_leaf(link.clone(), link.as_bytes().to_vec());
        }
        let tree = builder.build()?;

        // Get proof for the key
        let index = tree
            .get_index_for_key(key)
            .ok_or_else(|| ScionicError::InvalidLeaf(format!("Key not found: {}", key)))?;

        Ok(Some(ClassicTreeBranch {
            leaf: key.to_string(),
            proof: tree.proofs[index].clone(),
        }))
    }
}
