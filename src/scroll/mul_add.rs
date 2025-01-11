// //! Chip that implements instructions to check: a * b + c == d (mod 2^256) where
// //! a, b, c and d are all 256-bit words.
// //!
// //! The circuit layout is as follows:

// use halo2_proofs::plonk::{Advice, Column, Expression, TableColumn};

// use super::range::UIntRangeCheckChip;
// #[rustfmt::skip]
// // | q_step | col0      | col1      | col2      | col3      |
// // |--------|-----------|-----------|-----------|-----------|
// // | 1      | a_limb0   | a_limb1   | a_limb2   | a_limb3   |
// // | 0      | b_limb0   | b_limb1   | b_limb2   | b_limb3   |
// // | 0      | c_lo      | c_hi      | d_lo      | d_hi      |
// // | 0      | carry_lo0 | carry_lo1 | carry_lo2 | carry_lo3 |
// // | 0      | carry_lo4 | -         | -         | -         |
// // | 0      | carry_hi0 | carry_hi1 | carry_hi2 | carry_hi3 |
// // | 0      | carry_hi4 | -         | -         | -         |
// // | 0      | -         | -         | -         | -         |
// // |--------|-----------|-----------|-----------|-----------|
// // last row is padding to fit in 8 rows range_check_64 chip

// /// Config for the MulAddChip.
// #[derive(Clone, Debug)]
// pub struct MulAddConfig<F> {
//     /// First of the columns which we use over multiple rows to represent the
//     /// schema described above.
//     pub col0: Column<Advice>,
//     /// Second of the columns which we use over multiple rows to represent the
//     /// schema described above.
//     pub col1: Column<Advice>,
//     /// Third of the columns which we use over multiple rows to represent the
//     /// schema described above.
//     pub col2: Column<Advice>,
//     /// Fourth of the columns which we use over multiple rows to represent the
//     /// schema described above.
//     pub col3: Column<Advice>,
//     /// Sum of the parts higher than 256-bit in the product.
//     pub overflow: Expression<F>,
//     /// Lookup table for LtChips and carry_lo/hi.
//     pub u16_table: TableColumn,
//     /// Range check of a, b which needs to be in [0, 2^64)
//     pub range_check_64: UIntRangeCheckChip<F, { UIntRangeCheckChip::SIZE_U64 }, 8>,
//     /// Range check of c, d which needs to be in [0, 2^128)
//     pub range_check_128: UIntRangeCheckChip<F, { UIntRangeCheckChip::SIZE_U128 }, 4>,
// }