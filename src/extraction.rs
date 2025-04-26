use std::collections::BTreeMap;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use halo2_frontend::plonk::sealed::SealedPhase;
use halo2_frontend::plonk::{sealed, Phase};
use halo2_proofs::plonk::{Circuit, ConstraintSystem, Expression, FirstPhase};
use itertools::Itertools;

use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    plonk::{Advice, Any, Assigned, Assignment, Column, Error, Fixed, FloorPlanner, Instance, Selector},
};

use crate::field::TermField;
use crate::utils::{get_group_annotations, group_values, make_lean_comment, print_grouped_props, update_column_annotation, update_row_annotation};

const GROUPING_SIZE: usize = 10;

pub struct ExtractingAssignment<F: Field> {
    _marker: PhantomData<F>,
    advice_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    current_region: Option<String>,
    copies: Vec<((Column<Any>, usize), (Column<Any>, usize))>,
    selectors: BTreeMap<usize, BTreeMap<usize, String>>,
    fixed: BTreeMap<usize, BTreeMap<usize, String>>,
    fixed_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    fixed_fill: BTreeMap<usize, (usize, String)>,
    instance_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    current_phase: sealed::Phase,
    usable_rows_filename: String,
}

impl<F: Field> Drop for ExtractingAssignment<F> {
    fn drop(&mut self) {
        std::fs::remove_file(&self.usable_rows_filename).expect("Failed to delete usable_rows file. Feel free to manually delete");
    }
}

// impl<F: Field + From<String> + Display> ExtractingAssignment<F> {
impl ExtractingAssignment<TermField> {
    pub fn new() -> Self {
        let usable_rows_filename = if Path::new("./usable_rows").exists() {
            let mut i = 1;
            while Path::new(&format!("./usable_rows_{i}")).exists() {
                i += 1;
            }
            format!("usable_rows_{i}")
        } else {
            "usable_rows".to_string()
        };
        fs::write(&usable_rows_filename, "0").expect("Failed to write to usable_rows");
        Self {
            _marker: PhantomData,
            advice_column_annotations: BTreeMap::new(),
            current_region: None,
            copies: vec![],
            selectors: BTreeMap::new(),
            fixed: BTreeMap::new(),
            fixed_column_annotations: BTreeMap::new(),
            fixed_fill: BTreeMap::new(),
            instance_column_annotations: BTreeMap::new(),
            current_phase: FirstPhase.to_sealed(),
            usable_rows_filename,
        }
    }

    fn in_phase<P: Phase>(&self, phase: P) -> bool {
        self.current_phase == phase.to_sealed()
    }

    fn print_copy_constraints(&self) {

        let format_side = |col: &Column<Any>, row| {
            match col.column_type() {
                Any::Advice => format!("c.get_advice {} {}", col.index(), row),
                Any::Fixed => format!("c.get_fixed {} {}", col.index(), row),
                Any::Instance => format!("c.get_instance {} {}", col.index(), row),
            }
        };

        let props = self
            .copies
            .iter()
            .map(|((left_column, left_row), (right_column, right_row))| {
                format!("{} = {}", format_side(left_column, left_row), format_side(right_column, right_row))
            })
            .collect_vec();

        print_grouped_props("copy_", "all_copy_constraints", &props, GROUPING_SIZE);
    }

