/// Create DAG with specific chunk size for testing
use scionic_merkle_tree_rs::{create_dag_with_config, DagBuilderConfig, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <input_path> <output_cbor> <chunk_size>", args[0]);
        eprintln!("  chunk_size: size in bytes, or 0 to disable chunking");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_cbor = &args[2];
    let chunk_size: usize = args[3].parse().expect("Invalid chunk size");

    // Create config with specified chunk size
    let config = if chunk_size == 0 {
        DagBuilderConfig::new().without_chunking()
    } else {
        DagBuilderConfig::new().with_chunk_size(chunk_size)
    };

    // Create DAG
    let dag = create_dag_with_config(input_path, config)?;

    // Verify
    dag.verify()?;

    // Save to CBOR
    dag.save_to_file(output_cbor)?;

    // Output info
    println!(
        "Success! Root: {}, Leaves: {}, ChunkSize: {}",
        dag.root,
        dag.leaves.len(),
        if chunk_size == 0 {
            "disabled".to_string()
        } else {
            chunk_size.to_string()
        }
    );

    Ok(())
}
