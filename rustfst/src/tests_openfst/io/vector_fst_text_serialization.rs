use anyhow::Result;
use tempfile::tempdir;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{Fst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::io::generate_symbol_table;
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_text_serialization<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let dir = tempdir()?;

    let path_fst_serialized = dir.path().join("raw.txt");
    test_data.raw.write_text(&path_fst_serialized)?;

    let deserialized_fst = VectorFst::<W>::read_text(&path_fst_serialized)?;

    test_eq_fst(
        &test_data.raw,
        &deserialized_fst,
        "Serializer VectorFst Text",
    );

    Ok(())
}

pub fn test_vector_fst_text_serialization_with_symt<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let dir = tempdir()?;

    let path_fst_serialized = dir.path().join("raw.fst");
    let mut raw_with_symt = test_data.raw.clone();
    let (input_symt, output_symt) = generate_symbol_table("test", &raw_with_symt);
    raw_with_symt.set_input_symbols(input_symt);
    raw_with_symt.set_output_symbols(output_symt);

    raw_with_symt.write_text(&path_fst_serialized)?;

    let deserialized_fst = VectorFst::<W>::read_text(&path_fst_serialized)?;

    // Text serialization doesn't include the symbol table.
    let mut raw_without_symt = raw_with_symt;
    raw_without_symt.take_input_symbols();
    raw_without_symt.take_output_symbols();

    test_eq_fst(
        &raw_without_symt,
        &deserialized_fst,
        "Serializer VectorFst Text with Generated Symbol Table",
    );

    Ok(())
}
