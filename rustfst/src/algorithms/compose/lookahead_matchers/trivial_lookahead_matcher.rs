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

impl<W: Semiring, M: Matcher<W>> Matcher<W> for TrivialLookAheadMatcher<W, M> {
    type Iter = M::Iter;

    fn new(fst: &impl Fst<W>, match_type: MatchType) -> Result<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
            w: PhantomData,
        })
    }

    fn iter(&self, fst: &impl Fst<W>, state: usize, label: usize) -> Result<Self::Iter> {
        self.matcher.iter(fst, state, label)
    }

    fn final_weight(&self, fst: &impl Fst<W>, state: usize) -> Result<Option<W>> {
        self.matcher.final_weight(fst, state)
    }

    fn match_type(&self, fst: &impl Fst<W>, test: bool) -> Result<MatchType> {
        self.matcher.match_type(fst, test)
    }

    fn flags(&self, fst: &impl Fst<W>) -> MatcherFlags {
        self.matcher.flags(fst)
            | MatcherFlags::INPUT_LOOKAHEAD_MATCHER
            | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
    }

    fn priority(&self, state: usize) -> Result<usize> {
        self.matcher.priority(state)
    }
}

impl<W: Semiring, M: Matcher<W>> LookaheadMatcher<W> for TrivialLookAheadMatcher<W, M> {
    type MatcherData = ();

    fn data(&self) -> Option<&Arc<Self::MatcherData>> {
        None
    }

    fn new_with_data(
        self_fst: &impl Fst<W>,
        fst: &impl Fst<W>,
        match_type: MatchType,
        _data: Option<Arc<Self::MatcherData>>,
    ) -> Result<Self> {
        Self::new(fst, match_type)
    }

    fn create_data(
        self_fst: &impl Fst<W>,
        _fst: &impl Fst<W>,
        _match_type: MatchType,
    ) -> Result<Option<Self::MatcherData>> {
        Ok(None)
    }

    fn init_lookahead_fst(&mut self, self_fst: &impl Fst<W>, _lfst: &impl Fst<W>) -> Result<()> {
        Ok(())
    }

    fn lookahead_fst(
        &self,
        self_fst: &impl Fst<W>,
        _matcher_state: StateId,
        _lfst: &impl Fst<W>,
        _s: StateId,
    ) -> Result<Option<LookAheadMatcherData<W>>> {
        Ok(Some(LookAheadMatcherData::default()))
    }

    fn lookahead_label(&self, self_fst: &impl Fst<W>, _state: StateId, _label: Label) -> Result<bool> {
        Ok(true)
    }

    fn lookahead_prefix(
        &self,
        self_fst: &impl Fst<W>,
        _tr: &mut Tr<W>,
        _la_matcher_data: &LookAheadMatcherData<W>,
    ) -> bool {
        false
    }
}
