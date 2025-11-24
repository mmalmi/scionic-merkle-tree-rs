use scionic_merkle_tree_rs::merkle_tree::build_merkle_root;
use scionic_merkle_tree_rs::Dag;
use sha2::{Digest, Sha256};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cbor_file>", args[0]);
        std::process::exit(1);
    }

    let dag = Dag::load_from_file(&args[1]).expect("Failed to load DAG");
    let root_leaf = dag.leaves.get(&dag.root).unwrap();

    println!("Root hash: {}", root_leaf.hash);
    println!("Links: {}", root_leaf.links.len());
    println!("\nFirst 5 links:");
    for (i, link) in root_leaf.links.iter().take(5).enumerate() {
        println!("  [{}] {}", i, link);
    }

    // Rebuild merkle tree (matching TypeScript/Go approach)
    println!("\nRebuilding merkle tree...");

    // Sort links
    let mut sorted_links = root_leaf.links.clone();
    sorted_links.sort();

    // Hash each link
    let hashed_leaves: Vec<_> = sorted_links
        .iter()
        .map(|link| {
            let mut hasher = Sha256::new();
            hasher.update(link.as_bytes());
            hasher.finalize().to_vec()
        })
        .collect();

    let computed_root = build_merkle_root(&hashed_leaves);

    println!(
        "Computed ClassicMerkleRoot: {}",
        hex::encode(&computed_root)
    );
    println!(
        "Stored ClassicMerkleRoot:   {}",
        root_leaf
            .classic_merkle_root
            .as_ref()
            .map(|r| hex::encode(r))
            .unwrap_or_else(|| "None".to_string())
    );

    if root_leaf.classic_merkle_root.as_ref() == Some(&computed_root) {
        println!("✅ Merkle roots match!");
    } else {
        println!("❌ Merkle roots differ!");
    }
}
