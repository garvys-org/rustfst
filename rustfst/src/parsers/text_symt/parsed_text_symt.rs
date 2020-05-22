use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;

use crate::parsers::text_symt::nom_parser::parse_text_symt;
use crate::{Label, Symbol};

#[derive(Debug, PartialEq, Default)]
pub(crate) struct ParsedTextSymt {
    pub pairs: Vec<(Symbol, Label)>,
}

impl ParsedTextSymt {
    pub(crate) fn from_string(symt_string: &str) -> Result<Self> {
        let (_, parsed_symt) = parse_text_symt(symt_string)
            .map_err(|_| format_err!("Error while parsing text symt"))?;
        Ok(parsed_symt)
    }

    pub(crate) fn from_path<P: AsRef<Path>>(path_symt_text: P) -> Result<Self> {
        let symt_string = read_to_string(path_symt_text)?;
        Self::from_string(&symt_string)
    }
}
