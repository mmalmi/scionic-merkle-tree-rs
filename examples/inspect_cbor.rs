use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <cbor_file>", args[0]);
        std::process::exit(1);
    }

    let cbor_path = &args[1];
    let data = fs::read(cbor_path).expect("Failed to read CBOR file");

    println!("CBOR file size: {} bytes", data.len());
    println!("\nFirst 200 bytes (hex):");
    for (i, chunk) in data.iter().take(200).enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i);
        }
        print!("{:02x} ", chunk);
    }
    println!("\n");

    // Try to deserialize as raw CBOR value
    match serde_cbor::from_slice::<serde_cbor::Value>(&data) {
        Ok(value) => {
            println!("Successfully parsed as CBOR!");
            println!("\nStructure:");
            print_cbor_value(&value, 0);
        }
        Err(e) => {
            println!("Failed to parse as generic CBOR: {}", e);
        }
    }

    // Try to deserialize as our Dag type
    match scionic_merkle_tree_rs::Dag::from_cbor(&data) {
        Ok(dag) => {
            println!("\n✓ Successfully deserialized as Dag!");
            println!("Root: {}", dag.root);
            println!("Leaves: {}", dag.leaves.len());
        }
        Err(e) => {
            println!("\n✗ Failed to deserialize as Dag: {}", e);
        }
    }
}

fn print_cbor_value(value: &serde_cbor::Value, indent: usize) {
    let prefix = "  ".repeat(indent);

    match value {
        serde_cbor::Value::Map(map) => {
            println!("{}Map with {} entries:", prefix, map.len());
            for (k, v) in map.iter().take(5) {
                print!("{}  {:?}: ", prefix, k);
                print_cbor_value(v, indent + 1);
            }
            if map.len() > 5 {
                println!("{}  ... {} more entries", prefix, map.len() - 5);
            }
        }
        serde_cbor::Value::Array(arr) => {
            println!("{}Array[{}]", prefix, arr.len());
        }
        serde_cbor::Value::Text(s) => {
            println!("{}Text: {}", prefix, if s.len() > 50 { &s[..50] } else { s });
        }
        serde_cbor::Value::Bytes(b) => {
            println!("{}Bytes[{}]", prefix, b.len());
        }
        serde_cbor::Value::Integer(i) => {
            println!("{}Integer: {}", prefix, i);
        }
        _ => {
            println!("{}{:?}", prefix, value);
        }
    }
}
