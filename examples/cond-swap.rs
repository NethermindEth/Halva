use ff::{Field, PrimeField};
use halo2_extr::{
    extraction::{print_gates, print_preamble, print_postamble, ExtractingAssignment},
    field::TermField
};
use halo2_gadgets::utilities::{
    cond_swap::{CondSwapConfig, CondSwapChip, CondSwapInstructions},
    UtilitiesInstructions
};
use halo2_proofs::{
    circuit::{Value, SimpleFloorPlanner, Layouter},
    dev::CircuitGates,
    plonk::{Circuit, ConstraintSystem, Error, FloorPlanner}
};

#[derive(Default)]
struct MyCircuit<F: Field> {
    a: Value<F>,
    b: Value<F>,
    swap: Value<bool>,
}

impl<F: PrimeField> Circuit<F> for CondSwapChip<F> {
    type Config = CondSwapConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        CondSwapChip::<F>::configure(meta, advices)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = CondSwapChip::<F>::construct(config.clone());

        // Load the pair and the swap flag into the circuit.
        let a = chip.load_private(layouter.namespace(|| "a"), config.a, self.a)?;
        // Return the swapped pair.
        let swapped_pair = chip.swap(
            layouter.namespace(|| "swap"),
            (a.clone(), self.b),
            self.swap,
        )?;

        self.swap
            .zip(a.value().zip(self.b.as_ref()))
            .zip(swapped_pair.0.value().zip(swapped_pair.1.value()))
            .assert_if_known(|((swap, (a, b)), (a_swapped, b_swapped))| {
                if *swap {
                    // Check that `a` and `b` have been swapped
                    (a_swapped == b) && (b_swapped == a)
                } else {
                    // Check that `a` and `b` have not been swapped
                    (a_swapped == a) && (b_swapped == b)
                }
            });

        Ok(())
    }
}

fn main() {
    print_preamble("Zcash.CondSwap");
    // extract!(CondSwapChip, Target::AdviceGenerator);
    

    print_gates(CircuitGates::collect::<TermField, CondSwapChip<TermField>>());
    print_postamble("ZCash.CondSwap");
}
