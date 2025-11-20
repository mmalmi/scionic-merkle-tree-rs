use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScionicError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Hash mismatch: expected {expected}, got {got}")]
    HashMismatch { expected: String, got: String },

    #[error("Invalid leaf: {0}")]
    InvalidLeaf(String),

    #[error("Invalid DAG: {0}")]
    InvalidDag(String),

    #[error("Missing leaf: {0}")]
    MissingLeaf(String),

    #[error("Missing link: {0}")]
    MissingLink(String),

    #[error("Invalid proof")]
    InvalidProof,

    #[error("Merkle root mismatch")]
    MerkleRootMismatch,

    #[error("Invalid label: {0}")]
    InvalidLabel(String),

    #[error("Content hash mismatch")]
    ContentHashMismatch,

    #[error("Size mismatch: expected {expected}, got {got}")]
    SizeMismatch { expected: i64, got: i64 },

    #[error("Invalid CID: {0}")]
    InvalidCid(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid type: {0}")]
    InvalidType(String),
}

pub type Result<T> = std::result::Result<T, ScionicError>;
