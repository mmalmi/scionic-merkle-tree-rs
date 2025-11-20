use scionic_merkle_tree_rs::Dag;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cbor_file>", args[0]);
        std::process::exit(1);
    }

    let dag = Dag::load_from_file(&args[1]).expect("Failed to load DAG");

    println!("DAG Root: {}", dag.root);

    // Find file leaf
    for (_hash, leaf) in &dag.leaves {
        if leaf.leaf_type == scionic_merkle_tree_rs::LeafType::File {
            println!("\nFile leaf: {}", leaf.hash);
            println!("Links array (in order):");
            for (i, link) in leaf.links.iter().enumerate() {
                if let Some(chunk_leaf) = dag.leaves.get(link) {
                    println!("  [{}] {} -> {}", i, &link[..20], chunk_leaf.item_name);
                } else {
                    println!("  [{}] {} -> NOT FOUND", i, &link[..20]);
                }

                if i >= 5 {
                    println!("  ... {} more chunks", leaf.links.len() - i - 1);
                    break;
                }
            }
            break;
        }
    }
}
