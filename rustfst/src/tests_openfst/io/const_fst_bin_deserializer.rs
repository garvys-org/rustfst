use failure::Fallible;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::BinaryDeserializer;
use crate::semirings::Semiring;

use crate::tests_openfst::TestData;

pub fn test_const_fst_bin_deserializer<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32>,
{
    let parsed_fst_bin = ConstFst::<W>::read(&test_data.raw_const_bin_path)?;
    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    assert_eq!(
        raw_const,
        parsed_fst_bin,
        "{}",
        error_message_fst!(raw_const, parsed_fst_bin, "Deserializer ConstFst Bin")
    );
    Ok(())
}
