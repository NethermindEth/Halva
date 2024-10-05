use std::marker::PhantomData;

use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, ErrorFront},
};

// use halo2_proofs::dev::cost_model::{from_circuit_to_model_circuit, CommitmentScheme};
use halo2_proofs::plonk::{Expression, Selector, TableColumn};
use halo2_proofs::poly::Rotation;

use halo2_extr::{extract, extraction::Target, field::TermField};

// We use a lookup example
#[derive(Clone, Copy, Default)]
struct TestCircuit<TermField>(PhantomData<TermField>);

#[derive(Debug, Clone)]
struct MyConfig {
    selector: Selector,
    table: TableColumn,
    advice: Column<Advice>,
}

impl Circuit<TermField> for TestCircuit<TermField> {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<TermField>) -> MyConfig {
        let config = MyConfig {
            selector: meta.complex_selector(),
            table: meta.lookup_table_column(),
            advice: meta.advice_column(),
        };

        meta.lookup("lookup", |meta| {
            let selector = meta.query_selector(config.selector);
            let not_selector = Expression::Constant(TermField::ONE) - selector.clone();
            let advice = meta.query_advice(config.advice, Rotation::cur());
            vec![(selector * advice + not_selector, config.table)]
        });

        config
    }

    fn synthesize(
        &self,
        config: MyConfig,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<(), ErrorFront> {
        layouter.assign_table(
            || "8-bit table",
            |mut table| {
                for row in 0u64..(1 << 8) {
                    table.assign_cell(
                        || format!("row {row}"),
                        config.table,
                        row as usize,
                        || Value::known(TermField::from(row + 1)),
                    )?;
                }

                Ok(())
            },
        )?;

        layouter.assign_region(
            || "assign values",
            |mut region| {
                for offset in 0u64..(1 << 10) {
                    config.selector.enable(&mut region, offset as usize)?;
                    region.assign_advice(
                        || format!("offset {offset}"),
                        config.advice,
                        offset as usize,
                        || Value::known(TermField::from((offset % 256) + 1)),
                    )?;
                }

                Ok(())
            },
        )
    }
}

fn main() {
    extract!(TestCircuit, Target::AdviceGenerator);
}