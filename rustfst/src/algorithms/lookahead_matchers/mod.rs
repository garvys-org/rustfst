use failure::Fallible;

use crate::algorithms::matchers::Matcher;
use crate::fst_traits::Fst;
use crate::{Label, StateId};
use crate::semirings::Semiring;

mod arc_lookahead_matcher;
mod trivial_lookahead_matcher;

pub trait LookaheadMatcher<'fst, W: Semiring + 'fst>: Matcher<'fst, W> {
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?
    fn lookahead_fst<LF: Fst<W = W>>(&self, state: StateId, lfst: &LF) -> bool;

    // Can the label be read from the current matcher state after possibly
    // following epsilon transitions?
    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool>;
    fn lookahead_prefix(&self) -> bool;

    // Gives an estimate of the combined weight of the paths in the lookahead
    // and matcher FSTs for the last call to LookAheadFst. Non-trivial
    // implementations are useful for weight-pushing in composition.
    fn lookahead_weight(&self) -> W;
}
