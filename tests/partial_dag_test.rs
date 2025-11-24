use scionic_merkle_tree_rs::{create_dag, LeafType, Result};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_get_partial_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Create multiple files
    for i in 0..5 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Collect file leaf hashes
    let file_hashes: Vec<String> = dag
        .leaves
        .iter()
        .filter(|(_, leaf)| leaf.leaf_type == LeafType::File)
        .map(|(hash, _)| hash.clone())
        .collect();

    assert!(file_hashes.len() >= 2, "Need at least 2 files for test");

    // Get partial with first 2 files
    let partial_hashes = vec![file_hashes[0].clone(), file_hashes[1].clone()];
    let partial = dag.get_partial(&partial_hashes, true)?;

    // Verify partial
    partial.verify()?;

    // Should be marked as partial
    assert!(partial.is_partial());

    // Should have fewer leaves than full DAG
    assert!(partial.leaves.len() < dag.leaves.len());

    Ok(())
}

#[test]
fn test_get_partial_single_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "single file content")?;

    let dag = create_dag(&file, false)?;

    // Get the file hash
    let file_hash = dag
        .leaves
        .iter()
        .find(|(_, leaf)| leaf.leaf_type == LeafType::File)
        .map(|(hash, _)| hash.clone())
        .expect("No file leaf found");

    // Get "partial" (actually full for single file)
    let partial = dag.get_partial(&[file_hash], true)?;
    partial.verify()?;

    Ok(())
}

#[test]
fn test_get_partial_nested() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    let subdir = dir.join("subdir");
    fs::create_dir(&subdir)?;

    fs::write(dir.join("file1.txt"), "content1")?;
    fs::write(subdir.join("file2.txt"), "content2")?;
    fs::write(subdir.join("file3.txt"), "content3")?;

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Get a file from subdirectory
    let file_hashes: Vec<String> = dag
        .leaves
        .iter()
        .filter(|(_, leaf)| leaf.leaf_type == LeafType::File)
        .map(|(hash, _)| hash.clone())
        .take(1)
        .collect();

    let partial = dag.get_partial(&file_hashes, true)?;
    partial.verify()?;

    // Partial should include the file, parent dirs, and root
    assert!(partial.leaves.len() >= 2);

    Ok(())
}

#[test]
fn test_get_partial_deep_hierarchy() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut current = temp_dir.path().join("root");
    fs::create_dir(&current)?;

    // Create 3 levels of nesting with multiple files
    for i in 0..3 {
        current = current.join(format!("level{}", i));
        fs::create_dir(&current)?;
        // Add a file at each level
        fs::write(
            current.join(format!("file{}.txt", i)),
            format!("content{}", i),
        )?;
    }

    let root = temp_dir.path().join("root");
    let dag = create_dag(&root, false)?;
    dag.verify()?;

    // Get just one file from deep in hierarchy
    let file_hash = dag
        .leaves
        .iter()
        .find(|(_, leaf)| leaf.leaf_type == LeafType::File)
        .map(|(hash, _)| hash.clone())
        .expect("No file found");

    let partial = dag.get_partial(&[file_hash], true)?;
    partial.verify()?;

    // Partial should have verification path
    if dag.leaves.len() > 1 {
        assert!(partial.is_partial());
        assert!(partial.leaves.len() < dag.leaves.len());
    }

    Ok(())
}

#[test]
fn test_get_partial_errors() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "content")?;

    let dag = create_dag(&file, false)?;

    // Empty array should error
    assert!(dag.get_partial(&[], true).is_err());

    // Invalid hash should error
    assert!(dag
        .get_partial(&["invalid_hash_12345".to_string()], true)
        .is_err());

    Ok(())
}

#[test]
fn test_partial_dag_verification_with_proofs() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Create enough files to ensure merkle proofs are needed
    for i in 0..10 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Get partial with just one file
    let file_hash = dag
        .leaves
        .iter()
        .find(|(_, leaf)| leaf.leaf_type == LeafType::File)
        .map(|(hash, _)| hash.clone())
        .expect("No file found");

    let partial = dag.get_partial(&[file_hash], true)?;

    // Partial verification should use proofs
    partial.verify()?;

    assert!(partial.is_partial());

    Ok(())
}
