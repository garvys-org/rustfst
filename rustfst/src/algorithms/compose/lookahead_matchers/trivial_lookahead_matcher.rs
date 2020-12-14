use std::borrow::Borrow;
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

impl<W, F, B, M> Matcher<W, F, B> for TrivialLookAheadMatcher<W, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    type Iter = M::Iter;

    fn new(fst: B, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
            w: PhantomData,
        })
    }

    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: StateId) -> Result<Option<W>> {
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

    fn priority(&self, state: StateId) -> Result<usize> {
        self.matcher.priority(state)
    }

    fn fst(&self) -> &B {
        self.matcher.fst()
    }
}

impl<W, F, B, M> LookaheadMatcher<W, F, B> for TrivialLookAheadMatcher<W, M>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F>,
    M: Matcher<W, F, B>,
{
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        fst: B,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data<F2: Fst<W>, BF2: Borrow<F2>>(
        _fst: BF2,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(&mut self, _lfst: &BLF) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst<LF: Fst<W>, BLF: Borrow<LF>>(
        &self,
        _matcher_state: StateId,
        _lfst: &BLF,
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
