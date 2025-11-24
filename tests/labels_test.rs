use scionic_merkle_tree_rs::{create_dag, Result};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_calculate_labels_determinism() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    // Create multiple files
    for i in 0..10 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let mut dag = create_dag(&dir, false)?;

    // Calculate labels multiple times
    let mut label_snapshots = Vec::new();
    for _ in 0..5 {
        dag.calculate_labels()?;

        // Take snapshot
        let snapshot = dag.labels.clone();
        label_snapshots.push(snapshot);
    }

    // Verify all snapshots are identical
    for (i, snapshot) in label_snapshots.iter().enumerate().skip(1) {
        let first = &label_snapshots[0];
        assert_eq!(snapshot, first, "Iteration {} produced different labels", i);
    }

    Ok(())
}

#[test]
fn test_label_traversal_order() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    fs::write(dir.join("a.txt"), "content a")?;
    fs::write(dir.join("b.txt"), "content b")?;
    fs::write(dir.join("c.txt"), "content c")?;

    let mut dag = create_dag(&dir, false)?;
    dag.calculate_labels()?;

    let labels = dag.labels.as_ref().unwrap();

    // Verify labels are sequential from 1 to N
    for i in 1..=labels.len() {
        let label = i.to_string();
        assert!(
            labels.contains_key(&label),
            "Expected label {} not found",
            label
        );
    }

    // Verify root is NOT in labels
    for (_, hash) in labels {
        assert_ne!(hash, &dag.root, "Root should not be in labels map");
    }

    Ok(())
}

#[test]
fn test_get_hashes_by_label_range() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    for i in 0..20 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let mut dag = create_dag(&dir, false)?;

    // Should fail before calculating labels
    assert!(dag.get_hashes_by_label_range(1, 5).is_err());

    dag.calculate_labels()?;

    // Test valid range
    let hashes = dag.get_hashes_by_label_range(1, 5)?;
    assert_eq!(hashes.len(), 5);

    // Verify hashes match labels
    let labels = dag.labels.as_ref().unwrap();
    for (i, hash) in hashes.iter().enumerate() {
        let label = (i + 1).to_string();
        assert_eq!(hash, &labels[&label]);
    }

    // Test single label
    let hashes = dag.get_hashes_by_label_range(10, 10)?;
    assert_eq!(hashes.len(), 1);

    // Test invalid ranges
    assert!(dag.get_hashes_by_label_range(0, 5).is_err()); // start < 1
    assert!(dag.get_hashes_by_label_range(5, 3).is_err()); // end < start
    assert!(dag.get_hashes_by_label_range(1, 1000).is_err()); // end > total

    Ok(())
}

#[test]
fn test_get_label() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.path().join("test.txt");
    fs::write(&file, "content")?;

    let mut dag = create_dag(&file, false)?;
    dag.calculate_labels()?;

    // Test root hash (should always return "0")
    let root_label = dag.get_label(&dag.root)?;
    assert_eq!(root_label, "0");

    // Test all labeled hashes
    let labels = dag.labels.as_ref().unwrap();
    for (expected_label, hash) in labels {
        let label = dag.get_label(hash)?;
        assert_eq!(&label, expected_label);
    }

    // Test invalid hash
    assert!(dag.get_label("invalid_hash").is_err());

    Ok(())
}

#[test]
fn test_clear_labels() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let dir = temp_dir.path().join("test");
    fs::create_dir(&dir)?;

    for i in 0..5 {
        fs::write(dir.join(format!("file{}.txt", i)), format!("content{}", i))?;
    }

    let mut dag = create_dag(&dir, false)?;
    dag.calculate_labels()?;

    let initial_count = dag.labels.as_ref().unwrap().len();
    assert!(initial_count > 0);

    // Clear labels
    dag.labels = None;

    // Verify cleared
    assert!(dag.labels.is_none());

    // Recalculate
    dag.calculate_labels()?;
    assert_eq!(dag.labels.as_ref().unwrap().len(), initial_count);

    Ok(())
}
