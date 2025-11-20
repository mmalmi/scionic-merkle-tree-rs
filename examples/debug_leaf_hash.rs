use scionic_merkle_tree_rs::Dag;
use std::fs;

fn main() {
    let data = fs::read("/tmp/go_test.cbor").expect("Failed to read");
    let dag = Dag::from_cbor(&data).expect("Failed to deserialize");

    println!("Loaded DAG:");
    println!("Root: {}", dag.root);
    println!("Leaves: {}\n", dag.leaves.len());

    // Find a non-root leaf
    for (hash, leaf) in &dag.leaves {
        if hash != &dag.root {
            println!("Testing leaf: {}", hash);
            println!("  Type: {:?}", leaf.leaf_type);
            println!("  Name: {}", leaf.item_name);
            println!("  Links: {}", leaf.links.len());
            println!("  CurrentLinkCount: {}", leaf.current_link_count);

            // Try to verify this leaf
            match leaf.verify_leaf() {
                Ok(_) => println!("  ✓ Verification succeeded"),
                Err(e) => println!("  ✗ Verification failed: {}", e),
            }

            break;
        }
    }
}
