use ff::PrimeField;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::plonk::{Circuit, Column, ConstraintSystem, Error, Instance};
use crate::private_mul::chip::{PrivateMulChip, PrivateMulChipConfig, PrivateMulInstruction};

#[derive(Default)]
struct PrivateMulCircuit<F: PrimeField> {
    a : F,
    b : F
}

#[derive(Clone)]
struct PrivateMulCircuitConfig {
    private_mul_chip_conf: PrivateMulChipConfig,
    instance: Column<Instance>
}

impl <F: PrimeField> Circuit<F> for PrivateMulCircuit<F> {
    type Config = PrivateMulCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {

        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let instance = meta.instance_column();
        // We need to enable equality as we copy from advice column 3 to instance
        meta.enable_equality(instance);
        // Enable equality for columns as we copy values around them in the circuit
        for col in [a, b, c] {
            meta.enable_equality(col)
        }

        PrivateMulCircuitConfig { private_mul_chip_conf: PrivateMulChip::configure(meta, [a, b, c, ]), instance }

    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {

        // Assign circuit params
        let a = layouter.assign_region(|| "initial param", |mut region| {
            region.assign_advice(|| "initial param a", config.private_mul_chip_conf.advice[0], 0, || Value::known(self.a))
        })?;

        let b = layouter.assign_region(|| "initial param", |mut region| {
            region.assign_advice(|| "initial param b", config.private_mul_chip_conf.advice[1], 0, || Value::known(self.b))
        })?;

        // Generate the PrivateMul Gadget
        let private_mul = PrivateMulChip::<F>::construct(config.private_mul_chip_conf);

        // Calculate a * b * b * b
        let ab = private_mul.mul(&mut layouter, a, b.clone())?;
        let abb = private_mul.mul(&mut layouter, ab.clone(), b.clone())?;
        let abbb = private_mul.mul(&mut layouter, abb, b)?;

        // Assert that a * b * b * b = instance[0]
        layouter.constrain_instance(abbb.cell(), config.instance, 0)
    }
}


#[cfg(test)]
mod test {
    use ff::Field;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::plonk::Error;
    use pasta_curves::Fp;
    use crate::private_mul::circuit::PrivateMulCircuit;

    #[test]
    fn test_private_mul_circuit() -> Result<(), Error> {
        let circuit = PrivateMulCircuit::<Fp> {
            a: Fp::from(3),
            b: Fp::from(2),
        };

        let instance = circuit.a * circuit.b.pow([3]);


        let prover = MockProver::run(5, &circuit, vec![vec![instance]])?;
        prover.assert_satisfied();

        Ok(())
    }

    #[test]
    fn display_circuit() {
        // Prepare the circuit you want to render.
        // You don't need to include any witness variables.
        let circuit = PrivateMulCircuit::<Fp> {
            a: Fp::from(3),
            b: Fp::from(2),
        };


        // Create the area you want to draw on.
        // Use SVGBackend if you want to render to .svg instead.
        use plotters::prelude::*;
        let root = BitMapBackend::new("layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Example Circuit Layout", ("sans-serif", 60))
            .unwrap();

        halo2_proofs::dev::CircuitLayout::default()
            // You can optionally render only a section of the circuit.
            .view_height(0..16)
            // You can hide labels, which can be useful with smaller areas.
            .show_labels(true)
            // Render the circuit onto your area!
            // The first argument is the size parameter for the circuit.
            .render(5, &circuit, &root)
            .unwrap();

    }
}