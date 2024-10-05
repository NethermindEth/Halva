// Goal: This is an extension of example4 that performs the range check using lookup table
// Depending on the range, this helper uses either a range-check expression (for small range)
// or a look up (for larger ranges).
// The problem of the range-check expression is that this can get of a very high degree is the range is large
// The lookup has its own selector
// The lookup table have all the values that you are interested in
// v is a small value, v' is a large value
//   value  | q_range_check | q_lookup | table_value
//   v      | 1             |  0       |   0
//   v'     | 0             |  1       |   1

use std::marker::PhantomData;

use ff::PrimeField;

use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// here we are gonna define our lookup table
// A lookup table of values up to RANGE
// e.g. RANGE = 256, values = [0..255]
// Once this is create it can be used inside our main config!

// This is a table with a single column.
// TableColumn is a Fixed Column
#[derive(Debug, Clone)]
struct RangeCheckTable<F: PrimeField, const RANGE: usize> {
    value: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> RangeCheckTable<F, RANGE> {
    // create a configure function to allow to configure the table in the first place
    fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        // API to create this special fixed colum
        let value = meta.lookup_table_column();
        Self {
            value,
            _marker: PhantomData,
        }
    }

    // load function assign the values to our fixed table
    // This action is performed at key gen time
    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), ErrorFront> {
        // firstly, for some RANGE we want to load all the values and assign it to the lookup table
        // assign_table is a special api that only works for lookup tables
        layouter.assign_table(
            || "load range check table",
            |mut table| {
                let mut offset = 0;
                for i in 0..RANGE {
                    table.assign_cell(
                        || "assign cell",
                        self.value,
                        offset,
                        || Value::known(F::from(i as u64)),
                    )?;
                    offset += 1;
                }
                Ok(())
            },
        )
    }
}

//#[derive(Debug, Clone)]
/// A range-constrained value in the circuit produced by the RangeCheckConfig.
//struct RangeConstrained<F: PrimeField, const RANGE: usize>(AssignedCell<Assigned<F>, F>);

#[derive(Debug, Clone)]

// Now we add our RangeCheckTable to our config
struct RangeCheckConfig<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
    value: Column<Advice>,
    q_range_check: Selector,
    q_lookup: Selector,
    table: RangeCheckTable<F, LOOKUP_RANGE>,
}

