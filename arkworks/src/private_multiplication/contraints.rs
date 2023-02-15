use std::cmp::Ordering;
use std::ops::Mul;
use ark_ff::PrimeField;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::fields::fp::{AllocatedFp, FpVar};
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, LinearCombination, SynthesisError};


#[derive(Clone)]
pub struct PrivateMultiplication<ConstraintF: PrimeField> {
    // Public Inputs
    pub(crate) c : ConstraintF,

    // Private Inputs
    pub(crate) a : ConstraintF,
    pub(crate) b : ConstraintF,
}

impl <ConstraintF: PrimeField> PrivateMultiplication<ConstraintF> {
    fn multiplication_num_comparison(
        a : AllocatedFp<ConstraintF>,
        b : AllocatedFp<ConstraintF>,
        c : AllocatedFp<ConstraintF>
    ) -> Result<(), SynthesisError> {
        // Convert from AllocatedFp to FpVar to use the wrapper functions
        let a = FpVar::from(a);
        let b = FpVar::from(b);
        let c = FpVar::from(c);
        a.enforce_cmp(&c, Ordering::Less, false)?;     // Check that a < c
        a.enforce_cmp(&b, Ordering::Less, true)?;  // Check that a <= b
        // Check that a * b is less than 2^255
        let two_255 = FpVar::constant(ConstraintF::from(2_u128).pow(&[255, 0, 0, 0]));
        a.mul(&b).enforce_cmp(&two_255, Ordering::Less, true)?;
        Ok(())
    }
}

impl <ConstraintF: PrimeField> ConstraintSynthesizer<ConstraintF> for PrivateMultiplication<ConstraintF> {
    fn generate_constraints(self, cs: ConstraintSystemRef<ConstraintF>) -> ark_relations::r1cs::Result<()> {

        // We can create constraints by ourselves like in Circom:

        // First, we define our input and witness variables
        let a_var = cs.new_witness_variable(|| Ok(self.a)).unwrap();
        let b_var = cs.new_witness_variable(|| Ok(self.b)).unwrap();
        let c_var = cs.new_input_variable(|| Ok(self.c)).unwrap();

        // Then, we define our multiplication constraint as (2 * a_var) * (2 * b_var) = c_var using QAP
        // Note that we also used coefficient 2 for a_var and b_var, and coefficient 4 for c_var
        // This is to show what we can do with coefficients
        let a1 = LinearCombination::<ConstraintF>::from((ConstraintF::from(2u128), a_var));
        let b1 = LinearCombination::<ConstraintF>::from((ConstraintF::from(2u128), b_var));
        let c1 = LinearCombination::<ConstraintF>::from((ConstraintF::from(4u128), c_var));
        cs.enforce_constraint(a1, b1, c1).unwrap();

        // This version with coefficients is equivalent to the following version without coefficients
        cs.enforce_constraint(
            LinearCombination::<ConstraintF>::from(a_var),
            LinearCombination::<ConstraintF>::from(b_var),
            LinearCombination::<ConstraintF>::from(c_var)
        ).unwrap();

        // However, we need to enforce more complex constraints, so we use the helper functions provided by arkworks
        // First, we tell that we want to use Variables A, B, C as FpVar<ConstraintF> (default to the field of ConstraintF)
        let a_var_alloc = AllocatedFp::<ConstraintF>::new_witness(cs.clone(), || Ok(self.a)).unwrap();
        let b_var_alloc = AllocatedFp::<ConstraintF>::new_witness(ns!(cs, "b_var"), || Ok(self.b)).unwrap();
        let c_var_alloc = AllocatedFp::<ConstraintF>::new_witness(ns!(cs, "c_var"), || Ok(self.c)).unwrap();
        // As we have redefined a_var, b_var, c_var, we need make sure that they are equal to the original ones
        // Because we are using the typeless variables, we will need to do this manually through the constraint system
        let one_const = AllocatedFp::<ConstraintF>::new_constant(cs.clone(), ConstraintF::one()).unwrap();

        cs.enforce_constraint( // a_var = a_var_c
                               LinearCombination::<ConstraintF>::from(a_var),
                               LinearCombination::<ConstraintF>::from(one_const.variable),
                               LinearCombination::<ConstraintF>::from(a_var_alloc.variable)
        ).unwrap();

        cs.enforce_constraint( // b_var = b_var_c
                               LinearCombination::<ConstraintF>::from(b_var),
                               LinearCombination::<ConstraintF>::from(one_const.variable),
                               LinearCombination::<ConstraintF>::from(b_var_alloc.variable)
        ).unwrap();

        cs.enforce_constraint( // c_var = c_var_c
                               LinearCombination::<ConstraintF>::from(c_var),
                               LinearCombination::<ConstraintF>::from(one_const.variable),
                               LinearCombination::<ConstraintF>::from(c_var_alloc.variable)
        ).unwrap();

        // Now, we can use the helper functions to enforce the rest of multiplication constraints
        PrivateMultiplication::multiplication_num_comparison(a_var_alloc, b_var_alloc, c_var_alloc).unwrap();

        // Optimize and finalize the constraint system

        Ok(())
    }
}

