use scionic_merkle_tree_rs::{create_dag, LeafType, Result};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_empty_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("empty.txt");
    fs::write(&file, "")?;

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    // Recreate and verify
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let content = fs::read(output.join("empty.txt"))?;
    assert_eq!(content.len(), 0);

    Ok(())
}

#[test]
fn test_large_file_chunking() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("large.txt");

    // Create file larger than default chunk size (2MB)
    let size = 3 * 1024 * 1024; // 3MB
    let content = vec![b'A'; size];
    fs::write(&file, &content)?;

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    // Should have chunk leaves
    let chunk_count = dag
        .leaves
        .values()
        .filter(|leaf| leaf.leaf_type == LeafType::Chunk)
        .count();

    assert!(chunk_count > 0, "Large file should be chunked");

    // Recreate and verify content matches
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let recreated = fs::read(output.join("large.txt"))?;
    assert_eq!(recreated, content);

    Ok(())
}

#[test]
fn test_special_characters_in_filename() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Note: Some special characters may not be valid on all filesystems
    let filenames = vec!["file with spaces.txt", "file-dash.txt", "file_underscore.txt"];

    for name in &filenames {
        fs::write(dir.join(name), "content")?;
    }

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Recreate and verify
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    for name in &filenames {
        assert!(output.join(name).exists());
    }

    Ok(())
}

#[test]
fn test_deeply_nested_directories() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut current = temp_dir.path().join("root");
    fs::create_dir(&current)?;

    // Create 5 levels of nesting
    for i in 0..5 {
        current = current.join(format!("level{}", i));
        fs::create_dir(&current)?;
    }

    // Add file at deepest level
    fs::write(current.join("deep.txt"), "content")?;

    let root = temp_dir.path().join("root");
    let dag = create_dag(&root, false)?;
    dag.verify()?;

    // Recreate and verify structure
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let mut check = output.clone();
    for i in 0..5 {
        check = check.join(format!("level{}", i));
        assert!(check.exists());
    }
    assert!(check.join("deep.txt").exists());

    Ok(())
}

#[test]
fn test_single_file_at_chunk_boundary() -> Result<()> {
    use scionic_merkle_tree_rs::DEFAULT_CHUNK_SIZE;

    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("boundary.txt");

    // Create file exactly at chunk size
    let content = vec![b'B'; DEFAULT_CHUNK_SIZE];
    fs::write(&file, &content)?;

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    // Recreate
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let recreated = fs::read(output.join("boundary.txt"))?;
    assert_eq!(recreated, content);

    Ok(())
}

#[test]
fn test_binary_file_content() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("binary.bin");

    // Binary content with null bytes and all byte values
    let content: Vec<u8> = (0..=255).cycle().take(1000).collect();
    fs::write(&file, &content)?;

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    // Recreate and verify exact binary match
    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let recreated = fs::read(output.join("binary.bin"))?;
    assert_eq!(recreated, content);

    Ok(())
}

#[test]
fn test_empty_directory() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("empty_dir");
    fs::create_dir(&dir)?;

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Directory leaf should have no children
    let root_leaf = dag.leaves.get(&dag.root).unwrap();
    assert_eq!(root_leaf.links.len(), 0);

    Ok(())
}

#[test]
fn test_directory_with_only_subdirectories() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Create subdirectories without files
    fs::create_dir(dir.join("sub1"))?;
    fs::create_dir(dir.join("sub2"))?;
    fs::create_dir(dir.join("sub3"))?;

    let dag = create_dag(&dir, false)?;
    dag.verify()?;

    // Should have directory leaves but no file leaves
    let file_count = dag
        .leaves
        .values()
        .filter(|leaf| leaf.leaf_type == LeafType::File)
        .count();

    assert_eq!(file_count, 0);

    Ok(())
}

#[test]
fn test_very_small_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("tiny.txt");
    fs::write(&file, "x")?; // Single byte

    let dag = create_dag(&file, false)?;
    dag.verify()?;

    let output = temp_dir.path().join("output");
    dag.create_directory(&output)?;

    let content = fs::read_to_string(output.join("tiny.txt"))?;
    assert_eq!(content, "x");

    Ok(())
}
