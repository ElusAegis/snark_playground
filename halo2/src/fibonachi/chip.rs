use std::marker::PhantomData;
use std::ops::{Mul, Neg};
use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::plonk::{Advice, Column, Constraints, ConstraintSystem, Error, Expression, Selector};
use halo2_proofs::poly::Rotation;

#[derive(Clone)]
pub(crate) struct FibChipConfig {
    selector: Selector,
    initiator: Selector,
    num1: Column<Advice>,
    num2: Column<Advice>
}

pub(crate) struct FibChip<F: PrimeField> {
    config: FibChipConfig,
    _marker: PhantomData<F>
}

impl <F: PrimeField> FibChip<F> {

    pub(crate) fn configure_inside(
        meta: &mut ConstraintSystem<F>,
    ) -> FibChipConfig {
        let num1 = meta.advice_column();
        let num2 = meta.advice_column();

        let selector = meta.selector();
        let initiator = meta.complex_selector();

        meta.create_gate("next_num_check", |meta| {
            let next = meta.query_advice(num1, Rotation::cur());
            let prev = meta.query_advice(num1, Rotation::prev());
            let prev_prev = meta.query_advice(num2, Rotation::prev());


            // Constraints::with_selector(initiator.expr().neg().mul(selector.expr()), vec![(next - prev - prev_prev)])
            vec![(next - prev - prev_prev) * Expression::Selector(initiator) ]
        });

        meta.create_gate("initiation_check", |meta| {
            let init_1 = meta.query_advice(num1, Rotation::cur());
            let init_2 = meta.query_advice(num2, Rotation::cur());

            let enable = Expression::Selector(selector).mul(Expression::Constant(F::ONE) - Expression::Selector(initiator));

            Constraints::with_selector(enable, vec![init_1 - Expression::Constant(F::ONE), init_2 - Expression::Constant(F::ONE)])
        });

        FibChipConfig {
            selector,
            initiator,
            num1,
            num2
        }

    }

    #[allow(dead_code)]
    fn configure_outside(
        meta: &mut ConstraintSystem<F>,
        num1: Column<Advice>,
        num2: Column<Advice>,
        initiator: Selector
    ) -> FibChipConfig {
        let selector = meta.selector();

        meta.create_gate("next_num_check", |meta| {
            let next = meta.query_advice(num1, Rotation::next());
            let prev = meta.query_advice(num1, Rotation::cur());
            let prev_prev = meta.query_advice(num2, Rotation::cur());

            let neg_init = Expression::Selector(initiator).neg().mul(Expression::Selector(selector));

            Constraints::with_selector(neg_init, vec![next - prev - prev_prev])
        });

        meta.create_gate("initiation_check", |meta| {
            let init_1 = meta.query_advice(num1, Rotation::cur());
            let init_2 = meta.query_advice(num2, Rotation::cur());

            let enable = Expression::Selector(initiator).mul(Expression::Selector(selector));

            Constraints::with_selector(enable, vec![init_1 - Expression::Constant(F::ONE), init_2 - Expression::Constant(F::ONE)])
        });

        FibChipConfig {
            selector,
            initiator,
            num1,
            num2
        }
    }

    pub(crate) fn construct(config: FibChipConfig) -> Self {
        Self {
            config,
            _marker: Default::default(),
        }
    }

    pub(crate) fn assign(&self, layouter: &mut impl Layouter<F>, target: u64) -> Result<AssignedCell<F, F>, Error> {

        layouter.assign_region(|| "initial row", |mut region| {

            self.config.selector.enable(&mut region, 0)?;

            let mut last = region.assign_advice(
                || "initial 0",
                self.config.num1,
                0,
                || {Value::known(F::ONE)}
            )?;

            region.assign_advice(
                || "initial 1",
                self.config.num2,
                0,
                || {Value::known(F::ONE)}
            )?;


            let mut state = (1, 1);
            let mut i = 1;

            while target > state.0 {
                state = (state.0 + state.1, state.0);

                self.config.initiator.enable(&mut region, i)?;
                self.config.selector.enable(&mut region, i)?;

                // This is the value that we want to achieve
                last = region.assign_advice(
                    || format!("iter {} cur", i),
                    self.config.num1,
                    i,
                    || { Value::known(F::from(state.0)) }
                )?;

                region.assign_advice(
                    || format!("iter {} prev", i),
                    self.config.num2,
                    i,
                    || { Value::known(F::from(state.1)) }
                )?;

                i += 1;
            }

            if target != state.0 {
                return Err(Error::InvalidInstances);
            }

            Ok(last)
        })
    }

}