use crate::fst_traits::{MutableFst, TextParser};
use crate::semirings::{Semiring, WeightQuantize};
use crate::tests_openfst::FstTestData;
use failure::Fallible;

pub fn test_del_all_states<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst = test_data.raw.clone();

    fst.del_all_states();

    assert_eq!(fst.num_states(), 0);

    Ok(())
}
