use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::circuit::{AssignedCell, Layouter};
use halo2_proofs::plonk::{Advice, Column, Constraints, ConstraintSystem, Error, Selector};
use halo2_proofs::poly::Rotation;

pub(crate) trait PrivateMulInstruction<F: PrimeField> {
    type Num;

    fn mul(&self, layouter: &mut impl Layouter<F>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
}

#[derive(Debug, Clone)]
pub(crate) struct PrivateMulChipConfig {
    pub(crate) advice: [Column<Advice>; 3],
    pub(crate) selector: Selector,
}

#[derive(Debug, Clone)]
pub(crate) struct PrivateMulChip<F: PrimeField> {
    config: PrivateMulChipConfig,
    _marker: PhantomData<F>,
}

impl <F: PrimeField> PrivateMulChip<F> {
    pub(crate) fn construct(config: PrivateMulChipConfig) -> Self {
        Self {
            config,
            _marker: PhantomData::default(),
        }
    }

    pub(crate) fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
    ) -> PrivateMulChipConfig {

        let selector = meta.selector();

        meta.create_gate("multiplication check", |cs| {
            let a = cs.query_advice(advice[0], Rotation::cur());
            let b = cs.query_advice(advice[1], Rotation::cur());
            let c = cs.query_advice(advice[2], Rotation::cur());

            Constraints::with_selector(cs.query_selector(selector), vec![a * b - c])
        });

        PrivateMulChipConfig {
            advice,
            selector,
        }
    }
}

impl <F: PrimeField> PrivateMulInstruction<F> for PrivateMulChip<F> {
    type Num = AssignedCell<F, F>;

    fn mul(&self, layouter: &mut impl Layouter<F>, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        layouter.assign_region(|| "mul region", |mut region| {
            self.config.selector.enable(&mut region, 0)?;

            a.copy_advice(
                || "mul value a",
                &mut region,
                self.config.advice[0],
                0
            )?;

            b.copy_advice(
                || "mul value b",
                &mut region,
                self.config.advice[1],
                0
            )?;

            let c = a.value().copied() * b.value();

            region.assign_advice(
                || "mul result c",
                self.config.advice[2],
                0,
                || c
            )
        })
    }
}
