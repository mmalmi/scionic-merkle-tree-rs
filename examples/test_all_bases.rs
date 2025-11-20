use cid::Cid;
use multihash::Multihash;
use multibase::Base;
use sha2::{Digest, Sha256};

fn main() {
    // Create a simple CID
    let data = b"test";
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mh = Multihash::<64>::wrap(0x12, &hash).unwrap();
    let cid = Cid::new_v1(0x71, mh);

    println!("Testing different multibase encodings:\n");

    let bases = vec![
        (Base::Base32Lower, "Base32Lower"),
        (Base::Base32Upper, "Base32Upper"),
        (Base::Base16Lower, "Base16Lower"),
        (Base::Base58Btc, "Base58Btc"),
    ];

    for (base, name) in bases {
        match cid.to_string_of_base(base) {
            Ok(s) => println!("{:15} {}", name, s),
            Err(e) => println!("{:15} ERROR: {}", name, e),
        }
    }

    println!("\nGo example: bafireihmpivxmhx2bylyuqk2cwo2wlnomddgwjgwpn7z5jm63vlintthke");
    println!("Pattern: 'bafi' = ba + f (base32upper) + encoded data");
}
