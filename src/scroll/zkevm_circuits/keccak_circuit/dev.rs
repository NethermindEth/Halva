pub use super::KeccakCircuit;
use super::KeccakTable;

use crate::scroll::zkevm_circuits::{
    keccak_circuit::{KeccakCircuitConfig, KeccakCircuitConfigArgs},
    util::{Challenges, SubCircuit, SubCircuitConfig},
};

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, ConstraintSystem, ErrorFront}
};

use crate::field::TermField;

impl Circuit<TermField> for KeccakCircuit {
    type Config = (KeccakCircuitConfig, Challenges);
    type FloorPlanner = SimpleFloorPlanner;
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<TermField>) -> Self::Config {
        let keccak_table = KeccakTable::construct(meta);
        let challenges = Challenges::construct(meta);

        let config = {
            let challenges = challenges.exprs(meta);
            KeccakCircuitConfig::new(
                meta,
                KeccakCircuitConfigArgs {
                    keccak_table,
                    challenges,
                },
            )
        };
        (config, challenges)
    }

    fn synthesize(
        &self,
        (config, challenges): Self::Config,
        mut layouter: impl Layouter<TermField>,
    ) -> Result<(), ErrorFront> {
        let challenges = challenges.values(&layouter);
        self.synthesize_sub(&config, &challenges, &mut layouter)
    }
}
