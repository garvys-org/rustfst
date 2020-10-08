use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

pub use label_lookahead_matcher::LabelLookAheadMatcher;
pub(super) use label_lookahead_relabeler::LabelLookAheadRelabeler;
pub use tr_lookahead_matcher::TrLookAheadMatcher;
pub use trivial_lookahead_matcher::TrivialLookAheadMatcher;

use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr, NO_STATE_ID};

mod label_lookahead_matcher;
pub(super) mod label_lookahead_relabeler;
mod tr_lookahead_matcher;
mod trivial_lookahead_matcher;

pub trait MatcherFlagsTrait: Debug + Clone {
    fn flags() -> MatcherFlags;
}

#[derive(Clone, Debug)]
pub struct LookAheadMatcherData<W: Semiring> {
    pub lookahead_weight: W,
    pub prefix_tr: Tr<W>,
}

impl<W: Semiring> Default for LookAheadMatcherData<W> {
    fn default() -> Self {
        LookAheadMatcherData::new(W::one(), Tr::new(0, 0, W::one(), NO_STATE_ID))
    }
}

impl<W: Semiring> LookAheadMatcherData<W> {
    pub fn new(lookahead_weight: W, prefix_tr: Tr<W>) -> Self {
        Self {
            lookahead_weight,
            prefix_tr,
        }
    }

    pub fn clear_lookahead_weight(&mut self) {
        self.lookahead_weight = W::one();
    }
    pub fn set_lookahead_weight(&mut self, weight: W) {
        self.lookahead_weight = weight;
    }
    pub fn clear_lookahead_prefix(&mut self) {
        self.prefix_tr.nextstate = NO_STATE_ID;
    }
    pub fn set_lookahead_prefix(&mut self, tr: Tr<W>) {
        self.prefix_tr = tr;
    }
    pub fn default_lookahead_prefix(&self, tr: &mut Tr<W>) -> bool {
        if self.prefix_tr.nextstate != NO_STATE_ID {
            *tr = self.prefix_tr.clone();
            true
        } else {
            false
        }
    }
}

pub trait LookaheadMatcher<W: Semiring, F: Fst<W>, B: Borrow<F>>: Matcher<W, F, B> {
    type MatcherData: Clone;
    fn data(&self) -> Option<&Arc<Self::MatcherData>>;

    fn new_with_data(
        fst: B,
        match_type: MatchType,
        data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn create_data<F2: Fst<W>, BF2: Borrow<F2>>(
        fst: BF2,
        match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>>;

    fn init_lookahead_fst<LF: Fst<W>, BLF: Borrow<LF> + Clone>(&mut self, lfst: &BLF)
        -> Result<()>;
    // Are there paths from a state in the lookahead FST that can be read from
    // the curent matcher state?

    fn lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(
        &self,
        matcher_state: StateId,
        lfst: &BLF,
        lfst_state: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>>;

    // Can the label be read from the current matcher state after possibly
    // following epsilon transitions?
    fn lookahead_label(&self, state: StateId, label: Label) -> Result<bool>;
    fn lookahead_prefix(&self, tr: &mut Tr<W>, la_matcher_data: &LookAheadMatcherData<W>) -> bool;
}
