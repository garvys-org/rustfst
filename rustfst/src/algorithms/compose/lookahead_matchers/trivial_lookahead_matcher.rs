use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr};

#[derive(Debug, Clone)]
pub struct TrivialLookAheadMatcher<W, M> {
    matcher: M,
    w: PhantomData<W>,
}

impl<W: Semiring, M: Matcher<W>> Matcher<W> for TrivialLookAheadMatcher<W, M> {
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: Arc<Self::F>, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
            w: PhantomData,
        })
    }

    fn iter(&self, state: usize, label: usize) -> Result<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Result<Option<W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        self.matcher.match_type()
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
    }

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &Arc<Self::F> {
        self.matcher.fst()
    }
}

impl<W: Semiring, M: Matcher<W>> LookaheadMatcher<W> for TrivialLookAheadMatcher<W, M> {
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        fst: Arc<Self::F>,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data<F: ExpandedFst<W>>(
        _fst: &F,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst<LF: ExpandedFst<W>>(&mut self, _lfst: &Arc<LF>) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W>>(
        &self,
        _matcher_state: StateId,
        _lfst: &Arc<LF>,
        _s: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        Ok(Some(LookAheadMatcherData::default()))
    }

    fn lookahead_label(&self, _state: StateId, _label: Label) -> Result<bool> {
        Ok(true)
    }

    fn lookahead_prefix(
        &self,
        _tr: &mut Tr<W>,
        _la_matcher_data: &LookAheadMatcherData<W>,
    ) -> bool {
        false
    }
}
