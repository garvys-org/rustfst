use anyhow::Result;

use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::{StateId, Trs};

/// Return an acyclic FST where each SCC in the input FST has been condensed to
/// a single state with transitions between SCCs retained and within SCCs
/// dropped.
///
/// Also populates 'scc' with a mapping from input to output states.
pub fn condense<W: Semiring, FI: Fst<W> + ExpandedFst<W>, FO: MutableFst<W>>(
    ifst: &FI,
) -> Result<(Vec<i32>, FO)> {
    let mut visitor = SccVisitor::new(ifst, true, false);
    dfs_visit(ifst, &mut visitor, &AnyTrFilter {}, false);
    let scc = visitor.scc.unwrap();
    let mut ofst = FO::new();
    if let Some(max) = scc.iter().max() {
        let num_condensed_states = *max as usize + 1;
        ofst.add_states(num_condensed_states);
        unsafe {
            for (s, &c) in scc.iter().enumerate() {
                let c = c as StateId;
                let s = s as StateId;

                if s == ifst.start().unwrap() {
                    ofst.set_start_unchecked(c);
                }
                if let Some(final_weight) = ifst.final_weight_unchecked(s) {
                    let final_weight_ofst = match ofst.final_weight_unchecked(c) {
                        Some(w) => w.plus(final_weight)?,
                        None => final_weight,
                    };
                    ofst.set_final_unchecked(c, final_weight_ofst);
                }
                for tr in ifst.get_trs_unchecked(s).trs() {
                    let nextc = scc[tr.nextstate as usize] as StateId;
                    if nextc != c {
                        let mut condensed_tr = tr.clone();
                        condensed_tr.nextstate = nextc;
                        ofst.add_tr_unchecked(c, condensed_tr);
                    }
                }
            }
        }
    }
    Ok((scc, ofst))
}
