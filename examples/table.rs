use std::marker::PhantomData;

use ff::PrimeField;
use halo2_extr::{extraction::ExtractingAssignment, field::TermField};
use halo2_frontend::{circuit::*, plonk::*};
use halo2_proofs::poly::Rotation;

/// This helper checks that the value witnessed in a given cell is within a given range.
/// Depending on the range, this helper uses either a range-check expression (for small ranges),
/// or a lookup (for large ranges).
///
///        value     |    q_range_check    |   q_lookup  |  table_value  |
///       ----------------------------------------------------------------
///          v_0     |         1           |      0      |       0       |
///          v_1     |         0           |      1      |       1       |
///

/// A lookup table of values from 0..RANGE.
#[derive(Debug, Clone)]
struct RangeTableConfig<F: PrimeField, const RANGE: usize> {
    value: TableColumn,
    extra_column: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> RangeTableConfig<F, RANGE> {
    fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let value = meta.lookup_table_column();
        let extra_column = meta.lookup_table_column();

        Self {
            value,
            extra_column,
            _marker: PhantomData,
        }
    }

    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "load range-check table",
            |mut table| {
                let mut offset = 0;
                for value in 3..(RANGE+3) {
                    table.assign_cell(
                        || "num_bits",
                        self.value,
                        offset,
                        || Value::known(F::from(value as u64)),
                    )?;
                    offset += 1;
                }

                for value in 0..30 {
                    table.assign_cell(
                        || "num_bits",
                        self.extra_column,
                        value,
                        || Value::known(F::from(value as u64)),
                    )?;
                }

                Ok(())
            },
        )
    }
}

// ===== END halo2-examples/src/range_check/example2/table.rs

// ===== START halo2-examples/src/range_check/example2.rs
#[derive(Debug, Clone)]
/// A range-constrained value in the circuit produced by the RangeCheckConfig.
struct RangeConstrained<F: PrimeField, const RANGE: usize>(AssignedCell<Assigned<F>, F>);

#[derive(Debug, Clone)]
struct RangeCheckConfig<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
    q_range_check: Selector,
    q_lookup: Selector,
    value: Column<Advice>,
    table: RangeTableConfig<F, LOOKUP_RANGE>,
}

impl<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize>
    RangeCheckConfig<F, RANGE, LOOKUP_RANGE>
{
    pub fn configure(meta: &mut ConstraintSystem<F>, value: Column<Advice>) -> Self {
        let q_range_check = meta.selector();
        let q_lookup = meta.complex_selector();
        let table = RangeTableConfig::configure(meta);

        meta.create_gate("range check", |meta| {
            //        value     |    q_range_check
            //       ------------------------------
            //          v       |         1

            let q = meta.query_selector(q_range_check);
            let value = meta.query_advice(value, Rotation::cur());

            // Given a range R and a value v, returns the expression
            // (v) * (1 - v) * (2 - v) * ... * (R - 1 - v)
            let range_check = |range: usize, value: Expression<F>| {
                assert!(range > 0);
                (1..range).fold(value.clone(), |expr, i| {
                    expr * (Expression::Constant(F::from(i as u64)) - value.clone())
                })
            };

            Constraints::with_selector(q, [("range check", range_check(RANGE, value))])
        });

        meta.lookup("table look", |meta| {
            let q_lookup = meta.query_selector(q_lookup);
            let value = meta.query_advice(value, Rotation::cur());

            vec![(q_lookup * value, table.value)]
        });

        Self {
            q_range_check,
            q_lookup,
            value,
            table
        }
    }

    pub fn assign_simple(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<RangeConstrained<F, RANGE>, Error> {
        layouter.assign_region(
            || "Assign value for simple range check",
            |mut region| {
                let offset = 0;

                // Enable q_range_check
                self.q_range_check.enable(&mut region, offset)?;

                // Assign value
                region
                    .assign_advice(|| "value", self.value, offset, || value)
                    .map(RangeConstrained)
            },
        )
    }

    pub fn assign_lookup(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<RangeConstrained<F, LOOKUP_RANGE>, Error> {
        layouter.assign_region(
            || "Assign value for lookup range check",
            |mut region| {
                let offset = 0;

                // Enable q_lookup
                self.q_lookup.enable(&mut region, offset)?;

                // Assign value
                region
                    .assign_advice(|| "value", self.value, offset, || value)
                    .map(RangeConstrained)
            },
        )
    }
}

#[derive(Default)]
struct MyCircuit<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
    value: Value<Assigned<F>>,
    lookup_value: Value<Assigned<F>>,
}

impl<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> Circuit<F>
    for MyCircuit<F, RANGE, LOOKUP_RANGE>
{
    type Config = RangeCheckConfig<F, RANGE, LOOKUP_RANGE>;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let value = meta.advice_column();
        RangeCheckConfig::configure(meta, value)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.table.load(&mut layouter)?;

        config.assign_simple(layouter.namespace(|| "Assign simple value"), self.value)?;
        config.assign_lookup(
            layouter.namespace(|| "Assign lookup value"),
            self.lookup_value,
        )?;

        Ok(())
    }
}

