//! BatchedIsZero chip works as follows:
//!
//! Given a list of `values` to be checked if they are all zero:
//! - nonempty_witness = `inv(value)` for some non-zero `value` from `values` if it exists, `0`
//!   otherwise
//! - is_zero: 1 if all `values` are `0`, `0` otherwise

use ff::Field;
use halo2_proofs::{
    circuit::{Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Phase, VirtualCells},
    poly::Rotation,
};
use std::marker::PhantomData;

use crate::field::TermField;

use super::util::Expr;

// TODO: Configurable Phase

/// BatchedIsZeroChip configuration
#[derive(Clone, Debug)]
pub struct BatchedIsZeroConfig {
    /// All the values are 0
    pub is_zero: Column<Advice>,
    /// If some value is non-zero, this is its inverse
    pub nonempty_witness: Column<Advice>,
}

impl BatchedIsZeroConfig {
    /// Annotates columns of this gadget embedded within a circuit region.
    pub fn annotate_columns_in_region(&self, region: &mut Region<TermField>, prefix: &str) {
        [
            (self.is_zero, "GADGETS_BATCHED_IS_ZERO_is_zero"),
            (
                self.nonempty_witness,
                "GADGETS_BATCHED_IS_ZERO_nonempty_witness",
            ),
        ]
        .iter()
        .for_each(|(col, ann)| region.name_column(|| format!("{prefix}_{ann}"), *col));
    }
}

/// Verify that a list of values are all 0.
pub struct BatchedIsZeroChip<F, const N: usize> {
    config: BatchedIsZeroConfig,
    _marker: PhantomData<F>,
}

impl<const N: usize> BatchedIsZeroChip<TermField, N> {
    /// Configure the BatchedIsZeroChip
    pub fn configure<P: Phase>(
        meta: &mut ConstraintSystem<TermField>,
        // Phases for is_zero and nonempty_witness advice columns.
        (phase_a, phase_b): (P, P), // TODO: Remove once Phase is Copy
        q_enable: impl Fn(&mut VirtualCells<'_, TermField>) -> Expression<TermField>,
        values: impl Fn(&mut VirtualCells<'_, TermField>) -> [Expression<TermField>; N],
    ) -> BatchedIsZeroConfig {
        let is_zero = meta.advice_column_in(phase_a);
        let nonempty_witness = meta.advice_column_in(phase_b);
        meta.create_gate("is_zero is bool", |meta| {
            let is_zero = meta.query_advice(is_zero, Rotation::cur());
            [q_enable(meta) * is_zero.clone() * (is_zero - 1.expr())]
        });

        meta.create_gate("is_zero is 0 if there is any non-zero value", |meta| {
            let is_zero = meta.query_advice(is_zero, Rotation::cur());
            values(meta)
                .iter()
                .map(|value| q_enable(meta) * is_zero.clone() * value.clone())
                .collect::<Vec<_>>()
        });

        meta.create_gate("is_zero is 1 if values are all zero", |meta| {
            let is_zero = meta.query_advice(is_zero, Rotation::cur());
            let nonempty_witness = meta.query_advice(nonempty_witness, Rotation::cur());
            [q_enable(meta)
                * values(meta).iter().fold(1.expr() - is_zero, |acc, value| {
                    acc * (1.expr() - value.clone() * nonempty_witness.clone())
                })]
        });

        BatchedIsZeroConfig {
            is_zero,
            nonempty_witness,
        }
    }

    /// Assign the BatchedIsZeroChip
    pub fn assign(
        &self,
        region: &mut Region<'_, TermField>,
        offset: usize,
        values: Value<[TermField; N]>,
    ) -> Result<(), Error> {
        let config = &self.config;

        // Note the invert here will always produce a Some, but this doesn't matter because it's for advice assignment
        let is_zero_nonempty_witness = values.map(|values| {
            if let Some(inverse) = values
                .iter()
                .find_map(|value| Option::<TermField>::from(value.invert()))
            {
                (TermField::ZERO, inverse)
            } else {
                (TermField::ONE, TermField::ZERO)
            }
        });

        region.assign_advice(
            || "is_zero",
            config.is_zero,
            offset,
            || is_zero_nonempty_witness.map(|v| v.0),
        )?;
        region.assign_advice(
            || "nonempty_witness",
            config.nonempty_witness,
            offset,
            || is_zero_nonempty_witness.map(|v| v.1),
        )?;
        Ok(())
    }

    /// Given an `BatchedIsZeroConfig`, construct the chip.
    pub fn construct(config: BatchedIsZeroConfig) -> Self {
        BatchedIsZeroChip {
            config,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::extraction::ExtractingAssignment;

    use super::*;
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

    #[test]
    fn test_circuit() {
        let circuit = TestCircuit::<3> {
            values: Some([0,0,0]),
            expect_is_zero: Some(true),
            _marker: PhantomData,
        };
        ExtractingAssignment::run(&circuit, "BatchedIsZero", &[]).unwrap();
    }
}