/// Helper for Go tests: Create a DAG and save to CBOR
use scionic_merkle_tree_rs::{create_dag, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input_path> <output_cbor>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_cbor = &args[2];

    // Create DAG
    let dag = create_dag(input_path, false)?;

    // Verify
    dag.verify()?;

    // Save to CBOR
    dag.save_to_file(output_cbor)?;

    // Output info for Go test
    println!("Success! Root: {}, Leaves: {}", dag.root, dag.leaves.len());

    Ok(())
}
