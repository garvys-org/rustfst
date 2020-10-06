use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId, Tr};

#[derive(Debug, Clone)]
pub struct TrivialLookAheadMatcher<W, M> {
    matcher: M,
    w: PhantomData<W>,
}

impl<W: Semiring, F: Fst<W>, M: Matcher<W, F>> Matcher<W, F> for TrivialLookAheadMatcher<W, M> {
    type Iter = M::Iter;

    fn new(fst: Arc<F>, match_type: MatchType) -> Result<Self> {
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

    fn match_type(&self, test: bool) -> Result<MatchType> {
        self.matcher.match_type(test)
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags()
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
    }

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &Arc<F> {
        self.matcher.fst()
    }
}

impl<W: Semiring, F: Fst<W>, M: Matcher<W, F>> LookaheadMatcher<W, F> for TrivialLookAheadMatcher<W, M> {
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        fst: Arc<F>,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data<F2: Fst<W>>(
        _fst: &F2,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst<LF: Fst<W>>(&mut self, _lfst: &Arc<LF>) -> Result<()> {
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
