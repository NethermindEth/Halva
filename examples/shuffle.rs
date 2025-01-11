use std::{marker::PhantomData, vec};

use halo2_extr::{extraction::ExtractingAssignment, field::TermField};
use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        Advice, Circuit, Column,
        ConstraintSystem, ErrorFront, Fixed, Selector,
    },
    poly::Rotation,
};

struct ShuffleChip<F: Field> {
    config: ShuffleConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
struct ShuffleConfig {
    input_0: Column<Advice>,
    input_1: Column<Fixed>,
    shuffle_0: Column<Advice>,
    shuffle_1: Column<Advice>,
    s_input: Selector,
    s_shuffle: Selector,
}

impl<F: Field> ShuffleChip<F> {
    fn construct(config: ShuffleConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        input_0: Column<Advice>,
        input_1: Column<Fixed>,
        shuffle_0: Column<Advice>,
        shuffle_1: Column<Advice>,
    ) -> ShuffleConfig {
        let s_shuffle = meta.complex_selector();
        let s_input = meta.complex_selector();
        meta.shuffle("shuffle", |meta| {
            let s_input = meta.query_selector(s_input);
            let s_shuffle = meta.query_selector(s_shuffle);
            let input_0 = meta.query_advice(input_0, Rotation::cur());
            let input_1 = meta.query_fixed(input_1, Rotation::cur());
            let shuffle_0 = meta.query_advice(shuffle_0, Rotation::cur());
            let shuffle_1 = meta.query_advice(shuffle_1, Rotation::cur());
            vec![
                (s_input.clone() * input_0, s_shuffle.clone() * shuffle_0),
                (s_input * input_1, s_shuffle * shuffle_1),
            ]
        });
        ShuffleConfig {
            input_0,
            input_1,
            shuffle_0,
            shuffle_1,
            s_input,
            s_shuffle,
        }
    }
}

#[derive(Default)]
struct MyCircuit<F: Field> {
    input_0: Vec<Value<F>>,
    input_1: Vec<F>,
    shuffle_0: Vec<Value<F>>,
    shuffle_1: Vec<Value<F>>,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    // Since we are using a single chip for everything, we can just reuse its config.
    type Config = ShuffleConfig;
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let input_0 = meta.advice_column();
        let input_1 = meta.fixed_column();
        let shuffle_0 = meta.advice_column();
        let shuffle_1 = meta.advice_column();
        ShuffleChip::configure(meta, input_0, input_1, shuffle_0, shuffle_1)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        let ch = ShuffleChip::<F>::construct(config);
        layouter.assign_region(
            || "load inputs",
            |mut region| {
                for (i, (input_0, input_1)) in
                    self.input_0.iter().zip(self.input_1.iter()).enumerate()
                {
                    region.assign_advice(|| "input_0", ch.config.input_0, i, || *input_0)?;
                    region.assign_fixed(
                        || "input_1",
                        ch.config.input_1,
                        i,
                        || Value::known(*input_1),
                    )?;
                    ch.config.s_input.enable(&mut region, i)?;
                }
                Ok(())
            },
        )?;
        layouter.assign_region(
            || "load shuffles",
            |mut region| {
                for (i, (shuffle_0, shuffle_1)) in
                    self.shuffle_0.iter().zip(self.shuffle_1.iter()).enumerate()
                {
                    region.assign_advice(|| "shuffle_0", ch.config.shuffle_0, i, || *shuffle_0)?;
                    region.assign_advice(|| "shuffle_1", ch.config.shuffle_1, i, || *shuffle_1)?;
                    ch.config.s_shuffle.enable(&mut region, i)?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

fn main() {
    let input_0 = ["a1", "a2", "a3", "a4"]
        .map(|e| Value::known(TermField::create_symbol(e)))
        .to_vec();
    let input_1 = ["b1", "b2", "b3", "b4"]
        .map(|e| TermField::create_symbol(e))
        .to_vec();
    let shuffle_0 = ["c1", "c2", "c3", "c4"]
        .map(|e| Value::known(TermField::create_symbol(e)))
        .to_vec();
    let shuffle_1 = ["d1", "d2", "d3", "d4"]
        .map(|e| Value::known(TermField::create_symbol(e)))
        .to_vec();

    let circuit = MyCircuit::<TermField> {
        input_0,
        input_1,
        shuffle_0,
        shuffle_1,
    };

    ExtractingAssignment::run(&circuit, "ShuffleExample", &["a1", "a2", "a3", "a4", "b1", "b2", "b3", "b4", "c1", "c2", "c3", "c4", "d1", "d2", "d3", "d4"]).unwrap();
}

#[test]
fn test_shuffle_api() {
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::Fp;
    const K: u32 = 4;
    let input_0 = [1, 2, 1, 4]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let input_1 = [100, 20, 40, 100].map(Fp::from).to_vec();
    let shuffle_0 = [4, 1, 1, 2]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let shuffle_1 = [100a, 100, 40, 20]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let circuit = MyCircuit {
        input_0,
        input_1,
        shuffle_0,
        shuffle_1,
    };
    let prover = MockProver::run(K, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}
