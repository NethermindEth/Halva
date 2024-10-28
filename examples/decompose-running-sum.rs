

// use ff::{PrimeFieldBits};
// use halo2_extr::{extraction::{print_preamble, Target, ExtractingAssignment, print_gates, print_postamble}, field::TermField};
// use halo2_gadgets::utilities::decompose_running_sum::RunningSumConfig;
// use halo2_proofs::{
//     circuit::{Value, SimpleFloorPlanner, Layouter},
//     plonk::{ConstraintSystem, Error, FloorPlanner, Circuit}, dev::CircuitGates,
// };


// struct MyCircuit<
// 	F: PrimeFieldBits,
// 	const WORD_NUM_BITS: usize,
// 	const WINDOW_NUM_BITS: usize,
// 	const NUM_WINDOWS: usize,
// > {
// 	alpha: Value<F>,
// 	strict: bool,
// }

// impl<
// 		F: PrimeFieldBits,
// 		const WORD_NUM_BITS: usize,
// 		const WINDOW_NUM_BITS: usize,
// 		const NUM_WINDOWS: usize,
// 	> Circuit<F> for MyCircuit<F, WORD_NUM_BITS, WINDOW_NUM_BITS, NUM_WINDOWS>
// {
// 	type Config = RunningSumConfig<F, WINDOW_NUM_BITS>;
// 	type FloorPlanner = SimpleFloorPlanner;

// 	fn without_witnesses(&self) -> Self {
// 		Self {
// 			alpha: Value::unknown(),
// 			strict: self.strict,
// 		}
// 	}

// 	fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
// 		let z = meta.advice_column();
// 		let q_range_check = meta.selector();
// 		let constants = meta.fixed_column();
// 		meta.enable_constant(constants);

// 		RunningSumConfig::<F, WINDOW_NUM_BITS>::configure(meta, q_range_check, z)
// 	}

// 	fn synthesize(
// 		&self,
// 		config: Self::Config,
// 		mut layouter: impl Layouter<F>,
// 	) -> Result<(), Error> {
// 		layouter.assign_region(
// 			|| "decompose",
// 			|mut region| {
// 				let offset = 0;
// 				let zs = config.witness_decompose(
// 					&mut region,
// 					offset,
// 					self.alpha,
// 					self.strict,
// 					WORD_NUM_BITS,
// 					NUM_WINDOWS,
// 				)?;
// 				let alpha = zs[0].clone();

// 				let offset = offset + NUM_WINDOWS + 1;

// 				config.copy_decompose(
// 					&mut region,
// 					offset,
// 					alpha,
// 					self.strict,
// 					WORD_NUM_BITS,
// 					NUM_WINDOWS,
// 				)?;

// 				Ok(())
// 			},
// 		)
// 	}
// }

// // struct MyField(Fp);

// // impl From<String> for MyField {
// //     fn from(value: String) -> Self {
// //         todo!()
// //     }
// // }

// // impl Field for MyField {
// //     const ZERO: Self = MyField(Fp::ZERO);

// //     const ONE: Self = MyField(Fp::ONE);

// //     fn random(rng: impl rand_core::RngCore) -> Self {
// //         MyField(Fp::random(rng))
// //     }

// //     fn square(&self) -> Self {
// //         MyField(self.0.square())
// //     }

// //     fn double(&self) -> Self {
// //         MyField(self.0.double())
// //     }

// //     fn invert(&self) -> subtle::CtOption<Self> {
// //         MyField(self.0.invert())
// //     }

// //     fn sqrt_ratio(num: &Self, div: &Self) -> (subtle::Choice, Self) {
// //         MyField(Fp::sqrt_ratio(&num.0, &div.0))
// //     }
// // }

// // impl Neg for MyField {
// //     type Output = MyField;

// //     fn neg(self) -> Self::Output {
// //         MyField(self.0.neg())
// //     }
// // }

// // impl ConstantTimeEq for MyField {
// //     fn ct_eq(&self, other: &Self) -> subtle::Choice {
// //         self.0.ct_eq(&other.0)
// //     }
// // }

// // impl ConditionallySelectable for MyField {
// //     fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
// //         MyField(Fp::conditional_select(&a.0, &b.0, choice))
// //     }
// // }

// // impl <'a> Mul for MyField {
// //     type Output = MyField;

// //     fn mul(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.mul(rhs.0))
// //     }
// // }

// // impl <'a> Mul<&'a MyField> for MyField {
// //     type Output = MyField;

// //     fn mul(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.mul(rhs.0))
// //     }
// // }

