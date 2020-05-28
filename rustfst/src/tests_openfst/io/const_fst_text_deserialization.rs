use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{ExpandedFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;
use anyhow::Result;

pub fn test_const_fst_text_deserialization<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let fst: VectorFst<_> = ConstFst::from_text_string(test_data.raw_text.as_str())?.quantize()?;
    let fst_ref: VectorFst<_> = test_data.raw.quantize()?;
    assert_eq!(fst_ref, fst);
    Ok(())
}
