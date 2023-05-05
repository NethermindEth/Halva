use std::convert::TryFrom;

use halo2_proofs::plonk::Any;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug)]
pub struct ExtractorError;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Halo2Column {
    pub(crate) index: usize,
    pub(crate) column_type: Any,
}

impl Halo2Column {
    pub fn new(index: usize, column_type: Any) -> Self {
        Halo2Column { index, column_type }
    }
}

impl TryFrom<&str> for Halo2Column {
    type Error = ExtractorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re =
            Regex::new(r"Column\s\{\sindex:\s(\d+),\scolumn_type:\s(Advice|Instance|Fixed)\s\}")
                .map_err(|_| ExtractorError)?;

        for cap in re.captures_iter(value) {
            if cap.len() > 2 {
                return Ok(Halo2Column::new(
                    usize::from_str(&cap[1]).map_err(|_| ExtractorError)?,
                    match &cap[2] {
                        "Advice" => Any::Advice,
                        "Instance" => Any::Instance,
                        "Fixed" => Any::Fixed,
                        _ => panic!("Unknown column type \"{}\"", &cap[2]),
                    },
                ));
            } else {
                return Err(ExtractorError);
            }
        }

        Err(ExtractorError)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Halo2Cell {
    /// Identifies the region in which this cell resides.
    pub(crate) region_index: usize,
    /// The relative offset of this cell within its region.
    pub(crate) row_offset: usize,
    /// The column of this cell.
    pub(crate) column: Halo2Column,
}

impl Halo2Cell {
    pub(crate) fn new(region_index: usize, row_offset: usize, column: Halo2Column) -> Self {
        Halo2Cell {
            region_index,
            row_offset,
            column,
        }
    }
}

impl TryFrom<&str> for Halo2Cell {
    type Error = ExtractorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re = Regex::new(
            r"Cell\s\{\sregion_index:\sRegionIndex\((\d+)\),\srow_offset:\s(\d+),\scolumn:\s(.*)\s\}",
        )
        .map_err(|_| ExtractorError)?;

        for cap in re.captures_iter(&value) {
            if cap.len() > 3 {
                return Ok(Halo2Cell::new(
                    usize::from_str(&cap[1]).map_err(|_| ExtractorError)?,
                    usize::from_str(&cap[2]).map_err(|_| ExtractorError)?,
                    Halo2Column::try_from(&cap[3])?,
                ));
            } else {
                return Err(ExtractorError);
            }
        }

        Err(ExtractorError)
    }
}
