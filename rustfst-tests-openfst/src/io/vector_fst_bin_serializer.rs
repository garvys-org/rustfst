use failure::Fallible;

use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{BinarySerializer, BinaryDeserializer};
use rustfst::semirings::Semiring;

use tempfile::tempdir;
use crate::TestData;

pub fn test_vector_fst_bin_serializer<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
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
