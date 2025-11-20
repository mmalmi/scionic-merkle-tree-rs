use serde::{Deserialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct DebugDag {
    #[serde(rename = "Root")]
    root: String,

    #[serde(rename = "Leafs")]
    leaves: HashMap<String, serde_cbor::Value>,
}

fn main() {
    let data = fs::read("/tmp/go_test.cbor").expect("Failed to read");

    match serde_cbor::from_slice::<DebugDag>(&data) {
        Ok(dag) => {
            println!("Root: {}", dag.root);
            println!("Leaves: {}", dag.leaves.len());

            for (hash, leaf_value) in dag.leaves.iter().take(1) {
                println!("\nAnalyzing leaf: {}...", &hash[..20]);
                if let serde_cbor::Value::Map(map) = leaf_value {
                    for (key, value) in map {
                        if let serde_cbor::Value::Text(k) = key {
                            print!("  {}: ", k);
                            match value {
                                serde_cbor::Value::Text(s) => println!("Text({})", s),
                                serde_cbor::Value::Bytes(b) => println!("Bytes(len={})", b.len()),
                                serde_cbor::Value::Array(a) => {
                                    println!("Array(len={})", a.len());
                                    if k == "Links" || k == "Siblings" {
                                        for item in a.iter().take(2) {
                                            println!("    Item: {:?}", item);
                                        }
                                    }
                                }
                                serde_cbor::Value::Integer(i) => println!("Integer({})", i),
                                serde_cbor::Value::Null => println!("Null"),
                                serde_cbor::Value::Map(m) => println!("Map(len={})", m.len()),
                                _ => println!("{:?}", value),
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
