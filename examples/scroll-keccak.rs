use halo2_extr::{
    extraction::ExtractingAssignment, scroll::zkevm_circuits::keccak_circuit::KeccakCircuit,
};

fn main() {
    let num_rows = 2_usize.pow(10);
    println!("-- {:?}", KeccakCircuit::capacity_for_row(num_rows));
    let circuit = KeccakCircuit::new(num_rows, vec![vec![0,0]]);
    ExtractingAssignment::run(&circuit, "Keccak", &[]).unwrap();
}
