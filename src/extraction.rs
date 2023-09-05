use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::fmt::Display;
use std::marker::PhantomData;

use itertools::Itertools;

use halo2_proofs::dev::CircuitGates;
use halo2_proofs::plonk::ColumnType;
use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    plonk::{Advice, Any, Assigned, Assignment, Column, Fixed, Instance, Selector},
};
use regex::Regex;

use crate::utils::{Halo2Column, Halo2Selector};

pub enum Target {
    Constraints,
    AdviceGenerator,
}

pub struct ExtractingAssignment<F: Field> {
    _marker: PhantomData<F>,
    // current_region: Option<String>,
    target: Target,
    copy_count: u32,
    fixed_count: u32,
    selectors: BTreeMap<usize, BTreeSet<usize>>,
    advice: BTreeMap<usize, BTreeMap<usize, String>>,
    fixed: BTreeMap<usize, BTreeMap<usize, String>>,
}

impl<F: Field> ExtractingAssignment<F> {
    pub fn new(target: Target) -> Self {
        Self {
            _marker: PhantomData,
            // current_region: None,
            target: target,
            copy_count: 0,
            fixed_count: 0,
            selectors: BTreeMap::new(),
            advice: BTreeMap::new(),
            fixed: BTreeMap::new(),
        }
    }

    fn format_cell<T>(col: Column<T>) -> String
    where
        T: ColumnType,
    {
        let parsed_column: Halo2Column =
            Halo2Column::try_from(format!("{:?}", col).as_str()).unwrap();
        format!("{:?} {}", parsed_column.column_type, parsed_column.index)
    }

    // fn lemma_name<T>(col: Column<T>, row: usize) -> String
    // where
    //     T: ColumnType,
    // {
    //     let parsed_column = Halo2Column::try_from(format!("{:?}", col).as_str()).unwrap();
    //     let column_type = format!("{:?}", parsed_column.column_type).to_lowercase();
    //     let column_idx = parsed_column.index;
    //     format!("{column_type}_{column_idx}_{row}")
    // }

    // May need other column types adding
    fn add_lean_scoping(evaluated_expr: String) -> String {
        let s = evaluated_expr
            .replace(" Instance", " c.Instance")
            .replace("(Instance", "(c.Instance");
        if s.starts_with("Instance ") {
            format!("c.{s}")
        } else {
            s
        }
    }

    // fn print_annotation(annotation: String) {
    //     if !annotation.is_empty() {
    //         println!("--Annotation: {}", annotation);
    //     }
    // }

    pub fn print_grouping_props(&self) {
        let copy_constraints_body = if self.copy_count == 0 {
            "true".to_string()
        } else {
            (0..self.copy_count)
                .map(|val| format!("copy_{val} c"))
                .join(" ∧ ")
        };

        println!("def all_copy_constraints: Prop := {copy_constraints_body}");

        let fixed_body = if self.fixed_count == 0 {
            "true".to_string()
        } else {
            (0..self.fixed_count)
                .map(|val| format!("fixed_{val} c"))
                .join(" ∧ ")
        };

        println!("def all_fixed: Prop := {fixed_body}");

        println!("def selector_func : ℕ → ℕ → ZMod P :=");
        println!("  λ col row => match col, row with");
        for (col, row_set) in &self.selectors {
            for row in row_set {
                println!("    | {col}, {row} => 1");
            }
        }
        println!("    | _, _ => 0");

        println!("def advice_func : ℕ → ℕ → ZMod P :=");
        println!("  λ col row => match col, row with");
        for (col, row_set) in &self.advice {
            for (row, value) in row_set {
                if value != "0" {
                    println!("    | {col}, {row} => {value}");
                }
            }
        }
        println!("    | _, _ => 0");

        println!("def fixed_func : ℕ → ℕ → ZMod P :=");
        println!("  λ col row => match col, row with");
        for (col, row_set) in &self.fixed {
            for (row, value) in row_set {
                if value != "0" {
                    println!("    | {col}, {row} => {value}");
                }
            }
        }
        println!("    | _, _ => 0");
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

    fn set_advice(&mut self, col: usize, row: usize, val: String) {
        let s = self.advice.get_mut(&col);
        if let Some(m) = s {
            m.insert(row, val);
        } else {
            let mut new_map = BTreeMap::new();
            new_map.insert(row, val);
            self.advice.insert(col, new_map);
        };
    }

    fn set_fixed(&mut self, col: usize, row: usize, val: String) {
        let s = self.fixed.get_mut(&col);
        if let Some(m) = s {
            m.insert(row, val);
        } else {
            let mut new_map = BTreeMap::new();
            new_map.insert(row, val);
            self.fixed.insert(col, new_map);
        };
    }
}

impl<F: Field + From<String>> Assignment<F> for ExtractingAssignment<F>
where
    F: Display,
{
    fn enter_region<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // let x: String = name_fn().into();
        // println!("--REGION: {x}");
        // self.current_region = Some(x.clone());
    }

