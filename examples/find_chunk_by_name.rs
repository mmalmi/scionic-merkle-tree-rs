use scionic_merkle_tree_rs::Dag;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <cbor_file> <chunk_name>", args[0]);
        eprintln!("Example: {} dag.cbor \"bitcoin.pdf/0\"", args[0]);
        std::process::exit(1);
    }

    let dag = Dag::load_from_file(&args[1]).expect("Failed to load DAG");
    let target_name = &args[2];

    for (_hash, leaf) in &dag.leaves {
        if leaf.item_name == *target_name {
            println!("Found chunk: {}", target_name);
            println!("  Hash: {}", leaf.hash);
            println!("  Type: {:?}", leaf.leaf_type);
            println!("  Content length: {:?}", leaf.content.as_ref().map(|c| c.len()));
            println!("  ContentHash: {:?}", leaf.content_hash.as_ref().map(|h| hex::encode(h)));

            match leaf.verify_leaf() {
                Ok(_) => println!("  ✅ Verification: PASS"),
                Err(e) => println!("  ❌ Verification: FAIL - {}", e),
            }

            return;
        }
    }

    println!("Chunk '{}' not found", target_name);
}
