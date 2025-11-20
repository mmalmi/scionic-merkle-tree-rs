# Scionic Merkle Tree - Rust Implementation

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of **Scionic Merkle Trees**, a novel data structure that combines the strengths of Classic Merkle Trees and Merkle DAGs.

## What are Scionic Merkle Trees?

Scionic Merkle Trees are a hybrid data structure designed for efficient storage and verification of files and directories. They maintain the advantages of IPFS Merkle DAGs with the slim Merkle branches of Classic Merkle Trees, while providing LeafSync as a new feature that complements set reconciliation systems.

### Key Features

- **Hybrid Structure**: Combines Classic Merkle Trees and Merkle DAGs
- **Folder Support**: Store and verify entire directory structures
- **Chunked Parent Leaves**: Parent leaves use Classic Merkle Trees for efficiency
- **LeafSync Protocol**: Request ranges of leaves by numeric labels
- **Compact Branches**: Logarithmic growth instead of linear (50,000x smaller for 1M files)
- **Cryptographic Verification**: CID-based hashing with SHA256 and CBOR encoding
- **File Chunking**: Automatic chunking for large files (default 2MB)

### Performance Comparison

For a folder containing **1,000,000 files**:
- **Scionic Merkle Branch**: ~21 leaves required
- **IPFS Merkle DAG Branch**: 1,000,000 leaves required
- **Size Reduction**: **~50,000x smaller**

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
scionic-merkle-tree-rs = "0.1.0"
```

## Quick Start

```rust
use scionic_merkle_tree_rs::{create_dag, Result};

fn main() -> Result<()> {
    // Create a DAG from a directory
    let dag = create_dag("./my-directory", true)?;

    // Verify the DAG
    dag.verify()?;

    // Save to file
    dag.save_to_file("my-dag.cbor")?;

    // Load from file
    let loaded_dag = Dag::load_from_file("my-dag.cbor")?;

    // Recreate the directory
    loaded_dag.create_directory("./output-directory")?;

    Ok(())
}
```

## Examples

### Creating a DAG from Files

```rust
use scionic_merkle_tree_rs::{create_dag, Result};

fn main() -> Result<()> {
    // Create a DAG with timestamp
    let dag = create_dag("./my-files", true)?;

    println!("Root CID: {}", dag.root);
    println!("Total leaves: {}", dag.leaves.len());

    // Verify integrity
    dag.verify()?;

    Ok(())
}
```

### LeafSync Protocol

LeafSync allows you to request specific ranges of leaves by numeric labels:

```rust
use scionic_merkle_tree_rs::{create_dag, Result};

fn main() -> Result<()> {
    let mut dag = create_dag("./my-directory", false)?;

    // Calculate labels for all leaves
    dag.calculate_labels()?;

    // Request leaves 10-20
    let hashes = dag.get_hashes_by_label_range(10, 20)?;

    println!("Retrieved {} leaves", hashes.len());

    // Use these hashes to request specific data from peers
    for hash in hashes {
        println!("Leaf hash: {}", hash);
    }

    Ok(())
}
```

### Serialization Options

```rust
use scionic_merkle_tree_rs::{create_dag, Dag, Result};

fn main() -> Result<()> {
    let dag = create_dag("./my-directory", false)?;

    // JSON serialization (human-readable)
    let json = dag.to_json_pretty()?;
    std::fs::write("dag.json", json)?;

    // CBOR serialization (compact binary)
    let cbor = dag.to_cbor()?;
    std::fs::write("dag.cbor", cbor)?;

    // Deserialize
    let dag_from_json = Dag::from_json(&std::fs::read("dag.json")?)?;
    let dag_from_cbor = Dag::from_cbor(&std::fs::read("dag.cbor")?)?;

    Ok(())
}
```

### Transmission Packets

For efficient syncing over networks:

```rust
use scionic_merkle_tree_rs::{create_dag, Dag, Result};
use std::collections::HashMap;

fn main() -> Result<()> {
    let dag = create_dag("./my-directory", false)?;

    // Get all leaves as transmission packets
    let packets = dag.get_leaf_sequence();

    // Send packets over network...
    // On the receiving end:
    let mut received_dag = Dag {
        root: dag.root.clone(),
        leaves: HashMap::new(),
        labels: None,
    };

    for packet in packets {
        // Verify and apply each packet
        received_dag.apply_and_verify_transmission_packet(packet)?;
    }

    // Verify the complete received DAG
    received_dag.verify()?;

    Ok(())
}
```

### Custom Configuration

```rust
use scionic_merkle_tree_rs::{create_dag_with_config, DagBuilderConfig, Result};
use std::collections::HashMap;

fn main() -> Result<()> {
    let mut additional_data = HashMap::new();
    additional_data.insert("author".to_string(), "example".to_string());
    additional_data.insert("version".to_string(), "1.0.0".to_string());

    let config = DagBuilderConfig::new()
        .with_timestamp()
        .with_additional_data(additional_data);

    let dag = create_dag_with_config("./my-directory", config)?;

    dag.verify()?;

    Ok(())
}
```

## Architecture

### DAG Structure

```
Root (Directory)
├── File1.txt (single leaf if < 2MB)
├── LargeFile.zip
│   ├── Chunk 0
│   ├── Chunk 1
│   └── Chunk 2
└── Subfolder
    ├── File2.txt
    └── File3.txt
```

### Leaf Types

- **File**: Represents a complete file (if smaller than chunk size)
- **Chunk**: Represents a chunk of a large file
- **Directory**: Represents a folder containing files and subdirectories

### Verification

Each leaf contains:
- **Hash**: Content-addressed CID using SHA256 and CBOR
- **ContentHash**: SHA256 of the actual content
- **ClassicMerkleRoot**: Merkle root of child leaves
- **CurrentLinkCount**: Number of children (for verification)

## Performance

- **File Chunking**: Default 2MB chunks
- **Branch Size**: Logarithmic growth (log₂ n)
- **Verification**: O(log n) for partial DAGs
- **Storage**: Compact CBOR encoding

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

## Contributing

Contributions are welcome! This is a Rust port of the original Go implementation at [HORNET-Storage/Scionic-Merkle-Tree](https://github.com/HORNET-Storage/Scionic-Merkle-Tree).

## License

MIT License - see [LICENSE](LICENSE) file for details

## Learn More

- [Original Go Implementation](https://github.com/HORNET-Storage/Scionic-Merkle-Tree)
- [HORNET Storage](https://www.hornet.storage/)
- [Nostr Integration](https://github.com/damus-io/notedeck)

## Acknowledgments

This is a Rust implementation of the Scionic Merkle Tree data structure designed by the HORNET Storage team.
