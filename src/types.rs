use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod serde_base64_option {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{engine::general_purpose::STANDARD, Engine};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => {
                let encoded = STANDARD.encode(bytes);
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        Ok(opt.map(|s| STANDARD.decode(s).unwrap()))
    }
}

/// Type of leaf in the DAG
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeafType {
    File,
    Chunk,
    Directory,
}

impl std::fmt::Display for LeafType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafType::File => write!(f, "file"),
            LeafType::Chunk => write!(f, "chunk"),
            LeafType::Directory => write!(f, "directory"),
        }
    }
}

/// A leaf in the Scionic Merkle DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagLeaf {
    /// CID hash of this leaf
    #[serde(rename = "Hash")]
    pub hash: String,

    /// Name/path of the item
    #[serde(rename = "ItemName")]
    pub item_name: String,

    /// Type of leaf
    #[serde(rename = "Type")]
    pub leaf_type: LeafType,

    /// Hash of the content (SHA256)
    #[serde(
        rename = "ContentHash",
        skip_serializing_if = "Option::is_none",
        default,
        with = "serde_base64_option"
    )]
    pub content_hash: Option<Vec<u8>>,

    /// Actual content bytes
    #[serde(
        rename = "Content",
        skip_serializing_if = "Option::is_none",
        default,
        with = "serde_base64_option"
    )]
    pub content: Option<Vec<u8>>,

    /// Classic Merkle tree root for children
    #[serde(
        rename = "ClassicMerkleRoot",
        skip_serializing_if = "Option::is_none",
        default,
        with = "serde_base64_option"
    )]
    pub classic_merkle_root: Option<Vec<u8>>,

    /// Number of links this leaf has
    #[serde(rename = "CurrentLinkCount")]
    pub current_link_count: usize,

    /// Total number of leaves in DAG (root only)
    #[serde(rename = "LeafCount", skip_serializing_if = "Option::is_none")]
    pub leaf_count: Option<usize>,

    /// Total content size (root only)
    #[serde(rename = "ContentSize", skip_serializing_if = "Option::is_none")]
    pub content_size: Option<i64>,

    /// Total DAG size (root only)
    #[serde(rename = "DagSize", skip_serializing_if = "Option::is_none")]
    pub dag_size: Option<i64>,

    /// Links to child leaves (hashes)
    #[serde(rename = "Links", skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<String>,

    /// Parent hash (for traversal, not verified)
    #[serde(rename = "ParentHash", skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<String>,

    /// Additional metadata
    #[serde(rename = "AdditionalData", skip_serializing_if = "Option::is_none")]
    pub additional_data: Option<HashMap<String, String>>,

    /// Merkle proofs for partial DAG verification
    #[serde(rename = "stored_proofs", skip_serializing_if = "Option::is_none")]
    pub proofs: Option<HashMap<String, ClassicTreeBranch>>,
}

/// Classic Merkle tree branch/proof for a specific leaf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicTreeBranch {
    /// The leaf hash this proof is for
    #[serde(rename = "Leaf")]
    pub leaf: String,

    /// The Merkle proof
    #[serde(rename = "Proof")]
    pub proof: MerkleProof,
}

/// Merkle proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Sibling hashes along the path to root
    #[serde(rename = "Siblings")]
    pub siblings: Vec<serde_bytes::ByteBuf>,

    /// Path bitmap (uint32) indicating whether sibling is on left (0) or right (1)
    #[serde(rename = "Path")]
    pub path: u32,
}

/// The main Scionic Merkle DAG structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dag {
    /// Root leaf hash
    #[serde(rename = "Root")]
    pub root: String,

    /// All leaves indexed by hash
    #[serde(rename = "Leafs")]
    pub leaves: HashMap<String, DagLeaf>,

    /// Labels mapping (numeric labels to hashes)
    #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

/// Transmission packet for syncing individual leaves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmissionPacket {
    /// The leaf being transmitted
    #[serde(rename = "Leaf")]
    pub leaf: DagLeaf,

    /// Parent hash for context
    #[serde(rename = "ParentHash")]
    pub parent_hash: String,

    /// Merkle proofs needed to verify this leaf
    #[serde(rename = "proofs", skip_serializing_if = "HashMap::is_empty", default)]
    pub proofs: HashMap<String, ClassicTreeBranch>,
}

/// Configuration for DAG building
#[derive(Debug, Clone)]
pub struct DagBuilderConfig {
    /// Enable parallel processing
    pub enable_parallel: bool,

    /// Maximum number of workers (0 = auto)
    pub max_workers: usize,

    /// Add timestamp to root
    pub timestamp_root: bool,

    /// Additional metadata for root
    pub additional_data: HashMap<String, String>,

    /// Chunk size (None = use default, Some(0) = disable chunking)
    pub chunk_size: Option<usize>,
}

impl Default for DagBuilderConfig {
    fn default() -> Self {
        Self {
            enable_parallel: false,
            max_workers: 0,
            timestamp_root: false,
            additional_data: HashMap::new(),
            chunk_size: None,
        }
    }
}

impl DagBuilderConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parallel(mut self) -> Self {
        self.enable_parallel = true;
        self
    }

    pub fn with_workers(mut self, workers: usize) -> Self {
        self.max_workers = workers;
        self
    }

    pub fn with_timestamp(mut self) -> Self {
        self.timestamp_root = true;
        self
    }

    pub fn with_additional_data(mut self, data: HashMap<String, String>) -> Self {
        self.additional_data = data;
        self
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    pub fn without_chunking(mut self) -> Self {
        self.chunk_size = Some(0);
        self
    }
}

/// Chunk size configuration
pub const DEFAULT_CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2MB

/// Builder for constructing DAG leaves
pub struct DagLeafBuilder {
    pub(crate) item_name: String,
    pub(crate) leaf_type: Option<LeafType>,
    pub(crate) data: Option<Vec<u8>>,
    pub(crate) links: Vec<String>,
}

impl DagLeafBuilder {
    pub fn new(item_name: impl Into<String>) -> Self {
        Self {
            item_name: item_name.into(),
            leaf_type: None,
            data: None,
            links: Vec::new(),
        }
    }

    pub fn set_type(mut self, leaf_type: LeafType) -> Self {
        self.leaf_type = Some(leaf_type);
        self
    }

    pub fn set_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn add_link(mut self, hash: String) -> Self {
        self.links.push(hash);
        self
    }
}
