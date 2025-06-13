use halo2_proofs::plonk::{Circuit, ConstraintSystem};

use halo2_proofs::plonk::{Error, FloorPlanner};

use crate::collecting_assignment::CollectingAssignment;
use crate::field::TermField;

pub trait DelegatingProver {
    fn get_comment_prefix() -> String;

    fn print_preamble(&self, cs: &ConstraintSystem<TermField>);
    fn print_postamble(&self, assignment: &CollectingAssignment<TermField>, cs: &ConstraintSystem<TermField>);

    fn print_copy_constraints(assignment: &CollectingAssignment<TermField>);
    fn print_selectors(assignment: &CollectingAssignment<TermField>);
    fn print_fixed(assignment: &CollectingAssignment<TermField>);
    fn print_advice_phase(cs: &ConstraintSystem<TermField>);
    fn print_advice_annotations(assignment: &CollectingAssignment<TermField>);
    fn print_instance_annotations(assignment: &CollectingAssignment<TermField>);
    fn print_gates(cs: &ConstraintSystem<TermField>);
    fn print_lookups(cs: &ConstraintSystem<TermField>);
    fn print_shuffles(cs: &ConstraintSystem<TermField>);

    fn print_grouping_props(assignment: &CollectingAssignment<TermField>, cs: &ConstraintSystem<TermField>) {
        println!("");
        println!("");
        Self::print_copy_constraints(assignment);
        Self::print_selectors(assignment);
        Self::print_fixed(assignment);
        Self::print_advice_phase(&cs);
        Self::print_advice_annotations(assignment);
        Self::print_instance_annotations(assignment);
        Self::print_gates(&cs);
        Self::print_lookups(&cs);
        Self::print_shuffles(&cs);
    }

    fn run<ConcreteCircuit: Circuit<TermField>>(
        &self,
        circuit: &ConcreteCircuit,
        // namespace: &str,
        // symbol_names: &[&str]
    ) -> Result<(), Error> {
        
        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
        let cs = cs;
        self.print_preamble(&cs);

        let mut prover = CollectingAssignment::new(Self::get_comment_prefix());

        for current_phase in cs.phases() {
            prover.current_phase = current_phase;
            ConcreteCircuit::FloorPlanner::synthesize(
                &mut prover,
                circuit,
                config.clone(),
                cs.constants().clone(),
            )?;
        }

        Self::print_grouping_props(&prover, &cs);

        self.print_postamble(&prover, &cs);
        Ok(())

    }
}
