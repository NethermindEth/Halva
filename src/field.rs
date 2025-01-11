use std::{
    fmt::{Debug, Display}, iter::{Product, Sum}, ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign}
};

use arrayvec::ArrayString;
use ff::{Field, FromUniformBytes, PrimeField};
use num_bigint::BigUint;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

const EXPRESSION_MAX_SIZE: usize = 16384;

// Field requires Copy, Sized, and 'static
// This means we have to use a stack allocated, dynamically generatable, fixed length string
#[derive(Clone, Copy)]
pub enum TermField {
    Val(i64),
    Expr(*const String),
    TwoInv,
    MultiplicativeGenerator,
    S,
    RootOfUnity,
    RootOfUnityInv,
    Delta
}

impl PartialEq for TermField {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Val(x), Self::Val(y)) => if x == y {
                true
            } else { // Two identical values are equal, but distinct values could still be equal because the field modulus is symbolic
                #[cfg(not(feature = "unsafe-equality"))]
                panic!("Unable to determine whether {} and {} are equal without the unsafe-equality feature (which will consider them not equal)", x, y);

                #[cfg(feature = "unsafe-equality")]
                false
            }
            (Self::Expr(l0), Self::Expr(r0)) => {
                unsafe {
                    if l0 == r0 {
                        true
                    } else {
                        #[cfg(not(feature = "unsafe-equality"))]
                        panic!("Unable to determine whether {} and {} are equal without the unsafe-equality feature (which will consider them not equal)", **l0, **r0);
        
                        #[cfg(feature = "unsafe-equality")]
                        false
                    }
                }
            },
            _ => if core::mem::discriminant(self) == core::mem::discriminant(other) {
                true
            } else {
                #[cfg(not(feature = "unsafe-equality"))]
                panic!("Unable to determine whether {} and {} are equal without the unsafe-equality feature (which will consider them not equal)", self.to_expr(), other.to_expr());
        
                #[cfg(feature = "unsafe-equality")]
                false
            }
        }
    }
}

impl Eq for TermField {}

impl Debug for TermField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TermField::Val(_) => write!(f, "Val: {}", self.to_expr()),
            TermField::Expr(_) => write!(f, "Expr: {}", self.to_expr()),
            TermField::TwoInv => write!(f, "TwoInv: {}", self.to_expr()),
            TermField::MultiplicativeGenerator => write!(f, "MGen: {}", self.to_expr()),
            TermField::S => write!(f, "S: {}", self.to_expr()),
            TermField::RootOfUnity => write!(f, "RootOfUnity: {}", self.to_expr()),
            TermField::RootOfUnityInv => write!(f, "RootOfUnityInv: {}", self.to_expr()),
            TermField::Delta => write!(f, "Delta: {}", self.to_expr()),
        }
    }
}

impl Display for TermField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_expr())
    }
}

impl TermField {
    pub const fn zero() -> Self {
        TermField::Val(0)
    }

    pub const fn one() -> Self {
        TermField::Val(1)
    }

    pub const fn two_inv() -> Self {
        TermField::TwoInv
    }

    // Note that this requires the symbol also be passed to print_preamble
    pub fn create_symbol(name: &str) -> Self {
        TermField::from(format!("c.1.sym_{name}"))
    }

    fn to_expr(&self) -> String {
        match self {
            TermField::Val(x) => x.to_string(),
            TermField::Expr(x) => {
                unsafe {
                    (**x).clone()
                }
            }
            TermField::TwoInv => String::from("(2: ZMod P).inv"),
            TermField::MultiplicativeGenerator => String::from("c.mult_gen"),
            TermField::S => String::from("c.S"),
            TermField::RootOfUnity => String::from("c.root_of_unity"),
            TermField::RootOfUnityInv => String::from("c.root_of_unity.inv"),
            TermField::Delta => String::from("c.delta"),
        }
    }

    pub fn create_s() -> Self {
        Self::from("S")
    }
}

impl From<&str> for TermField {
    fn from(s: &str) -> Self {
        let num = str::parse::<i64>(s);
        if let Ok(val) = num {
            Self::Val(val)
        } else {
            let heap_str = Box::new(String::from(s));
            let ptr = Box::into_raw(heap_str).cast_const();
            Self::Expr(ptr)
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
            // TODO checks
            Self::Val(x) => Self::Val(-x),
            _ => Self::from(&format!("-({})", self.to_expr())),
        }
    }
}

impl Add for TermField {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Val(x), Self::Val(y)) => {
                let big_x = x as i128;
                let big_y = y as i128;
                let big_res = big_x + big_y;
                if let Ok(res) = TryInto::<i64>::try_into(big_res) {
                    Self::Val(res)
                } else {
                    Self::from(format!("{}", big_res))
                }
            },
            (Self::Val(0), _) => rhs,
            (_, Self::Val(0)) => self,
            _ => Self::from(&format!("({}) + ({})", self.to_expr(), rhs.to_expr())),
        }
    }
}

