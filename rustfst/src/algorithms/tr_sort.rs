use std::cmp::Ordering;

use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::{StateId, Tr};

pub trait TrCompare {
    fn compare<W: Semiring>(a: &Tr<W>, b: &Tr<W>) -> Ordering;
    fn properties(inprops: FstProperties) -> FstProperties;
}

/// Compare only input labels.
pub struct ILabelCompare {}

impl TrCompare for ILabelCompare {
    fn compare<W: Semiring>(a: &Tr<W>, b: &Tr<W>) -> Ordering {
        a.ilabel.cmp(&b.ilabel)
    }

    fn properties(inprops: FstProperties) -> FstProperties {
        let mut outprops =
            (inprops & FstProperties::arcsort_properties()) | FstProperties::I_LABEL_SORTED;
        if inprops.contains(FstProperties::ACCEPTOR) {
            outprops |= FstProperties::O_LABEL_SORTED;
        }
        outprops
    }
}

/// Compare only output labels.
pub struct OLabelCompare {}

impl TrCompare for OLabelCompare {
    fn compare<W: Semiring>(a: &Tr<W>, b: &Tr<W>) -> Ordering {
        a.olabel.cmp(&b.olabel)
    }

    fn properties(inprops: FstProperties) -> FstProperties {
        let mut outprops =
            (inprops & FstProperties::arcsort_properties()) | FstProperties::O_LABEL_SORTED;
        if inprops.contains(FstProperties::ACCEPTOR) {
            outprops |= FstProperties::I_LABEL_SORTED;
        }
        outprops
    }
}

/// Sorts trs leaving each state of the FST using a compare function
// The compare function could be passed only with the generic parameters but it seems less intuitive.
pub fn tr_sort<W, F, C>(fst: &mut F, _comp: C)
where
    W: Semiring,
    F: MutableFst<W>,
    C: TrCompare,
{
    let props = fst.properties();
    for state in 0..(fst.num_states() as StateId) {
        fst.sort_trs_unchecked(state, C::compare);
    }
    fst.set_properties_with_mask(C::properties(props), FstProperties::all_properties());
}
