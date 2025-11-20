/// Helper for Go tests: Load and verify a DAG from CBOR
use scionic_merkle_tree_rs::{Dag, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <cbor_path>", args[0]);
        std::process::exit(1);
    }

    let cbor_path = &args[1];

    // Load from CBOR
    let dag = Dag::load_from_file(cbor_path)?;

    println!("Loaded DAG:");
    println!("  Root: {}", dag.root);
    println!("  Leaves: {}", dag.leaves.len());

    // Verify
    dag.verify()?;

    println!("✓ Verification successful!");

    Ok(())
}
