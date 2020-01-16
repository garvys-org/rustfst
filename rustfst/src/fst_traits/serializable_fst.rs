use std::path::Path;

use failure::Fallible;

use crate::fst_traits::ExpandedFst;
use crate::parsers::text_fst::ParsedTextFst;
use crate::semirings::SerializableSemiring;

pub trait SerializableFst: ExpandedFst
where
    Self::W: SerializableSemiring,
{
    // BINARY

    fn read<P: AsRef<Path>>(path_bin_fst: P) -> Fallible<Self>;
    fn write<P: AsRef<Path>>(&self, path_bin_fst: P) -> Fallible<()>;

    // TEXT

    /// Turns a generic wFST format into the one of the wFST.
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst<Self::W>) -> Fallible<Self>;

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn from_text_string(fst_string: &str) -> Fallible<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    /// Deserializes a wFST in text from a path and returns a loaded wFST.
    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Fallible<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }
}
