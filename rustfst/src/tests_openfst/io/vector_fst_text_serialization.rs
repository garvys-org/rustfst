use failure::Fallible;
use tempfile::tempdir;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_text_serialization<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
{
    let dir = tempdir()?;

    let path_fst_serialized = dir.path().join("raw.txt");
    test_data.raw.write_text(&path_fst_serialized)?;

    let deserialized_fst = VectorFst::<W>::read_text(&path_fst_serialized)?;

    assert_eq!(
        test_data.raw,
        deserialized_fst,
        "{}",
        error_message_fst!(test_data.raw, deserialized_fst, "Serializer VectorFst Text")
    );
    Ok(())
}
