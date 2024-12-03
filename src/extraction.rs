use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

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

const GROUPING_SIZE: usize = 10;

pub struct ExtractingAssignment<F: Field> {
    _marker: PhantomData<F>,
    current_region: Option<String>,
    copies: Vec<((Column<Any>, usize), (Column<Any>, usize))>,
    selectors: BTreeMap<usize, BTreeSet<usize>>,
    fixed: BTreeMap<usize, BTreeMap<usize, String>>,
    fixed_fill: BTreeMap<usize, (usize, String)>,
    current_phase: sealed::Phase,
}

// impl<F: Field + From<String> + Display> ExtractingAssignment<F> {
impl ExtractingAssignment<TermField> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            current_region: None,
            copies: vec![],
            selectors: BTreeMap::new(),
            fixed: BTreeMap::new(),
            fixed_fill: BTreeMap::new(),
            current_phase: FirstPhase.to_sealed(),
        }
    }

    fn in_phase<P: Phase>(&self, phase: P) -> bool {
        self.current_phase == phase.to_sealed()
    }

    fn print_copy_constraints(&self) {
        if self.copies.len() == 0 {
            println!("def all_copy_constraints (_c: ValidCircuit P P_Prime): Prop := true");
        } else {
            for (idx, ((left_column, left_row), (right_column, right_row))) in self.copies.iter().enumerate() {
                let lhs = match left_column.column_type() {
                    Any::Advice => format!("c.1.Advice {} {}", left_column.index(), left_row),
                    Any:: Fixed => format!("c.1.Fixed {} {}", left_column.index(), left_row),
                    Any::Instance => format!("↑(c.1.Instance {} {})", left_column.index(), left_row),
                };
                let rhs = match right_column.column_type() {
                    Any::Advice => format!("c.1.Advice {} {}", right_column.index(), right_row),
                    Any:: Fixed => format!("c.1.Fixed {} {}", right_column.index(), right_row),
                    Any::Instance => format!("↑(c.1.Instance {} {})", right_column.index(), right_row),
                };
                println!("def copy_{idx} (c: ValidCircuit P P_Prime): Prop := {lhs} = {rhs}");
            }

            if GROUPING_SIZE < 2 {
                let copy_constraints_body = 
                    (0..self.copies.len())
                        .map(|val| format!("copy_{val} c"))
                        .join(" ∧ ");
    
                println!("def all_copy_constraints (c: ValidCircuit P P_Prime): Prop :=");
                println!("  {copy_constraints_body}");    
            } else {
                // Group the props into groups of size GROUPING_SIZE,
                // repeating recursively until all are accounted for
                let depth = self.copies.len().ilog(GROUPING_SIZE) + 1;
                for power in 1..depth {
                    let size = GROUPING_SIZE.pow(power);
                    let subgroup_size = GROUPING_SIZE.pow(power-1);
                    let mut start = 0;
                    let mut end = std::cmp::min(self.copies.len(), start + size);
                    while start < end - 1 {
                        println!("def copy_{start}_to_{} (c: ValidCircuit P P_Prime) : Prop :=", end-1);
                        let mut body = vec![];
                        let mut subgroup_start = start;
                        while subgroup_start < end {
                            let subgroup_end = std::cmp::min(end, subgroup_start + subgroup_size);
                            if subgroup_end == subgroup_start + 1 {
                                body.push(format!("copy_{subgroup_start} c"));
                            } else {
                                body.push(format!("copy_{subgroup_start}_to_{} c", subgroup_end-1));
                            }
                            subgroup_start += subgroup_size;
                        }
                        println!("  {}", body.join(" ∧ "));
    
                        start = std::cmp::min(self.copies.len(), start + size);
                        end = std::cmp::min(self.copies.len(), end + size);
                    }
                }
                println!("def all_copy_constraints (c: ValidCircuit P P_Prime): Prop :=");
    
                let mut body = vec![];
                let mut start = 0;
                let size = GROUPING_SIZE.pow(depth-1);
                while start < self.copies.len() {
                    let end = std::cmp::min(self.copies.len(), start + size);
                    if end == start + 1 {
                        body.push(format!("copy_{start} c"));
                    } else {
                        body.push(format!("copy_{start}_to_{} c", end-1));
                    }
                    start += size;
                }
                println!("  {}", body.join(" ∧ "));
            }
        } 
    }

    // TODO grouping
    fn print_selectors(&self) {
        for (col, row_set) in &self.selectors {
            if let Some(&start) = row_set.first() {
                let runs = {
                    let mut start = start;
                    // End is inclusive
                    let mut end = start;
                    let mut runs = vec![];
    
                    // Iterate through the true rows, to collect the consecutive runs
                    for &i in row_set.iter().skip(1) {
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
            if row_set.is_empty() {
                println!("def fixed_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → CellValue P :=");
                println!("  λ row => .Unassigned");
            } else if row_set.len() <= GROUPING_SIZE {
                println!("def fixed_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → CellValue P :=");
                println!("  λ row =>");
                let body = row_set
                    .iter()
                    .map(|(row, val)| {
                        format!("if row = {row} then .Assigned {val}")
                    })
                    .join("\n  else ");
                println!("  {body}");
                if let Some ((fill_row, fill_val)) = self.fixed_fill.get(col) {
                    println!("  else if row ≥ {fill_row} then .Assigned {fill_val}");
                }
                println!("  else .Unassigned");
            } else {
                let rows = row_set.iter().collect_vec();

                let mut i = 0;
                let mut boundaries = vec![];
                let mut lower_bound = 0;
                while i < rows.len() {
                    let end_index = std::cmp::min(i + GROUPING_SIZE, rows.len());
                    let end_row = rows[end_index - 1].0;
                    println!("def fixed_func_col_{col}_{}_to_{} (c: ValidCircuit P P_Prime) : ℕ → CellValue P :=", lower_bound, *end_row);
                    println!("  λ row =>");
                    let body = (i..end_index)
                        .map(|idx| {
                            format!("if row = {} then .Assigned {}", rows[idx].0, rows[idx].1)
                        })
                        .join("\n  else ");
                    println!("  {body}");
                    println!("  else .Unassigned");
                    boundaries.push(*end_row);
                    lower_bound = end_row + 1;
                    i += GROUPING_SIZE;
                }

                while boundaries.len() > GROUPING_SIZE {
                    let mut new_boundaries = vec![];
                    let mut i = 0;
                    let mut lower_bound: usize = 0;
                    loop {
                        let slice = boundaries.iter().skip(i).take(GROUPING_SIZE).collect_vec();
                        if slice.is_empty() {
                            break;
                        } else if slice.len() == 1 {
                            new_boundaries.push(*slice[0]);
                            break;
                        } else {
                            i += GROUPING_SIZE;
                        }

                        println!("def fixed_func_col_{col}_{lower_bound}_to_{} (c: ValidCircuit P P_Prime) : ℕ → CellValue P :=", slice[slice.len()-1]);
                        println!("  λ row =>");

                        let mut body = vec![];
                        for upper_bound in slice.iter() {
                            body.push(format!("if row ≤ {upper_bound} then fixed_func_col_{col}_{lower_bound}_to_{upper_bound} c row"));
                            lower_bound = **upper_bound + 1;
                        }

                        println!("  {}", body.join("\n  else "));
                        println!("  else .Unassigned");
                        
                        new_boundaries.push(*slice[slice.len()-1]);
                        lower_bound = slice[slice.len()-1] + 1;
                    }
                    boundaries = new_boundaries;
                }

                println!("def fixed_func_col_{col} (c: ValidCircuit P P_Prime) : ℕ → CellValue P :=");
                println!("  λ row =>");
                let mut body = vec![];
                if let Some ((fill_row, fill_val)) = self.fixed_fill.get(col) {
                    body.push(format!("if row ≥ {fill_row} then .Assigned {fill_val}"));
                }
                let mut lower_bound = 0;
                for bound in boundaries {
                    body.push(format!("if row ≤ {bound} then fixed_func_col_{col}_{lower_bound}_to_{bound} c row"));
                    lower_bound = bound + 1;
                }
                println!("  {}", body.join("\n  else "));
                println!("  else .Unassigned");
            }
        }

        println!("def fixed_func (c: ValidCircuit P P_Prime) : ℕ → ℕ → CellValue P :=");
        if self.fixed.keys().len() == 0 {
            println!("  λ col _ => match col with");
        } else {
            println!("  λ col row => match col with");
        }
        for col in self.fixed.keys() {
            println!("    | {col} => fixed_func_col_{col} c row");
        }
        println!("    | _ => .Unassigned");
    }

    pub fn print_grouping_props(&self, cs: &ConstraintSystem<TermField>) {
        println!("");
        println!("");
        self.print_copy_constraints();
        self.print_selectors();
        self.print_fixed();

        cs
            .gates()
            .iter()
            .enumerate()
            .for_each(|(gate_idx, gate)| {
                gate.polynomials()
                    .iter()
                    .enumerate()
                    .for_each(|(idx, constraint)| {
                        println!("def gate_{}_{}_{} (c: ValidCircuit P P_Prime) (row: ℕ) : Prop := ", gate_idx, idx, gate.constraint_name(idx).replace("_", "__").replace(" ", "_"));
                        println!("  {} = Value.Real 0", expression_to_value_string(constraint, "row"))
                    });
            });
        
        println!("def all_gates (c: ValidCircuit P P_Prime): Prop := ∀ row: ℕ,");
        let all_gates_body = if cs.gates().is_empty() {
            "true".to_string()
        } else {
            cs
                .gates()
                .iter()
                .enumerate()
                .flat_map(|(gate_idx, gate)| {
                    gate.polynomials()
                        .iter()
                        .enumerate()
                        .map(move |(idx, _)| {
                            format!("  gate_{}_{}_{} c row", gate_idx, idx, gate.constraint_name(idx).replace("_", "__").replace(" ", "_"))
                        })
                })
                .join(" ∧ \n")
        };
        println!("  {}", all_gates_body);

        // Lookups
        {
            let lookups = cs.lookups();
            let mut propnames = vec![];
    
            // Individual lookups
            for lookup in lookups.iter() {
                println!("--Lookup argument: {}", lookup.name());
                let processed_name = format!("lookup_{}", lookup.name().replace("_", "__").replace(" ", "_"));
                propnames.push(processed_name.clone());
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
                println!("def {processed_name} (c: ValidCircuit P P_Prime) : Prop := ∀ row : ℕ, row < c.usable_rows → ∃ lookup_row : ℕ, lookup_row < c.usable_rows ∧ ({lhs}) = ({rhs})");
            }
    
            // Group prop
            {
                let props = if propnames.is_empty() {
                    "true".to_string()
                } else {
                    propnames
                        .iter()
                        .map(|prop_name| format!("{prop_name} c"))
                        .join(" ∧ ")
                };
    
                println!("def all_lookups (c: ValidCircuit P P_Prime): Prop := {props}");
            }
        }

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

    fn set_selector(&mut self, col: usize, row: usize) {
        let s = self.selectors.get_mut(&col);
        if let Some(v) = s {
            v.insert(row);
        } else {
            let mut new_set = BTreeSet::new();
            new_set.insert(row);
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
        print_preamble(namespace, symbol_names);

        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
        let cs = cs;

        println!("def sufficient_rows (c: ValidCircuit P P_Prime) : Prop :=");
        println!("  c.n ≥ {} --cs.minimum_rows", cs.minimum_rows());

        let mut prover = ExtractingAssignment::new();

        println!("def assertions (c: ValidCircuit P P_Prime): Prop :=");
        println!("  true");

        // TODO phases
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
        _annotation: A,
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

        println!("  ∧ {row} < c.usable_rows -- Selector {} enabled", selector.index());

        self.set_selector(selector.index(), row);
        Ok(())
    }

    fn query_instance(
        &self,
        column: Column<Instance>,
        row: usize,
    ) -> Result<Value<TermField>, halo2_frontend::plonk::Error> {
        println!("  ∧ {row} < c.usable_rows -- Instance {} query", column.index());

        Ok(Value::known(TermField::from(format!(
            "instance_to_field (c.1.Instance {} {})",
            column.index(),
            row
        ))))
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _annotation: A,
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
            println!("  ∧ {row} < c.usable_rows -- Advice {} assignment", column.index());
        }

        // Aside from the above range assertion,
        // we ignore advice assignment as we are concerned only with constraint generation
        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _annotation: A,
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

        println!("  ∧ {row} < c.usable_rows -- Fixed {} assignment", column.index());

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

        println!("  ∧ {left_row} < c.usable_rows ∧ {right_row} < c.usable_rows -- Copy {} to {} assignment", left_column.index(), right_column.index());

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

        println!(" ∧ {row} < c.usable_rows -- Fixed {} fill from row", column.index());

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

    fn annotate_column<A, AR>(&mut self, _annotation: A, _column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        println!("--Annotate column");
    }

    fn get_challenge(&self, challenge: halo2_proofs::plonk::Challenge) -> Value<TermField> {
        Value::known(TermField::from(format!("c.get_challenge {} {}", challenge.index(), challenge.phase())))
    }
}

pub fn print_preamble(namespace: &str, symbol_names: &[&str]) {
    println!("import Mathlib.Data.Nat.Prime.Defs");
    println!("import Mathlib.Data.Nat.Prime.Basic");
    println!("import Mathlib.Data.ZMod.Defs");
    println!("import Mathlib.Data.ZMod.Basic\n");

    println!("namespace {namespace}\n");

    println!("inductive Value (P: ℕ) where");
    println!("  | Real (x: ZMod P)");
    println!("  | Poison");
    
    println!("inductive CellValue (P: ℕ) where");
    println!("  | Assigned (x: ZMod P)");
    println!("  | Unassigned");
    println!("  | Poison (row: ℕ)");

    println!("inductive InstanceValue (P: ℕ) where");
    println!("  | Assigned (x: ZMod P)");
    println!("  | Padding");
    
    println!("def S_T_from_P (S T P : ℕ) : Prop :=");
    println!("  (2^S * T = P - 1) ∧");
    println!("  (∀ s' t': ℕ, 2^s' * t' = P - 1 → s' ≤ S)");
    
    println!("def multiplicative_generator (P: ℕ) (mult_gen: ZMod P) : Prop :=");
    println!("  mult_gen ^ P = 1");
    
    println!("structure Circuit (P: ℕ) (P_Prime: Nat.Prime P) :=");
    println!("  Advice: ℕ → ℕ → CellValue P");
    println!("  Fixed: ℕ → ℕ → CellValue P");
    println!("  Instance: ℕ → ℕ → InstanceValue P");
    println!("  Selector: ℕ → ℕ → ZMod P");
    println!("  Challenges: ℕ → ℕ → ZMod P");
    println!("  num_blinding_factors: ℕ");
    println!("  S: ℕ");
    println!("  T: ℕ");
    println!("  k: ℕ");
    println!("  mult_gen: ZMod P");
    for symbol_name in symbol_names {
        println!("  sym_{symbol_name}: ZMod P")
    }
    
    println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}}");
    println!("def eval_cell (cell : CellValue P) : Value P :=");
    println!("  match cell with");
    println!("  | .Assigned (x : ZMod P) => .Real x");
    println!("  | .Unassigned            => .Real 0");
    println!("  | .Poison (_ : ℕ)      => .Poison");
    println!("");
    println!("def instance_to_field (cell : InstanceValue P) : ZMod P :=");
    println!("  match cell with");
    println!("  | .Assigned x => x");
    println!("  | .Padding    => 0");
    println!("");
    println!("def eval_instance (inst : InstanceValue P) : Value P :=");
    println!("  .Real (instance_to_field inst)");
    println!("");
    println!("def cell_of_inst (inst : InstanceValue P) : CellValue P :=");
    println!("  .Assigned (instance_to_field inst)");
    println!("");
    println!("instance {{P : ℕ}} : Coe (InstanceValue P) (CellValue P) where");
    println!("  coe := cell_of_inst");
    println!("");
    println!("instance : Neg (Value P) := ⟨λ x ↦");
    println!("  match x with");
    println!("  | .Real x => .Real (-x)");
    println!("  | .Poison => .Poison");
    println!("⟩");
    println!("");
    println!("instance : Add (Value P) := ⟨λ x y ↦");
    println!("  match x, y with");
    println!("  | .Real x₁, .Real x₂ => .Real (x₁ + x₂)");
    println!("  | .Poison, .Poison   => .Poison");
    println!("  | .Poison, .Real _   => .Poison");
    println!("  | .Real _, .Poison   => .Poison");
    println!("⟩");
    println!("");
    println!("-- Should this handle (x - x)?");
    println!("instance : Sub (Value P) := ⟨λ x y ↦ x + (-y)⟩");
    println!("");
    println!("instance : Mul (Value P) := ⟨λ x y ↦");
    println!("  match x, y with");
    println!("  | .Real x₁, .Real x₂ => .Real (x₁ * x₂)");
    println!("  | .Poison, .Poison   => .Poison");
    println!("  | .Poison, .Real x₁  => if x₁ = 0 then .Real 0 else .Poison");
    println!("  | .Real x₁, .Poison  => if x₁ = 0 then .Real 0 else .Poison");
    println!("⟩");
    println!("def Circuit.isValid (c: Circuit P P_Prime) : Prop :=");
    println!("  S_T_from_P c.S c.T P ∧");
    println!("  multiplicative_generator P c.mult_gen");

    println!("abbrev ValidCircuit (P: ℕ) (P_Prime: Nat.Prime P) : Type := {{c: Circuit P P_Prime // c.isValid}}");
    println!("namespace ValidCircuit");
    println!("def get_advice (c: ValidCircuit P P_Prime) : ℕ → ℕ → Value P :=");
    println!("  λ col row => eval_cell (c.1.Advice col row)");
    println!("def get_fixed (c: ValidCircuit P P_Prime) : ℕ → ℕ → Value P :=");
    println!("  λ col row => eval_cell (c.1.Fixed col row)");
    println!("def get_instance (c: ValidCircuit P P_Prime) : ℕ → ℕ → Value P :=");
    println!("  λ col row => eval_instance (c.1.Instance col row)");
    println!("def get_selector (c: ValidCircuit P P_Prime) : ℕ → ℕ → Value P :=");
    println!("  λ col row => .Real (c.1.Selector col row)");
    println!("def get_challenge (c: ValidCircuit P P_Prime) : ℕ → ℕ → Value P :=");
    println!("  λ idx phase => .Real (c.1.Challenges idx phase)");
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

    println!("--End preamble");
}

pub fn print_postamble(name: &str, cs: &ConstraintSystem<TermField>) {
    println!("def meets_constraints (c: ValidCircuit P P_Prime): Prop :=");
    println!("  sufficient_rows c ∧");
    println!("  c.1.num_blinding_factors = {} ∧", cs.blinding_factors());
    println!("  c.1.Selector = selector_func c ∧");
    println!("  c.1.Fixed = fixed_func c ∧");
    println!("  assertions c  ∧");
    println!("  all_gates c ∧");
    println!("  all_copy_constraints c ∧");
    println!("  all_lookups c ∧");
    println!("  all_shuffles c ∧");
    println!("  ∀ col row: ℕ, (row < c.n ∧ row ≥ c.usable_rows) → c.1.Instance col row = .Padding");
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
        Expression::Constant(value) => format!("(.Real {})", value),
        Expression::Selector(selector) => format!("c.get_selector {} {row_name}", selector.0),
        Expression::Fixed(query) => format_lookup("c.get_fixed", query.column_index(), query.rotation().0),
        Expression::Advice(query) => format_lookup("c.get_advice", query.column_index(), query.rotation().0),
        Expression::Instance(query) => format_lookup("c.get_isntance", query.column_index(), query.rotation().0),
        Expression::Challenge(challenge) => format!("c.get_challenge {}", challenge.index()),
        Expression::Negated(expression) => format!("-({})", expression_to_value_string(expression, row_name)),
        Expression::Sum(expression, expression1) =>
            format!("({}) + ({})", expression_to_value_string(expression, row_name), expression_to_value_string(expression1, row_name)),
        Expression::Product(expression, expression1) =>
            format!("({}) * ({})", expression_to_value_string(expression, row_name), expression_to_value_string(expression1, row_name)),
        Expression::Scaled(expression, factor) =>
        format!("(.Real {}) * ({})", factor.to_string(), expression_to_value_string(expression, row_name)),
    }
}
