use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct TestDag {
    #[serde(rename = "Root")]
    root: String,

    #[serde(rename = "Leafs")]
    leaves: HashMap<String, TestLeaf>,
}

#[derive(Debug, Deserialize)]
struct TestLeaf {
    #[serde(rename = "Hash")]
    hash: String,

    #[serde(rename = "ItemName")]
    item_name: String,

    #[serde(rename = "Type")]
    leaf_type: String,

    #[serde(rename = "Links", default)]
    links: Vec<String>,

    #[serde(rename = "CurrentLinkCount")]
    current_link_count: i32,

    #[serde(rename = "ContentHash", default)]
    content_hash: Option<Vec<u8>>,

    #[serde(rename = "Content", default)]
    content: Option<Vec<u8>>,

    #[serde(rename = "ClassicMerkleRoot", default)]
    classic_merkle_root: Option<Vec<u8>>,

    #[serde(rename = "stored_proofs", default)]
    stored_proofs: Option<HashMap<String, serde_cbor::Value>>,
}

fn main() {
    let data = fs::read("/tmp/go_test.cbor").expect("Failed to read");

    match serde_cbor::from_slice::<TestDag>(&data) {
        Ok(dag) => {
            println!("✓ Successfully deserialized!");
            println!("Root: {}", dag.root);
            println!("Leaves: {}", dag.leaves.len());

            for (hash, leaf) in dag.leaves.iter().take(2) {
                println!("\nLeaf: {}...", &hash[..20]);
                println!("  Type: {}", leaf.leaf_type);
                println!("  Name: {}", leaf.item_name);
                println!("  Links: {:?}", leaf.links);
            }
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
        }
    }
}
