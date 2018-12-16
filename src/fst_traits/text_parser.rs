use fst_traits::CoreFst;
use fst_traits::ExpandedFst;
use parsers::text::ParsedTextFst;
use semirings::TropicalWeight;
use std::path::Path;
use Result;

pub trait TextParser: ExpandedFst<W = TropicalWeight> {
    fn from_parsed_fst_text(parsed_fst_text: ParsedTextFst) -> Result<Self>;

    fn from_text_string(fst_string: String) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_string(fst_string)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }

    fn read_text<P: AsRef<Path>>(path_text_fst: P) -> Result<Self> {
        let parsed_text_fst = ParsedTextFst::from_path(path_text_fst)?;
        Self::from_parsed_fst_text(parsed_text_fst)
    }
}
