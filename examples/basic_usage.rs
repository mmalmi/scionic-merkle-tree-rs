use scionic_merkle_tree_rs::{create_dag, Result};
use std::fs;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("=== Scionic Merkle Tree - Basic Usage Example ===\n");

    // Create a temporary directory for demonstration
    let temp_dir = TempDir::new()?;
    let input_dir = temp_dir.path().join("example_input");
    fs::create_dir(&input_dir)?;

    // Create some example files
    println!("1. Creating example files...");
    fs::write(input_dir.join("readme.txt"), b"Welcome to Scionic Merkle Trees!")?;
    fs::write(input_dir.join("data.txt"), b"Some important data here.")?;

    let subdir = input_dir.join("documents");
    fs::create_dir(&subdir)?;
    fs::write(subdir.join("doc1.txt"), b"Document 1 content")?;
    fs::write(subdir.join("doc2.txt"), b"Document 2 content")?;

    println!("   Created 4 files in 2 directories\n");

    // Create a DAG from the directory
    println!("2. Creating Scionic Merkle DAG...");
    let dag = create_dag(&input_dir, true)?;

    println!("   Root CID: {}", dag.root);
    println!("   Total leaves: {}\n", dag.leaves.len());

    // Verify the DAG
    println!("3. Verifying DAG integrity...");
    dag.verify()?;
    println!("   ✓ DAG verified successfully!\n");

    // Save to CBOR file
    let dag_file = temp_dir.path().join("example.dag");
    println!("4. Saving DAG to file...");
    dag.save_to_file(&dag_file)?;

    let file_size = fs::metadata(&dag_file)?.len();
    println!("   Saved to: {}", dag_file.display());
    println!("   File size: {} bytes\n", file_size);

    // Load from file
    println!("5. Loading DAG from file...");
    let loaded_dag = scionic_merkle_tree_rs::Dag::load_from_file(&dag_file)?;
    println!("   ✓ Loaded successfully!\n");

    // Verify loaded DAG
    println!("6. Verifying loaded DAG...");
    loaded_dag.verify()?;
    println!("   ✓ Loaded DAG verified!\n");

    // Recreate directory structure
    let output_dir = temp_dir.path().join("example_output");
    println!("7. Recreating directory from DAG...");
    loaded_dag.create_directory(&output_dir)?;

    println!("   ✓ Directory recreated!\n");

    // Verify recreated files
    println!("8. Verifying recreated files...");
    let readme_content = fs::read_to_string(output_dir.join("readme.txt"))?;
    let data_content = fs::read_to_string(output_dir.join("data.txt"))?;
    let doc1_content = fs::read_to_string(output_dir.join("documents/doc1.txt"))?;
    let doc2_content = fs::read_to_string(output_dir.join("documents/doc2.txt"))?;

    assert_eq!(readme_content, "Welcome to Scionic Merkle Trees!");
    assert_eq!(data_content, "Some important data here.");
    assert_eq!(doc1_content, "Document 1 content");
    assert_eq!(doc2_content, "Document 2 content");

    println!("   ✓ All files match original content!\n");

    // Calculate labels for LeafSync
    println!("9. Calculating labels for LeafSync...");
    let mut dag_with_labels = loaded_dag.clone();
    dag_with_labels.calculate_labels()?;

    if let Some(labels) = &dag_with_labels.labels {
        println!("   Total labeled leaves: {}", labels.len());

        // Get a range of labels
        if labels.len() >= 2 {
            let hashes = dag_with_labels.get_hashes_by_label_range(1, 2)?;
            println!("   Labels 1-2 correspond to {} hashes\n", hashes.len());
        }
    }

    println!("=== Example completed successfully! ===");

    Ok(())
}
