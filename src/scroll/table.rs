// use ff::Field;
// use primitive_types::U256;
// use crate::field::TermField;
// use crate::scroll::*;
// use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Fixed, TableColumn};

// /// The number of rows assigned for each step in an exponentiation trace.
// /// It's max(MulAddChipRows, ExpCircuitRows) = max(8, 7) = 8
// pub(crate) const OFFSET_INCREMENT: usize = 8usize;
// /// The number of rows required for the exponentiation table within the circuit
// /// for each step.
// pub(crate) const ROWS_PER_STEP: usize = 4usize;
// /// The gate "verify all but the last step" at constraint "`base_limb[i]` is the
// /// same across all steps" uses rotation 10 in `exp_table.base_limb` which is
// /// enabled with `q_usable`, which in turn is enabled in all steps.  This means
// /// this circuit requires these extra rows after the last enabled `q_usable`.
// pub(crate) const UNUSABLE_EXP_ROWS: usize = 10usize;


// pub type Word = U256;

// /// Lookup table for [0, MAX) range
// #[derive(Clone, Copy, Debug)]
// pub struct RangeTable<const MAX: usize>(TableColumn);

// /// Type Alias of u8 table, [0, 1 << 8)
// pub type U8Table = RangeTable<{ 1 << 8 }>;
// /// Type Alias of u16 table, [0, 1 << 16)
// pub type U16Table = RangeTable<{ 1 << 16 }>;

// /// Intermediary multiplication step, representing `a * b == d (mod 2^256)`
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct ExpStep {
//     /// First multiplicand.
//     pub a: Word,
//     /// Second multiplicand.
//     pub b: Word,
//     /// Multiplication result.
//     pub d: Word,
// }

// impl From<(Word, Word, Word)> for ExpStep {
//     fn from(values: (Word, Word, Word)) -> Self {
//         Self {
//             a: values.0,
//             b: values.1,
//             d: values.2,
//         }
//     }
// }

// /// Event representing an exponentiation `a ^ b == d (mod 2^256)`.
// #[derive(Clone, Debug)]
// pub struct ExpEvent {
//     /// Base `a` for the exponentiation.
//     pub base: Word,
//     /// Exponent `b` for the exponentiation.
//     pub exponent: Word,
//     /// Exponentiation result.
//     pub exponentiation: Word,
//     /// Intermediate multiplication results.
//     pub steps: Vec<ExpStep>,
// }

// impl Default for ExpEvent {
//     fn default() -> Self {
//         Self {
//             base: 2.into(),
//             exponent: 2.into(),
//             exponentiation: 4.into(),
//             steps: vec![ExpStep {
//                 a: 2.into(),
//                 b: 2.into(),
//                 d: 4.into(),
//             }],
//         }
//     }
// }

// /// Returns tuple consists of low and high part of U256
// pub fn split_u256(value: &U256) -> (U256, U256) {
//     (
//         U256([value.0[0], value.0[1], 0, 0]),
//         U256([value.0[2], value.0[3], 0, 0]),
//     )
// }

// /// Split a U256 value into 4 64-bit limbs stored in U256 values.
// pub fn split_u256_limb64(value: &U256) -> [U256; 4] {
//     [
//         U256([value.0[0], 0, 0, 0]),
//         U256([value.0[1], 0, 0, 0]),
//         U256([value.0[2], 0, 0, 0]),
//         U256([value.0[3], 0, 0, 0]),
//     ]
// }

// /// Lookup table within the Exponentiation circuit.
// #[derive(Clone, Copy, Debug)]
// pub struct ExpTable {
//     /// Whether the row is enabled.
//     pub q_enable: Column<Fixed>,
//     /// Whether the row is the start of a step.
//     pub is_step: Column<Fixed>,
//     /// Whether this row is the last row in the exponentiation operation's
//     /// trace.
//     pub is_last: Column<Advice>,
//     /// The integer base of the exponentiation.
//     pub base_limb: Column<Advice>,
//     /// The integer exponent of the exponentiation.
//     pub exponent_lo_hi: Column<Advice>,
//     /// The intermediate result of exponentiation by squaring.
//     pub exponentiation_lo_hi: Column<Advice>,
// }

// impl ExpTable {
//     /// Construct the Exponentiation table.
//     pub fn construct<F: Field>(meta: &mut ConstraintSystem<F>) -> Self {
//         Self {
//             q_enable: meta.fixed_column(),
//             is_step: meta.fixed_column(),
//             is_last: meta.advice_column(),
//             base_limb: meta.advice_column(),
//             exponent_lo_hi: meta.advice_column(),
//             exponentiation_lo_hi: meta.advice_column(),
//         }
//     }

//     /// Given an exponentiation event and randomness, get assignments to the
//     /// exponentiation table.
//     pub fn assignments(exp_event: &ExpEvent) -> Vec<[TermField; 4]> {
//         let mut assignments = Vec::new();
//         let base_limbs = split_u256_limb64(&exp_event.base);
//         let mut exponent = exp_event.exponent;
//         for (step_idx, exp_step) in exp_event.steps.iter().rev().enumerate() {
//             let is_last = if step_idx == exp_event.steps.len() - 1 {
//                 TermField::ONE
//             } else {
//                 TermField::ZERO
//             };
//             let (exp_lo, exp_hi) = split_u256(&exp_step.d);
//             let (exponent_lo, exponent_hi) = split_u256(&exponent);

//             // row 1
//             assignments.push([
//                 is_last,
//                 base_limbs[0].as_u64().into(),
//                 TermField::from(exponent_lo.to_string()),
//                 TermField::from(exp_lo.to_string()),
//             ]);
//             // row 2
//             assignments.push([
//                 TermField::ZERO,
//                 base_limbs[1].as_u64().into(),
//                 TermField::from(exponent_hi.to_string()),
//                 TermField::from(exp_hi.to_string())
//             ]);
//             // row 3
//             assignments.push([
//                 TermField::ZERO,
//                 base_limbs[2].as_u64().into(),
//                 TermField::ZERO,
//                 TermField::ZERO,
//             ]);
//             // row 4
//             assignments.push([
//                 TermField::ZERO,
//                 base_limbs[3].as_u64().into(),
//                 TermField::ZERO,
//                 TermField::ZERO,
//             ]);
//             for _ in ROWS_PER_STEP..OFFSET_INCREMENT {
//                 assignments.push([TermField::ZERO, TermField::ZERO, TermField::ZERO, TermField::ZERO]);
//             }

//             // update intermediate exponent.
//             let (exponent_div2, remainder) = exponent.div_mod(U256::from(2));
//             if remainder.is_zero() {
//                 // exponent is even
//                 exponent = exponent_div2;
//             } else {
//                 // exponent is odd
//                 exponent = exponent - 1;
//             }
//         }
//         assignments
//     }
// }

// fn main() {
    
// }