fn main() {
    // Go CID example
    let go_cid = "bafireihmpivxmhx2bylyuqk2cwo2wlnomddgwjgwpn7z5jm63vlintthke";
    // Rust CID example
    let rust_cid = "bafyreibtckmrl3ygmezqykrndghvgm6mxgmampdd6rgx4iohvnn7h6v26m";

    println!("Go CID:   {} (len={})", go_cid, go_cid.len());
    println!("Rust CID: {} (len={})", rust_cid, rust_cid.len());

    println!("\nGo prefix:   {}", &go_cid[..7]);
    println!("Rust prefix: {}", &rust_cid[..7]);

    println!("\nAnalysis:");
    println!("Go starts with 'bafi' - this is base32 upper (no padding)");
    println!("Rust starts with 'bafy' - this is base32 lower");

    println!("\nThe difference is the multibase encoding:");
    println!("'f' = base32upper");
    println!("'b' = base32 (lowercase)");

    println!("\nTo match Go, Rust CID should use:");
    println!("- CID::to_string_of_base(multibase::Base::Base32Upper)");
}
