use failure::Fallible;
use itertools::Itertools;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

fn do_test_fst_into_iterator<F: ExpandedFst>(fst: F) -> Fallible<()> {
    let mut fst_data_ref = vec![];

    for state in 0..fst.num_states() {
        fst_data_ref.push((
            state,
            fst.arcs_iter(state)?.cloned().collect_vec(),
            fst.final_weight(state)?.cloned(),
            fst.num_arcs(state)?,
        ));
    }

    let mut fst_data = vec![];
    for fst_iter_data in fst.fst_into_iter() {
        fst_data.push((
            fst_iter_data.state_id,
            fst_iter_data.arcs.collect_vec(),
            fst_iter_data.final_weight,
            fst_iter_data.num_arcs,
        ));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

fn do_test_fst_iterator<F: ExpandedFst>(fst: &F) -> Fallible<()> {
    let mut fst_data_ref = vec![];

    for state in 0..fst.num_states() {
        fst_data_ref.push((
            state,
            fst.arcs_iter(state)?.collect_vec(),
            fst.final_weight(state)?,
            fst.num_arcs(state)?,
        ));
    }

    let mut fst_data = vec![];
    for data in fst.fst_iter() {
        fst_data.push((
            data.state_id,
            data.arcs.collect_vec(),
            data.final_weight,
            data.num_arcs,
        ));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

fn do_test_fst_iterator_mut<F: MutableFst>(mut fst: F) -> Fallible<()> {
    let mut fst_data_ref = vec![];

    for state in 0..fst.num_states() {
        fst_data_ref.push((
            state,
            fst.arcs_iter(state)?.cloned().collect_vec(),
            fst.final_weight(state)?.cloned(),
        ));
    }

    let mut fst_data = vec![];
    for data in fst.fst_iter_mut() {
        fst_data.push((
            data.state_id,
            data.arcs.map(|v| v.clone()).collect_vec(),
            data.final_weight.cloned(),
        ));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

pub fn test_fst_into_iterator_const<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + 'static,
{
    let raw_fst: ConstFst<_> = test_data.raw.clone().into();

    do_test_fst_iterator(&raw_fst)?;
    do_test_fst_into_iterator(raw_fst)?;

    Ok(())
}

pub fn test_fst_into_iterator_vector<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + 'static,
{
    let raw_fst = test_data.raw.clone();

    do_test_fst_iterator(&raw_fst)?;
    do_test_fst_into_iterator(raw_fst.clone())?;
    do_test_fst_iterator_mut(raw_fst)?;

    Ok(())
}
