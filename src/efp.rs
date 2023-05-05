use std::fmt::Display;
use std::marker::PhantomData;
use std::{convert::TryFrom, fmt::Debug};

use halo2_proofs::{
    arithmetic::Field,
    circuit::{layouter::RegionLayouter, Cell, Layouter, Region, Table, Value},
    plonk::{
        Advice, Any, Assigned, Assignment, Circuit, Column, ConstraintSystem, Fixed, FloorPlanner,
        Instance, Selector,
    },
};

use crate::utils::Halo2Cell;

#[derive(Debug)]
pub struct ExtractingFloorPlanner<P: FloorPlanner> {
    _marker: PhantomData<P>,
}

impl<P: FloorPlanner> FloorPlanner for ExtractingFloorPlanner<P> {
    fn synthesize<F: Field, CS: Assignment<F>, C: Circuit<F>>(
        cs: &mut CS,
        circuit: &C,
        config: C::Config,
        constants: Vec<Column<Fixed>>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        P::synthesize(
            &mut ExtractingAssignment::new(cs),
            &ExtractingCircuit::borrowed(circuit),
            config,
            constants,
        )
    }
}

pub enum ExtractingCircuit<'c, F: Field, C: Circuit<F>> {
    Borrowed {
        circuit: &'c C,
        assignment: Vec<(Cell, F)>,
    },
    Owned {
        circuit: C,
        assignment: Vec<(Cell, F)>,
    },
}

impl<'c, F: Field, C: Circuit<F>> ExtractingCircuit<'c, F, C> {
    fn borrowed(circuit: &'c C) -> Self {
        Self::Borrowed {
            circuit,
            assignment: vec![],
        }
    }

    fn owned(circuit: C) -> Self {
        Self::Owned {
            circuit,
            assignment: vec![],
        }
    }

    fn inner_ref(&self) -> &C {
        match self {
            ExtractingCircuit::Borrowed { circuit, .. } => circuit,
            ExtractingCircuit::Owned { circuit, .. } => circuit,
        }
    }

    pub fn assignment(&self) -> &Vec<(Cell, F)> {
        match self {
            ExtractingCircuit::Borrowed {
                circuit: _circuit,
                assignment,
            } => assignment,
            ExtractingCircuit::Owned {
                circuit: _circuit,
                assignment,
            } => assignment,
        }
    }
}

impl<'c, F: Field, C: Circuit<F>> Circuit<F> for ExtractingCircuit<'c, F, C> {
    type Config = C::Config;
    type FloorPlanner = C::FloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::owned(self.inner_ref().without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        C::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        println!("Synthesize");
        let mut vec: Vec<(Cell, F)> = vec![];
        let mut equal_cells = vec![];
        let layouter = ExtractingLayouter::new(layouter, &mut vec, &mut equal_cells);
        self.inner_ref().synthesize(config, layouter).map(|()| {
            println!("Assigned cells");
            println!("-----------------------------------");

            for (cell, value) in vec {
                let cell = Halo2Cell::try_from(format!("{:?}", cell).as_str())
                    .map_err(|_| halo2_proofs::plonk::Error::Synthesis)
                    .unwrap();

                println!(
                    "Cell {} {} {} {:?} = {:?}",
                    cell.region_index,
                    cell.row_offset,
                    cell.column.index,
                    cell.column.column_type,
                    value
                );
            }

            println!("Equal cells");
            println!("-----------------------------------");

            for (lhs, rhs) in equal_cells {
                let lhs = Halo2Cell::try_from(format!("{:?}", lhs).as_str())
                    .map_err(|_| halo2_proofs::plonk::Error::Synthesis)
                    .unwrap();
                let rhs = Halo2Cell::try_from(format!("{:?}", rhs).as_str())
                    .map_err(|_| halo2_proofs::plonk::Error::Synthesis)
                    .unwrap();
                println!(
                    "Cell {} {} {} {:?} = Cell {} {} {} {:?}",
                    lhs.region_index,
                    lhs.row_offset,
                    lhs.column.index,
                    lhs.column.column_type,
                    rhs.region_index,
                    rhs.row_offset,
                    rhs.column.index,
                    rhs.column.column_type,
                );
            }
        })
    }
}

struct ExtractingLayouter<'a, F: Field, L: Layouter<F>> {
    layouter: L,
    assigned_values: &'a mut Vec<(Cell, F)>,
    equal_cells: &'a mut Vec<(Cell, Cell)>,
}

impl<'a, F: Field, L: Layouter<F>> ExtractingLayouter<'a, F, L> {
    fn new(
        layouter: L,
        assigned_values: &'a mut Vec<(Cell, F)>,
        equal_cells: &'a mut Vec<(Cell, Cell)>,
    ) -> Self {
        Self {
            layouter,
            assigned_values,
            equal_cells,
        }
    }
}

impl<'a, F: Field, L: Layouter<F>> Layouter<F> for ExtractingLayouter<'a, F, L> {
    type Root = Self;

