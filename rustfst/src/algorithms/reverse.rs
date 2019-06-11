use failure::Fallible;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;

/// Reverses an FST. The reversed result is written to an output mutable FST.
/// If A transduces string x to y with weight a, then the reverse of A
/// transduces the reverse of x to the reverse of y with weight a.Reverse().
///
/// Typically, a = a.Reverse() and an arc is its own reverse (e.g., for
/// TropicalWeight or LogWeight). In general, e.g., when the weights only form a
/// left or right semiring, the output arc type must match the input arc type
/// except having the reversed Weight type.
///
/// A superinitial state is always created.
pub fn reverse<W, F1, F2>(ifst: &F1) -> Fallible<F2>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W::ReverseWeight> + ExpandedFst<W = W::ReverseWeight>,
{
    let mut ofst = F2::new();
    ofst.reserve_states(ifst.num_states());
    let istart = ifst.start();
    let ostart = ofst.add_state();

    for _ in 0..ifst.num_states() {
        ofst.add_state();
    }

    let mut c_arcs = vec![0; ifst.num_states() + 1];
    for is in 0..ifst.num_states() {
        for iarc in ifst.arcs_iter_unchecked(is) {
            c_arcs[iarc.nextstate + 1] += 1;
        }
    }

    let mut states_arcs: Vec<_> = c_arcs.into_iter().map(|c| Vec::with_capacity(c)).collect();

    for is in 0..ifst.num_states() {
        let os = is + 1;
        if Some(is) == istart {
            ofst.set_final(os, W::ReverseWeight::one())?;
        }
        let weight = ifst.final_weight(is);
        if let Some(w) = weight {
            states_arcs[0].push(Arc::new(0, 0, w.reverse()?, os));
        }

        for iarc in ifst.arcs_iter_unchecked(is) {
            let nos = iarc.nextstate + 1;
            let weight = iarc.weight.reverse()?;
            let w = Arc::new(iarc.ilabel, iarc.olabel, weight, os);
            states_arcs[nos].push(w);
        }
    }
    states_arcs
        .into_iter()
        .enumerate()
        .for_each(|(s, arcs)| ofst.set_arcs_unchecked(s, arcs));
    ofst.set_start(ostart)?;

    Ok(ofst)
}
