use serde::Serialize;
use sha2::{Digest, Sha256};

fn main() {
    // Test CBOR encoding that matches Go's leaf hash computation

    #[derive(Serialize)]
    struct LeafDataRust {
        #[serde(rename = "ItemName")]
        item_name: String,
        #[serde(rename = "Type")]
        leaf_type: String,
        #[serde(rename = "MerkleRoot")]
        #[serde(with = "serde_bytes")]
        merkle_root: Vec<u8>,
        #[serde(rename = "CurrentLinkCount")]
        current_link_count: usize,
        #[serde(rename = "ContentHash")]
        content_hash: Option<Vec<u8>>,
        #[serde(rename = "AdditionalData")]
        additional_data: Vec<(String, String)>,
    }

    let test_data = LeafDataRust {
        item_name: "file.txt".to_string(),
        leaf_type: "file".to_string(),
        merkle_root: vec![],
        current_link_count: 0,
        content_hash: Some(vec![1, 2, 3, 4]), // test hash
        additional_data: vec![],
    };

    let cbor = serde_cbor::to_vec(&test_data).unwrap();

    println!("CBOR bytes ({} total):", cbor.len());
    for (i, byte) in cbor.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", byte);
    }
    println!("\n");

    // Hash it
    let mut hasher = Sha256::new();
    hasher.update(&cbor);
    let hash = hasher.finalize();

    println!("SHA256: {}", hex::encode(hash));
}
