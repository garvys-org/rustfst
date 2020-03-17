use crate::algorithms::arc_filters::AnyArcFilter;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::lookahead_matchers::interval_set::IntervalSet;
use crate::algorithms::visitors::IntervalReachVisitor;
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;

// Tests reachability of final states from a given state. To test for
// reachability from a state s, first do SetState(s). Then a final state f can
// be reached from state s of FST iff Reach(f) is true. The input can be cyclic,
// but no cycle may contain a final state.
struct StateReachable {
    isets: Vec<IntervalSet>,
    state2index: Vec<usize>,
}

impl StateReachable {
    fn new<F: ExpandedFst>(fst: &F, acyclic: bool) -> Self {
        if acyclic {
            Self::new_acyclic(fst)
        } else {
            Self::new_cyclic(fst)
        }
    }

    fn new_cyclic<F: ExpandedFst>(fst: &F) -> Self {
        let mut reach_visitor = IntervalReachVisitor::new(fst);
        dfs_visit(fst, &mut reach_visitor, &AnyArcFilter {}, false);
        Self {
            isets: reach_visitor.isets,
            state2index: reach_visitor.state2index,
        }
    }

    fn new_acyclic<F: Fst>(fst: &F) -> Self {
        todo!()
    }
}
