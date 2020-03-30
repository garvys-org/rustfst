use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use failure::Fallible;

pub use arc_lookahead_matcher::ArcLookAheadMatcher;
pub use label_lookahead_matcher::LabelLookAheadMatcher;

use crate::algorithms::matchers::MatcherFlags;
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Arc, Label, StateId, NO_STATE_ID};

mod arc_lookahead_matcher;
pub(crate) mod interval_set;
mod label_lookahead_matcher;
pub mod label_lookahead_relabeler;
pub mod label_reachable;
pub(crate) mod state_reachable;
// mod trivial_lookahead_matcher;
pub mod add_on;
pub mod matcher_fst;

pub trait MatcherFlagsTrait: Debug {
    fn flags() -> MatcherFlags;
}

pub trait LookaheadMatcher<'fst, W: Semiring + 'fst>: Matcher<'fst, W> {
    type MatcherData: Clone;
    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>>;

    fn new_with_data(
        fst: &'fst Self::F,
        match_type: MatchType,
        data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Fallible<Self>
    where
        Self: std::marker::Sized;

    fn create_data(fst: &Self::F, match_type: MatchType) -> Option<Rc<RefCell<Self::MatcherData>>>;

    fn init_lookahead_fst<LF: ExpandedFst<W = W>>(&mut self, lfst: &LF) -> Fallible<()>;
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?
    fn lookahead_fst<LF: ExpandedFst<W = W>>(
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
