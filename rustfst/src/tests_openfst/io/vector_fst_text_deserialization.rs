use anyhow::Result;

use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, SerializableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

pub fn test_vector_fst_text_deserialization<W>(
    test_data: &FstTestData<W, VectorFst<W>>,
) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let fst: VectorFst<_> = VectorFst::from_text_string(test_data.raw_text.as_str())?.quantize()?;
    let fst_ref: VectorFst<_> = test_data.raw.clone().quantize()?;
    assert_eq!(fst_ref, fst);
    Ok(())
}