// // impl <'a> MulAssign for MyField {
// //     fn mul_assign(&mut self, rhs: &'a MyField) {
// //         self.0 *= rhs.0;
// //     }
// // }

// // impl <'a> MulAssign<&'a MyField> for MyField {
// //     fn mul_assign(&mut self, rhs: &'a MyField) {
// //         self.0 *= rhs.0;
// //     }
// // }

// // impl <'a> Sub for MyField {
// //     type Output = MyField;

// //     fn sub(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.sub(rhs.0))
// //     }
// // }

// // impl <'a> Sub<&'a MyField> for MyField {
// //     type Output = MyField;

// //     fn sub(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.sub(rhs.0))
// //     }
// // }

// // impl <'a> SubAssign for MyField {
// //     fn sub_assign(&mut self, rhs: &'a MyField) {
// //         self.0 -= rhs.0;
// //     }
// // }

// // impl <'a> SubAssign<&'a MyField> for MyField {
// //     fn sub_assign(&mut self, rhs: &'a MyField) {
// //         self.0 -= rhs.0;
// //     }
// // }

// // impl <'a> Add for MyField {
// //     type Output = MyField;

// //     fn add(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.add(rhs.0))
// //     }
// // }

// // impl <'a> Add<&'a MyField> for MyField {
// //     type Output = MyField;

// //     fn add(self, rhs: &'a MyField) -> Self::Output {
// //         MyField(self.0.add(rhs.0))
// //     }
// // }

// // impl <'a> AddAssign for MyField {
// //     fn add_assign(&mut self, rhs: &'a MyField) {
// //         self.0 += rhs.0;
// //     }
// // }

// // impl <'a> AddAssign<&'a MyField> for MyField {
// //     fn add_assign(&mut self, rhs: &'a MyField) {
// //         self.0 += rhs.0;
// //     }
// // }

// // impl<T: ::core::borrow::Borrow<MyField>> ::core::iter::Sum<T> for MyField {
// //     fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
// //         iter.fold(Self::ZERO, |acc, item| acc + item.borrow())
// //     }
// // }

// // impl<T: ::core::borrow::Borrow<Fp>> ::core::iter::Product<T> for MyField {
// //     fn product<I: Iterator<Item = T>>(iter: I) -> Self {
// //         iter.fold(Self::ONE, |acc, item| acc * item.borrow())
// //     }
// // }

// // MyField
// // fn main() {
// // 	print_preamble("Zcash.DecomposeRunningSum");

// // 	// Instantiate the circuit with the private inputs.
// // 	let circuit = MyCircuit {
// // 		alpha: Value::unknown(),
// // 		strict: true
// // 	};

// //     let mut cs = ConstraintSystem::<MyField>::default();
// //     let config = MyCircuit::<MyField, 255, 3, 83>::configure(&mut cs);

// //     println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

// //     let mut extr_assn = ExtractingAssignment::<MyField>::new(Target::AdviceGenerator);
// //     <MyCircuit<MyField, 255, 3, 83> as Circuit<MyField>>::FloorPlanner::synthesize(
// //         &mut extr_assn,
// //         &circuit,
// //         config,
// //         vec![],
// //     )
// //     .unwrap();

// //     extr_assn.print_grouping_props();
// //     print_gates(CircuitGates::collect::<MyField, MyCircuit<MyField, 255, 3, 83>>());
// //     print_postamble("Zcash.DecomposeRunningSum");
// // }

// //TermField
// fn main() {
// 	print_preamble("Zcash.DecomposeRunningSum");

// 	// Instantiate the circuit with the private inputs.
// 	let circuit: MyCircuit<TermField, 255, 3, 83> = MyCircuit {
// 		alpha: Value::unknown(),
// 		strict: true
// 	};

//     let mut cs = ConstraintSystem::<TermField>::default();
//     let config = MyCircuit::<TermField, 255, 3, 83>::configure(&mut cs);

//     println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

//     let mut extr_assn = ExtractingAssignment::<TermField>::new(Target::AdviceGenerator);
//     <MyCircuit<TermField, 255, 3, 83> as Circuit<TermField>>::FloorPlanner::synthesize(
//         &mut extr_assn,
//         &circuit,
//         config,
//         vec![cs.fixed_column()],
//     )
//     .unwrap();

//     extr_assn.print_grouping_props();
//     print_gates(CircuitGates::collect::<TermField, MyCircuit<TermField, 255, 3, 83>>());
//     print_postamble("Zcash.DecomposeRunningSum");
// }

// TEMP
fn main () {}