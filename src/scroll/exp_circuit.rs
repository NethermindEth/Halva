// use halo2_proofs::plonk::{Column, Fixed};

// use super::{mul_add::MulAddConfig, table::{ExpTable, U16Table}};

// /// Layout for the Exponentiation circuit.
// #[derive(Clone, Debug)]
// pub struct ExpCircuitConfig<F> {
//     /// Whether the row is enabled.
//     pub q_enable: Column<Fixed>,
//     /// Mark the last step of the last event within usable rows.
//     pub is_final_step: Column<Fixed>,
//     /// The Exponentiation circuit's table.
//     pub exp_table: ExpTable,
//     /// u16 lookup table,
//     pub u16_table: U16Table,
//     /// Multiplication gadget for verification of each step.
//     pub mul_gadget: MulAddConfig<F>,
//     /// Multiplication gadget to perform 2*n + k.
//     pub parity_check: MulAddConfig<F>,
// }
