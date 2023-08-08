use std::convert::TryFrom;
use std::fmt::Display;
use std::marker::PhantomData;

use halo2_proofs::dev::CircuitGates;
use halo2_proofs::plonk::{Circuit, ColumnType};
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
        .map(|gate| {
            println!(
                "{}",
                gate.replace("S", "Selector ")
                    .replace("A", "Advice ")
                    .replace("I", "Instance ")
                    .replace("F", "Fixed ")
                    .replace("@", " ")
            );
        })
        .for_each(drop);
}

#[macro_export]
macro_rules! extract_constraints {
    ($a:ident, $b:expr, $c:ident) => {
        let mut cs = ConstraintSystem::<TermField>::default();
        let config = $a::<TermField>::configure(&mut cs);

        let mut extr_assn = ExtractingAssignment::<TermField>::new($b);
        <$a<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
            &mut extr_assn,
            &$c,
            config,
            vec![],
        )
        .unwrap();

        use efp::print_gates;
        print_gates(CircuitGates::collect::<Fp, $a<Fp>>());
    };
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use halo2_proofs::{
        arithmetic::Field,
        circuit::{AssignedCell, Layouter, SimpleFloorPlanner},
        dev::CircuitGates,
        pasta::Fp,
        plonk::{Advice, Circuit, Column, ConstraintSystem, FloorPlanner, Instance, Selector},
        poly::Rotation,
    };

    use crate::field::TermField;

    use super::{ExtractingAssignment, Target};

    #[derive(Debug, Clone)]
    struct FibonacciConfig {
        pub col_a: Column<Advice>,
        pub col_b: Column<Advice>,
        pub col_c: Column<Advice>,
        pub selector: Selector,
        pub instance: Column<Instance>,
    }

    #[derive(Debug, Clone)]
    struct FibonacciChip<F: Field> {
        config: FibonacciConfig,
        _marker: PhantomData<F>,
    }

    impl<F: Field> FibonacciChip<F> {
        pub fn construct(config: FibonacciConfig) -> Self {
            Self {
                config,
                _marker: PhantomData,
            }
        }

        pub fn configure(meta: &mut ConstraintSystem<F>) -> FibonacciConfig {
            let col_a = meta.advice_column();
            let col_b = meta.advice_column();
            let col_c = meta.advice_column();
            let selector = meta.selector();
            let instance = meta.instance_column();

            meta.enable_equality(col_a);
            meta.enable_equality(col_b);
            meta.enable_equality(col_c);
            meta.enable_equality(instance);

            meta.create_gate("add", |meta| {
                //
                // col_a | col_b | col_c | selector |
                //   a      b        c       s
                //
                let s = meta.query_selector(selector);
                let a = meta.query_advice(col_a, Rotation::cur());
                let b = meta.query_advice(col_b, Rotation::cur());
                let c = meta.query_advice(col_c, Rotation::cur());
                vec![s * (a + b - c)]
            });

            FibonacciConfig {
                col_a,
                col_b,
                col_c,
                selector,
                instance,
            }
        }

        #[allow(clippy::type_complexity)]
        pub fn assign_first_row(
            &self,
            mut layouter: impl Layouter<F>,
        ) -> Result<
            (AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>),
            halo2_proofs::plonk::Error,
        > {
            layouter.assign_region(
                || "first row",
                |mut region| {
                    self.config.selector.enable(&mut region, 0)?;

                    let a_cell = region.assign_advice_from_instance(
                        || "f(0)",
                        self.config.instance,
                        0,
                        self.config.col_a,
                        0,
                    )?;

                    let b_cell = region.assign_advice_from_instance(
                        || "f(1)",
                        self.config.instance,
                        1,
                        self.config.col_b,
                        0,
                    )?;

                    let c_cell = region.assign_advice(
                        || "a + b",
                        self.config.col_c,
                        0,
                        || a_cell.value().copied() + b_cell.value(),
                    )?;

                    Ok((a_cell, b_cell, c_cell))
                },
            )
        }

        pub fn assign_row(
            &self,
            mut layouter: impl Layouter<F>,
            prev_b: &AssignedCell<F, F>,
            prev_c: &AssignedCell<F, F>,
        ) -> Result<AssignedCell<F, F>, halo2_proofs::plonk::Error> {
            layouter.assign_region(
                || "next row",
                |mut region| {
                    self.config.selector.enable(&mut region, 0)?;

                    // Copy the value from b & c in previous row to a & b in current row
                    prev_b.copy_advice(|| "a", &mut region, self.config.col_a, 0)?;
                    prev_c.copy_advice(|| "b", &mut region, self.config.col_b, 0)?;

                    let c_cell = region.assign_advice(
                        || "c",
                        self.config.col_c,
                        0,
                        || prev_b.value().copied() + prev_c.value(),
                    )?;

                    Ok(c_cell)
                },
            )
        }

        pub fn expose_public(
            &self,
            mut layouter: impl Layouter<F>,
            cell: &AssignedCell<F, F>,
            row: usize,
        ) -> Result<(), halo2_proofs::plonk::Error> {
            layouter.constrain_instance(cell.cell(), self.config.instance, row)
        }
    }

    #[derive(Default)]
    struct MyCircuit<F>(PhantomData<F>);

    impl<F: Field> Circuit<F> for MyCircuit<F> {
        type Config = FibonacciConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            FibonacciChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), halo2_proofs::plonk::Error> {
            let chip = FibonacciChip::construct(config);

            let (_, mut prev_b, mut prev_c) =
                chip.assign_first_row(layouter.namespace(|| "first row"))?;

            for _i in 3..10 {
                let c_cell =
                    chip.assign_row(layouter.namespace(|| "next row"), &prev_b, &prev_c)?;
                prev_b = prev_c;
                prev_c = c_cell;
            }

            chip.expose_public(layouter.namespace(|| "out"), &prev_c, 2)?;

            Ok(())
        }
    }

    #[test]
    fn fibonacci_example1() {
        let circuit: MyCircuit<TermField> = MyCircuit(PhantomData);

        extract_constraints!(MyCircuit, Target::AdviceGenerator, circuit);
    }
}
