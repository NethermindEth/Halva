use std::{
    fmt::Display,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use arrayvec::ArrayString;
use ff::Field;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

const EXPRESSION_MAX_SIZE: usize = 16384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermField {
    Zero,
    One,
    Expr(ArrayString<EXPRESSION_MAX_SIZE>),
}

impl TermField {
    const fn zero() -> Self {
        TermField::Zero
    }

    const fn one() -> Self {
        TermField::One
    }

    fn to_expr(&self) -> ArrayString<EXPRESSION_MAX_SIZE> {
        ArrayString::from(&match self {
            TermField::Zero => "0",
            TermField::One => "1",
            TermField::Expr(x) => x,
        })
        .unwrap()
    }

    pub fn from(s: &str) -> Self {
        match s.trim() {
            "0" => Self::Zero,
            "1" => Self::One,
            s => {
                let array_str = ArrayString::from(s);
                if let Err(_) = array_str {
                    panic!("{}", s)
                } else {
                    Self::Expr(array_str.unwrap())
                }
            }
        }
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

impl Field for TermField {
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
        panic!("Not supposed to call \"invert\"")
    }

    fn sqrt_ratio(_num: &Self, _div: &Self) -> (Choice, Self) {
        panic!("Not supposed to call \"sqrt_ratio\"")
    }
}
