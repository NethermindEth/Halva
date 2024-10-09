use std::{
    fmt::Debug, fmt::Display,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arrayvec::ArrayString;
// use eth_types::Field;
use ff::{Field as Halo2Field, FromUniformBytes, PrimeField};
use num_bigint::BigUint;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

const EXPRESSION_MAX_SIZE: usize = 16384;

// Field requires Copy, however String is not copyable
// It would be possible to use a more restricted enum representation of all possible expressions
// instead in the future if there are efficiency issues
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TermField {
    Zero,
    One,
    Expr(ArrayString<EXPRESSION_MAX_SIZE>),
}

// TODO fix hack
//   implementation of decompose-running-sum expects a number starting with 0x
//   the default implementation of Debug for TermField therefore causes a crash when it cannot remove the 0x prefix
//   to get around this, a meaningless 0x prefix is added. The output is NOT in hexadecimal
impl Debug for TermField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_expr())
    }
}

impl TermField {
    pub const fn zero() -> Self {
        TermField::Zero
    }

    pub const fn one() -> Self {
        TermField::One
    }

    // Note this must be called at the start of the extraction, since it adds to the Circuit structure
    // TODO: Switch to a paradigm in which data is collected rather than printed during extraction, then printed at the end
    pub fn create_symbol(name: &str) -> Self {
        println!("  {name}: ZMod P");
        TermField::from(format!("c.{name}"))
    }

    fn to_expr(&self) -> ArrayString<EXPRESSION_MAX_SIZE> {
        ArrayString::from(match self {
            TermField::Zero => "0",
            TermField::One => "1",
            TermField::Expr(x) => x,
        })
        .unwrap()
    }
}

impl From<&str> for TermField {
    fn from(s: &str) -> Self {
        match s.trim() {
            "0" => Self::Zero,
            "1" => Self::One,
            s => {
                let array_str = ArrayString::from(s);
                if let Ok(str) = array_str {
                    Self::Expr(str)
                } else {
                    panic!("{}", s)
                }
            }
        }
    }
}

impl From<String> for TermField {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for TermField {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for TermField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_expr())
    }
}

impl Default for TermField {
    fn default() -> Self {
        Self::zero()
    }
}

unsafe impl Sync for TermField {}
unsafe impl Send for TermField {}

impl ConditionallySelectable for TermField {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        if choice.unwrap_u8() == 0 {
            *a
        } else {
            *b
        }
    }
}

impl ConstantTimeEq for TermField {
    fn ct_eq(&self, other: &Self) -> Choice {
        if self.eq(other) {
            Choice::from(1u8)
        } else {
            Choice::from(0u8)
        }
    }
}

impl Neg for TermField {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Zero => Self::Zero,
            _ => Self::from(&format!("-({})", self.to_expr())),
        }
    }
}

impl Add for TermField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Zero, _) => rhs,
            (_, Self::Zero) => self,
            _ => Self::from(&format!("({}) + ({})", self.to_expr(), rhs.to_expr())),
        }
    }
}

impl Sub for TermField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Zero, _) => rhs.neg(),
            (_, Self::Zero) => self,
            _ => Self::from(&format!("({}) - ({})", self.to_expr(), rhs.to_expr())),
        }
    }
}

impl Mul for TermField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::One, _) => rhs,
            (_, Self::One) => self,
            _ => Self::from(&format!("({}) * ({})", self.to_expr(), rhs.to_expr())),
        }
    }
}

impl Sum for TermField {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Self::add)
    }
}

impl Product for TermField {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Self::mul)
    }
}

impl<'a> Add<&'a TermField> for TermField {
    type Output = TermField;

    fn add(self, rhs: &'a TermField) -> Self::Output {
        self.add(*rhs)
    }
}

impl<'a> Sub<&'a TermField> for TermField {
    type Output = TermField;

    fn sub(self, rhs: &'a TermField) -> Self::Output {
        self.sub(*rhs)
    }
}

impl<'a> Mul<&'a TermField> for TermField {
    type Output = TermField;

    fn mul(self, rhs: &'a TermField) -> Self::Output {
        self.mul(*rhs)
    }
}

impl AddAssign for TermField {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for TermField {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for TermField {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<'a> AddAssign<&'a TermField> for TermField {
    fn add_assign(&mut self, rhs: &'a TermField) {
        *self = *self + *rhs;
    }
}

impl<'a> MulAssign<&'a TermField> for TermField {
    fn mul_assign(&mut self, rhs: &'a TermField) {
        *self = *self * *rhs;
    }
}

impl<'a> SubAssign<&'a TermField> for TermField {
    fn sub_assign(&mut self, rhs: &'a TermField) {
        *self = *self - *rhs;
    }
}

impl<'a> Sum<&'a TermField> for TermField {
    fn sum<I: Iterator<Item = &'a TermField>>(iter: I) -> Self {
        iter.fold(Self::zero(), Self::add)
    }
}

impl<'a> Product<&'a TermField> for TermField {
    fn product<I: Iterator<Item = &'a TermField>>(iter: I) -> Self {
        iter.fold(Self::one(), Self::mul)
    }
}

impl PartialOrd for TermField {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Less)
    }
}

impl Ord for TermField {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Less
    }
}

impl Halo2Field for TermField {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();

    fn random(_rng: impl rand_core::RngCore) -> Self {
        panic!("Not supposed to call \"random\"")
    }

    fn square(&self) -> Self {
        *self * self
    }

    fn double(&self) -> Self {
        Self::from("2") * self
    }

    fn invert(&self) -> CtOption<Self> {
        CtOption::new(
            Self::from(format!("(({}: ZMod P).inv)", self)),
            Choice::from(1),
        )
    }

    fn sqrt_ratio(_num: &Self, _div: &Self) -> (Choice, Self) {
        panic!("Not supposed to call \"sqrt_ratio\"")
    }
}

impl From<u64> for TermField {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::zero(),
            1 => Self::one(),
            value => Self::from(value.to_string()),
        }
    }
}

impl PrimeField for TermField {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let x = BigUint::from_bytes_le(&repr);
        CtOption::new(Self::from(x.to_str_radix(10)), Choice::from(1))
    }

    fn to_repr(&self) -> Self::Repr {
        Self::Repr::default()
    }

    fn is_odd(&self) -> Choice {
        Choice::from(1)
    }

    const MODULUS: &'static str = "214";

    const NUM_BITS: u32 = 100;

    const CAPACITY: u32 = 100;

    const TWO_INV: Self = Self::zero();

    const MULTIPLICATIVE_GENERATOR: Self = Self::zero();

    const S: u32 = 100;

    const ROOT_OF_UNITY: Self = Self::zero();

    const ROOT_OF_UNITY_INV: Self = Self::zero();

    const DELTA: Self = Self::zero();
}

impl FromUniformBytes<64> for TermField {
    fn from_uniform_bytes(bytes: &[u8; 64]) -> Self {
        let x = BigUint::from_bytes_le(bytes);
        Self::from(x.to_str_radix(10))
    }
}

// impl Field for TermField {}

// impl PrimeFieldBits for TermField {
//     type ReprBits = [u64; 4];

//     fn to_le_bits(&self) -> ff::FieldBits<Self::ReprBits> {
//         // println!("Running to_le_bits for {}", self.to_expr());
//         let f: Fp = self.to_expr().as_str().parse::<u64>().unwrap().into();
//         // println!("    Got {}", f.to_le_bits());
//         f.to_le_bits()
//     }

//     fn char_le_bits() -> ff::FieldBits<Self::ReprBits> {
//         todo!()
//     }
// }
