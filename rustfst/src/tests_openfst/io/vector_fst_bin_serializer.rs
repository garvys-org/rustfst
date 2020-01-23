use failure::Fallible;
use tempfile::tempdir;

use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_bin_serializer<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
{
    let dir = tempdir()?;

    let path_fst_serialized = dir.path().join("raw.fst");
    test_data.raw.write(&path_fst_serialized)?;

    let deserialized_fst = VectorFst::<W>::read(&path_fst_serialized)?;

    assert_eq!(
        test_data.raw,
        deserialized_fst,
        "{}",
        error_message_fst!(test_data.raw, deserialized_fst, "Serializer VectorFst Bin")
    );
    Ok(())
}
