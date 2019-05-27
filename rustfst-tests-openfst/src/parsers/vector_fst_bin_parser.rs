use failure::Fallible;

use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::BinaryParser;
use rustfst::semirings::Semiring;

use crate::TestData;

pub fn test_vector_fst_bin_parser<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32>,
{
    let parsed_fst_bin = VectorFst::<W>::read(&test_data.raw_vector_bin_path)?;

    assert_eq!(
        test_data.raw,
        parsed_fst_bin,
        "{}",
        error_message_fst!(test_data.raw, parsed_fst_bin, "Parser VectorFst Bin")
    );
    Ok(())
}
