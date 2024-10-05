use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Expression, Advice, Circuit, TableColumn, Column, ConstraintSystem, Instance, Selector},
    poly::Rotation,
};

use halo2_extr::{extract, extraction::Target, field::TermField};

#[derive(Debug, Clone)]
struct FibonacciConfig {
    pub col_a: Column<Advice>,
    pub col_b: Column<Advice>,
    pub col_c: Column<Advice>,
    pub col_t: Column<Advice>,
    pub table: TableColumn,
    pub selector: Selector,
    pub selector_c: Selector,
    pub instance: Column<Instance>,
}

#[derive(Debug, Clone)]
struct FibonacciChip<TermField> {
    config: FibonacciConfig,
    _marker: PhantomData<TermField>,
}

impl FibonacciChip<TermField> {
    pub fn construct(config: FibonacciConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<TermField>) -> FibonacciConfig {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let col_t = meta.advice_column();
        let table = meta.lookup_table_column();
        let selector = meta.selector();
        let selector_c = meta.complex_selector();
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

        meta.lookup("lookup", |meta| {
            let selector = meta.query_selector(selector_c);
            let not_selector = Expression::Constant(TermField::One) - selector.clone();
            let advice = meta.query_advice(col_t, Rotation::cur());
            vec![(selector * advice + not_selector, table)]
        });

        FibonacciConfig {
            col_a,
            col_b,
            col_c,
            col_t,
            table,
            selector,
            selector_c,
            instance,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn assign_first_row(
        &self,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<
        (AssignedCell<TermField, TermField>, AssignedCell<TermField, TermField>, AssignedCell<TermField, TermField>),
        halo2_frontend::plonk::Error,
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
        mut layouter: impl Layouter<TermField>,
        prev_b: &AssignedCell<TermField, TermField>,
        prev_c: &AssignedCell<TermField, TermField>,
    ) -> Result<AssignedCell<TermField, TermField>, halo2_frontend::plonk::Error> {
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
        mut layouter: impl Layouter<TermField>,
        cell: &AssignedCell<TermField, TermField>,
        row: usize,
    ) -> Result<(), halo2_frontend::plonk::Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct MyCircuit<TermField>(PhantomData<TermField>);

impl Circuit<TermField> for MyCircuit<TermField> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<TermField>) -> Self::Config {
        FibonacciChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<(), halo2_frontend::plonk::Error> {
        let chip = FibonacciChip::construct(config);

        let (_, mut prev_b, mut prev_c) =
            chip.assign_first_row(layouter.namespace(|| "first row"))?;

        for _i in 3..10 {
            let c_cell = chip.assign_row(layouter.namespace(|| "next row"), &prev_b, &prev_c)?;
            prev_b = prev_c;
            prev_c = c_cell;
        }

        chip.expose_public(layouter.namespace(|| "out"), &prev_c, 2)?;

        layouter.assign_table(
            || "id table",
            |mut table| {
                for row in 0..100 {
                    table.assign_cell(
                        || format!("row {row}"),
                        chip.config.table,
                        row as usize,
                        || Value::known(TermField::from(row)),
                    )?;
                }

                Ok(())
            },
        )?;

        // layouter.assign_region(
        //     || "assign values",
        //     |mut region| {
        //         for offset in 0u64..(1 << 10) {
        //             chip.config.selector_c.enable(&mut region, offset as usize)?;
        //             region.assign_advice(
        //                 || format!("offset {offset}"),
        //                 chip.config.col_t,
        //                 offset as usize,
        //                 || Value::known(TermField::from((offset % 256) + 1)),
        //             )?;
        //         }

        //         Ok(())
        //     },
        // )?;

        Ok(())
    }
}

fn main() {
    use halo2_extr::extraction::{print_gates, ExtractingAssignment};
    use halo2_extr::field::TermField;
    use halo2_frontend::dev::CircuitGates;
    use halo2_proofs::halo2curves::pasta::Fp;
    use halo2_proofs::plonk::{Circuit, ConstraintSystem, FloorPlanner};
    let circuit: MyCircuit<TermField> = MyCircuit::default();

    let mut cs = ConstraintSystem::<TermField>::default();
    let config = MyCircuit::<TermField>::configure(&mut cs);

    let mut extr_assn = ExtractingAssignment::<TermField>::new(Target::AdviceGenerator);
    <MyCircuit<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
        &mut extr_assn,
        &circuit,
        config,
        vec![],
    )
    .unwrap();

    let test_gates = cs.gates();
    println!("\n\nGATES");
    println!("\n\n{:?}\n\n", test_gates);

    let test_lookups = cs.lookups();
    println!("\n\nLOOKUPS");
    println!("\n\n{:?}\n\n", test_lookups);
    extract!(MyCircuit, Target::AdviceGenerator);
}
