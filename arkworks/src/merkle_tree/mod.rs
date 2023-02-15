mod common;
pub(crate) mod constraints;

use ark_crypto_primitives::CRH;
use ark_crypto_primitives::merkle_tree::Config;
use ark_std::UniformRand;
use common::*;

#[derive(Clone)]
struct MerkleConfig;

// Define the hash functions that will be used in the Merkle Tree
impl Config for MerkleConfig {
    type LeafHash = LeafHash;
    type TwoToOneHash = InnerHash;
}

// Define the Merkle Tree with the above hash functions
type MyMerkleTree = ark_crypto_primitives::merkle_tree::MerkleTree<MerkleConfig>;

// Define the type of the Merkle Tree's root
type Root = <InnerHash as CRH>::Output;

// Define the type of the path using the inbuilt Path type
type MyPath = ark_crypto_primitives::merkle_tree::Path<MerkleConfig>;


pub fn main() {
    // Get rng
    let rng = &mut ark_std::test_rng();

    // Setup public parameters for the hash functions
    let leaf_hash_params = LeafHash::setup(rng).unwrap();
    let inner_hash_params = InnerHash::setup(rng).unwrap();

    let tree_values = (0..16).map(|_| u8::rand(rng)).collect::<Vec<_>>();

    // Construct the Merkle Tree
    let mut tree = MyMerkleTree::blank(&leaf_hash_params, &inner_hash_params, 5).unwrap();
    // Include 2^4 leaves in the Merkle Tree with random values
    tree_values.iter().enumerate().for_each(
        |(i, num)| {
            tree.update(i, &num).unwrap();
            println!("Leaf {}: {}", i, num)
        }
    );

    // Read user input for the leaf number that we want to prove
    let mut input = String::new();
    println!("Enter the leaf number that you want to prove:");
    std::io::stdin().read_line(&mut input).unwrap();
    let leaf_number: usize = input.trim().parse().unwrap();

    // Generate the proof for the leaf number
    let proof = tree.generate_proof(leaf_number).unwrap();
    println!("Generated proof for leaf {}: {:?}", leaf_number, proof.auth_path);

    // Get the root of the Merkle Tree
    let root = tree.root();

    // Check the correctness of the proof
    // We pass proof and all other parameters are public inputs to the circuit
    println!("Verifying the proof...");
    let result = proof.verify(&leaf_hash_params, &inner_hash_params, &root, &tree_values[leaf_number]).unwrap();
    assert!(result, "Proof verification failed");
    println!("Successfully verified that the leaf {} at index {} is indeed in the Merkle Tree with root has {}", &tree_values[leaf_number], leaf_number, root);

    println!("Hello, world!");
}
