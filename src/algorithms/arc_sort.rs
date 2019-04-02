use std::cmp::Ordering;
use std::marker::PhantomData;

use failure::Fallible;

use crate::algorithms::{state_map, StateMapper};
use crate::fst_traits::{CoreFst, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

/// Compare only input labels.
pub fn ilabel_compare<W: Semiring>(a: &Arc<W>, b: &Arc<W>) -> Ordering {
    a.ilabel.cmp(&b.ilabel)
}

/// Compare only output labels.
pub fn olabel_compare<W: Semiring>(a: &Arc<W>, b: &Arc<W>) -> Ordering {
    a.olabel.cmp(&b.olabel)
}

struct ArcSortMapper<W: Semiring, F: Fn(&Arc<W>, &Arc<W>) -> Ordering> {
    f: F,
    ghost: PhantomData<W>,
}

impl<FST, F> StateMapper<FST> for ArcSortMapper<FST::W, F>
where
    FST: MutableFst,
    F: Fn(&Arc<FST::W>, &Arc<FST::W>) -> Ordering,
{
    fn map_final_weight(&self, _weight: Option<&mut <FST as CoreFst>::W>) -> Fallible<()> {
        Ok(())
    }

    fn map_arcs(&self, fst: &mut FST, state: usize) -> Fallible<()> {
        let mut arcs = fst.pop_arcs(state).unwrap();
        arcs.sort_by(&self.f);
        fst.reserve_arcs(state, arcs.len()).unwrap();
        arcs.into_iter()
            .for_each(|arc| fst.add_arc(state, arc).unwrap());
        Ok(())
    }
}

/// Sorts arcs leaving each state of the FST using a compare function
pub fn arc_sort<F>(fst: &mut F, comp: impl Fn(&Arc<F::W>, &Arc<F::W>) -> Ordering) -> Fallible<()>
where
    F: MutableFst,
{
    let mut mapper = ArcSortMapper {
        f: comp,
        ghost: PhantomData,
    };
    state_map(fst, &mut mapper)
}
