use halo2_proofs::plonk::Expression;

use crate::field::TermField;

use crate::scroll::zkevm_circuits::util::Expr;

pub trait ConstrainBuilderCommon {
    fn add_constraint(&mut self, name: &'static str, constraint: Expression<TermField>);

    fn require_zero(&mut self, name: &'static str, constraint: Expression<TermField>) {
        self.add_constraint(name, constraint);
    }

    fn require_equal(&mut self, name: &'static str, lhs: Expression<TermField>, rhs: Expression<TermField>) {
        self.add_constraint(name, lhs - rhs);
    }

    fn require_boolean(&mut self, name: &'static str, value: Expression<TermField>) {
        self.add_constraint(name, value.clone() * (1.expr() - value));
    }

    fn require_in_set(
        &mut self,
        name: &'static str,
        value: Expression<TermField>,
        set: Vec<Expression<TermField>>,
    ) {
        self.add_constraint(
            name,
            set.iter()
                .fold(1.expr(), |acc, item| acc * (value.clone() - item.clone())),
        );
    }

    fn add_constraints(&mut self, constraints: Vec<(&'static str, Expression<TermField>)>) {
        for (name, constraint) in constraints {
            self.add_constraint(name, constraint);
        }
    }
}

#[derive(Default)]
pub struct BaseConstraintBuilder {
    pub constraints: Vec<(&'static str, Expression<TermField>)>,
    pub max_degree: usize,
    pub condition: Option<Expression<TermField>>,
}

impl ConstrainBuilderCommon for BaseConstraintBuilder {
    fn add_constraint(&mut self, name: &'static str, constraint: Expression<TermField>) {
        let constraint = match &self.condition {
            Some(condition) => condition.clone() * constraint,
            None => constraint,
        };
        self.validate_degree(constraint.degree(), name);
        self.constraints.push((name, constraint));
    }
}

impl BaseConstraintBuilder {
    pub(crate) fn new(max_degree: usize) -> Self {
        BaseConstraintBuilder {
            constraints: Vec::new(),
            max_degree,
            condition: None,
        }
    }

    pub fn condition<R>(
        &mut self,
        condition: Expression<TermField>,
        constraint: impl FnOnce(&mut Self) -> R,
    ) -> R {
        assert!(
            self.condition.is_none(),
            "Nested condition is not supported"
        );
        self.condition = Some(condition);
        let ret = constraint(self);
        self.condition = None;
        ret
    }

    pub(crate) fn validate_degree(&self, degree: usize, name: &'static str) {
        if self.max_degree > 0 {
            assert!(
                degree <= self.max_degree,
                "Expression {} degree too high: {} > {}",
                name,
                degree,
                self.max_degree,
            );
        }
    }

    pub fn gate(&self, selector: Expression<TermField>) -> Vec<(&'static str, Expression<TermField>)> {
        self.constraints
            .clone()
            .into_iter()
            .map(|(name, constraint)| (name, selector.clone() * constraint))
            .filter(|(name, constraint)| {
                self.validate_degree(constraint.degree(), name);
                true
            })
            .collect()
    }
}