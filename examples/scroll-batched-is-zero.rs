use std::marker::PhantomData;

use halo2_extr::{extraction::ExtractingAssignment, field::TermField, scroll::gadgets::batched_is_zero::{BatchedIsZeroChip, BatchedIsZeroConfig}};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront, FirstPhase, Selector},
    poly::Rotation,
};

#[derive(Clone, Debug)]
struct TestCircuitConfig<const N: usize> {
    q_enable: Selector,
    values: [Column<Advice>; N],
    is_zero: BatchedIsZeroConfig,
    expect_is_zero: Column<Advice>,
}

#[derive(Default)]
struct TestCircuit<const N: usize> {
    values: Option<[u64; N]>,
    expect_is_zero: Option<bool>,
    _marker: PhantomData<TermField>,
}

impl<const N: usize> Circuit<TermField> for TestCircuit<N> {
    type Config = TestCircuitConfig<N>;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<TermField>) -> Self::Config {
        let q_enable = meta.complex_selector();
        let values = [(); N].map(|_| meta.advice_column());
        let expect_is_zero = meta.advice_column();

        let is_zero = BatchedIsZeroChip::configure(
            meta,
            (FirstPhase, FirstPhase),
            |meta| meta.query_selector(q_enable),
            |meta| values.map(|value| meta.query_advice(value, Rotation::cur())),
        );

        let config = Self::Config {
            q_enable,
            values,
            expect_is_zero,
            is_zero,
        };

        meta.create_gate("check is_zero", |meta| {
            let q_enable = meta.query_selector(q_enable);

            // This verifies is_zero is calculated correctly
            let expect_is_zero = meta.query_advice(config.expect_is_zero, Rotation::cur());
            let is_zero = meta.query_advice(config.is_zero.is_zero, Rotation::cur());
            vec![q_enable * (is_zero - expect_is_zero)]
        });

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<(), ErrorFront> {
        let is_zero = BatchedIsZeroChip::construct(config.is_zero);

        let values: [TermField; N] = self
            .values
            .as_ref()
            .map(|values| values.map(|value| TermField::from(value)))
            .ok_or(halo2_frontend::plonk::Error::Synthesis)?;
        let expect_is_zero = self.expect_is_zero.as_ref().ok_or(halo2_frontend::plonk::Error::Synthesis)?;

        layouter.assign_region(
            || "witness",
            |mut region| {
                config.q_enable.enable(&mut region, 0)?;
                region.assign_advice(
                    || "expect_is_zero",
                    config.expect_is_zero,
                    0,
                    || Value::known(TermField::from(*expect_is_zero as u64)),
                )?;
                for (value_column, value) in config.values.iter().zip(values.iter()) {
                    region.assign_advice(
                        || "value",
                        *value_column,
                        0,
                        || Value::known(*value),
                    )?;
                }
                is_zero.assign(&mut region, 0, Value::known(values)).unwrap();
                Ok(())
            },
        )
    }
}

fn main() {
    let circuit = TestCircuit::<3> {
        values: Some([0,0,0]),
        expect_is_zero: Some(true),
        _marker: PhantomData,
    };
    ExtractingAssignment::run(&circuit, "BatchedIsZero", &[]).unwrap();
}
