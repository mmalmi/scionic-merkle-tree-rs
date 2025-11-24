//! # Scionic Merkle Tree - Rust Implementation
//!
//! A Rust implementation of Scionic Merkle Trees, combining the strengths of
//! Classic Merkle Trees and Merkle DAGs.
//!
//! ## Features
//!
//! - **Hybrid Structure**: Combines Classic Merkle Trees and Merkle DAGs
//! - **Folder Support**: Store and verify entire directory structures
//! - **Chunked Parent Leaves**: Parent leaves use Classic Merkle Trees for efficiency
//! - **LeafSync Protocol**: Request ranges of leaves by numeric labels
//! - **Compact Branches**: Logarithmic growth instead of linear
//! - **Cryptographic Verification**: CID-based hashing with SHA256
//!
//! ## Quick Start
//!
//! ```no_run
//! use scionic_merkle_tree_rs::{create_dag, Dag};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a DAG from a directory
//! let dag = create_dag("./my-directory", true)?;
//!
//! // Verify the DAG
//! dag.verify()?;
//!
//! // Save to file
//! dag.save_to_file("my-dag.cbor")?;
//!
//! // Load from file
//! let loaded_dag = Dag::load_from_file("my-dag.cbor")?;
//!
//! // Recreate the directory
//! loaded_dag.create_directory("./output-directory")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## LeafSync Protocol
//!
//! ```no_run
//! use scionic_merkle_tree_rs::{create_dag};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut dag = create_dag("./my-directory", false)?;
//!
//! // Calculate labels for all leaves
//! dag.calculate_labels()?;
//!
//! // Request a range of leaves (e.g., leaves 10-20)
//! let hashes = dag.get_hashes_by_label_range(10, 20)?;
//!
//! println!("Found {} leaves in range", hashes.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Serialization
//!
//! ```no_run
//! use scionic_merkle_tree_rs::{create_dag, Dag};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let dag = create_dag("./my-directory", false)?;
//!
//! // Serialize to JSON
//! let json = dag.to_json()?;
//!
//! // Serialize to CBOR (more compact)
//! let cbor = dag.to_cbor()?;
//!
//! // Deserialize
//! let dag_from_json = Dag::from_json(&json)?;
//! let dag_from_cbor = Dag::from_cbor(&cbor)?;
//! # Ok(())
//! # }
//! ```

pub mod dag;
pub mod error;
pub mod leaf;
pub mod merkle_tree;
pub mod serialize;
pub mod streaming;
pub mod types;

// Re-export commonly used items
pub use dag::{create_dag, create_dag_with_config};
pub use error::{Result, ScionicError};
pub use streaming::{create_dag_from_stream, StreamingDagBuilder};
pub use types::{
    ClassicTreeBranch, Dag, DagBuilderConfig, DagLeaf, DagLeafBuilder, LeafType, MerkleProof,
    TransmissionPacket, DEFAULT_CHUNK_SIZE,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_end_to_end() -> Result<()> {
        // Create a temporary directory with test files
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        fs::create_dir(&input_dir)?;

        fs::write(input_dir.join("file1.txt"), b"Hello, World!")?;
        fs::write(input_dir.join("file2.txt"), b"Rust is awesome!")?;

        let subdir = input_dir.join("subdir");
        fs::create_dir(&subdir)?;
        fs::write(subdir.join("file3.txt"), b"Nested file content")?;

        // Create DAG
        let dag = create_dag(&input_dir, true)?;

        // Verify DAG
        dag.verify()?;

        // Save and load
        let dag_file = temp_dir.path().join("test.dag");
        dag.save_to_file(&dag_file)?;
        let loaded_dag = Dag::load_from_file(&dag_file)?;

        // Verify loaded DAG
        loaded_dag.verify()?;

        // Recreate directory
        let output_dir = temp_dir.path().join("output");
        loaded_dag.create_directory(&output_dir)?;

        // Verify files were recreated
        assert!(output_dir.exists());
        assert!(output_dir.join("file1.txt").exists());
        assert!(output_dir.join("file2.txt").exists());
        assert!(output_dir.join("subdir").join("file3.txt").exists());

        // Verify content
        let content1 = fs::read_to_string(output_dir.join("file1.txt"))?;
        assert_eq!(content1, "Hello, World!");

        Ok(())
    }

    #[test]
    fn test_leafsync() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_dir = temp_dir.path().join("input");
        fs::create_dir(&input_dir)?;

        for i in 0..10 {
            fs::write(
                input_dir.join(format!("file{}.txt", i)),
                format!("Content {}", i),
            )?;
        }

        let mut dag = create_dag(&input_dir, false)?;
        dag.calculate_labels()?;

        // Request a range
        let hashes = dag.get_hashes_by_label_range(1, 5)?;
        assert_eq!(hashes.len(), 5);

        Ok(())
    }

    #[test]
    fn test_transmission_packets() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"Test content for transmission")?;

        let dag = create_dag(&file_path, false)?;

        // Get transmission packets
        let packets = dag.get_leaf_sequence();
        assert!(!packets.is_empty());

        // Create a new DAG and apply packets
        let mut new_dag = Dag {
            root: dag.root.clone(),
            leaves: std::collections::HashMap::new(),
            labels: None,
        };

        for packet in packets {
            new_dag.apply_and_verify_transmission_packet(packet)?;
        }

        // Verify the reconstructed DAG
        new_dag.verify()?;

        Ok(())
    }
}
