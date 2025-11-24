use scionic_merkle_tree_rs::Dag;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cbor_file>", args[0]);
        std::process::exit(1);
    }

    let dag = Dag::load_from_file(&args[1]).expect("Failed to load DAG");

    println!("Root: {}", dag.root);
    println!("Total leaves: {}\n", dag.leaves.len());

    let root_leaf = dag.leaves.get(&dag.root).expect("Root not found");
    println!("Root leaf:");
    println!("  ItemName: {:?}", root_leaf.item_name);
    println!("  Type: {:?}", root_leaf.leaf_type);
    println!("  Links: {}", root_leaf.links.len());
    println!("  LeafCount: {:?}", root_leaf.leaf_count);
    println!("  ContentSize: {:?}", root_leaf.content_size);
    println!("  DagSize: {:?}", root_leaf.dag_size);
    println!(
        "  ClassicMerkleRoot len: {:?}",
        root_leaf.classic_merkle_root.as_ref().map(|v| v.len())
    );
    println!();

    // Find file leaf
    for (_hash, leaf) in &dag.leaves {
        if leaf.leaf_type == scionic_merkle_tree_rs::LeafType::File {
            println!("File leaf:");
            println!("  Hash: {}", leaf.hash);
            println!("  ItemName: {:?}", leaf.item_name);
            println!("  Links (chunks): {}", leaf.links.len());
            break;
        }
    }

    // Show first chunk
    for (_hash, leaf) in &dag.leaves {
        if leaf.leaf_type == scionic_merkle_tree_rs::LeafType::Chunk {
            println!("\nFirst chunk:");
            println!("  Hash: {}", leaf.hash);
            println!("  ItemName: {:?}", leaf.item_name);
            break;
        }
    }
}
