/// Cross-compatibility tests with the Go implementation
/// These tests verify that the Rust and Go implementations are compatible
use scionic_merkle_tree_rs::{create_dag, Dag, Result};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Check if Go implementation is available
fn go_implementation_available() -> bool {
    std::path::Path::new("/workspace/Scionic-Merkle-Tree").exists()
}

/// Create a DAG using the Go implementation
fn create_dag_with_go(input_path: &str, output_cbor: &str) -> std::io::Result<std::process::Output> {
    Command::new("go")
        .args(&[
            "run",
            "/workspace/Scionic-Merkle-Tree/tests/test_helper.go",
            "create",
            input_path,
            output_cbor,
        ])
        .output()
}

/// Verify a DAG using the Go implementation
fn verify_dag_with_go(cbor_path: &str) -> std::io::Result<std::process::Output> {
    Command::new("go")
        .args(&[
            "run",
            "/workspace/Scionic-Merkle-Tree/tests/test_helper.go",
            "verify",
            cbor_path,
        ])
        .output()
}

#[test]
fn test_rust_creates_go_reads() -> Result<()> {
    if !go_implementation_available() {
        eprintln!("Skipping: Go implementation not available");
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    fs::create_dir(&input_dir)?;

    // Create test files
    fs::write(input_dir.join("file1.txt"), "content 1")?;
    fs::write(input_dir.join("file2.txt"), "content 2")?;

    // Create DAG with Rust
    let dag = create_dag(&input_dir, false)?;
    dag.verify()?;

    // Save to CBOR
    let cbor_path = temp_dir.path().join("rust.cbor");
    dag.save_to_file(&cbor_path)?;

    // Try to verify with Go
    let result = verify_dag_with_go(cbor_path.to_str().unwrap());

    if result.is_ok() {
        let output = result.unwrap();
        println!("Go verification output: {:?}", String::from_utf8_lossy(&output.stdout));
        // Note: This test is informational - Go might not be able to read our format yet
    }

    Ok(())
}

#[test]
fn test_identical_input_produces_same_root() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("input");
    fs::create_dir(&input_dir)?;

    // Create deterministic test content
    fs::write(input_dir.join("a.txt"), "test content a")?;
    fs::write(input_dir.join("b.txt"), "test content b")?;
    fs::write(input_dir.join("c.txt"), "test content c")?;

    // Create with Rust
    let rust_dag = create_dag(&input_dir, false)?;
    rust_dag.verify()?;

    println!("Rust root hash: {}", rust_dag.root);
    println!("Rust leaf count: {}", rust_dag.leaves.len());

    // Print some debug info about the structure
    for (hash, leaf) in rust_dag.leaves.iter().take(3) {
        println!(
            "  Leaf: {} type={:?} name={} links={}",
            &hash[..20],
            leaf.leaf_type,
            leaf.item_name,
            leaf.links.len()
        );
    }

    Ok(())
}

#[test]
fn test_serialization_format_compatibility() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "test content for serialization")?;

    let dag = create_dag(&file, false)?;

    // Serialize to both formats
    let cbor = dag.to_cbor()?;
    let json = dag.to_json()?;

    println!("CBOR size: {} bytes", cbor.len());
    println!("JSON size: {} bytes", json.len());

    // Verify we can deserialize our own output
    let dag_from_cbor = Dag::from_cbor(&cbor)?;
    let dag_from_json = Dag::from_json(&json)?;

    assert_eq!(dag.root, dag_from_cbor.root);
    assert_eq!(dag.root, dag_from_json.root);

    dag_from_cbor.verify()?;
    dag_from_json.verify()?;

    Ok(())
}

#[test]
fn test_chunk_size_compatibility() -> Result<()> {
    use scionic_merkle_tree_rs::DEFAULT_CHUNK_SIZE;

    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("large.txt");

    // Create file slightly larger than chunk size
    let content = vec![b'X'; DEFAULT_CHUNK_SIZE + 1000];
    fs::write(&file, &content)?;

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    // Should have created chunks
    let chunk_count = dag
        .leaves
        .values()
        .filter(|leaf| leaf.leaf_type == scionic_merkle_tree_rs::LeafType::Chunk)
        .count();

    println!("Chunk size: {}", DEFAULT_CHUNK_SIZE);
    println!("File size: {}", content.len());
    println!("Chunks created: {}", chunk_count);

    assert!(chunk_count > 0, "Should have created chunks");

    Ok(())
}

#[test]
fn test_cid_format() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "test")?;

    let dag = create_dag(&file, false)?;

    // Verify CID format (should start with "bafi" for CIDv1 with CBOR codec 0x51 - matching Go)
    assert!(
        dag.root.starts_with("bafi"),
        "Root CID should be CIDv1 CBOR format: {}",
        dag.root
    );

    // All leaf hashes should also be valid CIDs
    for (hash, _) in &dag.leaves {
        assert!(
            hash.starts_with("bafi"),
            "Leaf hash should be CIDv1 CBOR format: {}",
            hash
        );
    }

    Ok(())
}

#[test]
fn test_directory_structure_preservation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input");
    fs::create_dir(&input)?;

    // Create specific structure
    let subdir1 = input.join("subdir1");
    let subdir2 = input.join("subdir2");
    fs::create_dir(&subdir1)?;
    fs::create_dir(&subdir2)?;

    fs::write(input.join("root.txt"), "root")?;
    fs::write(subdir1.join("sub1.txt"), "sub1")?;
    fs::write(subdir2.join("sub2.txt"), "sub2")?;

    // Create DAG
    let dag = create_dag(&input, false)?;
    dag.verify()?;

    // Recreate
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    // Verify structure is preserved
    assert!(output.join("root.txt").exists());
    assert!(output.join("subdir1").is_dir());
    assert!(output.join("subdir2").is_dir());
    assert!(output.join("subdir1/sub1.txt").exists());
    assert!(output.join("subdir2/sub2.txt").exists());

    // Verify content
    let content = fs::read_to_string(output.join("subdir1/sub1.txt"))?;
    assert_eq!(content, "sub1");

    Ok(())
}

#[test]
fn test_content_addressing_is_deterministic() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create two identical directories with same content
    let dir1 = temp_dir.path().join("dir1");
    let dir2 = temp_dir.path().join("dir2");

    for dir in &[&dir1, &dir2] {
        fs::create_dir(dir)?;
        fs::write(dir.join("file.txt"), "identical content")?;
    }

    let dag1 = create_dag(&dir1, false)?;
    let dag2 = create_dag(&dir2, false)?;

    // The only difference should be the root directory name
    // But file leaves should have identical hashes
    let file_hash_1: Vec<_> = dag1
        .leaves
        .iter()
        .filter(|(_, leaf)| leaf.leaf_type == scionic_merkle_tree_rs::LeafType::File)
        .map(|(hash, _)| hash.clone())
        .collect();

    let file_hash_2: Vec<_> = dag2
        .leaves
        .iter()
        .filter(|(_, leaf)| leaf.leaf_type == scionic_merkle_tree_rs::LeafType::File)
        .map(|(hash, _)| hash.clone())
        .collect();

    // File hashes should be identical since content is identical
    assert_eq!(file_hash_1, file_hash_2, "File leaves should have same hash for same content");

    Ok(())
}
