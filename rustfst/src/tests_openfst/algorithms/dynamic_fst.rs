use crate::fst_traits::{ExpandedFst, Fst};
use counter::Counter;
use failure::Fallible;

pub fn compare_fst_static_dynamic<FS, FD>(fst_static: &FS, fst_dynamic: &FD) -> Fallible<()>
where
    FS: ExpandedFst,
    FD: Fst<W = FS::W>,
{
    assert_eq!(fst_dynamic.states_iter().count(), fst_static.num_states());

    assert_eq!(fst_dynamic.start(), fst_static.start());

    for i in 0..fst_static.num_states() {
        assert_eq!(fst_dynamic.final_weight(i)?, fst_static.final_weight(i)?);
        unsafe {
            assert_eq!(
                fst_dynamic.final_weight_unchecked(i),
                fst_static.final_weight_unchecked(i)
            )
        };
        assert_eq!(fst_dynamic.num_arcs(i)?, fst_static.num_arcs(i)?);
        unsafe {
            assert_eq!(
                fst_dynamic.num_arcs_unchecked(i),
                fst_static.num_arcs_unchecked(i)
            )
        };

        let mut arcs_dynamic: Counter<_, usize> = Counter::new();
        arcs_dynamic.update(fst_dynamic.arcs_iter(i)?.cloned());

        let mut arcs_static: Counter<_, usize> = Counter::new();
        arcs_static.update(fst_static.arcs_iter(i)?.cloned());

        assert_eq!(arcs_dynamic, arcs_static);
    }

    Ok(())
}
