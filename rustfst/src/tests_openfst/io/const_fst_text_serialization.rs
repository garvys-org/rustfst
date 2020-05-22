use anyhow::Result;
use tempfile::tempdir;

use crate::algorithms::fst_convert_from_ref;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{Fst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::io::generate_symbol_table;
use crate::tests_openfst::FstTestData;

pub fn test_const_fst_text_serialization<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring,
{
    let const_fst_ref: ConstFst<_> = test_data.raw.clone().into();

    let dir = tempdir()?;

    let path_fst_serialized = dir.path().join("raw.txt");
    const_fst_ref.write_text(&path_fst_serialized)?;

    let deserialized_fst = ConstFst::<W>::read_text(&path_fst_serialized)?;

    // Assert const fst written and then pared is equal to the orginal one.
    assert_eq!(
        const_fst_ref,
        deserialized_fst,
        "{}",
        error_message_fst!(const_fst_ref, deserialized_fst, "Serializer ConstFst Text")
    );

    let deserialized_fst_vec: VectorFst<_> = fst_convert_from_ref(&deserialized_fst);

    // Same assert but at the VectorFst level
    assert_eq!(
        test_data.raw,
        deserialized_fst_vec,
        "{}",
        error_message_fst!(
            test_data.raw,
            deserialized_fst_vec,
            "Serializer VectorFst Text"
        )
    );
    Ok(())
}

pub fn test_const_fst_text_serialization_with_symt<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring,
{
    let dir = tempdir()?;

    let mut raw_const_with_symt: ConstFst<_> = test_data.raw.clone().into();
    let (input_symt, output_symt) = generate_symbol_table("test", &raw_const_with_symt);
    raw_const_with_symt.set_input_symbols(input_symt);
    raw_const_with_symt.set_output_symbols(output_symt);

    let path_fst_serialized = dir.path().join("raw_const.fst");
    raw_const_with_symt.write_text(&path_fst_serialized)?;

    let deserialized_fst = ConstFst::<W>::read_text(&path_fst_serialized)?;

    // Text serialization doesn't include the symbol table.
    let mut raw_const_without_symt = raw_const_with_symt;
    raw_const_without_symt.take_input_symbols();
    raw_const_without_symt.take_output_symbols();

    assert_eq!(
        raw_const_without_symt,
        deserialized_fst,
        "{}",
        error_message_fst!(
            raw_const_without_symt,
            deserialized_fst,
            "Serializer ConstFst Text with Generated Symbol Table"
        )
    );
    Ok(())
}
