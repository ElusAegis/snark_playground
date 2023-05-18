use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner};
use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};
use crate::fibonachi::chip::{FibChip, FibChipConfig};


#[derive(Clone, Debug, Default)]
struct FibCircuit<F: PrimeField> {
    target: u64,
    _marker: PhantomData<F>,
}

#[derive(Clone)]
struct FibCircuitConfig {
    fib_config: FibChipConfig
}

impl <F: PrimeField> Circuit<F> for FibCircuit<F> {
    type Config = FibCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        FibCircuit::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        Self::Config {
            fib_config: FibChip::configure_inside(meta)
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let fib_chip = FibChip::construct(config.fib_config);

        fib_chip.assign(&mut layouter, self.target).and_then(|_| {Ok(())})
    }
}


// Add a testing suite for the circuit
#[cfg(test)]
mod test {
    use std::marker::PhantomData;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pasta::Fp;
    use halo2_proofs::plonk::Error;
    use crate::fibonachi::circuit::FibCircuit;

    #[test]
    fn test_run_circuit() -> Result<(), Error>{
        let circuit = FibCircuit::<Fp> {
            target: 21u64,
            _marker: PhantomData::default()
        };

        let prover = MockProver::run(5, &circuit, vec![])?;
        prover.assert_satisfied();

        Ok(())
    }
}