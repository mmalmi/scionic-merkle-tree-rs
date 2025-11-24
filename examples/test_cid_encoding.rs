use cid::Cid;
use multihash::Multihash;
use sha2::{Digest, Sha256};

fn main() {
    // Create a simple CID
    let data = b"test data";
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mh = Multihash::<64>::wrap(0x12, &hash).unwrap();
    let cid = Cid::new_v1(0x71, mh);

    println!("Default to_string(): {}", cid.to_string());
    println!(
        "Base32Upper: {}",
        cid.to_string_of_base(multibase::Base::Base32Upper).unwrap()
    );
    println!(
        "Base32Lower: {}",
        cid.to_string_of_base(multibase::Base::Base32Lower).unwrap()
    );

    println!(
        "\nFirst char of default: {}",
        cid.to_string().chars().nth(2).unwrap()
    );
    println!(
        "First char of upper: {}",
        cid.to_string_of_base(multibase::Base::Base32Upper)
            .unwrap()
            .chars()
            .nth(2)
            .unwrap()
    );
}
