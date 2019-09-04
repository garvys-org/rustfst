use failure::Fallible;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{BinaryDeserializer, BinarySerializer};
use crate::semirings::Semiring;

use crate::tests_openfst::FstTestData;
use tempfile::tempdir;

pub fn test_vector_fst_bin_serializer<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32>,
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
