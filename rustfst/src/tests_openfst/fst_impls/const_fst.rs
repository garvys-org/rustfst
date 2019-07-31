use failure::Fallible;

use crate::algorithms::fst_convert;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::semirings::{Semiring, WeightQuantize};
use crate::tests_openfst::TestData;

pub fn test_const_fst_convert_convert<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize,
{
    let raw_fst = test_data.raw.clone();
    let const_fst: ConstFst<_> = raw_fst.clone().into();
    let pred_fst: VectorFst<_> = fst_convert(&const_fst);
    assert_eq!(
        raw_fst,
        pred_fst,
        "{}",
        error_message_fst!(
            raw_fst,
            pred_fst,
            "Convert VectorFst -> ConstFst -> VectorFst"
        )
    );
    Ok(())
}
