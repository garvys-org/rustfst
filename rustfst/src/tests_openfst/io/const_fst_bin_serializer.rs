use anyhow::Result;
use tempfile::tempdir;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{Fst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::io::generate_symbol_table;
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_const_fst_bin_serializer<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let dir = tempdir()?;

    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    let path_fst_serialized = dir.path().join("raw_const.fst");
    raw_const.write(&path_fst_serialized)?;

    let deserialized_fst = ConstFst::<W>::read(&path_fst_serialized)?;

    test_eq_fst(&raw_const, &deserialized_fst, "Serializer ConstFst Bin");

    Ok(())
}

pub fn test_const_fst_bin_serializer_with_symt<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let dir = tempdir()?;

    let mut raw_const_with_symt: ConstFst<_> = test_data.raw.clone().into();
    let (input_symt, output_symt) = generate_symbol_table("test", &raw_const_with_symt);
    raw_const_with_symt.set_input_symbols(input_symt);
    raw_const_with_symt.set_output_symbols(output_symt);

    let path_fst_serialized = dir.path().join("raw_const.fst");
    raw_const_with_symt.write(&path_fst_serialized)?;

    let deserialized_fst = ConstFst::<W>::read(&path_fst_serialized)?;

    test_eq_fst(
        &raw_const_with_symt,
        &deserialized_fst,
        "Serializer ConstFst Bin with Generated Symbol Table",
    );

    Ok(())
}
