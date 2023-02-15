// Here we will do the same as before, but with custom constraints.

use ark_crypto_primitives::crh::{TwoToOneCRH, TwoToOneCRHGadget};
use ark_crypto_primitives::{CRH, PathVar};
use ark_ff::Field;
use ark_r1cs_std::prelude::{AllocVar, Boolean, EqGadget, UInt8};
use ark_relations::r1cs::{ConstraintLayer, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, TracingMode};
use tracing_subscriber::layer::SubscriberExt;
use crate::merkle_tree::common::{ConstraintF, InnerHash, InnerHashGadget, InnerHashParamVars, LeafHash, LeafHashGadget, LeafHashParamVars};
use crate::merkle_tree::{MerkleConfig, MyMerkleTree, MyPath, Root};

// First, we define a R1CS equivalent to the Merkle Tree root
pub type RootVar = <InnerHashGadget as TwoToOneCRHGadget<InnerHash, ConstraintF>>::OutputVar;

// Then, we define a R1CS equivalent to the Merkle Tree path
type MyPathVar = PathVar<MerkleConfig, LeafHashGadget, InnerHashGadget, ConstraintF>;

////////////////////////////////////////////////////////////////////////////////////////

struct MerkleTreeVerification {
    // These are part of the setup
    pub leaf_crh_parameters: <LeafHash as CRH>::Parameters,
    pub inner_crh_parameters: <InnerHash as TwoToOneCRH>::Parameters,

    // Public inputs to the circuit
    pub root: Root,
    pub leaf: u8,

    // Private inputs to the circuit
    pub path: MyPath,
}

impl ConstraintSynthesizer<ConstraintF> for MerkleTreeVerification {
    #[tracing::instrument(target = "r1cs", skip(self, cs))]
    fn generate_constraints(self, cs: ConstraintSystemRef<ConstraintF>) -> ark_relations::r1cs::Result<()> {
        // Generate public inputs
        let root = RootVar::new_input(ark_relations::ns!(cs, "root_var"), || Ok(self.root))?;
        let leaf : UInt8<ConstraintF> = UInt8::new_input(ark_relations::ns!(cs, "leaf_var"), || Ok(self.leaf))?; // This is the leaf that we want to verify

        // Generate private inputs
        let path = MyPathVar::new_witness(ark_relations::ns!(cs, "path_var"), || Ok(self.path))?;

        // Generate constants for input parameters for the hash functions
        let inner_crh_parameters = InnerHashParamVars::new_constant(cs.clone(), &self.inner_crh_parameters)?;
        let leaf_crh_parameters = LeafHashParamVars::new_constant(cs.clone(), &self.leaf_crh_parameters)?;

        // Verify the correctness of the path
        let leaf_bytes = vec![leaf; 1];


        let res = path.verify_membership(
            &leaf_crh_parameters,
            &inner_crh_parameters,
            &root,
            &leaf_bytes.as_slice()
        )?;

        res.enforce_equal(&Boolean::Constant(true))?;

        Ok(())
    }
}

pub fn custom_circuit() {
    let rng = &mut ark_std::test_rng();

    // Generate the parameters for the hash functions
    let leaf_crh_parameters = <LeafHash as CRH>::setup(rng).unwrap();
    let inner_crh_parameters = <InnerHash as TwoToOneCRH>::setup(rng).unwrap();

    // Construct the tree
    let tree = MyMerkleTree::new(
        &leaf_crh_parameters,
        &inner_crh_parameters,
        &[0u8, 8u8, 12u8, 16u8, 20u8, 24u8, 28u8, 32u8]
    ).unwrap();

    // Generate proof of membership for the 1st leaf
    let proof = tree.generate_proof(0).unwrap();

    // Prompt the user to enter the leaf value
    println!("Enter the leaf value: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let leaf = input.trim().parse::<u8>().unwrap();

    // Generate the circuit
    let circuit = MerkleTreeVerification {
        leaf_crh_parameters,
        inner_crh_parameters,
        root: tree.root(),
        leaf,
        path: proof,
    };

    // Some debugging helpers
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Generate circuit
    let cs = ConstraintSystem::<ConstraintF>::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();

    // Check that the circuit is satisfied
    let satisfied = cs.is_satisfied().unwrap();
    if !satisfied {
        println!("Unsatisfied constraints: {:?}\n", cs.which_is_unsatisfied().unwrap());
    } else {
        println!("Circuit is satisfied!");
    }
    assert!(satisfied);
}