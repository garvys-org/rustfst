use std::path::Path;

use crate::fst_traits::ExpandedFst;
use crate::parsers::text::ParsedTextFst;
use crate::semirings::Semiring;
use crate::Result;

pub trait TextParser: ExpandedFst
where
    Self::W: Semiring<Type = f32>,
{
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst) -> Result<Self>;

    fn from_text_string(fst_string: &str) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }
}