// Write the gate for our range check Config
// It's good practive to pass advice columns to the config (rather than creating it within the config)
// because these are very likely to be shared across multiple config
impl<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize>
    RangeCheckConfig<F, RANGE, LOOKUP_RANGE>
{
    // REMEMBER THAT THE CONFIGURATION HAPPEN AT KEYGEN TIME
    fn configure(meta: &mut ConstraintSystem<F>, value: Column<Advice>) -> Self {
        // Toggles the range check constraint
        let q_range_check = meta.selector();

        // Toggles the lookup argument
        // I use a complex selector for that.
        // Simple selector cannot appear in lookup arguments.
        let q_lookup = meta.complex_selector();

        // We also need to configure our look up table and pass it to config
        let table = RangeCheckTable::configure(meta);

        let config = Self {
            q_range_check,
            q_lookup,
            value,
            table: table.clone(),
        };

        // range-check gate
        // For a value v and a range R, check that v < R
        // v * (1 - v) * (2 - v) * ... (R - 1 - v) = 0 if v is any of these values!
        meta.create_gate("range check", |meta| {
            let q_range_check = meta.query_selector(q_range_check);
            // note that we don't need to specify the rotation when querying the selctor
            // That's because the selector always get queried at the current row
            // While the advice columns get queried relatively to the selector offset, so we need to specify the relative rotation
            let value = meta.query_advice(value, Rotation::cur());

            // This is a closure that produce the expression defined previously
            let range_check = |range: usize, value: Expression<F>| {
                (0..range).fold(value.clone(), |expr: Expression<F>, i: usize| {
                    expr * (Expression::Constant(F::from(i as u64)) - value.clone())
                })
            };
            // This is a way to return the constrain from our create_gate function.
            // similar to what we were doing previously using "vec![s * (a + b - c)]"
            // this api, behind the scene, multiplies the specified constraint by the selector
            Constraints::with_selector(q_range_check, [("range check", range_check(RANGE, value))])
        });

        // range-check using lookup argument
        // Check that a value is contained within a lookup table of values 0..RANGE (exclusive)
        // api to instantiate a lookup argument
        // Similar to create gate as an api so we need to query the selector and our value
        meta.lookup("table lookup", |meta| {
            let q_lookup = meta.query_selector(q_lookup);
            let value = meta.query_advice(value, Rotation::cur());

            // The meta.lookup api expect to return a vector of tuples, where the first element
            // is what you are looking at, and the second element is the corresponding table we are looking into
            vec![(q_lookup * value, table.value)]
        });

        config
    }

    // assign value to each cell inside the advise column
    // we can modify this assign function such that under a certain range enables the simple range check expression
    // and over a certain range enables the look up argument
    // the range passed in is the actual claimed range
    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
        range: usize,
    ) -> Result<(), ErrorFront> {
        assert!(range <= LOOKUP_RANGE);

        println!("range: {range}");
        if range < RANGE {
            println!("Menor!");
            layouter.assign_region(
                || "Assign value for poly range check",
                |mut region| {
                    let offset = 0;

                    // Enable q range check
                    self.q_range_check.enable(&mut region, offset)?;

                    // assign given value and return RangeConstrained struct
                    region.assign_advice(|| "assign value", self.value, offset, || value)?;

                    Ok(())
                },
            )
        } else {
            println!("Maior igual");
            layouter.assign_region(
                || "Assign value for lookup range check",
                |mut region| {
                    let offset = 0;

                    // Enable q range check
                    self.q_lookup.enable(&mut region, offset)?;

                    // assign given value and return RangeConstrained struct
                    region.assign_advice(|| "assign value", self.value, offset, || value)?;

                    Ok(())
                },
            )
        }
    }
}

#[derive(Default)]
struct MyCircuit<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
    value: Value<Assigned<F>>,
    large_value: Value<Assigned<F>>,
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
    ) -> Result<(), ErrorFront> {
        config.assign(layouter.namespace(|| "Assign value"), self.value, 5)?;
        config.assign(layouter.namespace(|| "Assign value"), self.large_value, 10)?;
        // We need to load the values inside the lookup table!
        config.table.load(&mut layouter)?;
        Ok(())
    }
}

fn main() {
    use halo2_extr::extraction::print_gates;
    use halo2_frontend::dev::CircuitGates;
    use halo2_proofs::halo2curves::bn256::Fq;
    use halo2_extr::{extraction::Target, field::TermField, extraction::ExtractingAssignment};

    const RANGE: usize = 8;
    const LOOKUP_RANGE: usize = 20;
    let circuit = MyCircuit::<TermField, RANGE, LOOKUP_RANGE> {
        value: Value::known(TermField::from(6 as u64).into()),
        large_value: Value::known(TermField::from(2 as u64).into()),
    };

    let mut cs = ConstraintSystem::<TermField>::default();
    let config = MyCircuit::<TermField, RANGE, LOOKUP_RANGE>::configure(&mut cs);

    let mut extr_assn = ExtractingAssignment::<TermField>::new(Target::AdviceGenerator);
    <MyCircuit<TermField, RANGE, LOOKUP_RANGE> as Circuit<TermField>>::FloorPlanner::synthesize(
        &mut extr_assn,
        &circuit,
        config,
        vec![],
    )
    .unwrap();

    print_gates(CircuitGates::collect::<Fq, MyCircuit<Fq, RANGE, LOOKUP_RANGE>>(
        <MyCircuit<Fq, RANGE, LOOKUP_RANGE> as Circuit<Fq>>::Params::default(),
    ));

    let lookups = cs.lookups();
    println!("{:#?}", lookups);
    // let gates = cs.gates();
    // println!("{:#?}", lookups);
}

