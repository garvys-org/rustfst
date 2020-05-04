use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::Result;

pub use arc_lookahead_matcher::TrLookAheadMatcher;
pub use label_lookahead_matcher::LabelLookAheadMatcher;
pub use label_lookahead_relabeler::LabelLookAheadRelabeler;
pub use trivial_lookahead_matcher::TrivialLookAheadMatcher;

use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Tr, Label, StateId, NO_STATE_ID};

mod arc_lookahead_matcher;
mod label_lookahead_matcher;
pub mod label_lookahead_relabeler;
mod trivial_lookahead_matcher;

pub trait MatcherFlagsTrait: Debug {
    fn flags() -> MatcherFlags;
}

pub trait LookaheadMatcher<W: Semiring>: Matcher<W> {
    type MatcherData: Clone;
    fn data(&self) -> Option<&Rc<RefCell<Self::MatcherData>>>;

    fn new_with_data(
        fst: Rc<Self::F>,
        match_type: MatchType,
        data: Option<Rc<RefCell<Self::MatcherData>>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn create_data<F: ExpandedFst<W = W>>(
        fst: &F,
        match_type: MatchType,
    ) -> Result<Option<Rc<RefCell<Self::MatcherData>>>>;

    fn init_lookahead_fst<LF: ExpandedFst<W = W>>(&mut self, lfst: &Rc<LF>) -> Result<()>;
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?
    fn lookahead_fst<LF: ExpandedFst<W = W>>(
        &mut self,
        matcher_state: StateId,
        lfst: &Rc<LF>,
        lfst_state: StateId,
    ) -> Result<bool>;

    // Can the label be read from the current matcher state after possibly
    // following epsilon transitions?
    fn lookahead_label(&self, state: StateId, label: Label) -> Result<bool>;
    fn lookahead_prefix(&self, arc: &mut Tr<W>) -> bool;

    // Gives an estimate of the combined weight of the paths in the lookahead
    // and matcher FSTs for the last call to LookAheadFst. Non-trivial
    // implementations are useful for weight-pushing in composition.
    fn lookahead_weight(&self) -> &W;

    fn prefix_arc(&self) -> &Tr<W>;
    fn prefix_arc_mut(&mut self) -> &mut Tr<W>;
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
    fn set_lookahead_prefix(&mut self, arc: Tr<W>) {
        *self.prefix_arc_mut() = arc;
    }

    fn default_lookahead_prefix(&self, arc: &mut Tr<W>) -> bool {
        let prefix_arc = self.prefix_arc();
        if prefix_arc.nextstate != NO_STATE_ID {
            *arc = prefix_arc.clone();
            true
        } else {
            false
        }
    }
}
