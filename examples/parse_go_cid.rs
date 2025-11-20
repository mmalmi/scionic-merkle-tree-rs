use cid::Cid;

fn main() {
    let go_cid_str = "bafireihmpivxmhx2bylyuqk2cwo2wlnomddgwjgwpn7z5jm63vlintthke";

    match Cid::try_from(go_cid_str) {
        Ok(cid) => {
            println!("Successfully parsed Go CID!");
            println!("Version: {:?}", cid.version());
            println!("Codec: 0x{:x}", cid.codec());
            println!("Hash: {:?}", cid.hash());

            // Try converting back
            println!("\nConverting back to string:");
            println!("Default: {}", cid.to_string());
            println!("Base32Upper: {}", cid.to_string_of_base(multibase::Base::Base32Upper).unwrap());
        }
        Err(e) => {
            println!("Failed to parse Go CID: {}", e);
        }
    }
}
