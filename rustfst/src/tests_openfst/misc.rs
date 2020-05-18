use anyhow::Result;

use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_del_all_states<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: MutableFst<W> + SerializableFst<W>,
    W: SerializableSemiring,
{
    let mut fst = test_data.raw.clone();

    fst.del_all_states();

    assert_eq!(fst.num_states(), 0);

    Ok(())
}