    fn exit_region(&mut self) {
        // println!("--EXITED REGION: {}", self.current_region.as_ref().unwrap());
        // self.current_region = None;
    }

    fn enable_selector<A, AR>(
        &mut self,
        _annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), halo2_proofs::plonk::Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        let halo2_selector = Halo2Selector::try_from(format!("{:?}", selector).as_str()).unwrap();
        self.set_selector(halo2_selector.0, row);
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
        _annotation: A,
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
                // Self::print_annotation(annotation().into());
                to().map(|v| {
                    let halo2_column = Halo2Column::try_from(format!("{:?}", column).as_str()).unwrap();
                    self.set_advice(halo2_column.index, row, Self::add_lean_scoping(v.into().evaluate().to_string()));
                });
                Ok(())
            }
        }
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _annotation: A,
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
        // Self::print_annotation(annotation().into());
        to().map(|v| {
            // println!(
            //     "Assign fixed: {} row: {} = {}",
            //     Self::format_cell(column),
            //     row,
            //     v.into().evaluate()
            // );
            // println!(
            //     "def fixed_{}: Prop := c.{} {} = {}",
            //     self.fixed_count,
            //     Self::format_cell(column),
            //     row,
            //     v.into().evaluate()
            // );
            // self.fixed_count += 1;
            let halo2_column = Halo2Column::try_from(format!("{:?}", column).as_str()).unwrap();
            self.set_fixed(halo2_column.index, row, Self::add_lean_scoping(v.into().evaluate().to_string()));
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
        // println!(
        //     "Copy: {} row: {} = {} row: {}",
        //     Self::format_cell(left_column),
        //     left_row,
        //     Self::format_cell(right_column),
        //     right_row
        // );
        println!(
            "def copy_{}: Prop := c.{} {} = c.{} {}",
            self.copy_count,
            Self::format_cell(left_column),
            left_row,
            Self::format_cell(right_column),
            right_row
        );
        self.copy_count += 1;
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

    // privacy-scaling-solutions
    // fn annotate_column<A, AR>(&mut self, _annotation: A, _column: Column<Any>)
    // where
    //     A: FnOnce() -> AR,
    //     AR: Into<String>,
    // {
    //     println!("--Annotate column");
    // }

    // fn get_challenge(&self, _challenge: halo2_proofs::plonk::Challenge) -> Value<F> {
    //     println!("--Get challenge");
    //     Value::unknown()
    // }
}

pub fn print_gates(gates: CircuitGates) {
    println!("------GATES-------");
    let selector_regex = Regex::new(r"S(?P<column>\d+)").unwrap();
    let cell_ref_regex = Regex::new(r"(?P<type>[AIF])(?P<column>\d+)@(?P<row>\d+)").unwrap();
    let gate_string = gates.to_string();
    let gate_strings = gate_string
        .lines()
        .filter(|x| !x.contains(':'))
        .enumerate()
        .collect_vec();
    gate_strings.iter().for_each(|(idx, gate)| {
            // println!("{gate}");
            let s = cell_ref_regex
                .replace_all(
                    selector_regex
                        .replace_all(gate, "c.Selector $column row")
                        .as_ref(),
                    "$type $column (row + $row)",
                )
                .as_ref()
                .replace("A", "c.Advice ")
                .replace("I", "c.Instance ")
                .replace("F", "c.Fixed ")
                .replace("@", " ")
                .replace(" + 0", "");
            println!(
                // "def gate_{idx}: Prop := {}",
                "def gate_{idx}: Prop := ∀ row : ℕ, {} = 0",
                if s.starts_with('-') {
                    s.strip_prefix('-').unwrap()
                } else {
                    &s
                }
            );
        });
    let all_gates_body = if gate_strings.len() == 0 {
        "true".to_string()
    } else {
        (0..gate_strings.len())
            .map(|val| format!("gate_{val} c"))
            .join(" ∧ ")
    };

    println!("def all_gates: Prop := {all_gates_body}");
}

pub fn print_preamble(name: &str) {
    println!("import Mathlib.Data.Nat.Prime");
    println!("import Mathlib.Data.ZMod.Defs");
    println!("import Mathlib.Data.ZMod.Basic\n");

    println!("namespace {name}\n");

    println!("structure Circuit (P: ℕ) (P_Prime: Nat.Prime P) :=");
    println!("  Advice: ℕ → ℕ → ZMod P");
    println!("  Fixed: ℕ → ℕ → ZMod P");
    println!("  Instance: ℕ → ℕ → ZMod P");
    println!("  Selector: ℕ → ℕ → ZMod P");
}

pub fn print_postamble(name: &str) {
    println!("def meets_constraints: Prop := c.Selector = selector_func ∧ all_gates c ∧ all_copy_constraints c ∧ c.Fixed = fixed_func");
    println!("end {name}");
}

#[macro_export]
macro_rules! extract {
    ($a:ident, $b:expr) => {
        use halo2_extr::extraction::{print_gates, ExtractingAssignment};
        use halo2_extr::field::TermField;
        use halo2_proofs::dev::CircuitGates;
        // privacy-scaling-solutions
        // use halo2_proofs::halo2curves::pasta::Fp;
        // zcash
        use group::ff::Field;
        use halo2_proofs::plonk::{Circuit, ConstraintSystem, FloorPlanner};
        let circuit: $a<TermField> = $a::default();

        let mut cs = ConstraintSystem::<TermField>::default();
        let config = $a::<TermField>::configure(&mut cs, [
            Column::new(0, Advice),
            Column::new(1, Advice),
            Column::new(2, Advice),
            Column::new(3, Advice),
            Column::new(4, Advice),
        ]);

        // println!("variable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

        // let mut extr_assn = ExtractingAssignment::<TermField>::new($b);
        // <$a<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
        //     &mut extr_assn,
        //     &circuit,
        //     config,
        //     vec![],
        // )
        // .unwrap();

        // extr_assn.print_grouping_props();
        // privacy-scaling-solutions
        // print_gates(CircuitGates::collect::<Fp, $a<Fp>>());
        // zcash
        // print_gates(CircuitGates::collect::<TermField, $a<TermField>>());
    };
    ($a:ident, $b:expr, $c:expr) => {
        use halo2_extr::extraction::{print_gates, ExtractingAssignment};
        use halo2_extr::field::TermField;
        use halo2_proofs::dev::CircuitGates;
        use halo2_proofs::halo2curves::bn256::Fq;
        use halo2_proofs::plonk::{Circuit, ConstraintSystem, FloorPlanner};
        let circuit: $a<TermField> = $c;

        let mut cs = ConstraintSystem::<TermField>::default();
        let config = $a::<TermField>::configure(&mut cs);

        println!("\nvariable {{P: ℕ}} {{P_Prime: Nat.Prime P}} (c: Circuit P P_Prime)");

        let mut extr_assn = ExtractingAssignment::<TermField>::new($b);
        <$a<TermField> as Circuit<TermField>>::FloorPlanner::synthesize(
            &mut extr_assn,
            &circuit,
            config,
            vec![],
        )
        .unwrap();

        extr_assn.print_grouping_props();
        print_gates(CircuitGates::collect::<Fq, $a<Fq>>(<$a<Fq> as Circuit<
            Fq,
        >>::Params::default(
        )));
    };
}

#[cfg(test)]
mod tests {}
