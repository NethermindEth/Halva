use halo2_extr::{
    delegating_prover::DelegatingProver, lean_delegating_prover::LeanDelegatingProver, scroll::zkevm_circuits::keccak_circuit::KeccakCircuit
};

fn main() {
    let num_rows = 2_usize.pow(10);
    println!("-- {:?}", KeccakCircuit::capacity_for_row(num_rows));
    let circuit = KeccakCircuit::new(num_rows, vec![vec![0,0]]);
    let prover = LeanDelegatingProver::new("Keccak".to_string(), vec![]);
    prover.run(&circuit).unwrap();
}