// // Now let's test it! Here we define a circuit with a single value. and in syntesize function we assign that value
// #[cfg(test)]
// mod tests {
//     use halo2_proofs::{
//         circuit::floor_planner::V1,
//         dev::{FailureLocation, MockProver, VerifyFailure},
//         pasta::Fp,
//         plonk::{Any, Circuit},
//     };

//     use super::*;

//     #[derive(Default)]
//     struct MyCircuit<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> {
//         value: Value<Assigned<F>>,
//         large_value: Value<Assigned<F>>
//     }

//     impl<F: PrimeField, const RANGE: usize, const LOOKUP_RANGE: usize> Circuit<F> for MyCircuit<F, RANGE, LOOKUP_RANGE> {
//         type Config = RangeCheckConfig<F, RANGE, LOOKUP_RANGE>;
//         type FloorPlanner = V1;

//         fn without_witnesses(&self) -> Self {
//             Self::default()
//         }

//         fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//             let value = meta.advice_column();
//             RangeCheckConfig::configure(meta, value)
//         }

//         fn synthesize(
//             &self,
//             config: Self::Config,
//             mut layouter: impl Layouter<F>,
//         ) -> Result<(), Error> {
//             config.assign(layouter.namespace(|| "Assign value"), self.value, RANGE)?;
//             config.assign(layouter.namespace(|| "Assign value"), self.value, LOOKUP_RANGE)?;
//             // We need to load the values inside the lookup table!
//             config.table.load(&mut layouter)?;
//             Ok(())
//         }
//     }

//     #[test]
//     fn test_range_check_2() {
//         // our lookup table is 256 rows + last few rows or the advise colums
//         // are automatically allocated to random values which are bliding factors
//         // so we need to use k=9
//         let k = 9;
//         const RANGE: usize = 8; // 3-bit value table
//         const LOOKUP_RANGE: usize = 256; // 8-bit value table

//         // Successful cases value=0,1,2,3,4,5,6,7
//         // Successful cases large_value=0,1,2,3,4,5,6,7 (these should also pass the lookup range check)
//         for i in 0..RANGE {
//             let circuit = MyCircuit::<Fp, RANGE, LOOKUP_RANGE> {
//                 value: Value::known(Fp::from(i as u64).into()),
//                 large_value : Value::known(Fp::from(i as u64).into())
//             };

//             let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//             prover.assert_satisfied();
//         }

//         // // Out-of-range `value = 8`
//         // {
//         //     let circuit = MyCircuit::<Fp, RANGE> {
//         //         value: Value::known(Fp::from(RANGE as u64).into()),
//         //     };
//         //     let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         //     // prover.assert_satisfied(); // this should fail!
//         //     assert_eq!(
//         //         prover.verify(),
//         //         Err(vec![VerifyFailure::ConstraintNotSatisfied {
//         //             constraint: ((0, "range check").into(), 0, "range check").into(),
//         //             location: FailureLocation::InRegion {
//         //                 region: (0, "Assign value").into(),
//         //                 offset: 0
//         //             },
//         //             cell_values: vec![(((Any::Advice, 0).into(), 0).into(), "0x8".to_string())]
//         //         }])
//         //     );
//         // }
//     }

//     #[cfg(feature = "dev-graph")]
//     #[test]
//     fn print_range_check_2() {
//         use plotters::prelude::*;

//         let root = BitMapBackend::new("range-check-2-layout.png", (1024, 3096)).into_drawing_area();
//         root.fill(&WHITE).unwrap();
//         let root = root
//             .titled("Range Check 2 Layout", ("sans-serif", 60))
//             .unwrap();

//         let circuit = MyCircuit::<Fp, 8, 256> {
//             value: Value::unknown(),
//             large_value: Value::unknown()
//         };
//         halo2_proofs::dev::CircuitLayout::default()
//             .render(3, &circuit, &root)
//             .unwrap();
//     }
// }
