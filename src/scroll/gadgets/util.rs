use halo2_proofs::plonk::Expression;

use crate::field::TermField;

/// Returns the sum of the passed in cells
pub mod sum {
    use crate::{field::TermField, scroll::gadgets::util::Expr};
    use halo2_proofs::plonk::Expression;

    /// Returns an expression for the sum of the list of expressions.
    pub fn expr<E: Expr, I: IntoIterator<Item = E>>(inputs: I) -> Expression<TermField> {
        inputs
            .into_iter()
            .fold(0.expr(), |acc, input| acc + input.expr())
    }

    // /// Returns the sum of the given list of values within the field.
    // pub fn value<F: Field>(values: &[u8]) -> F {
    //     values
    //         .iter()
    //         .fold(F::ZERO, |acc, value| acc + F::from(*value as u64))
    // }
}

/// Returns `1` when `expr[0] && expr[1] && ... == 1`, and returns `0`
/// otherwise. Inputs need to be boolean
pub mod and {
    use crate::{field::TermField, scroll::gadgets::util::Expr};
    use halo2_proofs::plonk::Expression;

    /// Returns an expression that evaluates to 1 only if all the expressions in
    /// the given list are 1, else returns 0.
    pub fn expr<E: Expr, I: IntoIterator<Item = E>>(inputs: I) -> Expression<TermField> {
        inputs
            .into_iter()
            .fold(1.expr(), |acc, input| acc * input.expr())
    }

    /// Returns the product of all given values.
    pub fn value(inputs: Vec<TermField>) -> TermField {
        inputs.iter().fold(TermField::one(), |acc, input| acc * input)
    }
}

/// Returns `1` when `b == 0`, and returns `0` otherwise.
/// `b` needs to be boolean
pub mod not {
    use crate::{field::TermField, scroll::gadgets::util::Expr};
    use halo2_proofs::plonk::Expression;

    /// Returns an expression that represents the NOT of the given expression.
    pub fn expr<E: Expr>(b: E) -> Expression<TermField> {
        1.expr() - b.expr()
    }

    /// Returns a value that represents the NOT of the given value.
    pub fn value(b: TermField) -> TermField {
        TermField::one() - b
    }
}

/// Returns `when_true` when `selector == 1`, and returns `when_false` when
/// `selector == 0`. `selector` needs to be boolean.
pub mod select {
    use crate::{field::TermField, scroll::gadgets::util::Expr};
    use halo2_proofs::plonk::Expression;

    /// Returns the `when_true` expression when the selector is true, else
    /// returns the `when_false` expression.
    pub fn expr(
        selector: Expression<TermField>,
        when_true: Expression<TermField>,
        when_false: Expression<TermField>,
    ) -> Expression<TermField> {
        selector.clone() * when_true + (1.expr() - selector) * when_false
    }

    /// Returns the `when_true` value when the selector is true, else returns
    /// the `when_false` value.
    pub fn value(selector: TermField, when_true: TermField, when_false: TermField) -> TermField {
        selector * when_true + (TermField::one() - selector) * when_false
    }

    // /// Returns the `when_true` word when selector is true, else returns the
    // /// `when_false` word.
    // pub fn value_word(
    //     selector: F,
    //     when_true: [u8; 32],
    //     when_false: [u8; 32],
    // ) -> [u8; 32] {
    //     if selector == F::ONE {
    //         when_true
    //     } else {
    //         when_false
    //     }
    // }
}

/// Trait that implements functionality to get a constant expression from
/// commonly used types.
pub trait Expr {
    /// Returns an expression for the type.
    fn expr(&self) -> Expression<TermField>;
}

/// Implementation trait `Expr` for type able to be casted to u64
#[macro_export]
macro_rules! impl_expr {
    ($type:ty) => {
        impl $crate::scroll::gadgets::util::Expr for $type {
            #[inline]
            fn expr(&self) -> Expression<$crate::field::TermField> {
                Expression::Constant($crate::field::TermField::from(*self as u64))
            }
        }
    };
    ($type:ty, $method:path) => {
        impl $crate::scroll::gadgets::util::Expr for $type {
            #[inline]
            fn expr(&self) -> Expression<$crate::field::TermField> {
                Expression::Constant($crate::field::TermField::from($method(self) as u64))
            }
        }
    };
}

impl_expr!(bool);
impl_expr!(u8);
impl_expr!(u64);
impl_expr!(usize);
// impl_expr!(OpcodeId, OpcodeId::as_u8);
// impl_expr!(GasCost, GasCost::as_u64);

impl Expr for i32 {
    #[inline]
    fn expr(&self) -> Expression<TermField> {
        Expression::Constant(
            TermField::from(self.unsigned_abs() as u64) * if self.is_negative() { -TermField::one() } else { TermField::one() },
        )
    }
}

impl Expr for Expression<TermField> {
    fn expr(&self) -> Expression<TermField> {
        self.clone()
    }
}
