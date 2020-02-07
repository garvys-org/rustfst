use failure::Fallible;
use itertools::Itertools;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::ExpandedFst;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

fn do_test<F: ExpandedFst>(fst: F) -> Fallible<()> {
    let mut fst_data_ref = vec![];

    for state in 0..fst.num_states() {
        fst_data_ref.push((
            state,
            fst.arcs_iter(state)?.cloned().collect_vec(),
            fst.final_weight(state)?.cloned(),
        ));
    }

    let mut fst_data = vec![];
    for (state_id, arcs, final_weight) in fst.fst_into_iter() {
        fst_data.push((state_id, arcs.collect_vec(), final_weight));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

pub fn test_fst_into_iterator_const<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + 'static,
{
    let raw_fst: ConstFst<_> = test_data.raw.clone().into();

    do_test(raw_fst)?;

    Ok(())
}

pub fn test_fst_into_iterator_vector<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + 'static,
{
    let raw_fst = test_data.raw.clone();

    do_test(raw_fst)?;

    Ok(())
}
