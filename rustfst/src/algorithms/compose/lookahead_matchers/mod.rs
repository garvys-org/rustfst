use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

// pub use label_lookahead_matcher::LabelLookAheadMatcher;
// pub use label_lookahead_relabeler::LabelLookAheadRelabeler;
// pub use tr_lookahead_matcher::TrLookAheadMatcher;
// pub use trivial_lookahead_matcher::TrivialLookAheadMatcher;

use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, NO_STATE_ID};

// mod label_lookahead_matcher;
// pub mod label_lookahead_relabeler;
// mod tr_lookahead_matcher;
// mod trivial_lookahead_matcher;

pub trait MatcherFlagsTrait: Debug {
    fn flags() -> MatcherFlags;
}

pub trait LookaheadMatcher<W: Semiring>: Matcher<W> {
    type MatcherData: Clone;
    fn data(&self) -> Option<&Arc<RefCell<Self::MatcherData>>>;

    fn new_with_data(
        fst: Arc<Self::F>,
        match_type: MatchType,
        data: Option<Arc<RefCell<Self::MatcherData>>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn create_data<F: ExpandedFst<W>>(
        fst: &F,
        match_type: MatchType,
    ) -> Result<Option<Arc<RefCell<Self::MatcherData>>>>;

    // Previously init_lookahead_fst
    fn check_lookahead_fst<LF: ExpandedFst<W>>(&mut self, lfst: &Arc<LF>) -> Result<()>;
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?
    fn lookahead_fst<LF: ExpandedFst<W>>(
        &self,
        matcher_state: StateId,
        lfst: &Arc<LF>,
        lfst_state: StateId,
    ) -> Result<bool>;

    // Can the label be read from the current matcher state after possibly
    // following epsilon transitions?
    fn lookahead_label(&self, state: StateId, label: Label) -> Result<bool>;
    fn lookahead_prefix(&self, tr: &mut Tr<W>) -> bool;

    // Gives an estimate of the combined weight of the paths in the lookahead
    // and matcher FSTs for the last call to LookAheadFst. Non-trivial
    // implementations are useful for weight-pushing in composition.
    fn lookahead_weight(&self) -> &W;

    fn prefix_tr(&self) -> &Tr<W>;
    fn prefix_tr_mut(&mut self) -> &mut Tr<W>;
    fn lookahead_weight_mut(&mut self) -> &mut W;

    fn clear_lookahead_weight(&mut self) {
        *self.lookahead_weight_mut() = W::one();
    }
    fn set_lookahead_weight(&mut self, weight: W) {
        *self.lookahead_weight_mut() = weight;
    }
    fn clear_lookahead_prefix(&mut self) {
        self.prefix_tr_mut().nextstate = NO_STATE_ID;
    }
    fn set_lookahead_prefix(&mut self, tr: Tr<W>) {
        *self.prefix_tr_mut() = tr;
    }

    fn default_lookahead_prefix(&self, tr: &mut Tr<W>) -> bool {
        let prefix_tr = self.prefix_tr();
        if prefix_tr.nextstate != NO_STATE_ID {
            *tr = prefix_tr.clone();
            true
        } else {
            false
        }
    }
}
