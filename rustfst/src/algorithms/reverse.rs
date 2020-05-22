use anyhow::Result;

use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::tr::Tr;
use crate::{Trs, EPS_LABEL};

/// Reverses an FST. The reversed result is written to an output mutable FST.
/// If A transduces string x to y with weight a, then the reverse of A
/// transduces the reverse of x to the reverse of y with weight a.Reverse().
///
/// Typically, a = a.Reverse() and a transition is its own reverse (e.g., for
/// TropicalWeight or LogWeight). In general, e.g., when the weights only form a
/// left or right semiring, the output transition type must match the input transition type
/// except having the reversed Weight type.
///
/// A superinitial state is always created.
///
/// # Example
///
/// ## Input
///
/// ![reverse_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/reverse_in.svg?sanitize=true)
///
/// ## Output
///
/// ![reverse_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/reverse_out.svg?sanitize=true)
///
pub fn reverse<W, F1, F2>(ifst: &F1) -> Result<F2>
where
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: MutableFst<W::ReverseWeight> + AllocableFst<W::ReverseWeight>,
{
    let mut ofst = F2::new();
    ofst.reserve_states(ifst.num_states());
    let istart = ifst.start();
    let ostart = ofst.add_state();

    ofst.add_states(ifst.num_states());

    let mut c_trs = vec![0; ifst.num_states() + 1];
    for is in 0..ifst.num_states() {
        for iarc in unsafe { ifst.get_trs_unchecked(is).trs() } {
            c_trs[iarc.nextstate + 1] += 1;
        }
    }

    let mut states_trs: Vec<_> = c_trs.into_iter().map(Vec::with_capacity).collect();

    for is in 0..ifst.num_states() {
        let os = is + 1;
        if Some(is) == istart {
            ofst.set_final(os, W::ReverseWeight::one())?;
        }
        let weight = unsafe { ifst.final_weight_unchecked(is) };
        if let Some(w) = weight {
            states_trs[0].push(Tr::new(EPS_LABEL, EPS_LABEL, w.reverse()?, os));
        }

        for iarc in unsafe { ifst.get_trs_unchecked(is).trs() } {
            let nos = iarc.nextstate + 1;
            let weight = iarc.weight.reverse()?;
            let w = Tr::new(iarc.ilabel, iarc.olabel, weight, os);
            states_trs[nos].push(w);
        }
    }
    states_trs
        .into_iter()
        .enumerate()
        .for_each(|(s, trs)| unsafe { ofst.set_trs_unchecked(s, trs) });
    ofst.set_start(ostart)?;

    ofst.set_symts_from_fst(ifst);

    Ok(ofst)
}
