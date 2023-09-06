use ff::PrimeFieldBits;
use halo2_extr::{extraction::{print_preamble, Target, ExtractingAssignment, print_gates, print_postamble}, field::TermField};
use halo2_gadgets::utilities::decompose_running_sum::RunningSumConfig;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Value, SimpleFloorPlanner, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, FloorPlanner, Selector, Circuit},
    poly::Rotation, pasta::{pallas, Fp}, dev::CircuitGates,
};

struct MyCircuit<
	F: PrimeFieldBits,
	const WORD_NUM_BITS: usize,
	const WINDOW_NUM_BITS: usize,
	const NUM_WINDOWS: usize,
> {
	alpha: Value<F>,
	strict: bool,
}

impl<
		F: PrimeFieldBits,
		const WORD_NUM_BITS: usize,
		const WINDOW_NUM_BITS: usize,
		const NUM_WINDOWS: usize,
	> Circuit<F> for MyCircuit<F, WORD_NUM_BITS, WINDOW_NUM_BITS, NUM_WINDOWS>
{
	type Config = RunningSumConfig<F, WINDOW_NUM_BITS>;
	type FloorPlanner = SimpleFloorPlanner;

	fn without_witnesses(&self) -> Self {
		Self {
			alpha: Value::unknown(),
			strict: self.strict,
		}
	}

	fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
		let z = meta.advice_column();
		let q_range_check = meta.selector();
		let constants = meta.fixed_column();
		meta.enable_constant(constants);

		RunningSumConfig::<F, WINDOW_NUM_BITS>::configure(meta, q_range_check, z)
	}

	fn synthesize(
		&self,
		config: Self::Config,
		mut layouter: impl Layouter<F>,
	) -> Result<(), Error> {
		layouter.assign_region(
			|| "decompose",
			|mut region| {
				let offset = 0;
				let zs = config.witness_decompose(
					&mut region,
					offset,
					self.alpha,
					self.strict,
					WORD_NUM_BITS,
					NUM_WINDOWS,
				)?;
				let alpha = zs[0].clone();

				let offset = offset + NUM_WINDOWS + 1;

				config.copy_decompose(
					&mut region,
					offset,
					alpha,
					self.strict,
					WORD_NUM_BITS,
					NUM_WINDOWS,
				)?;

				Ok(())
			},
		)
	}
}

fn main() {
	print_preamble("Zcash.DecomposeRunningSum");

	// Instantiate the circuit with the private inputs.
	let circuit = MyCircuit {
		alpha: Value::unknown(),
		strict: true
	};

    let mut cs = ConstraintSystem::<Fp>::default();
    let config = MyCircuit::<Fp, 255, 3, 83>::configure(&mut cs);

    println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

    let mut extr_assn = ExtractingAssignment::<Fp>::new(Target::AdviceGenerator);
    <MyCircuit<Fp, 255, 3, 83> as Circuit<Fp>>::FloorPlanner::synthesize(
        &mut extr_assn,
        &circuit,
        config,
        vec![],
    )
    .unwrap();

    extr_assn.print_grouping_props();
    print_gates(CircuitGates::collect::<Fp, MyCircuit<Fp, 255, 3, 83>>());
    print_postamble("Zcash.DecomposeRunningSum");
}