use sha2::{Digest, Sha256};

fn main() {
    // Test with just 3 simple links to see the algorithm
    let links = vec!["link1", "link2", "link3"];

    println!("Building merkle tree from 3 links:\n");

    // Hash each link
    let mut hashed_leaves = Vec::new();
    for link in &links {
        let mut hasher = Sha256::new();
        hasher.update(link.as_bytes());
        let hash = hasher.finalize().to_vec();
        println!("  Leaf {}: {}", link, hex::encode(&hash));
        hashed_leaves.push(hash);
    }

    println!("\nBuilding tree...");
    let mut current_level = hashed_leaves.clone();
    let mut level_num = 0;

    while current_level.len() > 1 {
        println!("\nLevel {}: {} nodes", level_num, current_level.len());
        let mut next_level = Vec::new();

        for (i, chunk) in current_level.chunks(2).enumerate() {
            if chunk.len() == 2 {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                hasher.update(&chunk[1]);
                let hash = hasher.finalize().to_vec();
                println!("  Pair {}: hash({} + {}) = {}",
                    i,
                    hex::encode(&chunk[0][..8]),
                    hex::encode(&chunk[1][..8]),
                    hex::encode(&hash[..8])
                );
                next_level.push(hash);
            } else {
                println!("  Odd node: promoting {}", hex::encode(&chunk[0][..8]));
                next_level.push(chunk[0].clone());
            }
        }

        current_level = next_level;
        level_num += 1;
    }

    println!("\nRoot: {}", hex::encode(&current_level[0]));
}
