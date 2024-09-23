// use halo2_extr::{extract, extraction::Target};
// use halo2_proofs::arithmetic::Field as Halo2Field;
// use zkevm_circuits::bytecode_circuit::circuit::BytecodeCircuit;

// fn main() {
//     let k = 9;
//     let bytecodes = vec![vec![1, 2, 3, 4]];

//     let _bytecode_len = bytecodes[0].len();
//     let circuit = BytecodeCircuit::<TermField>::new(bytecodes.into(), 2usize.pow(k));
//     extract!(BytecodeCircuit, Target::AdviceGenerator, circuit);
// }
