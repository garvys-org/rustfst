use std::fs::read_to_string;
use std::path::Path;

use nom::types::CompleteStr;

use crate::parsers::text_symt::nom_parser::parse_text_symt;
use crate::Result as ResultRustFst;
use crate::{Label, Symbol};

#[derive(Debug, PartialEq, Default)]
pub(crate) struct ParsedTextSymt {
    pub pairs: Vec<(Symbol, Label)>,
}

impl ParsedTextSymt {
    pub(crate) fn from_string(symt_string: &str) -> ResultRustFst<Self> {
        let complete_symt_str = CompleteStr(symt_string);
        let (_, parsed_symt) = parse_text_symt(complete_symt_str)
            .map_err(|_| format_err!("Error while parsing text symt"))?;
        Ok(parsed_symt)
    }

    pub(crate) fn from_path<P: AsRef<Path>>(path_symt_text: P) -> ResultRustFst<Self> {
        let symt_string = read_to_string(path_symt_text)?;
        Self::from_string(&symt_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_table::SymbolTable;

    #[test]
    fn test_parse_text_symt() -> ResultRustFst<()> {
        let mut symt = SymbolTable::new();
        symt.add_symbol("a");
        symt.add_symbol("b");
        let s = symt.text()?;
        println!("{:?}", ParsedTextSymt::from_string(&s));
        Ok(())
    }

}
