use failure::Fallible;

use crate::arc::Arc;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use crate::semirings::Semiring;
use crate::EPS_LABEL;

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
#[allow(unused)]
pub fn reverse<W, F1, F2>(ifst: &F1) -> Fallible<F2>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W::ReverseWeight> + ExpandedFst<W = W::ReverseWeight>,
{
    let mut ofst = F2::new();
    ofst.reserve_states(ifst.num_states());
    let istart = ifst.start();
    let mut ostart = ofst.add_state();
    let mut offset = 1;
    let istates: Vec<_> = ifst.states_iter().collect();
    for is in istates {
        let os = is + offset;
        while ofst.num_states() <= os {
            ofst.add_state();
        }
        if Some(is) == istart {
            ofst.set_final(os, W::ReverseWeight::one());
        }
        let weight = ifst.final_weight(is);
        if weight.is_some() && offset == 1 {
            ofst.add_arc(0, Arc::new(0, 0, weight.unwrap().reverse()?, os));
        }
        for iarc in ifst.arcs_iter(is)? {
            let nos = iarc.nextstate + offset;
            let weight = iarc.weight.reverse()?;
            while ofst.num_states() <= nos {
                ofst.add_state();
            }
            ofst.add_arc(nos, Arc::new(iarc.ilabel, iarc.olabel, weight, os))?;
        }
    }
    ofst.set_start(ostart);

    Ok(ofst)
}
