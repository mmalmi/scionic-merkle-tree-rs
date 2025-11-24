/// Test that our merkle tree building matches Go's txaty/go-merkletree
/// for odd numbers of leaves (which duplicate the last node)
use scionic_merkle_tree_rs::merkle_tree::build_merkle_root;
use sha2::{Digest, Sha256};

#[test]
fn test_merkle_root_with_3_leaves() {
    // Test with 3 leaves - Go duplicates the 3rd leaf
    let leaves: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

    let root = build_merkle_root(&leaves);

    // Manually compute what Go does:
    // Level 0: [leaf1, leaf2, leaf3, leaf3]  <- duplicates last
    // Level 1: [hash(leaf1+leaf2), hash(leaf3+leaf3)]
    // Root: hash(level1[0] + level1[1])

    let mut hasher1 = Sha256::new();
    hasher1.update(&leaves[0]);
    hasher1.update(&leaves[1]);
    let level1_0 = hasher1.finalize().to_vec();

    let mut hasher2 = Sha256::new();
    hasher2.update(&leaves[2]);
    hasher2.update(&leaves[2]); // Duplicate!
    let level1_1 = hasher2.finalize().to_vec();

    let mut hasher_root = Sha256::new();
    hasher_root.update(&level1_0);
    hasher_root.update(&level1_1);
    let expected_root = hasher_root.finalize().to_vec();

    assert_eq!(
        root, expected_root,
        "Merkle root should match Go's duplication algorithm"
    );
}

#[test]
fn test_merkle_root_with_5_leaves() {
    // Test with 5 leaves
    let leaves: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![3], vec![4], vec![5]];

    let root = build_merkle_root(&leaves);

    // Go's algorithm:
    // Level 0: [1, 2, 3, 4, 5, 5]  <- duplicates last
    // Level 1: [hash(1+2), hash(3+4), hash(5+5)]
    // Level 1 (after fixing): [hash(1+2), hash(3+4), hash(5+5), hash(5+5)]  <- duplicates last again
    // Level 2: [hash(hash(1+2)+hash(3+4)), hash(hash(5+5)+hash(5+5))]
    // Root: hash(level2[0] + level2[1])

    let mut h1 = Sha256::new();
    h1.update(&leaves[0]);
    h1.update(&leaves[1]);
    let l1_0 = h1.finalize().to_vec();

    let mut h2 = Sha256::new();
    h2.update(&leaves[2]);
    h2.update(&leaves[3]);
    let l1_1 = h2.finalize().to_vec();

    let mut h3 = Sha256::new();
    h3.update(&leaves[4]);
    h3.update(&leaves[4]); // Duplicate
    let l1_2 = h3.finalize().to_vec();

    // Now level 1 has 3 nodes, duplicate last
    let mut h4 = Sha256::new();
    h4.update(&l1_0);
    h4.update(&l1_1);
    let l2_0 = h4.finalize().to_vec();

    let mut h5 = Sha256::new();
    h5.update(&l1_2);
    h5.update(&l1_2); // Duplicate
    let l2_1 = h5.finalize().to_vec();

    let mut h_root = Sha256::new();
    h_root.update(&l2_0);
    h_root.update(&l2_1);
    let expected_root = h_root.finalize().to_vec();

    assert_eq!(
        root, expected_root,
        "Merkle root with 5 leaves should match Go's algorithm"
    );
}

#[test]
fn test_merkle_root_with_even_leaves() {
    // Test with 4 leaves (even - no duplication needed)
    let leaves: Vec<Vec<u8>> = vec![vec![1], vec![2], vec![3], vec![4]];

    let root = build_merkle_root(&leaves);

    // Level 0: [1, 2, 3, 4]  <- no duplication
    // Level 1: [hash(1+2), hash(3+4)]
    // Root: hash(level1[0] + level1[1])

    let mut h1 = Sha256::new();
    h1.update(&leaves[0]);
    h1.update(&leaves[1]);
    let l1_0 = h1.finalize().to_vec();

    let mut h2 = Sha256::new();
    h2.update(&leaves[2]);
    h2.update(&leaves[3]);
    let l1_1 = h2.finalize().to_vec();

    let mut h_root = Sha256::new();
    h_root.update(&l1_0);
    h_root.update(&l1_1);
    let expected_root = h_root.finalize().to_vec();

    assert_eq!(
        root, expected_root,
        "Merkle root with even leaves should work correctly"
    );
}
