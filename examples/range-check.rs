use std::marker::PhantomData;

use ff::PrimeField;
use halo2_extr::{field::TermField, extraction::ExtractingAssignment};
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, Value, SimpleFloorPlanner},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Expression, Selector, Circuit, ErrorFront},
    poly::Rotation,
};

use halo2_frontend::plonk::Assigned;

/// This helper checks that the value witnessed in a given cell is within a given range.
///
///        value     |    q_range_check
///       ------------------------------
///          v       |         1
///

//#[derive(Debug, Clone)]
/// A range-constrained value in the circuit produced by the RangeCheckConfig.
//struct RangeConstrained<F: PrimeField, const RANGE: usize>(AssignedCell<F, F>);

#[derive(Debug, Clone)]
struct RangeCheckConfig<F: PrimeField, const RANGE: usize> {
    value: Column<Advice>,
    q_range_check: Selector,
    _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> RangeCheckConfig<F, RANGE> {
    pub fn configure(meta: &mut ConstraintSystem<F>, value: Column<Advice>) -> Self {
        let q_range_check = meta.selector();

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

        Self {
            q_range_check,
            value,
            _marker: PhantomData,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<AssignedCell<Assigned<F>, F>, ErrorFront> {
        layouter.assign_region(
            || "RangeCheck Region",
            |mut region| {
                let offset = 0;

                // Enable q_range_check
                self.q_range_check.enable(&mut region, offset)?;

                // Assign value
                let cell_value = region
                    .assign_advice(|| "value", self.value, offset, || value)?;

               Ok(cell_value)
            },
        )
    }
}

    #[derive(Default)]
    struct MyCircuit<F: PrimeField, const RANGE: usize> {
        value: Value<Assigned<F>>,
    }

    impl<F: PrimeField, const RANGE: usize> Circuit<F> for MyCircuit<F, RANGE> {
        type Config = RangeCheckConfig<F, RANGE>;
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
            config.assign(layouter.namespace(|| "Assign value"), self.value)?;

            Ok(())
        }
    }

fn main() {
    const RANGE: usize = 10;
    let circuit = MyCircuit::<TermField, RANGE> {value: Value::known(TermField::from(5).into())};

    ExtractingAssignment::run(&circuit, "RangeCheck", &[]).unwrap();
}

//     #[test]
//     fn test_range_check_1() {
//         let k = 4;
//         const RANGE: usize = 8; // 3-bit value

//         // Successful cases
//         for i in 0..RANGE {
//             let circuit = MyCircuit::<Fp, RANGE> {
//                 value: Value::known(Fp::from(i as u64).into()),
//             };

//             let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//             prover.assert_satisfied();
//         }

//         // Out-of-range `value = 8`
//         {
//             let circuit = MyCircuit::<Fp, RANGE> {
//                 value: Value::known(Fp::from(RANGE as u64).into()),
//             };
//             let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//             assert_eq!(
//                 prover.verify(),
//                 Err(vec![VerifyFailure::ConstraintNotSatisfied {
//                     constraint: ((0, "range check").into(), 0, "range check").into(),
//                     location: FailureLocation::InRegion {
//                         region: (0, "Assign value").into(),
//                         offset: 0
//                     },
//                     cell_values: vec![(((Any::Advice, 0).into(), 0).into(), "0x8".to_string())]
//                 }])
//             );
//         }
//     }

//     #[cfg(feature = "dev-graph")]
//     #[test]
//     fn print_range_check_1() {
//         use plotters::prelude::*;

//         let root = BitMapBackend::new("range-check-1-layout.png", (1024, 3096)).into_drawing_area();
//         root.fill(&WHITE).unwrap();
//         let root = root
//             .titled("Range Check 1 Layout", ("sans-serif", 60))
//             .unwrap();

//         let circuit = MyCircuit::<Fp, 8> {
//             value: Value::unknown(),
//         };
//         halo2_proofs::dev::CircuitLayout::default()
//             .render(3, &circuit, &root)
//             .unwrap();
//     }
// }