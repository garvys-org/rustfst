use crate::algorithms::compose::IntervalReachVisitor;
use crate::algorithms::compose::IntervalSet;
use crate::algorithms::condense;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst};
use crate::StateId;

use crate::fst_properties::FstProperties;
use crate::semirings::Semiring;
use anyhow::Result;

static UNASSIGNED: usize = std::usize::MAX;

// Tests reachability of final states from a given state. To test for
// reachability from a state s, first do SetState(s). Then a final state f can
// be reached from state s of FST iff Reach(f) is true. The input can be cyclic,
// but no cycle may contain a final state.
pub struct StateReachable {
    pub(crate) isets: Vec<IntervalSet>,
    pub(crate) state2index: Vec<usize>,
}

impl StateReachable {
    pub fn new<W: Semiring, F: ExpandedFst<W>>(fst: &F) -> Result<Self> {
        let props = fst.properties_check(FstProperties::ACYCLIC)?;
        let acyclic = props.contains(FstProperties::ACYCLIC);
        if acyclic {
            Ok(Self::new_acyclic(fst))
        } else {
            Self::new_cyclic(fst)
        }
    }

    pub fn new_cyclic<W: Semiring, F: ExpandedFst<W>>(fst: &F) -> Result<Self> {
        let (scc, cfst): (_, VectorFst<_>) = condense(fst).unwrap();
        let reachable = StateReachable::new_acyclic(&cfst);
        let mut nscc = vec![];

        // Gets the number of states per SCC.
        for &c in scc.iter() {
            let c = c as usize;
            while c >= nscc.len() {
                nscc.push(0);
            }
            nscc[c] += 1;
        }

        // Constructs the interval sets and state index mapping for the original
        // FST from the condensation FST.
        let mut state2index = vec![UNASSIGNED; scc.len()];
        let mut isets: Vec<IntervalSet> = vec![];
        isets.resize_with(scc.len(), Default::default);
        for (s, &c) in scc.iter().enumerate() {
            let c = c as StateId;
            isets[s] = reachable.isets[c as usize].clone();
            state2index[s] = reachable.state2index[c as usize];

            // Checks that each final state in an input FST is not contained in a
            // cycle (i.e., not in a non-trivial SCC).
            if unsafe { cfst.is_final_unchecked(c) } && nscc[c as usize] > 1 {
                bail!("StateReachable: Final state contained in a cycle")
            }
        }
        Ok(Self { isets, state2index })
    }

    pub fn new_acyclic<W: Semiring, F: ExpandedFst<W>>(fst: &F) -> Self {
        let mut reach_visitor = IntervalReachVisitor::new(fst);
        dfs_visit(fst, &mut reach_visitor, &AnyTrFilter {}, false);
        Self {
            isets: reach_visitor.isets,
            state2index: reach_visitor.state2index,
        }
    }

    #[allow(unused)]
    // Can reach this final state from current state?
    pub fn reach(&self, current_state: StateId, s: StateId) -> Result<bool> {
        if let Some(i) = self.state2index.get(s as usize) {
            Ok(self.isets[current_state as usize].member(*i))
        } else {
            bail!("StateReachable: State non-final {}", s)
        }
    }
}
