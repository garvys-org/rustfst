use anyhow::Result;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{Fst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_bin_deserializer<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let parsed_fst_bin = VectorFst::<W>::read(&test_data.raw_vector_bin_path)?;

    test_eq_fst(
        &test_data.raw,
        &parsed_fst_bin,
        "Deserializer VectorFst Bin",
    );
    Ok(())
}

pub fn test_vector_fst_bin_with_symt_deserializer<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let mut parsed_fst_bin = VectorFst::<W>::read(&test_data.raw_vector_with_symt_bin_path)?;

    parsed_fst_bin.take_input_symbols();
    parsed_fst_bin.take_output_symbols();

    test_eq_fst(
        &test_data.raw,
        &parsed_fst_bin,
        "Deserializer VectorFst Bin With symt",
    );
    Ok(())
}
