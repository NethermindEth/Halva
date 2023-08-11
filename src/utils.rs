use std::convert::TryFrom;

use regex::Regex;
use std::str::FromStr;

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
pub(crate) struct Halo2Column {
    pub(crate) index: usize,
    pub(crate) column_type: Halo2Any,
}

impl Halo2Column {
    pub fn new(index: usize, column_type: Halo2Any) -> Self {
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
                        "Instance" => Halo2Any::Instance,
                        "Fixed" => Halo2Any::Fixed,
                        "Advice" => Halo2Any::Advice,
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

pub(crate) struct Halo2Selector(pub(crate) usize);

impl TryFrom<&str> for Halo2Selector {
    type Error = ExtractorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re = Regex::new(r"Selector\((\d+),\s(.*)\)").map_err(|_| ExtractorError)?;

        for cap in re.captures_iter(&value) {
            if cap.len() > 2 {
                return Ok(Halo2Selector(
                    usize::from_str(&cap[1]).map_err(|_| ExtractorError)?,
                ));
            } else {
                return Err(ExtractorError);
            }
        }

        Err(ExtractorError)
    }
}
