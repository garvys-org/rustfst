use anyhow::Result;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{Fst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_bin_deserializer<W>(test_data: &FstTestData<VectorFst<W>>) -> Result<()>
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

pub fn test_vector_fst_bin_with_symt_deserializer<W>(
    test_data: &FstTestData<VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + 'static,
{
    let mut parsed_fst_bin = VectorFst::<W>::read(&test_data.raw_vector_with_symt_bin_path)?;

    parsed_fst_bin.take_input_symbols();
    parsed_fst_bin.take_output_symbols();

    assert_eq!(
        test_data.raw,
        parsed_fst_bin,
        "{}",
        error_message_fst!(
            test_data.raw,
            parsed_fst_bin,
            "Deserializer VectorFst Bin With symt"
        )
    );
    Ok(())
}