    // TODO grouping, annotations
    fn print_selectors(&self) {
        for (col, row_set) in &self.selectors {
            if let Some((&start, _)) = row_set.first_key_value() {
                let runs = {
                    let mut start = start;
                    // End is inclusive
                    let mut end = start;
                    let mut runs = vec![];
    
                    // Iterate through the true rows, to collect the consecutive runs
                    for (&i, _) in row_set.iter().skip(1) {
                        if end == i - 1 {
                            // We have found a row that connects to the current run
                            end = i;
                        } else {
                            runs.push((start, end));
                            start = i;
                            end = i;
                        }
                    }
    
                    runs.push((start, end));
                    runs
                };

                let body = runs
                    .iter()
                    .map(|(start, end)| {
                        if *start == 0 {
                            format!("if row < {} then 1", end+1)
                        } else {
                            format!("if row < {start} then 0\n  else if row < {} then 1", end+1)
                        }
                    })
                    .join("\n  else ");
                println!("def selector_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → ZMod P :=");
                println!("  λ row =>");
                println!("  {body}");
                println!("  else 0");
            } else {
                println!("def selector_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → ZMod P :=");
                println!("  λ _ => 0");
            }

        }
        println!("def selector_func (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
        println!("  λ col row => match col with");
        for col in self.selectors.keys() {
            println!("    | {col} => selector_func_col_{col} c row")
        }
        println!("    | _ => 0");
    }

    fn print_fixed(&self) {
        for (col, row_set) in &self.fixed {
            // (value, start, end, annotations already printed)
            let mut entries = group_values(row_set)
                .into_iter()
                .map(|(a,b,c)| (a,b,c,false))
                .collect_vec();

            assert!(GROUPING_SIZE > 1);

            while entries.len() > GROUPING_SIZE {
                let mut new_entries = vec![];
    
                while entries.len() > GROUPING_SIZE {
                    {
                        let start = entries[0].1;
                        let end = entries[GROUPING_SIZE-1].2.unwrap_or(entries[GROUPING_SIZE-1].1);
                        let name = format!("fixed_func_col_{col}_{start}_to_{end}");
                        println!("def {name} (c: ValidCircuit P P_Prime) : ℕ → ZMod P :=");
                        println!("  λ row =>");
                        new_entries.push((
                            format!("{name} c row"),
                            start,
                            Some(end),
                            true
                        ));
                    }
                    let mut first = true;
                    for _ in 0..GROUPING_SIZE {
                        let value = &entries[0].0;
                        let start = entries[0].1;
                        let prefix = if first {
                            first = false;
                            "  "
                        } else {
                            "  else "
                        };
                        let print_annotations = !entries[0].3;
                        if let Some(end) = entries[0].2 {
                            let annotation = match (print_annotations, self.fixed_column_annotations.get(&col)) {
                                (true, Some((_, row_annotations))) => get_group_annotations(row_annotations, start, end),
                                _ => None,
                            };
                            
                            if let Some(annotation) = annotation {
                                if annotation.contains("\n") {
                                    println!("{annotation}");
                                    println!("{prefix}if row ≥ {start} ∧ row ≤ {end} then {value}")
                                } else {
                                    println!("{prefix}if row ≥ {start} ∧ row ≤ {end} then {value} -- {annotation}")
                                }
                            } else {
                                println!("{prefix}if row ≥ {start} ∧ row ≤ {end} then {value}")
                            }
                        } else {
                            let annotation = match (print_annotations, self.fixed_column_annotations.get(&col)) {
                                (true, Some((_, row_annotations))) => get_group_annotations(row_annotations, start, start),
                                _ => None,
                            };

                            if let Some(annotation) = annotation {
                                println!("{prefix}if row = {start} then {value}{annotation}")
                            } else {
                                println!("{prefix}if row = {start} then {value}")
                            }
                        }
                        entries.remove(0);
                    }
                    println!("  else c.1.FixedUnassigned {col} row");
                }

                for new_entry in new_entries.into_iter().rev() {
                    entries.insert(0, new_entry);
                }
            }

            println!("def fixed_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → ZMod P :=");
            println!("  λ row =>");
            let mut first = true;
            for (value, start, end, _) in entries {
                if first {
                    first = false;
                    print!("  ");
                } else {
                    print!("  else ");
                }
                if let Some(end) = end {
                    println!("if row ≥ {start} ∧ row ≤ {end} then {value}")
                } else {
                    println!("if row = {start} then {value}")
                }
            }
            if let Some ((fill_row, fill_value)) = self.fixed_fill.get(&col) {
                if first {
                    print!("  ");
                } else {
                    print!("  else ");
                }
                println!("if row ≥ {fill_row} ∧ row < c.usable_rows then {fill_value}")
            }
            println!("  else c.1.FixedUnassigned {col} row");
        }

        println!("def fixed_func (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
        println!("  λ col row => match col with");
        for col in self.fixed.keys() {
            if let Some((Some(annotation), _)) = self.fixed_column_annotations.get(col) {
                println!("    | {col} => fixed_func_col_{col} c row {}", make_lean_comment(annotation));
            } else {
                println!("    | {col} => fixed_func_col_{col} c row");
            }
        }
        println!("    | _ => c.1.FixedUnassigned col row");
    }

    fn print_advice_phase(&self, cs: &ConstraintSystem<TermField>) {
        println!("def advice_phase (c: ValidCircuit P P_Prime) : ℕ → ℕ :=");
        println!("  λ col => match col with");
        for (col, phase) in cs.advice_column_phase().iter().enumerate() {
            if *phase != 0 {
                println!("  | {col} => {phase}");
            }
        }
        println!("  | _ => 0");
    }

    fn print_advice_annotations(&self) {
        println!("  -- Advice column annotations:");
        if self.advice_column_annotations.is_empty() {
            println!("  -- None");
        }
        self.advice_column_annotations
            .iter()
            .for_each(|(col, (column_annotation, rows))| {
                println!("-- Advice Column {col}");
                if let Some(column_annotation) = column_annotation {
                    println!("{}", make_lean_comment(column_annotation));
                }
                if let Some((start, _)) = rows.first_key_value() {
                    if let Some ((end, _)) = rows.last_key_value() {
                        if let Some(comments) = get_group_annotations(rows, *start, *end) {
                            println!("{comments}");
                        }
                    }
                }
            });
    }

    fn print_instance_annotations(&self) {
        println!("  -- Instance column annotations:");
        if self.instance_column_annotations.is_empty() {
            println!("  -- None");
        }
        self.instance_column_annotations
            .iter()
            .for_each(|(col, (column_annotation, rows))| {
                println!("-- Instance Column {col}");
                if let Some(column_annotation) = column_annotation {
                    println!("{}", make_lean_comment(column_annotation));
                }
                if let Some((start, _)) = rows.first_key_value() {
                    if let Some ((end, _)) = rows.last_key_value() {
                        if let Some(comments) = get_group_annotations(rows, *start, *end) {
                            println!("{comments}");
                        }
                    }
                }
            });
    }

    fn print_gates(&self, cs: &ConstraintSystem<TermField>) {
        let constraints = cs
            .gates()
            .iter()
            .enumerate()
            .flat_map(|(gate_idx, gate)| {
                // Each gate can contain many polynomials, so we need an inner iteration
                gate
                    .polynomials()
                    .iter()
                    .enumerate()
                    .filter(move |(poly_idx, polynomial)| {
                        match polynomial {
                            Expression::Constant(TermField::Val(0)) => {
                                println!(
                                    "  -- Gate number {} name: \"{}\" part {}/{} {} is trivially true",
                                    gate_idx+1,
                                    gate.name(),
                                    poly_idx+1,
                                    gate.polynomials().len(),
                                    gate.constraint_name(*poly_idx)
                                );
                                false
                            },
                            _ => true
                        }
                    })
                    .map(move |(poly_idx, polynomial)| {
                        format!(
                            "-- Gate number {} name: \"{}\" part {}/{} {}\n  ∀ row: ℕ, {} = 0",
                            gate_idx+1,
                            gate.name(),
                            poly_idx+1,
                            gate.polynomials().len(),
                            gate.constraint_name(poly_idx),
                            expression_to_value_string(polynomial, "row")
                        )
                    })
            })
            .collect_vec();

        print_grouped_props("gate_", "all_gates", &constraints, GROUPING_SIZE);
    }

    fn print_lookups(&self, cs: &ConstraintSystem<TermField>) {
        let lookups = cs
            .lookups()
            .iter()
            .enumerate()
            .map(|(idx, lookup)| {
                let lhs = lookup.input_expressions()
                    .iter()
                    .map(|expr| {
                        expression_to_value_string(expr, "row")
                    })
                    .join(", ");
                let rhs = lookup.table_expressions()
                    .iter()
                    .map(|expr| {
                        expression_to_value_string(expr, "lookup_row")
                    })
                    .join(", ");
                format!(
                    "∀ row : ℕ, row < c.usable_rows → ∃ lookup_row : ℕ, lookup_row < c.usable_rows ∧ -- Lookup number {} name: \"{}\"\n  ({lhs}) = ({rhs})\n  ",
                    idx+1,
                    lookup.name()
                )
            })
            .collect_vec();

        print_grouped_props("lookup_", "all_lookups", &lookups, GROUPING_SIZE);
    }

    pub fn print_grouping_props(&self, cs: &ConstraintSystem<TermField>) {
        println!("");
        println!("");
        self.print_copy_constraints();
        self.print_selectors();
        self.print_fixed();
        self.print_advice_phase(&cs);
        self.print_advice_annotations();
        self.print_instance_annotations();
        self.print_gates(&cs);
        self.print_lookups(&cs);
        

        // Shuffles
        {
            let mut shuffle_names = vec![];
            for shuffle in cs.shuffles() {
                let name = format!("shuffle_{}", shuffle.name().replace("_", "__").replace(" ", "_")); // TODO mangle if necessary
                shuffle_names.push(name.clone());
                let lhs = shuffle.input_expressions()
                    .iter()
                    .map(|expr| {
                        expression_to_value_string(expr, "row")
                    })
                    .join(", ");
                let rhs = shuffle.shuffle_expressions()
                    .iter()
                    .map(|expr| {
                        expression_to_value_string(expr, "(shuffle row)")
                    })
                    .join(", ");
                println!("def {name} (c: ValidCircuit P P_Prime): Prop := ∃ shuffle, is_shuffle c shuffle ∧ (∀ row : ℕ, row < c.usable_rows → ({lhs}) = ({rhs}))");
            }
    
            let all_shuffles_body = if shuffle_names.is_empty() {
                "true".to_string()
            } else {
                shuffle_names
                    .iter()
                    .map(|name| format!("{name} c"))
                    .join(" ∧ ")
            };
            println!("def all_shuffles (c: ValidCircuit P P_Prime) : Prop := {all_shuffles_body}");
        }
    }

    fn set_selector(&mut self, col: usize, row: usize, annotation: String) {
        let s = self.selectors.get_mut(&col);
        if let Some(v) = s {
            v.insert(row, annotation);
        } else {
            let mut new_set = BTreeMap::new();
            new_set.insert(row, annotation);
            self.selectors.insert(col, new_set);
        };
    }

    // Assign a cell in the fixed map, adjusting fixed_fill if necessary
    fn set_fixed_checked(&mut self, col: usize, row: usize, val: String) {
        let fill = self.fixed_fill.get(&col);

        // If assigning a cell beyond the fill, push the fill back and write it into the map
        if let Some((fill_row, fill_val)) = fill {
            if *fill_row <= row {
                // This handles the writing of the lower rows automatically
                self.set_fixed_fill(col, row + 1, (*fill_val).clone());
            }
        }

        self.set_fixed_unchecked(col, row, val);
    }

    // Assign a cell into the fixed map, creating a new inner map if necessary
    // Does not check fixed_fill
    fn set_fixed_unchecked(&mut self, col: usize, row: usize, val: String) {
        let fixed_column_opt = self.fixed.get_mut(&col);

        if let Some(fixed_column) = fixed_column_opt {
            fixed_column.insert(row, val);
        } else {
            let mut new_map = BTreeMap::new();
            new_map.insert(row, val);
            self.fixed.insert(col, new_map);
        };
    }

    fn set_fixed_fill(&mut self, col: usize, row: usize, val: String) {
        // Insert the new fill, and get the old one
        let old_fill = self.fixed_fill.insert(col, (row, val));

        // If there was a fill in place that started lower than the new one,
        // fill it in manually
        if let Some ((fill_row, fill_val)) = old_fill {
            if fill_row < row {
                for i in fill_row..row {
                    self.set_fixed_unchecked(col, i, fill_val.clone());
                }
            }
        }
    }

    pub fn run<ConcreteCircuit: Circuit<TermField>>(
        circuit: &ConcreteCircuit,
        namespace: &str,
        symbol_names: &[&str]
    ) -> Result<(), Error> {
        
        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
        let cs = cs;
        print_preamble(namespace, symbol_names, &cs);

        let mut prover = ExtractingAssignment::new();

        for current_phase in cs.phases() {
            prover.current_phase = current_phase;
            ConcreteCircuit::FloorPlanner::synthesize(
                &mut prover,
                circuit,
                config.clone(),
                cs.constants().clone(),
            )?;
        }

        prover.print_grouping_props(&cs);

        print_postamble(namespace, &cs);
        Ok(())

    }

    fn assert_row_usable(&self, row: usize) {
        let usable_rows = str::parse::<usize>(
            &fs::read_to_string(&self.usable_rows_filename)
                .expect("Failed to read usable_rows file")
        ).expect("Failed to parse contents of usable_rows file");
        if row >= usable_rows {
            // row+1 because of 0-indexing
            fs::write(&self.usable_rows_filename, (row+1).to_string()).expect("Failed to write usable_rows file");
        }
    }
}

impl Assignment<TermField> for ExtractingAssignment<TermField>
{
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        let x: String = name_fn().into();
        println!("\n-- Entered region: {x}");
        self.current_region = Some(x.clone());
    }

    fn exit_region(&mut self) {
        println!("-- Exited region: {}", self.current_region.as_ref().unwrap());
        self.current_region = None;
    }

    fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), halo2_frontend::plonk::Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.in_phase(FirstPhase) {
            println!("--WARNING: Attempted to assign selector {} {} outside or first phase", selector.index(), row);
            return Ok(());
        }