fn main() {
    // Instantiate the circuit with the private inputs.
    const RANGE: usize = 8;
    const LOOKUP_RANGE: usize = 30;
    let circuit = MyCircuit::<TermField, RANGE, LOOKUP_RANGE> {
        value: Value::known(TermField::create_symbol("value").into()),
        lookup_value: Value::known(TermField::create_symbol("lookup_value").into()),
    };

    ExtractingAssignment::run(&circuit, "LookupExamples.Table", &["value", "lookup_value"]);
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{
        circuit::floor_planner::V1,
        dev::{FailureLocation, MockProver, VerifyFailure},
        halo2curves::pasta::Fp,
        plonk::{Any, Circuit},
    };

    use super::*;

    #[derive(Default)]
    struct MyCircuit<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
        value: Value<Assigned<F>>,
        lookup_value: Value<Assigned<F>>,
    }

    impl<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> Circuit<F>
        for MyCircuit<F, RANGE, LOOKUP_RANGE>
    {
        type Config = RangeCheckConfig<F, RANGE, LOOKUP_RANGE>;
        type FloorPlanner = V1;
        type Params = ();

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let value = meta.advice_column();
            RangeCheckConfig::configure(meta, value)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            config.table.load(&mut layouter)?;

            config.assign_simple(layouter.namespace(|| "Assign simple value"), self.value)?;
            config.assign_lookup(layouter.namespace(|| "Assign lookup value"), self.lookup_value,)?;

            Ok(())
        }
    }

    #[test]
    fn test_range_check_2() {
        let k = 9;
        const RANGE: usize = 8; // 3-bit value
        const LOOKUP_RANGE: usize = 256; // 8-bit value

        println!("Successful cases");
        // Successful cases
        for i in 0..RANGE {
            for j in 0..LOOKUP_RANGE {
                let circuit = MyCircuit::<Fp, RANGE, LOOKUP_RANGE> {
                    value: Value::known(Fp::from(i as u64).into()),
                    lookup_value: Value::known(Fp::from(j as u64).into()),
                };

                let prover = MockProver::run(k, &circuit, vec![]).unwrap();
                prover.assert_satisfied();
            }
        }

        // println!("Unsuccessful cases");
        // // Out-of-range `value = 8`, `lookup_value = 256`
        // {
        //     let circuit = MyCircuit::<Fp, RANGE, LOOKUP_RANGE> {
        //         value: Value::known(Fp::from(RANGE as u64).into()),
        //         lookup_value: Value::known(Fp::from(LOOKUP_RANGE as u64).into()),
        //     };
        //     let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        //     assert_eq!(
        //         prover.verify(),
        //         Err(vec![
        //             VerifyFailure::ConstraintNotSatisfied {
        //                 constraint: ((0, "range check").into(), 0, "range check").into(),
        //                 location: FailureLocation::InRegion {
        //                     region: (1, "Assign value for simple range check").into(),
        //                     offset: 0
        //                 },
        //                 // TODO
        //                 cell_values: vec![]
        //             },
        //             VerifyFailure::Lookup {
        //                 name: "PLACEHOLDER NAME".to_string(),
        //                 lookup_index: 0,
        //                 location: FailureLocation::InRegion {
        //                     region: (2, "Assign value for lookup range check").into(),
        //                     offset: 0
        //                 }
        //             }
        //         ])
        //     );
        // }
    }

    // #[cfg(feature = "dev-graph")]
    // #[test]
    // fn print_range_check_2() {
    //     use plotters::prelude::*;

    //     let root = BitMapBackend::new("range-check-2-layout.png", (1024, 3096)).into_drawing_area();
    //     root.fill(&WHITE).unwrap();
    //     let root = root
    //         .titled("Range Check 2 Layout", ("sans-serif", 60))
    //         .unwrap();

    //     let circuit = MyCircuit::<Fp, 8, 256> {
    //         value: Value::unknown(),
    //         lookup_value: Value::unknown(),
    //     };
    //     halo2_proofs::dev::CircuitLayout::default()
    //         .render(9, &circuit, &root)
    //         .unwrap();
    // }
}