impl Sub for TermField {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Val(x), Self::Val(y)) => Self::Val(x-y),
            (Self::Val(0), _) => rhs.neg(),
            (_, Self::Val(0)) => self,
            _ => Self::from(&format!("({}) - ({})", self.to_expr(), rhs.to_expr())),
        }
    }
}

impl Mul for TermField {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Val(x), Self::Val(y)) => {
                let big_x = x as i128;
                let big_y = y as i128;
                let big_res = big_x * big_y;
                if let Ok(res) = TryInto::<i64>::try_into(big_res) {
                    Self::Val(res)
                } else {
                    Self::from(format!("{}", big_res))
                }
            },
            (Self::Val(1), _) => rhs,
            (_, Self::Val(1)) => self,
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
        #[cfg(not(feature = "unsafe-ord"))]
        panic!("partial_cmp requires the unsafe-ord feature enabled because it cannot be calculated correctly for symbolic values. Enabling it will return a placeholder value (Less)");

        #[cfg(feature = "unsafe-ord")]
        Some(std::cmp::Ordering::Less)
    }
}

impl Ord for TermField {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        #[cfg(not(feature = "unsafe-ord"))]
        panic!("cmp requires the unsafe-ord feature enabled because it cannot be calculated correctly for symbolic values. Enabling it will return a placeholder value (Less)");

        #[cfg(feature = "unsafe-ord")]
        Some(std::cmp::Ordering::Less)
    }
}

impl Field for TermField {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();

    fn random(_rng: impl rand_core::RngCore) -> Self {
        panic!("Random with a random number generator is not supported. Use create_symbol for an unknown value")
    }

    fn square(&self) -> Self {
        *self * self
    }

    fn double(&self) -> Self {
        Self::from("2") * self
    }

    fn invert(&self) -> CtOption<Self> {
        #[cfg(not(feature = "unsafe-invert"))]
        panic!("Field::invert requires the unsafe-invert flag. This is because it is not always possible to determine whether a TermField is equal to zero");

        #[cfg(feature = "unsafe-invert")]
        CtOption::new(
            Self::from(format!("(({}: ZMod P).inv)", self)),
            Choice::from(1),
        )
    }

    fn sqrt_ratio(_num: &Self, _div: &Self) -> (Choice, Self) {
        panic!("Sqrt_ratio is not supported, because it is not possible to determine whether a TermField equals zero. Hence the Choice return cannot be determined")
    }
}

impl From<bool> for TermField {
    fn from(value: bool) -> Self {
        if value {
            Self::one()
        } else {
            Self::zero()
        }
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

#[derive(Clone, Copy)]
pub struct TermFieldBytes (ArrayString<EXPRESSION_MAX_SIZE>);

impl Default for TermFieldBytes {
    fn default() -> Self {
        Self(ArrayString::<EXPRESSION_MAX_SIZE>::default())
    }
}

impl AsMut<[u8]> for TermFieldBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        todo!()
    }
}

impl AsRef<[u8]> for TermFieldBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_str().as_bytes()
    }
}

impl PrimeField for TermField {
    type Repr = TermFieldBytes;

    #[cfg(all(feature = "repr-text", feature = "repr-number"))]
    fn from_repr(_repr: Self::Repr) -> CtOption<Self> {
        compile_error!("features `halo2-extractor/repr-text` and `halo2-extractor/repr-number` are mutually exclusive");
    }

    #[cfg(not(any(feature = "repr-text", feature = "repr-number")))]
    fn from_repr(_repr: Self::Repr) -> CtOption<Self> {        
        panic!("from_repr requires either the repr-text flag or the repr-number feature to be enabled");
    }

    #[cfg(all(feature = "repr-number", not(feature = "repr-text")))]
    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        let x = BigUint::from_bytes_le(&repr.0.as_bytes());
        CtOption::new(Self::from(x.to_str_radix(10)), Choice::from(1))
    }

    #[cfg(all(feature = "repr-text", not(feature = "repr-number")))]
    fn from_repr(repr: Self::Repr) -> CtOption<Self> {
        CtOption::new(Self::Expr(repr.0), Choice::from(1))
    }

    fn to_repr(&self) -> Self::Repr {
        #[cfg(not(feature = "repr-text"))]
        unimplemented!("to_repr requires the repr-text feature");
        
        #[cfg(feature = "repr-text")]
        TermFieldBytes(self.to_expr())
    }

    fn is_odd(&self) -> Choice {
        unimplemented!("Cannot deterministically decide whether a TermField is odd")
    }

    const MODULUS: &'static str = "P";

    const NUM_BITS: u32 = unimplemented!();

    const CAPACITY: u32 = unimplemented!();

    const TWO_INV: Self = Self::TwoInv;

    const MULTIPLICATIVE_GENERATOR: Self = Self::MultiplicativeGenerator;

    // The value of S cannot be known at the Rust level
    // However the create_s method does exist for referring to it at the Lean level
    const S: u32 = unimplemented!();

    const ROOT_OF_UNITY: Self = Self::RootOfUnity;

    const ROOT_OF_UNITY_INV: Self = Self::RootOfUnityInv;

    const DELTA: Self = Self::Delta;
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
