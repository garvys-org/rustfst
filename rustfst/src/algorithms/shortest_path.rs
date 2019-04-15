use failure::Fallible;

use crate::algorithms::queues::AutoQueue;
use crate::algorithms::{reverse, shortest_distance};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{Semiring, SemiringProperties};

pub fn shortest_path<FI, FO>(
    ifst: &FI,
    nshortest: usize,
    unique: bool,
    first_path: bool,
) -> Fallible<FO>
where
    FI: ExpandedFst + MutableFst,
    FO: MutableFst<W = FI::W>,
    FI::W: 'static,
{
    let queue = AutoQueue::new(ifst, None)?;

    if nshortest == 0 {
        return Ok(FO::new());
    }

    if nshortest == 1 {
        unimplemented!()
    }

    if !FI::W::properties().contains(SemiringProperties::PATH | SemiringProperties::SEMIRING) {
        bail!("ShortestPath : Weight need to have the Path property and be distributive")
    }

//    let distance = shortest_distance(ifst, false)?;
//    let rfst: VectorFst<_> = reverse(ifst)?;
//    let mut d = FI::W::zero();
    unimplemented!()
}
