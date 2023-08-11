extern crate ff;
extern crate halo2_extr;
extern crate halo2_proofs;
extern crate zkevm_circuits;
//use bus_mapping::{evm::OpcodeId, state_db::CodeDB};
use eth_types::Field;
use halo2_extr::{extract, field::TermField};
use halo2_proofs::{arithmetic::Field as Halo2Field, dev::MockProver, halo2curves::bn256::Fr};
use log::error;
use zkevm_circuits::{
    bytecode_circuit::circuit::BytecodeCircuit,
    util::{log2_ceil, unusable_rows, SubCircuit},
};

use super::circuit::BytecodeCircuitRow;

fn main() {
    let k = 9;
    let bytecodes = vec![vec![1, 2, 3, 4]];

    let bytecode_len = bytecodes[0].len();
    let circuit = BytecodeCircuit::<TermField>::new(bytecodes.into(), 2usize.pow(k));
    // let circuit = BytecodeCircuit::<TermField>::from_bytes(bytecodes, k).mut_rows(|rows| {
    //     let code_hash = rows[0].code_hash;
    //     let mut index = bytecode_len as u64;
    //     let size = 100;
    //     let minimum_rows = 8;
    //     let len = rows.len();
    //     for i in len..size - minimum_rows {
    //         rows.push(BytecodeCircuitRow::new(
    //             code_hash,
    //             Fr::ONE,
    //             Fr::from(index),
    //             Fr::ONE,
    //             Fr::from((i % 10 + 1) as u64),
    //         ));
    //         index += 1;
    //     }
    // });
    // extract!(BytecodeCircuit,)
    //     .verify(false);
}
