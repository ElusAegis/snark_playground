use ark_crypto_primitives::crh::injective_map::{PedersenCRHCompressor, TECompressor};
use ark_crypto_primitives::crh::injective_map::constraints::{PedersenCRHCompressorGadget, TECompressorGadget};
use ark_crypto_primitives::crh::pedersen;
use ark_crypto_primitives::CRHGadget;
use ark_ed_on_bls12_381::constraints::EdwardsVar;
use ark_ed_on_bls12_381::EdwardsProjective;

// This file defines the hash functions that we will use in our circuit
// As well as the gadgets that will be used to verify the correctness of the hash functions
// Finally, in the end we define the parameters of the hash verification gadgets


// This will do the calculation of the inner hash
pub type InnerHash = PedersenCRHCompressor<EdwardsProjective, TECompressor, InnerHashWindow>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InnerHashWindow;

impl pedersen::Window for InnerHashWindow {
    const WINDOW_SIZE: usize = 4; // This is a 4-bit window, default for PedersenCRH
    const NUM_WINDOWS: usize = 128; // This is the number of 4-bit windows, we need to make sure that our two children hashes fit into this
}

pub type LeafHash = PedersenCRHCompressor<EdwardsProjective, TECompressor, LeafHashWindow>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LeafHashWindow;

impl pedersen::Window for LeafHashWindow {
    const WINDOW_SIZE: usize = 4; // This is a 4-bit window, default for PedersenCRH
    const NUM_WINDOWS: usize = 2; // This is the number of 4-bit windows, we need to make sure that our children data fits into this (u8 requires only 2 windows)
}

// This will generate the proof of correctness of the inner hash
pub type InnerHashGadget = PedersenCRHCompressorGadget<EdwardsProjective, TECompressor, InnerHashWindow, EdwardsVar, TECompressorGadget>;

// This will generate the proof of correctness of the leaf hash
pub type LeafHashGadget = PedersenCRHCompressorGadget<EdwardsProjective, TECompressor, LeafHashWindow, EdwardsVar, TECompressorGadget>;

// Define parameters variables that would be input to the gadget for leaf hash
pub type LeafHashParamVars = <LeafHashGadget as CRHGadget<LeafHash, ConstraintF>>::ParametersVar;

// Define parameters variables that would be input to the gadget for inner hash
pub type InnerHashParamVars = <InnerHashGadget as CRHGadget<InnerHash, EdwardsVar>>::ParametersVar;

pub type ConstraintF = ark_ed_on_bls12_381::Fq;