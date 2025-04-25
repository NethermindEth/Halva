use halo2_extr::{extraction::ExtractingAssignment, field::TermField, scroll::gadgets::binary_number::{AsBits, BinaryNumberChip, BinaryNumberConfig}};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, Column, ConstraintSystem, Fixed},
};

use halo2_frontend::plonk::Error;

use strum::EnumIter;

#[derive(Clone, Debug, EnumIter)]
enum TestEnum {
    A,
    B,
    C,
}

impl AsBits<2> for TestEnum {
    fn as_bits(&self) -> [bool; 2] {
        match self {
            TestEnum::A => [false, false],
            TestEnum::B => [false, true],
            TestEnum::C => [true, false],
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct TestCircuitConfig {
    q_enable: Column<Fixed>,
    value: Column<Fixed>,
    binary_number_chip_config: BinaryNumberConfig<TestEnum, 2>,
}

#[derive(Default)]
struct TestCircuit {}

impl Circuit<TermField> for TestCircuit {
    type Config = TestCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<TermField>) -> Self::Config {
        // Binary Config requires a fixed rather than a selector as its 'selector' column
        let q_enable = meta.fixed_column();
        let value = meta.fixed_column();

        let binary_number_chip_config =
            BinaryNumberChip::configure(meta, q_enable, Some(value.into()));

        let config = Self::Config {
            q_enable,
            value,
            binary_number_chip_config,
        };

        config
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "witness",
            |mut _region| {
                
                Ok(())
            }
        )
    }
}

fn main() {
    let circuit = TestCircuit {};
    ExtractingAssignment::run(&circuit, "BinaryNumber", &[]).unwrap();
}