use std::path::Path;

use crate::fst_traits::ExpandedFst;
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::Semiring;
use crate::Result;

/// Trait to allow serialization and deserialization of a wFST in text format.
pub trait TextParser: ExpandedFst
where
    Self::W: Semiring<Type = f32>,
{
    /// Turns a generic wFST format into the one of the wFST.
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst) -> Result<Self>;

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn from_text_string(fst_string: &str) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }
}
