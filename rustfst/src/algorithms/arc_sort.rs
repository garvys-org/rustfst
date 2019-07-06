use std::cmp::Ordering;

use crate::fst_traits::{ExpandedFst, MutableFst};
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

/// Sorts arcs leaving each state of the FST using a compare function
pub fn arc_sort<F>(fst: &mut F, comp: impl Fn(&Arc<F::W>, &Arc<F::W>) -> Ordering)
where
    F: MutableFst + ExpandedFst,
{
    for state in 0..fst.num_states() {
        fst.sort_arcs_unchecked(state, &comp);
    }
}
