//! The binary number chip implements functionality to represent any given value
//! in binary bits, which can be compared against a value or expression for
//! equality.

use crate::{
    field::TermField,
    scroll::gadgets::util::{and, not, Expr},
};
use halo2_proofs::{
    circuit::{Region, Value},
    plonk::{Advice, Any, Column, ConstraintSystem, Error, Expression, Fixed, VirtualCells},
    poly::Rotation,
};
use std::{collections::BTreeSet, marker::PhantomData};
use strum::IntoEnumIterator;

/// Helper trait that implements functionality to represent a generic type as
/// array of N-bits.
pub trait AsBits<const N: usize> {
    /// Return the bits of self, starting from the most significant.
    fn as_bits(&self) -> [bool; N];
}

impl<T, const N: usize> AsBits<N> for T
where
    T: Copy + Into<usize>,
{
    fn as_bits(&self) -> [bool; N] {
        let mut bits = [false; N];
        let mut x: usize = (*self).into();
        for i in 0..N {
            bits[N - 1 - i] = x % 2 == 1;
            x /= 2;
        }
        bits
    }
}

/// Config for the binary number chip.
#[derive(Clone, Copy, Debug)]
pub struct BinaryNumberConfig<T, const N: usize> {
    /// Must be constrained to be binary for correctness.
    pub bits: [Column<Advice>; N],
    _marker: PhantomData<T>,
}

impl<T, const N: usize> BinaryNumberConfig<T, N>
where
    T: AsBits<N>,
{
    /// Returns the expression value of the bits at the given rotation.
    pub fn value(
        &self,
        rotation: Rotation,
    ) -> impl FnOnce(&mut VirtualCells<'_, TermField>) -> Expression<TermField> {
        let bits = self.bits;
        move |meta: &mut VirtualCells<'_, TermField>| {
            let bits = bits.map(|bit| meta.query_advice(bit, rotation));
            bits.iter()
                .fold(0.expr(), |result, bit| bit.clone() + result * 2.expr())
        }
    }

    /// Return the constant that represents a given value. To be compared with the value expression.
    pub fn constant_expr(&self, value: T) -> Expression<TermField> {
        let f = value
            .as_bits()
            .iter()
            .fold(TermField::zero(), |result, bit| {
                TermField::from(*bit as u64) + result * TermField::from(2)
            });
        Expression::Constant(f)
    }

    /// Returns a function that can evaluate to a binary expression, that
    /// evaluates to 1 if value is equal to value as bits. The returned
    /// expression is of degree N.
    pub fn value_equals<S: AsBits<N>>(
        &self,
        value: S,
        rotation: Rotation,
    ) -> impl FnOnce(&mut VirtualCells<'_, TermField>) -> Expression<TermField> {
        let bits = self.bits;
        move |meta| Self::value_equals_expr(value, bits.map(|bit| meta.query_advice(bit, rotation)))
    }

    /// Returns a binary expression that evaluates to 1 if expressions are equal
    /// to value as bits. The returned expression is of degree N.
    pub fn value_equals_expr<S: AsBits<N>>(
        value: S,
        expressions: [Expression<TermField>; N], // must be binary.
    ) -> Expression<TermField> {
        and::expr(
            value
                .as_bits()
                .iter()
                .zip(&expressions)
                .map(|(&bit, expression)| {
                    if bit {
                        expression.clone()
                    } else {
                        not::expr(expression.clone())
                    }
                }),
        )
    }

    /// Annotates columns of this gadget embedded within a circuit region.
    pub fn annotate_columns_in_region(&self, region: &mut Region<TermField>, prefix: &str) {
        let mut annotations = Vec::new();
        for (i, _) in self.bits.iter().enumerate() {
            annotations.push(format!("GADGETS_binary_number_{i}"));
        }
        self.bits
            .iter()
            .zip(annotations.iter())
            .for_each(|(col, ann)| region.name_column(|| format!("{prefix}_{ann}"), *col));
    }
}

/// This chip helps working with binary encoding of integers of length N bits
/// by:
///  - enforcing that the binary representation is in the valid range defined by T.
///  - creating expressions (via the Config) that evaluate to 1 when the bits match a specific value
///    and 0 otherwise.
#[derive(Clone, Debug)]
pub struct BinaryNumberChip<T, const N: usize> {
    config: BinaryNumberConfig<T, N>,
    _marker: PhantomData<TermField>,
}

