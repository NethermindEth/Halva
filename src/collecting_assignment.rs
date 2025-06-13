use std::collections::BTreeMap;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;

use halo2_frontend::plonk::sealed::SealedPhase;
use halo2_frontend::plonk::{sealed, Phase};
use halo2_proofs::plonk::FirstPhase;

use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    plonk::{Advice, Any, Assigned, Assignment, Column, Fixed, Instance, Selector},
};

use crate::field::TermField;
use crate::utils::{update_column_annotation, update_row_annotation};

pub struct CollectingAssignment<F: Field> {
    _marker: PhantomData<F>,
    pub advice_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    current_region: Option<String>,
    pub copies: Vec<((Column<Any>, usize), (Column<Any>, usize))>,
    pub selectors: BTreeMap<usize, BTreeMap<usize, String>>,
    pub fixed: BTreeMap<usize, BTreeMap<usize, String>>,
    pub fixed_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    pub fixed_fill: BTreeMap<usize, (usize, String)>,
    pub instance_column_annotations: BTreeMap<usize, (Option<String>, BTreeMap<usize, String>)>,
    pub current_phase: sealed::Phase,
    usable_rows_filename: String,
    comment_prefix: String,
}

impl<F: Field> Drop for CollectingAssignment<F> {
    fn drop(&mut self) {
        std::fs::remove_file(&self.usable_rows_filename).expect("Failed to delete usable_rows file. Feel free to manually delete");
    }
}

impl CollectingAssignment<TermField> {
    pub fn new(comment_prefix: String) -> Self {
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
            comment_prefix
        }
    }

    fn in_phase<P: Phase>(&self, phase: P) -> bool {
        self.current_phase == phase.to_sealed()
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

    pub fn get_usable_rows(&self) -> usize {
        str::parse::<usize>(
            &fs::read_to_string(&self.usable_rows_filename)
                .expect("Failed to read usable_rows")
        ).expect("Failed to parse usable_rows")
    }
}

impl Assignment<TermField> for CollectingAssignment<TermField>
{
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        let x: String = name_fn().into();
        println!("\n{} Entered region: {x}", self.comment_prefix);
        self.current_region = Some(x.clone());
    }

    fn exit_region(&mut self) {
        println!("{} Exited region: {}", self.comment_prefix, self.current_region.as_ref().unwrap());
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
            println!(
                "{} WARNING: Attempted to assign selector {} {} outside or first phase",
                self.comment_prefix,
                selector.index(),
                row
            );
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

        Ok(Value::known(TermField::create_instance(column.index(), row)))
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
        Value::known(TermField::create_challenge(challenge))
    }
}
