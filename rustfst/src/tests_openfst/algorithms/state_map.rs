use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::tr_sum;
use crate::algorithms::tr_unique;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_state_map_tr_sum<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring,
{
    let mut fst_state_map = test_data.raw.clone();
    tr_sum(&mut fst_state_map);

    assert_eq!(
        test_data.state_map_tr_sum,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_tr_sum,
            fst_state_map,
            "StateMap : TrSum"
        )
    );

    Ok(())
}

pub fn test_state_map_tr_unique<W, F>(test_data: &FstTestData<W, F>) -> Result<()>
where
    F: SerializableFst<W> + MutableFst<W> + Display,
    W: SerializableSemiring,
{
    let mut fst_state_map = test_data.raw.clone();
    tr_unique(&mut fst_state_map);

    assert_eq!(
        test_data.state_map_tr_unique,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_tr_unique,
            fst_state_map,
            "StateMap : TrUnique"
        )
    );

    Ok(())
}
