use failure::Fallible;

use crate::algorithms::matchers::Matcher;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, NO_STATE_ID};

mod arc_lookahead_matcher;
mod label_lookahead_matcher;
// mod trivial_lookahead_matcher;

pub trait LookaheadMatcher<'fst, W: Semiring + 'fst>: Matcher<'fst, W> {
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?
    fn lookahead_fst<LF: Fst<W = W>>(
        &mut self,
        matcher_state: StateId,
        lfst: &LF,
        lfst_state: StateId,
    ) -> Fallible<bool>;

    // Can the label be read from the current matcher state after possibly
    // following epsilon transitions?
    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool>;
    fn lookahead_prefix(&self, arc: &mut Arc<W>) -> bool;

    // Gives an estimate of the combined weight of the paths in the lookahead
    // and matcher FSTs for the last call to LookAheadFst. Non-trivial
    // implementations are useful for weight-pushing in composition.
    fn lookahead_weight(&self) -> &W;

    fn prefix_arc(&self) -> &Arc<W>;
    fn prefix_arc_mut(&mut self) -> &mut Arc<W>;
    fn lookahead_weight_mut(&mut self) -> &mut W;

    fn clear_lookahead_weight(&mut self) {
        *self.lookahead_weight_mut() = W::one();
    }
    fn set_lookahead_weight(&mut self, weight: W) {
        *self.lookahead_weight_mut() = weight;
    }
    fn clear_lookahead_prefix(&mut self) {
        self.prefix_arc_mut().nextstate = NO_STATE_ID;
    }
    fn set_lookahead_prefix(&mut self, arc: Arc<W>) {
        *self.prefix_arc_mut() = arc;
    }

    fn default_lookahead_prefix(&self, arc: &mut Arc<W>) -> bool {
        let prefix_arc = self.prefix_arc();
        if prefix_arc.nextstate != NO_STATE_ID {
            *arc = prefix_arc.clone();
            true
        } else {
            false
        }
    }
}
