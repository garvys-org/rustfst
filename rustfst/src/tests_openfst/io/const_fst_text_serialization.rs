use failure::Fallible;
use tempfile::tempdir;

use crate::algorithms::fst_convert;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::SerializableFst;
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_const_fst_text_serialization<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
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

    let deserialized_fst_vec: VectorFst<_> = fst_convert(&deserialized_fst);

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
