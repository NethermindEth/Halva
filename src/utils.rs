use std::{convert::TryFrom, error::Error};

use halo2_proofs::{plonk::{Selector, Column, ColumnType}};
use regex::Regex;
use std::str::FromStr;
use itertools::Itertools;

#[derive(Debug)]
pub struct ExtractorError;

#[derive(Clone, Copy, Debug)]
pub enum Halo2Any {
    /// An Advice variant
    Advice,
    /// A Fixed variant
    Fixed,
    /// An Instance variant
    Instance,
}

#[derive(Clone, Copy, Debug)]
pub struct Halo2Column {
    pub index: usize,
    pub column_type: Halo2Any,
}

impl Halo2Column {
    pub fn new(index: usize, column_type: Halo2Any) -> Self {
        Halo2Column { index, column_type }
    }
}

impl<C: ColumnType> TryFrom<&Column<C>> for Halo2Column {
    type Error = ExtractorError;

    fn try_from(column: &Column<C>) -> Result<Self, Self::Error> {
        let re = Regex::new(
            r"Column\s\{\sindex:\s(\d+),\scolumn_type:\s(Advice(?: \{.*\})?|Instance|Fixed)\s\}",
        )
        .map_err(|_| ExtractorError)?;

        let res = if let Some((_, [index, column_type])) = re.captures_iter(format!("{column:?}").as_str()).next().map(|c| c.extract()) {
            Ok(Halo2Column::new(
                usize::from_str(index).map_err(|_| ExtractorError)?,
                match column_type {
                    "Instance" => Halo2Any::Instance,
                    "Fixed" => Halo2Any::Fixed,
                    _ if column_type.starts_with("Advice") => Halo2Any::Advice,
                    _ => panic!("Unknown column type \"{}\"", &column_type)
                },
            ))
        } else {
            Err(ExtractorError)
        };

        res
    }
}

pub struct Halo2Selector(pub usize, pub bool);

// This is required because Assignment is a public trait, yet enable_selector takes a Selector whose row member is pub(crate)
pub fn extract_selector_row(selector: &Selector) -> Result<usize, ExtractorError> {
    let re = Regex::new(r"Selector\((\d+),\s(false|true)\)").map_err(|_| ExtractorError)?;

    let res = if let Some((_, [col, _enabled])) = re.captures_iter(format!("{selector:?}").as_str()).next().map(|c| c.extract()) {
        usize::from_str(col).map_err(|_err| ExtractorError)
    } else {
        Err(ExtractorError)
    };

    res
}
