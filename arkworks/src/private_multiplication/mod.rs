mod contraints;

use std::io;
use std::io::Write;
use ark_bls12_381::Bls12_381;
use ark_ec::AffineCurve;
use ark_ff::ToBytes;
use ark_r1cs_std::prelude::*;
use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
use ark_relations::r1cs::{ConstraintLayer, ConstraintSynthesizer, ConstraintSystem, TracingMode};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};

use tracing_subscriber::layer::SubscriberExt;
use crate::private_multiplication::contraints::PrivateMultiplication;


type TargetFp = ark_bls12_381::Fr;

pub fn main() {

    // Greets the user
    println!("\n\n\nWelcome to the Private Multiplication example!");
    println!("This example shows how to use the arkworks library to create a circuit that multiplies two private inputs and checks that the result is less than a public input.");

    let (a, b, c) = get_parameters();

    // Set up the circuit
    let circuit = PrivateMultiplication::<TargetFp> { a, b, c };

    // Some debugging helpers
    let mut layer = ConstraintLayer::default();
    layer.mode = TracingMode::OnlyConstraints;
    let subscriber = tracing_subscriber::Registry::default().with(layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    // We can generate the constraints ourselves and print them
    // However, this is not necessary as Groth16::circuit_specific_setup does it for us
    generate_constraints_ourself(circuit.clone());

    let rng = &mut ark_std::test_rng();
    let (proving_key, verifying_key) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, rng).unwrap();
    prompt_to_save_keys(&proving_key, &verifying_key);

    let circuit = PrivateMultiplication::<TargetFp> { a, b, c };
    let proof = Groth16::<Bls12_381>::prove(&proving_key, circuit, rng).unwrap();
    println!("Proof: {:?}", proof);
    let verified = Groth16::<Bls12_381>::verify(&verifying_key, &[c], &proof).unwrap();
    if verified {
        println!("Verification successful!");
    } else {
        println!("Verification failed!");
    }

    // We could also have the verification done by the verifier separately
    // For that, we would only need the verifying key
    // Note that the process is split into two steps as step one is only needed once for a circuit
    let processed_key = Groth16::<Bls12_381>::process_vk(&verifying_key).unwrap();
    // This step is proof-specific and needs to be done for each proof
    let result = Groth16::<Bls12_381>::verify_with_processed_vk(&processed_key, &[c], &proof).unwrap();

    if result {
        println!("Verification successful by external verifier!");
    } else {
        println!("Verification by external verifier failed!");
    }
}

fn prompt_to_save_keys(proving_key: &ProvingKey<Bls12_381>, verifying_key: &VerifyingKey<Bls12_381>) {
    // Ask the user if they want to save the keys to disk
    println!("Do you want to save the keys to disk? (y/n)");
    let mut save = String::new();
    io::stdin().read_line(&mut save).expect("Failed to read line");
    if save.trim() == "y" {
        // Save the keys to disk
        // Place the keys into a `target/circuit` directory
        // Check if the directory exists, and create it if it doesn't
        let path = std::path::Path::new("target/circuit");
        if !path.exists() {
            std::fs::create_dir(path).unwrap();
        }
        let mut pk_file = std::fs::File::create(path.join("proving_key")).unwrap();
        let mut vk_file = std::fs::File::create(path.join("verifying_key")).unwrap();

        // Write the keys to a file
        write!(pk_file, "{:?}", proving_key).unwrap();
        write!(vk_file, "{:?}", verifying_key).unwrap();
    }
}

fn generate_constraints_ourself(circuit: PrivateMultiplication<TargetFp>) {
// Populate the constraint system
    let cs = ConstraintSystem::<TargetFp>::new_ref();
    circuit.generate_constraints(cs.clone()).unwrap();
    cs.finalize();

    // Get the matrices representation of the constraint system
    let m = cs.to_matrices().unwrap();
    let error = cs.which_is_unsatisfied().unwrap();
    if error.is_some() {
        println!("Error: {:?}", error);
    } else {
        println!("Circuit satisfied!");
    }

    // Print the R1CS constraints
    println!("I can print number of constraints: {:?}", m.num_constraints);
    println!("Number of variables: {}", cs.num_instance_variables() + cs.num_witness_variables());
    println!("Number of instance variables: {}", cs.num_instance_variables());
}

fn get_parameters() -> (TargetFp, TargetFp, TargetFp) {
// Prompt the user to input the input `a`:
    println!("\n\nPlease input a: ");
    let mut a = String::new();
    io::stdin().read_line(&mut a).expect("Failed to read line");
    let a: u128 = a.trim().parse().expect("Please type a number!");

    // Prompt the user to input the input `b`:
    println!("Please input b: ");
    let mut b = String::new();
    io::stdin().read_line(&mut b).expect("Failed to read line");
    let b: u128 = b.trim().parse().expect("Please type a number!");

    // Prompt the user to input the input `c` (should be equal to `a * b`):
    println!("Please input c: ");
    let mut c = String::new();
    io::stdin().read_line(&mut c).expect("Failed to read line");
    let c: u128 = c.trim().parse().expect("Please type a number!");

    let a = TargetFp::from(a);
    let b = TargetFp::from(b);
    let c = TargetFp::from(c);
    (a, b, c)
}