    fn assign_region<A, AR, N, NR>(
        &mut self,
        name: N,
        mut assignment: A,
    ) -> Result<AR, halo2_proofs::plonk::Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, halo2_proofs::plonk::Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        let layouter = &mut self.layouter;
        let eqs = &mut self.assigned_values;
        let cell_eqs = &mut self.equal_cells;
        layouter.assign_region(name, |region| {
            let mut region = ExtractingRegion(region, *eqs, *cell_eqs);
            let region: &mut dyn RegionLayouter<F> = &mut region;
            assignment(region.into())
        })
    }

    fn assign_table<A, N, NR>(
        &mut self,
        name: N,
        assignment: A,
    ) -> Result<(), halo2_proofs::plonk::Error>
    where
        A: FnMut(Table<'_, F>) -> Result<(), halo2_proofs::plonk::Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.layouter.assign_table(name, assignment)
    }

    fn constrain_instance(
        &mut self,
        cell: Cell,
        column: Column<Instance>,
        row: usize,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.layouter.constrain_instance(cell, column, row)
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.layouter.push_namespace(name_fn);
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        self.layouter.pop_namespace(gadget_name)
    }
}

#[derive(Debug)]
struct ExtractingRegion<'a, 'b, F: Field>(
    Region<'a, F>,
    &'b mut Vec<(Cell, F)>,
    &'b mut Vec<(Cell, Cell)>,
);

impl<'a, 'b, F: Field> RegionLayouter<F> for ExtractingRegion<'a, 'b, F> {
    fn enable_selector<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        selector: &Selector,
        offset: usize,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.0.enable_selector(annotation, selector, offset)
    }

    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<Cell, halo2_proofs::plonk::Error> {
        self.0
            .assign_advice(annotation, column, offset, to)
            .map(|value| {
                value.value().and_then(|v| {
                    self.1.push((value.cell(), v.evaluate()));
                    Value::known(())
                });
                value.cell()
            })
    }

    fn assign_advice_from_constant<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: halo2_proofs::plonk::Column<halo2_proofs::plonk::Advice>,
        offset: usize,
        constant: halo2_proofs::plonk::Assigned<F>,
    ) -> Result<halo2_proofs::circuit::Cell, halo2_proofs::plonk::Error> {
        self.0
            .assign_advice_from_constant(annotation, column, offset, constant)
            .map(|value| {
                value.value().and_then(|v| {
                    self.1.push((value.cell(), v.evaluate()));
                    Value::known(())
                });
                value.cell()
            })
    }

    fn assign_advice_from_instance<'v>(
        &mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        instance: Column<Instance>,
        row: usize,
        advice: Column<Advice>,
        offset: usize,
    ) -> Result<(Cell, Value<F>), halo2_proofs::plonk::Error> {
        self.0
            .assign_advice_from_instance(annotation, instance, row, advice, offset)
            .map(|value| {
                value.value().and_then(|v| {
                    self.1.push((value.cell(), *v));
                    Value::known(())
                });
                (value.cell(), value.value().cloned())
            })
    }

    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<Cell, halo2_proofs::plonk::Error> {
        self.0
            .assign_fixed(annotation, column, offset, to)
            .map(|value| {
                value.value().and_then(|v| {
                    self.1.push((value.cell(), v.evaluate()));
                    Value::known(())
                });
                value.cell()
            })
    }

    fn constrain_constant(
        &mut self,
        cell: Cell,
        constant: Assigned<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.0.constrain_constant(cell, constant)
    }

    fn constrain_equal(
        &mut self,
        left: Cell,
        right: Cell,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.2.push((left, right));
        self.0.constrain_equal(left, right)
    }

    fn instance_value(
        &mut self,
        instance: Column<Instance>,
        row: usize,
    ) -> Result<Value<F>, halo2_proofs::plonk::Error> {
        self.0.instance_value(instance, row)
    }
}

struct ExtractingAssignment<'cs, F: Field, CS: Assignment<F>> {
    cs: &'cs mut CS,
    _marker: PhantomData<F>,
}

impl<'cs, F: Field, CS: Assignment<F>> ExtractingAssignment<'cs, F, CS> {
    fn new(cs: &'cs mut CS) -> Self {
        Self {
            cs,
            _marker: PhantomData,
        }
    }
}

impl<'cs, F: Field, CS: Assignment<F>> Assignment<F> for ExtractingAssignment<'cs, F, CS> {
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.enter_region(name_fn);
    }

    fn exit_region(&mut self) {
        self.cs.exit_region();
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
        let annotation = annotation().into();
        self.cs.enable_selector(|| annotation, selector, row)
    }

    fn query_instance(
        &self,
        column: Column<Instance>,
        row: usize,
    ) -> Result<Value<F>, halo2_proofs::plonk::Error> {
        self.cs.query_instance(column, row)
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
        let annotation = annotation().into();
        self.cs.assign_advice(|| annotation, column, row, to)
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
        let annotation = annotation().into();
        self.cs.assign_fixed(|| annotation, column, row, to)
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.cs.copy(left_column, left_row, right_column, right_row)
    }

    fn fill_from_row(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        to: Value<Assigned<F>>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        self.cs.fill_from_row(column, row, to)
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.push_namespace(name_fn)
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        self.cs.pop_namespace(gadget_name);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use arrayvec::ArrayString;
    use halo2_proofs::{
        arithmetic::Field,
        circuit::{AssignedCell, Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Instance, Selector},
        poly::Rotation,
    };

    use crate::field::TermField;

    use super::ExtractingFloorPlanner;

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
        type FloorPlanner = ExtractingFloorPlanner<SimpleFloorPlanner>;

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
        let k = 4;

        let a = TermField::from("a"); // F[0]
        let b = TermField::from("b"); // F[0] // F[1]
        let out = TermField::from("c"); // F[9]

        let circuit = MyCircuit(PhantomData);

        let public_input = vec![a, b, out];

        MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    }
}
