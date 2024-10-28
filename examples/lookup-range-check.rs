// use std::marker::PhantomData;

// use ff::PrimeFieldBits;
// use halo2_extr::{extraction::{print_preamble, ExtractingAssignment, Target, print_postamble, print_gates}, field::TermField};
// use halo2_gadgets::{utilities::{lookup_range_check::LookupRangeCheckConfig, lebs2ip}, sinsemilla::primitives::K};
// use halo2_proofs::{plonk::{Circuit, ConstraintSystem, Error, FloorPlanner}, circuit::{SimpleFloorPlanner, Layouter, Value}, dev::CircuitGates};

// #[derive(Clone, Copy)]
// struct MyCircuit<F: PrimeFieldBits> {
// 	num_words: usize,
// 	_marker: PhantomData<F>,
// }

// impl<F: PrimeFieldBits> Circuit<F> for MyCircuit<F> {
// 	type Config = LookupRangeCheckConfig<F, K>;
// 	type FloorPlanner = SimpleFloorPlanner;

// 	fn without_witnesses(&self) -> Self {
// 		*self
// 	}

// 	fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
// 		let running_sum = meta.advice_column();
// 		let table_idx = meta.lookup_table_column();
// 		let constants = meta.fixed_column();
// 		meta.enable_constant(constants);

// 		LookupRangeCheckConfig::<F, K>::configure(meta, running_sum, table_idx)
// 	}

// 	fn synthesize(
// 		&self,
// 		config: Self::Config,
// 		mut layouter: impl Layouter<F>,
// 	) -> Result<(), Error> {
// 		// Load table_idx
// 		config.load(&mut layouter)?;

// 		// Lookup constraining element to be no longer than num_words * K bits.
// 		let elements_and_expected_final_zs = [
// 			(F::from((1 << (self.num_words * K)) - 1), F::ZERO, true), // a word that is within self.num_words * K bits long
// 			(F::from(1 << (self.num_words * K)), F::ONE, false), // a word that is just over self.num_words * K bits long
// 		];

// 		// println!("{:?}", elements_and_expected_final_zs[0].0);
// 		// println!("{:?}", elements_and_expected_final_zs[0].1);
// 		// println!("{}", elements_and_expected_final_zs[0].2);
// 		// println!("{:?}", elements_and_expected_final_zs[1].0);
// 		// println!("{:?}", elements_and_expected_final_zs[1].1);
// 		// println!("{}", elements_and_expected_final_zs[1].2);

// 		fn expected_zs<F: PrimeFieldBits, const K: usize>(
// 			element: F,
// 			num_words: usize,
// 		) -> Vec<F> {
// 			let chunks = {
// 				element
// 					.to_le_bits()
// 					.iter()
// 					.by_vals()
// 					.take(num_words * K)
// 					.collect::<Vec<_>>()
// 					.chunks_exact(K)
// 					.map(|chunk| F::from(lebs2ip::<K>(chunk.try_into().unwrap())))
// 					.collect::<Vec<_>>()
// 			};
// 			// println!("Chunks: {:?}", chunks);
// 			let expected_zs = {
// 				let inv_two_pow_k = F::from(1 << K).invert().unwrap();
// 				chunks.iter().fold(vec![element], |mut zs, a_i| {
// 					// z_{i + 1} = (z_i - a_i) / 2^{K}
// 					let z = (zs[zs.len() - 1] - a_i) * inv_two_pow_k;
// 					zs.push(z);
// 					zs
// 				})
// 			};
// 			expected_zs
// 		}

// 		for (element, expected_final_z, strict) in elements_and_expected_final_zs.iter() {
// 			let expected_zs = expected_zs::<F, K>(*element, self.num_words);

// 			let zs = config.witness_check(
// 				layouter.namespace(|| format!("Lookup {:?}", self.num_words)),
// 				Value::known(*element),
// 				self.num_words,
// 				*strict,
// 			)?;

// 			// assert_eq!(*expected_zs.last().unwrap(), *expected_final_z);
// 			println!("--VERIFY EQUAL: {:?} ={:?}", expected_zs.last().unwrap(), expected_final_z);

// 			for (expected_z, z) in expected_zs.into_iter().zip(zs.iter()) {
// 				z.value().assert_if_known(|z| &&expected_z == z);
// 			}
// 		}
// 		Ok(())
// 	}
// }

// fn main() {
// 	print_preamble("Zcash.LookupRangeCheck");

// 	let circuit: MyCircuit<TermField> = MyCircuit {
// 		num_words: 6,
// 		_marker: PhantomData,
// 	};

// 	let mut cs = ConstraintSystem::<TermField>::default();
// 	let config = MyCircuit::<TermField>::configure(&mut cs);

// 	println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

// 	let mut extr_assn = ExtractingAssignment::<TermField>::new(Target::AdviceGenerator);
// 	<MyCircuit<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
// 		&mut extr_assn,
// 		&circuit,
// 		config,
// 		vec![cs.fixed_column()],
// 	).unwrap();

// 	extr_assn.print_grouping_props();
// 	print_gates(CircuitGates::collect::<TermField, MyCircuit<TermField>>());
// 	print_postamble("Zcash.LookupRangeCheck");
// }

// TEMP
fn main() {}