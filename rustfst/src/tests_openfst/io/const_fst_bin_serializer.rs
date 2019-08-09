use failure::Fallible;
use tempfile::tempdir;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{BinaryDeserializer, BinarySerializer, ExpandedFst};
use crate::semirings::Semiring;
use crate::tests_openfst::TestData;

pub fn test_const_fst_bin_serializer<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32>,
{
    let dir = tempdir()?;

    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    let path_fst_serialized = dir.path().join("raw_const.fst");
    raw_const.write(&path_fst_serialized)?;

    let deserialized_fst = ConstFst::<W>::read(&path_fst_serialized)?;

    assert_eq!(
        raw_const,
        deserialized_fst,
        "{}",
        error_message_fst!(raw_const, deserialized_fst, "Serializer ConstFst Bin")
    );
    Ok(())
}
