use scionic_merkle_tree_rs::Dag;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <go_cbor> <rust_cbor>", args[0]);
        std::process::exit(1);
    }

    let go_dag = Dag::load_from_file(&args[1]).expect("Failed to load Go DAG");
    let rust_dag = Dag::load_from_file(&args[2]).expect("Failed to load Rust DAG");

    let go_root = go_dag.leaves.get(&go_dag.root).unwrap();
    let rust_root = rust_dag.leaves.get(&rust_dag.root).unwrap();

    println!("=== Go Root Leaf ===");
    println!("Hash: {}", go_root.hash);
    println!("ItemName: {:?}", go_root.item_name);
    println!("Type: {:?}", go_root.leaf_type);
    println!("Links: {} (first 3: {:?})", go_root.links.len(), &go_root.links[..3.min(go_root.links.len())]);
    println!("CurrentLinkCount: {}", go_root.current_link_count);
    println!("LeafCount: {:?}", go_root.leaf_count);
    println!("ContentSize: {:?}", go_root.content_size);
    println!("DagSize: {:?}", go_root.dag_size);
    println!("ClassicMerkleRoot: {:?}", go_root.classic_merkle_root.as_ref().map(|r| hex::encode(r)));
    println!("ContentHash: {:?}", go_root.content_hash.as_ref().map(|h| hex::encode(h)));
    println!("Content: {:?}", go_root.content.as_ref().map(|c| c.len()));
    println!("AdditionalData: {:?}", go_root.additional_data);

    println!("\n=== Rust Root Leaf ===");
    println!("Hash: {}", rust_root.hash);
    println!("ItemName: {:?}", rust_root.item_name);
    println!("Type: {:?}", rust_root.leaf_type);
    println!("Links: {} (first 3: {:?})", rust_root.links.len(), &rust_root.links[..3.min(rust_root.links.len())]);
    println!("CurrentLinkCount: {}", rust_root.current_link_count);
    println!("LeafCount: {:?}", rust_root.leaf_count);
    println!("ContentSize: {:?}", rust_root.content_size);
    println!("DagSize: {:?}", rust_root.dag_size);
    println!("ClassicMerkleRoot: {:?}", rust_root.classic_merkle_root.as_ref().map(|r| hex::encode(r)));
    println!("ContentHash: {:?}", rust_root.content_hash.as_ref().map(|h| hex::encode(h)));
    println!("Content: {:?}", rust_root.content.as_ref().map(|c| c.len()));
    println!("AdditionalData: {:?}", rust_root.additional_data);

    println!("\n=== Comparison ===");
    if go_root.classic_merkle_root == rust_root.classic_merkle_root {
        println!("✅ ClassicMerkleRoot matches");
    } else {
        println!("❌ ClassicMerkleRoot differs!");
    }

    if &go_root.links[..5] == &rust_root.links[..5] {
        println!("✅ First 5 links match");
    } else {
        println!("❌ Links differ!");
    }
}
