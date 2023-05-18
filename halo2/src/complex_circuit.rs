use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Circuit, Column, ConstraintSystem, Error, Fixed, Instance};
use crate::fibonachi::chip::{FibChip, FibChipConfig};
use crate::private_mul::chip::{PrivateMulChip, PrivateMulChipConfig, PrivateMulInstruction};

#[derive(Clone)]
struct ComplexCircuitConfig {
    fib_conf: FibChipConfig,
    prv_mul_conf: PrivateMulChipConfig,
    fixed: Column<Fixed>,
    instance: Column<Instance>
    // instance: Column<Instance>
}


// This circuit checks that I know a number A, such that for given public B:
// Such that A^4 * X = B, where X is some constant predefined for the circuit
// And A is a Fibonacci number
#[derive(Default)]
struct ComplexCircuit<F: PrimeField> {
    a: u64,
    x: u64,
    _marker: PhantomData<F>
}

impl <F: PrimeField> Circuit<F> for ComplexCircuit<F> {
    type Config = ComplexCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {

        let fib_conf = FibChip::configure_inside(meta);

        let advice = [fib_conf.num1.clone(), fib_conf.num2.clone(), meta.advice_column()];
        let prv_mul_conf = PrivateMulChip::configure(meta, advice);

        let fixed = meta.fixed_column();
        let instance = meta.instance_column();

        for col in prv_mul_conf.advice {
            meta.enable_equality(col)
        }
        meta.enable_equality(instance);
        meta.enable_equality(fixed);

        Self::Config {
            fib_conf,
            prv_mul_conf,
            fixed,
            instance
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {

        let fib_chip = FibChip::<F>::construct(config.fib_conf);

        let a = fib_chip.assign(&mut layouter, self.a)?;

        let five = layouter.assign_region(|| "assign_const", |mut region| {
            region.assign_fixed(|| "const five", config.fixed, 0, || Value::known(F::from(self.x)))
        })?;

        let mul_chip = PrivateMulChip::<F>::construct(config.prv_mul_conf);

        let aa = mul_chip.mul(&mut layouter, a.clone(), a)?;
        let aaaa = mul_chip.mul(&mut layouter, aa.clone(), aa.clone())?;

        let aaaa_five = mul_chip.mul(&mut layouter, aaaa, five)?;

        layouter.constrain_instance(aaaa_five.cell(), config.instance, 0)
    }
}


#[cfg(test)]
mod test {
    use std::marker::PhantomData;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::plonk::Error;
    use pasta_curves::Fp;
    use crate::complex_circuit::ComplexCircuit;

    #[test]
    fn test_run_complex_circuit() -> Result<(), Error> {

        let a = 13u64;
        let x = 5;
        let b = a.pow(4) * x;

        let circuit = ComplexCircuit::<Fp> {
            a,
            x,
            _marker: PhantomData::default()
        };

        // Create the area you want to draw on.
        // Use SVGBackend if you want to render to .svg instead.
        use plotters::prelude::*;
        let root = BitMapBackend::new("complex_circuit_layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Complex Circuit Layout", ("sans-serif", 60))
            .unwrap();

        halo2_proofs::dev::CircuitLayout::default()
            // You can optionally render only a section of the circuit.
            .view_height(0..16)
            // You can hide labels, which can be useful with smaller areas.
            .show_labels(true)
            .mark_equality_cells(true)
            .show_equality_constraints(true)
            // Render the circuit onto your area!
            // The first argument is the size parameter for the circuit.
            .render(4, &circuit, &root)
            .unwrap();

        let prover = MockProver::run(4, &circuit, vec![vec![Fp::from(b)]])?;
        Ok(prover.assert_satisfied())
    }


}
