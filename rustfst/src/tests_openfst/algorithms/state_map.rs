use failure::Fallible;

use crate::algorithms::state_mappers::ArcSumMapper;
use crate::algorithms::state_mappers::ArcUniqueMapper;
use crate::fst_traits::MutableFst;
use crate::fst_traits::TextParser;
use crate::semirings::Semiring;

use crate::tests_openfst::TestData;

pub fn test_state_map_arc_sum<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    let mut fst_state_map = test_data.raw.clone();
    let mut mapper = ArcSumMapper {};
    fst_state_map.state_map(&mut mapper)?;

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

pub fn test_state_map_arc_unique<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    let mut fst_state_map = test_data.raw.clone();
    let mut mapper = ArcUniqueMapper {};
    fst_state_map.state_map(&mut mapper)?;

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
