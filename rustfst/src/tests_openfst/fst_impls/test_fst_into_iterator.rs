use anyhow::Result;
use itertools::Itertools;

use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::ExpandedFst;
use crate::semirings::{SerializableSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;
use crate::{Semiring, Trs};

fn do_test_fst_into_iterator<W: Semiring, F: ExpandedFst<W>>(fst: F) -> Result<()> {
    let mut fst_data_ref = vec![];

    for state in fst.states_range() {
        fst_data_ref.push((
            state,
            fst.get_trs(state)?.trs().iter().cloned().collect_vec(),
            fst.final_weight(state)?,
            fst.num_trs(state)?,
        ));
    }

    let mut fst_data = vec![];
    for fst_iter_data in fst.fst_into_iter() {
        fst_data.push((
            fst_iter_data.state_id,
            fst_iter_data.trs.collect_vec(),
            fst_iter_data.final_weight,
            fst_iter_data.num_trs,
        ));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

fn do_test_fst_iterator<W: Semiring, F: ExpandedFst<W>>(fst: &F) -> Result<()> {
    let mut fst_data_ref = vec![];

    for state in fst.states_range() {
        fst_data_ref.push((
            state,
            fst.get_trs(state)?.trs().iter().cloned().collect_vec(),
            fst.final_weight(state)?,
            fst.num_trs(state)?,
        ));
    }

    let mut fst_data = vec![];
    for data in fst.fst_iter() {
        fst_data.push((
            data.state_id,
            data.trs.trs().iter().cloned().collect_vec(),
            data.final_weight,
            data.num_trs,
        ));
    }
    assert_eq!(fst_data, fst_data_ref);

    Ok(())
}

pub fn test_fst_into_iterator_const<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let raw_fst: ConstFst<_> = test_data.raw.clone().into();

    do_test_fst_iterator(&raw_fst)?;
    do_test_fst_into_iterator(raw_fst)?;

    Ok(())
}

pub fn test_fst_into_iterator_vector<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    let raw_fst = test_data.raw.clone();

    do_test_fst_iterator(&raw_fst)?;
    do_test_fst_into_iterator(raw_fst.clone())?;

    Ok(())
}
