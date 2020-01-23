use failure::Fallible;

use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_del_all_states<F>(test_data: &FstTestData<F>) -> Fallible<()>
where
    F: MutableFst + SerializableFst,
    F::W: SerializableSemiring,
{
    let mut fst = test_data.raw.clone();

    fst.del_all_states();

    assert_eq!(fst.num_states(), 0);

    Ok(())
}
