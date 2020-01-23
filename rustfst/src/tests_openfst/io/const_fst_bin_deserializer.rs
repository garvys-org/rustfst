use failure::Fallible;
use failure::ResultExt;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::SerializableFst;
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_const_fst_bin_deserializer<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
{
    let parsed_fst_bin = ConstFst::<W>::read(&test_data.raw_const_bin_path)
        .with_context(|_| format_err!("Failed parsing ConstFst Aligned"))?;
    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    assert_eq!(
        raw_const,
        parsed_fst_bin,
        "{}",
        error_message_fst!(raw_const, parsed_fst_bin, "Deserializer ConstFst Bin")
    );
    Ok(())
}

pub fn test_const_fst_aligned_bin_deserializer<W>(
    test_data: &FstTestData<VectorFst<W>>,
) -> Fallible<()>
where
    W: SerializableSemiring + 'static,
{
    let parsed_fst_bin = ConstFst::<W>::read(&test_data.raw_const_aligned_bin_path)
        .with_context(|_| format_err!("Failed parsing ConstFst Aligned Bin"))?;
    let raw_const: ConstFst<_> = test_data.raw.clone().into();

    assert_eq!(
        raw_const,
        parsed_fst_bin,
        "{}",
        error_message_fst!(
            raw_const,
            parsed_fst_bin,
            "Deserializer ConstFst Aligned Bin"
        )
    );
    Ok(())
}
