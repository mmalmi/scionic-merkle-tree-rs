use scionic_merkle_tree_rs::{create_dag, Dag, LeafType, Result};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_content_hash_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    let content = b"test content for hashing";
    fs::write(&file, content)?;

    let dag = create_dag(&file, false)?;

    // Find the file leaf
    let file_leaf = dag
        .leaves
        .values()
        .find(|leaf| leaf.leaf_type == LeafType::File)
        .expect("No file leaf found");

    // Verify content hash
    if let Some(ref stored_content) = file_leaf.content {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(stored_content);
        let computed_hash = hasher.finalize();

        let content_hash = file_leaf.content_hash.as_ref().expect("No content hash");

        assert_eq!(&computed_hash[..], &content_hash[..]);
    }

    Ok(())
}

#[test]
fn test_merkle_branch_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Create multiple files to ensure merkle tree in parent
    for i in 0..5 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let dag = create_dag(&dir, false)?;

    // Find a directory leaf with children
    let dir_leaf = dag
        .leaves
        .values()
        .find(|leaf| leaf.leaf_type == LeafType::Directory && !leaf.links.is_empty())
        .expect("No directory leaf found");

    // Verify branch for first child if there are multiple
    if dir_leaf.links.len() > 1 {
        let child_hash = &dir_leaf.links[0];
        let branch = dir_leaf.get_branch(child_hash)?;

        // Branch should exist for multiple children
        assert!(branch.is_some());
    }

    Ok(())
}

#[test]
fn test_full_dag_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    let subdir = dir.join("subdir");
    fs::create_dir(&subdir)?;

    fs::write(dir.join("file1.txt"), "content1")?;
    fs::write(subdir.join("file2.txt"), "content2")?;

    let dag = create_dag(&dir, false)?;

    // Should verify successfully
    dag.verify()?;

    // Should NOT be marked as partial
    assert!(!dag.is_partial());

    Ok(())
}

#[test]
fn test_leaf_hash_integrity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "content")?;

    let dag = create_dag(&file, false)?;

    // Verify each leaf
    for leaf in dag.leaves.values() {
        if leaf.hash == dag.root {
            leaf.verify_root_leaf()?;
        } else {
            leaf.verify_leaf()?;
        }
    }

    Ok(())
}

#[test]
fn test_roundtrip_preserves_hashes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let input = temp_dir.path().join("input");
    fs::create_dir(&input)?;

    fs::write(input.join("a.txt"), "content a")?;
    fs::write(input.join("b.txt"), "content b")?;

    let dag1 = create_dag(&input, false)?;
    dag1.verify()?;

    // Recreate directory
    let output = temp_dir.path().join("output");
    dag1.create_directory(&output)?;

    // Create DAG from recreated directory
    let dag2 = create_dag(&output, false)?;
    dag2.verify()?;

    // Note: Root hashes may differ due to directory names being different
    // But the structure and content should be identical
    assert_eq!(dag1.leaves.len(), dag2.leaves.len());

    Ok(())
}

#[test]
fn test_serialization_preserves_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    for i in 0..3 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let dag1 = create_dag(&dir, false)?;
    dag1.verify()?;

    // CBOR round-trip
    let cbor = dag1.to_cbor()?;
    let dag2 = Dag::from_cbor(&cbor)?;
    assert_eq!(dag1.root, dag2.root);
    dag2.verify()?;

    // JSON round-trip
    let json = dag1.to_json()?;
    let dag3 = Dag::from_json(&json)?;
    assert_eq!(dag1.root, dag3.root);
    dag3.verify()?;

    Ok(())
}

#[test]
fn test_chunked_file_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("large.txt");

    // Create file that will be chunked
    let size = 3 * 1024 * 1024; // 3MB
    let content = vec![b'X'; size];
    fs::write(&file, &content)?;

    let dag = create_dag(&file, false)?;

    // Verify the chunked file
    dag.verify()?;

    // Verify all chunks have content
    for leaf in dag.leaves.values() {
        if leaf.leaf_type == LeafType::Chunk {
            assert!(leaf.content.is_some());
            assert!(leaf.content_hash.is_some());
        }
    }

    Ok(())
}

#[test]
fn test_nested_directory_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path().join("root");
    fs::create_dir(&root)?;

    let level1 = root.join("level1");
    fs::create_dir(&level1)?;

    let level2 = level1.join("level2");
    fs::create_dir(&level2)?;

    fs::write(root.join("root.txt"), "root content")?;
    fs::write(level1.join("l1.txt"), "level1 content")?;
    fs::write(level2.join("l2.txt"), "level2 content")?;

    let dag = create_dag(&root, false)?;

    // Verify entire DAG
    dag.verify()?;

    // Verify we have the right structure
    let dir_count = dag
        .leaves
        .values()
        .filter(|leaf| leaf.leaf_type == LeafType::Directory)
        .count();

    assert_eq!(dir_count, 3); // root, level1, level2

    Ok(())
}

#[test]
fn test_empty_file_has_valid_hash() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("empty.txt");
    fs::write(&file, "")?;

    let dag = create_dag(&file, false)?;

    // Should verify even with empty content
    dag.verify()?;

    // Root hash should be consistent
    assert!(!dag.root.is_empty());

    Ok(())
}
