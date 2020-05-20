use anyhow::Result;
use counter::Counter;
use itertools::Itertools;

use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::WeightQuantize;
use crate::{Semiring, Tr, Trs, KDELTA};

pub fn compare_fst_static_lazy<W, FS, FD>(fst_static: &FS, fst_lazy: &FD) -> Result<()>
where
    FS: ExpandedFst<W>,
    FD: Fst<W>,
    W: Semiring + WeightQuantize,
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

        let mut trs_lazy: Counter<Tr<W>, usize> = Counter::new();
        let trs_lazy_owner = fst_lazy.get_trs(i)?;
        trs_lazy.update(trs_lazy_owner.trs().iter().map(|tr| {
            Tr::new(
                tr.ilabel,
                tr.olabel,
                tr.weight.quantize(KDELTA).unwrap(),
                tr.nextstate,
            )
        }));

        let mut trs_static: Counter<Tr<W>, usize> = Counter::new();
        let trs_static_owner = fst_static.get_trs(i)?;
        trs_static.update(trs_static_owner.trs().iter().map(|tr| {
            Tr::new(
                tr.ilabel,
                tr.olabel,
                tr.weight.quantize(KDELTA).unwrap(),
                tr.nextstate,
            )
        }));

        assert_eq!(trs_lazy, trs_static);
    }

    let fst_data_static = fst_static
        .fst_iter()
        .map(|data| {
            (
                data.state_id,
                data.trs
                    .trs()
                    .iter()
                    .map(|tr| {
                        Tr::<W>::new(
                            tr.ilabel,
                            tr.olabel,
                            tr.weight.quantize(KDELTA).unwrap(),
                            tr.nextstate,
                        )
                    })
                    .collect_vec(),
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
                data.trs
                    .trs()
                    .iter()
                    .map(|tr| {
                        Tr::<W>::new(
                            tr.ilabel,
                            tr.olabel,
                            tr.weight.quantize(KDELTA).unwrap(),
                            tr.nextstate,
                        )
                    })
                    .collect_vec(),
                data.final_weight,
                data.num_trs,
            )
        })
        .collect_vec();

    assert_eq!(fst_data_static, fst_data_lazy);

    Ok(())
}
