#[macro_use]
mod macros;
mod text_fst_001;
mod text_fst_002;

use fst_impls::vector::vector_fst::VectorFst;
use parsers::text::ParsedTextFst;
use semirings::ProbabilityWeight;
use std::path::PathBuf;
use std::vec::IntoIter;

#[cfg(test)]
pub(crate) struct TextParserTest {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) parsed_text_fst: ParsedTextFst,
    pub(crate) vector_fst: VectorFst<ProbabilityWeight>,
}

#[cfg(test)]
pub(crate) fn get_test_data_for_text_parser() -> IntoIter<TextParserTest> {
    let mut res = vec![];
    res.push(text_fst_001::text_fst_001());
    res.push(text_fst_002::text_fst_002());
    res.into_iter()
}