impl<T: IntoEnumIterator, const N: usize> BinaryNumberChip<T, N>
where
    T: AsBits<N>,
{
    /// Construct the binary number chip given a config.
    pub fn construct(config: BinaryNumberConfig<T, N>) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    /// Configure constraints for the binary number chip.
    pub fn configure(
        meta: &mut ConstraintSystem<TermField>,
        selector: Column<Fixed>,
        value: Option<Column<Any>>,
    ) -> BinaryNumberConfig<T, N> {
        let bits = [0; N].map(|_| meta.advice_column());
        bits.map(|bit| {
            meta.create_gate("bit column is 0 or 1", |meta| {
                let selector = meta.query_fixed(selector, Rotation::cur());
                let bit = meta.query_advice(bit, Rotation::cur());
                vec![selector * bit.clone() * (1.expr() - bit)]
            })
        });

        let config = BinaryNumberConfig {
            bits,
            _marker: PhantomData,
        };

        if let Some(value) = value {
            meta.create_gate("binary number value", |meta| {
                let selector = meta.query_fixed(selector, Rotation::cur());
                vec![
                    selector
                        * (config.value(Rotation::cur())(meta)
                            - meta.query_any(value, Rotation::cur())),
                ]
            });
        }

        // Disallow bit patterns (if any) that don't correspond to a variant of T.
        let valid_values: BTreeSet<usize> = T::iter().map(|t| from_bits(&t.as_bits())).collect();
        let mut invalid_values = (0..1 << N).filter(|i| !valid_values.contains(i)).peekable();
        if invalid_values.peek().is_some() {
            meta.create_gate("binary number value in range", |meta| {
                let selector = meta.query_fixed(selector, Rotation::cur());
                invalid_values
                    .map(|i| {
                        let value_equals_i = config.value_equals(i, Rotation::cur());
                        selector.clone() * value_equals_i(meta)
                    })
                    .collect::<Vec<_>>()
            });
        }

        config
    }

    /// Assign a value to the binary number chip. A generic type that implements
    /// the AsBits trait can be provided for assignment.
    pub fn assign(
        &self,
        region: &mut Region<'_, TermField>,
        offset: usize,
        value: &T,
    ) -> Result<(), Error> {
        for (&bit, &column) in value.as_bits().iter().zip(&self.config.bits) {
            region.assign_advice(
                || format!("binary number {column:?}"),
                column,
                offset,
                || Value::known(TermField::from(bit as u64)),
            )?;
        }
        Ok(())
    }
}

/// Helper function to get a decimal representation given the bits.
pub fn from_bits(bits: &[bool]) -> usize {
    bits.iter()
        .fold(0, |result, &bit| bit as usize + 2 * result)
}

#[cfg(test)]
mod test {
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::Circuit,
    };

    use crate::{extraction::ExtractingAssignment, field::TermField};

    use super::*;

    use halo2_frontend::plonk::Error;

    use strum::EnumIter;

    #[derive(Clone, Debug, EnumIter)]
    enum TestEnum {
        A,
        B,
        C,
    }

    impl AsBits<2> for TestEnum {
        fn as_bits(&self) -> [bool; 2] {
            match self {
                TestEnum::A => [false, false],
                TestEnum::B => [false, true],
                TestEnum::C => [true, false],
            }
        }
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct TestCircuitConfig {
        q_enable: Column<Fixed>,
        value: Column<Fixed>,
        binary_number_chip_config: BinaryNumberConfig<TestEnum, 2>,
    }

    #[derive(Default)]
    struct TestCircuit {}

    impl Circuit<TermField> for TestCircuit {
        type Config = TestCircuitConfig;
        type FloorPlanner = SimpleFloorPlanner;
        type Params = ();

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<TermField>) -> Self::Config {
            // Binary Config requires a fixed rather than a selector as its 'selector' column
            let q_enable = meta.fixed_column();
            let value = meta.fixed_column();

            let binary_number_chip_config =
                BinaryNumberChip::configure(meta, q_enable, Some(value.into()));

            let config = Self::Config {
                q_enable,
                value,
                binary_number_chip_config,
            };

            // meta.create_gate("check_matches", |meta| {
            //     let q_enable = meta.query_selector(q_enable);
            //     let is_a = binary_number_chip_config.value_equals(TestEnum::A, Rotation::cur());
            //     let is_b = binary_number_chip_config.value_equals(TestEnum::B, Rotation::cur());
            //     let is_c = binary_number_chip_config.value_equals(TestEnum::C, Rotation::cur());


            // });

            config
        }

        fn synthesize(
            &self,
            _config: Self::Config,
            mut layouter: impl Layouter<TermField>,
        ) -> Result<(), Error> {
            layouter.assign_region(
                || "witness",
                |mut _region| {
                    
                    Ok(())
                }
            )
        }
    }

    #[test]
    fn test_circuit() {
        let circuit = TestCircuit {};
        ExtractingAssignment::run(&circuit, "BinaryNumber", &[]).unwrap();
    }
}
