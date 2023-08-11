use std::convert::TryFrom;
use std::fmt::Display;
use std::marker::PhantomData;

use halo2_proofs::dev::CircuitGates;
use halo2_proofs::plonk::ColumnType;
use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    plonk::{Advice, Any, Assigned, Assignment, Column, Fixed, Instance, Selector},
};

use crate::utils::{Halo2Column, Halo2Selector};

pub enum Target {
    Constraints,
    AdviceGenerator,
}

pub struct ExtractingAssignment<F: Field> {
    _marker: PhantomData<F>,
    current_region: Option<String>,
    target: Target,
}

impl<F: Field> ExtractingAssignment<F> {
    pub fn new(target: Target) -> Self {
        Self {
            _marker: PhantomData,
            current_region: None,
            target: target,
        }
    }

    fn format_cell<T>(col: Column<T>) -> String
    where
        T: ColumnType,
    {
        let parsed_column = Halo2Column::try_from(format!("{:?}", col).as_str()).unwrap();
        format!("{:?} {}", parsed_column.column_type, parsed_column.index)
    }

    fn print_annotation(annotation: String) {
        if !annotation.is_empty() {
            println!("--{}", annotation);
        }
    }
}

impl<F: Field + From<String>> Assignment<F> for ExtractingAssignment<F>
where
    F: Display,
{
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        let x: String = name_fn().into();
        println!("--REGION: {x}");
        self.current_region = Some(x.clone());
    }

    fn exit_region(&mut self) {
        println!(
            "--EXITTED REGION: {}",
            self.current_region.as_ref().unwrap()
        );
        self.current_region = None;
    }

    fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), halo2_proofs::plonk::Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        Self::print_annotation(annotation().into());
        let halo2_selector = Halo2Selector::try_from(format!("{:?}", selector).as_str()).unwrap();
        println!("EnableSelector {} {}", halo2_selector.0, row);
        Ok(())
    }

    fn query_instance(
        &self,
        column: Column<Instance>,
        row: usize,
    ) -> Result<Value<F>, halo2_proofs::plonk::Error> {
        Ok(Value::known(F::from(format!(
            "{} {}",
            Self::format_cell(column),
            row
        ))))
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), halo2_proofs::plonk::Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        match self.target {
            Target::Constraints => Ok(()),
            Target::AdviceGenerator => {
                Self::print_annotation(annotation().into());
                to().map(|v| {
                    println!(
                        "{} {} = {}",
                        Self::format_cell(column),
                        row,
                        v.into().evaluate()
                    );
                });
                Ok(())
            }
        }
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), halo2_proofs::plonk::Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        Self::print_annotation(annotation().into());
        to().map(|v| {
            println!(
                "{} {} = {}",
                Self::format_cell(column),
                row,
                v.into().evaluate()
            );
        });
        Ok(())
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        println!(
            "{} {} = {} {}",
            Self::format_cell(left_column),
            left_row,
            Self::format_cell(right_column),
            right_row
        );
        Ok(())
    }

    fn fill_from_row(
        &mut self,
        _column: Column<Fixed>,
        _row: usize,
        _to: Value<Assigned<F>>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        // todo: Not sure what should be done here
        Ok(())
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn pop_namespace(&mut self, _gadget_name: Option<String>) {}
}

pub fn print_gates(gates: CircuitGates) {
    println!("------GATES-------");
    gates
        .to_string()
        .lines()
        .filter(|x| !x.contains(':'))
        .for_each(|gate| {
            println!(
                "{}",
                gate.replace("S", "Selector ")
                    .replace("A", "Advice ")
                    .replace("I", "Instance ")
                    .replace("F", "Fixed ")
                    .replace("@", " ")
            )
        })
}

#[macro_export]
macro_rules! extract {
    ($circuit_type:ident, $target:expr) => {
        use halo2_extr::extraction::{print_gates, ExtractingAssignment};
        use halo2_extr::field::TermField;
        use halo2_proofs::dev::CircuitGates;
        use halo2_proofs::pasta::Fp;
        use halo2_proofs::plonk::FloorPlanner;
        let circuit: MyCircuit<TermField> = $circuit_type::default();

        let mut cs = ConstraintSystem::<TermField>::default();
        let config = $circuit_type::<TermField>::configure(&mut cs);

        let mut extr_assn = ExtractingAssignment::<TermField>::new($target);
        <$circuit_type<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
            &mut extr_assn,
            &circuit,
            config,
            vec![],
        )
        .unwrap();

        print_gates(CircuitGates::collect::<Fp, $circuit_type<Fp>>());
    };
    ($circuit_type:ident, $target:expr, $circuit_initializer:expr) => {
        use halo2_extr::extraction::{print_gates, ExtractingAssignment};
        use halo2_extr::field::TermField;
        use halo2_proofs::dev::CircuitGates;
        use halo2_proofs::pasta::Fp;
        use halo2_proofs::plonk::FloorPlanner;
        let circuit: MyCircuit<TermField> = $circuit_initializer;

        let mut cs = ConstraintSystem::<TermField>::default();
        let config = $circuit_type::<TermField>::configure(&mut cs);

        let mut extr_assn = ExtractingAssignment::<TermField>::new($target);
        <$circuit_type<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
            &mut extr_assn,
            &circuit,
            config,
            vec![],
        )
        .unwrap();

        print_gates(CircuitGates::collect::<Fp, $circuit_type<Fp>>());
    };
}

#[cfg(test)]
mod tests {}
