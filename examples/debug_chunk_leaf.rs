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

    // Find and analyze the first chunk
    for (_hash, leaf) in &dag.leaves {
        if leaf.leaf_type == scionic_merkle_tree_rs::LeafType::Chunk {
            println!("\nFirst chunk found:");
            println!("  Hash: {}", leaf.hash);
            println!("  ItemName: {:?}", leaf.item_name);
            println!(
                "  Content length: {:?}",
                leaf.content.as_ref().map(|c| c.len())
            );
            println!(
                "  ContentHash: {:?}",
                leaf.content_hash.as_ref().map(|h| hex::encode(h))
            );
            println!("  CurrentLinkCount: {}", leaf.current_link_count);
            println!(
                "  ClassicMerkleRoot: {:?}",
                leaf.classic_merkle_root.as_ref().map(|r| r.len())
            );
            println!("  AdditionalData: {:?}", leaf.additional_data);

            // Try to verify
            match leaf.verify_leaf() {
                Ok(_) => println!("  ✅ Verification: PASS"),
                Err(e) => println!("  ❌ Verification: FAIL - {}", e),
            }

            break;
        }
    }

    // Analyze the root/file leaf
    let root_leaf = dag.leaves.get(&dag.root).expect("Root not found");
    println!("\nRoot/File leaf:");
    println!("  Hash: {}", root_leaf.hash);
    println!("  ItemName: {:?}", root_leaf.item_name);
    println!("  Type: {:?}", root_leaf.leaf_type);
    println!("  Links: {}", root_leaf.links.len());
    println!("  CurrentLinkCount: {}", root_leaf.current_link_count);
    println!(
        "  ClassicMerkleRoot len: {:?}",
        root_leaf.classic_merkle_root.as_ref().map(|r| r.len())
    );

    match root_leaf.verify_root_leaf() {
        Ok(_) => println!("  ✅ Root verification: PASS"),
        Err(e) => println!("  ❌ Root verification: FAIL - {}", e),
    }
}
