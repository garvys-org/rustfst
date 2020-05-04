use anyhow::Result;
use counter::Counter;
use itertools::Itertools;

use crate::fst_traits::{ExpandedFst, Fst};

pub fn compare_fst_static_lazy<FS, FD>(fst_static: &FS, fst_lazy: &FD) -> Result<()>
where
    FS: ExpandedFst,
    FD: Fst<W = FS::W>,
{
    assert_eq!(fst_lazy.states_iter().count(), fst_static.num_states());

    assert_eq!(fst_lazy.start(), fst_static.start());

    for i in 0..fst_static.num_states() {
        assert_eq!(fst_lazy.final_weight(i)?, fst_static.final_weight(i)?);
        unsafe {
            assert_eq!(
                fst_lazy.final_weight_unchecked(i),
                fst_static.final_weight_unchecked(i)
            )
        };
        assert_eq!(fst_lazy.num_trs(i)?, fst_static.num_trs(i)?);
        unsafe {
            assert_eq!(
                fst_lazy.num_trs_unchecked(i),
                fst_static.num_trs_unchecked(i)
            )
        };

        let mut arcs_lazy: Counter<_, usize> = Counter::new();
        arcs_lazy.update(fst_lazy.arcs_iter(i)?.cloned());

        let mut arcs_static: Counter<_, usize> = Counter::new();
        arcs_static.update(fst_static.arcs_iter(i)?.cloned());

        assert_eq!(arcs_lazy, arcs_static);
    }

    let fst_data_static = fst_static
        .fst_iter()
        .map(|data| {
            (
                data.state_id,
                data.arcs.collect_vec(),
                data.final_weight,
                data.num_trs,
            )
        })
        .collect_vec();
    let fst_data_lazy = fst_lazy
        .fst_iter()
        .map(|data| {
            (
                data.state_id,
                data.arcs.collect_vec(),
                data.final_weight,
                data.num_trs,
            )
        })
        .collect_vec();

    assert_eq!(fst_data_static, fst_data_lazy);

    Ok(())
}
