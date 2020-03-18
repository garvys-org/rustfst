use crate::algorithms::visitors::SccVisitor;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};

use crate::algorithms::arc_filters::AnyArcFilter;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::semirings::Semiring;
use failure::Fallible;

// Returns an acyclic FST where each SCC in the input FST has been condensed to
// a single state with transitions between SCCs retained and within SCCs
// dropped. Also populates 'scc' with a mapping from input to output states.
pub fn condense<FI: Fst + ExpandedFst, FO: MutableFst<W = FI::W>>(
    ifst: &FI,
) -> Fallible<(Vec<i32>, FO)> {
    let mut visitor = SccVisitor::new(ifst, true, false);
    dfs_visit(ifst, &mut visitor, &AnyArcFilter {}, false);
    let scc = visitor.scc.unwrap();
    let mut ofst = FO::new();
    if let Some(max) = scc.iter().max() {
        let num_condensed_states = *max as usize + 1;
        ofst.add_states(num_condensed_states);
        unsafe {
            for (s, &c) in scc.iter().enumerate() {
                let c = c as usize;
                if s == ifst.start().unwrap() {
                    ofst.set_start_unchecked(c);
                }
                if let Some(final_weight) = ifst.final_weight_unchecked(s) {
                    match ofst.final_weight_unchecked_mut(c) {
                        Some(w) => w.plus_assign(final_weight)?,
                        None => ofst.set_final_unchecked(c, final_weight.clone()),
                    };
                }
                for arc in ifst.arcs_iter_unchecked(s) {
                    let nextc = scc[arc.nextstate] as usize;
                    if nextc != c {
                        let mut condensed_arc = arc.clone();
                        condensed_arc.nextstate = nextc;
                        ofst.add_arc_unchecked(c, condensed_arc);
                    }
                }
            }
        }
    }
    Ok((scc, ofst))
}