        self.assert_row_usable(row);

        self.set_selector(selector.index(), row, annotation().into());
        Ok(())
    }

    fn query_instance(
        &self,
        column: Column<Instance>,
        row: usize,
    ) -> Result<Value<TermField>, halo2_frontend::plonk::Error> {
        self.assert_row_usable(row);

        Ok(Value::known(TermField::from(format!(
            "instance_to_field (c.1.Instance {} {})",
            column.index(),
            row
        ))))
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        _to: V,
    ) -> Result<(), halo2_frontend::plonk::Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<TermField>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if self.in_phase(FirstPhase) {
            self.assert_row_usable(row);
        }

        update_row_annotation(&mut self.advice_column_annotations, column.index(), row, annotation().into());

        // Aside from the above range assertion,
        // we ignore advice assignment as we are concerned only with constraint generation
        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), halo2_frontend::plonk::Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<TermField>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        update_row_annotation(&mut self.fixed_column_annotations, column.index(), row, annotation().into());
        self.assert_row_usable(row);

        to().map(|v| {
            self.set_fixed_checked(
                column.index(),
                row,
                v.into().evaluate().to_string()
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
    ) -> Result<(), halo2_frontend::plonk::Error> {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        self.assert_row_usable(left_row);
        self.assert_row_usable(right_row);

        self.copies.push(((left_column, left_row), (right_column, right_row)));
        Ok(())
    }

    fn fill_from_row(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        to: Value<Assigned<TermField>>,
    ) -> Result<(), halo2_frontend::plonk::Error> {
        if !self.in_phase(FirstPhase) {
            return Ok(());
        }

        self.assert_row_usable(row);

        let fill_val = to.assign()?.evaluate().to_string();
        self.set_fixed_fill(column.index(), row, fill_val);
        Ok(())
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn pop_namespace(&mut self, _gadget_name: Option<String>) {}

    fn annotate_column<A, AR>(&mut self, annotation: A, column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        match column.column_type {
            Any::Advice => update_column_annotation(&mut self.advice_column_annotations, column.index, annotation().into()),
            Any::Fixed => update_column_annotation(&mut self.fixed_column_annotations, column.index, annotation().into()),
            Any::Instance => update_column_annotation(&mut self.instance_column_annotations, column.index, annotation().into()),
        };
    }

    fn get_challenge(&self, challenge: halo2_proofs::plonk::Challenge) -> Value<TermField> {
        Value::known(TermField::from(format!("c.get_challenge {} {}", challenge.index(), challenge.phase())))
    }
}

pub fn print_preamble(namespace: &str, symbol_names: &[&str], cs: &ConstraintSystem<TermField>) {
    println!("import Mathlib.Data.Nat.Prime.Defs");
    println!("import Mathlib.Data.Nat.Prime.Basic");
    println!("import Mathlib.Data.ZMod.Defs");
    println!("import Mathlib.Data.ZMod.Basic\n");

    println!("set_option linter.unusedVariables false\n");

    println!("namespace {namespace}\n");

    println!("def S_T_from_P (S T P : ℕ) : Prop :=");
    println!("  (2^S * T = P - 1) ∧");
    println!("  (∀ s' t': ℕ, 2^s' * t' = P - 1 → s' ≤ S)");
    
    println!("def multiplicative_generator (P: ℕ) (mult_gen: ZMod P) : Prop :=");
    println!("  mult_gen ^ P = 1");
    
    println!("structure Circuit (P: ℕ) (P_Prime: Nat.Prime P) where");
    println!("  Advice: ℕ → ℕ → ZMod P");
    println!("  AdviceUnassigned: ℕ → ℕ → ZMod P");
    println!("  AdvicePhase: ℕ → ℕ");
    println!("  Fixed: ℕ → ℕ → ZMod P");
    println!("  FixedUnassigned: ℕ → ℕ → ZMod P");
    println!("  Instance: ℕ → ℕ → ZMod P");
    println!("  InstanceUnassigned: ℕ → ℕ → ZMod P");
    println!("  Selector: ℕ → ℕ → ZMod P");
    println!("  Challenges: (ℕ → ℕ → ZMod P) → ℕ → ℕ → ZMod P");
    println!("  num_blinding_factors: ℕ");
    println!("  S: ℕ");
    println!("  T: ℕ");
    println!("  k: ℕ");
    println!("  mult_gen: ZMod P");
    for symbol_name in symbol_names {
        println!("  sym_{symbol_name}: ZMod P")
    }
    
    println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}}");
    println!("def Circuit.isValid (c: Circuit P P_Prime) : Prop :=");
    println!("  S_T_from_P c.S c.T P ∧");
    println!("  multiplicative_generator P c.mult_gen ∧ (");
    println!("  ∀ advice1 advice2: ℕ → ℕ → ZMod P, ∀ phase: ℕ,");
    println!("    (∀ row col, (col < {} ∧ c.AdvicePhase col ≤ phase) → advice1 col row = advice2 col row) →", cs.num_advice_columns());
    println!("    (∀ i, c.Challenges advice1 i phase = c.Challenges advice2 i phase)");
    println!("  )");

    println!("abbrev ValidCircuit (P: ℕ) (P_Prime: Nat.Prime P) : Type := {{c: Circuit P P_Prime // c.isValid}}");
    println!("namespace ValidCircuit");
    println!("def get_advice (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
    println!("  λ col row => c.1.Advice col row");
    println!("def get_fixed (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
    println!("  λ col row => c.1.Fixed col row");
    println!("def get_instance (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
    println!("  λ col row => c.1.Instance col row");
    println!("def get_selector (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
    println!("  λ col row => c.1.Selector col row");
    println!("def get_challenge (c: ValidCircuit P P_Prime) : ℕ → ℕ → ZMod P :=");
    println!("  λ idx phase => c.1.Challenges c.1.Advice idx phase");
    println!("def k (c: ValidCircuit P P_Prime) := c.1.k");
    println!("def n (c: ValidCircuit P P_Prime) := 2^c.k");
    println!("def usable_rows (c: ValidCircuit P P_Prime) := c.n - (c.1.num_blinding_factors + 1)");
    println!("def S (c: ValidCircuit P P_Prime) := c.1.S");
    println!("def T (c: ValidCircuit P P_Prime) := c.1.T");
    println!("def mult_gen (c: ValidCircuit P P_Prime) := c.1.mult_gen");
    println!("def root_of_unity (c: ValidCircuit P P_Prime) : ZMod P := c.mult_gen ^ c.T");
    println!("def delta (c: ValidCircuit P P_Prime) : ZMod P := c.mult_gen ^ (2^c.S)");
    println!("end ValidCircuit");

    println!("def is_shuffle (c: ValidCircuit P P_Prime) (shuffle: ℕ → ℕ): Prop :=");
    println!("  ∃ inv: ℕ → ℕ,");
    println!("  ∀ row: ℕ,");
    println!("    inv (shuffle row) = row ∧");
    println!("    (row ≥ c.usable_rows → shuffle row = row)");

    println!("def sufficient_rows (c: ValidCircuit P P_Prime) : Prop :=");
    println!("  c.n ≥ {} --cs.minimum_rows", cs.minimum_rows());

    println!("--End preamble");
}

pub fn print_postamble(name: &str, cs: &ConstraintSystem<TermField>) {
    let usable_rows = str::parse::<usize>(&fs::read_to_string("./usable_rows").expect("Failed to read usable_rows")).expect("Failed to parse usable_rows");

    println!("def meets_constraints (c: ValidCircuit P P_Prime): Prop :=");
    println!("  sufficient_rows c ∧");
    println!("  c.1.num_blinding_factors = {} ∧", cs.blinding_factors());
    println!("  c.1.Selector = selector_func c ∧");
    println!("  c.1.Fixed = fixed_func c ∧");
    println!("  c.1.AdvicePhase = advice_phase c ∧");
    println!("  c.usable_rows ≥ {usable_rows} ∧");
    println!("  all_gates c ∧");
    println!("  all_copy_constraints c ∧");
    println!("  all_lookups c ∧");
    println!("  all_shuffles c ∧");
    println!("  ∀ col row: ℕ, (row < c.n ∧ row ≥ c.usable_rows) → c.1.Instance col row = c.1.InstanceUnassigned col row");
    println!("end {name}");
}

pub fn expression_to_value_string(expr: &Expression<TermField>, row_name: &str) -> String {
    let format_lookup = |identifier, column, rotation: i32| {
        if rotation == 0 {
            format!("{} {} {row_name}", identifier, column)
        } else if rotation > 0 {
            format!("{} {} (({row_name} + {}) % c.n)", identifier, column, rotation)
        } else {
            format!("{} {} (({row_name} + c.n - ({} % c.n)) % c.n)", identifier, column, -rotation)
        }
    };

    match expr {
        Expression::Constant(value) => format!("({})", value),
        Expression::Selector(selector) => format!("c.get_selector {} {row_name}", selector.0),
        Expression::Fixed(query) => format_lookup("c.get_fixed", query.column_index(), query.rotation().0),
        Expression::Advice(query) => format_lookup("c.get_advice", query.column_index(), query.rotation().0),
        Expression::Instance(query) => format_lookup("c.get_instance", query.column_index(), query.rotation().0),
        Expression::Challenge(challenge) => format!("c.get_challenge {} {}", challenge.index(), challenge.phase()),
        Expression::Negated(expression) => format!("-({})", expression_to_value_string(expression, row_name)),
        Expression::Sum(expression, expression1) =>
            format!("({}) + ({})", expression_to_value_string(expression, row_name), expression_to_value_string(expression1, row_name)),
        Expression::Product(expression, expression1) =>
            format!("({}) * ({})", expression_to_value_string(expression, row_name), expression_to_value_string(expression1, row_name)),
        Expression::Scaled(expression, factor) =>
        format!("({}) * ({})", factor.to_string(), expression_to_value_string(expression, row_name)),
    }
}
