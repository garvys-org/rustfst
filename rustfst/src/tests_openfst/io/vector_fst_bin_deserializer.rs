use failure::Fallible;

use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_bin_deserializer<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
{
    let parsed_fst_bin = VectorFst::<W>::read(&test_data.raw_vector_bin_path)?;

    assert_eq!(
        test_data.raw,
        parsed_fst_bin,
        "{}",
        error_message_fst!(test_data.raw, parsed_fst_bin, "Deserializer VectorFst Bin")
    );
    Ok(())
}
