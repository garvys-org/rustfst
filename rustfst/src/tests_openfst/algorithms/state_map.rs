use std::fmt::Display;

use anyhow::Result;

use crate::algorithms::arc_sum;
use crate::algorithms::arc_unique;
use crate::fst_traits::{MutableFst, SerializableFst};
use crate::semirings::SerializableSemiring;
use crate::tests_openfst::FstTestData;

pub fn test_state_map_arc_sum<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring,
{
    let mut fst_state_map = test_data.raw.clone();
    arc_sum(&mut fst_state_map);

    assert_eq!(
        test_data.state_map_arc_sum,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_arc_sum,
            fst_state_map,
            "StateMap : ArcSum"
        )
    );

    Ok(())
}

pub fn test_state_map_arc_unique<F>(test_data: &FstTestData<F>) -> Result<()>
where
    F: SerializableFst + MutableFst + Display,
    F::W: SerializableSemiring,
{
    let mut fst_state_map = test_data.raw.clone();
    arc_unique(&mut fst_state_map);

    assert_eq!(
        test_data.state_map_arc_unique,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_arc_unique,
            fst_state_map,
            "StateMap : ArcUnique"
        )
    );

    Ok(())
}
