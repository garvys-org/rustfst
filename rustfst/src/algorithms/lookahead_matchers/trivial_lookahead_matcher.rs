use failure::Fallible;

use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId};

#[derive(Debug)]
struct TrivialLookAheadMatcher<M> {
    matcher: M,
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> Matcher<'fst, W> for TrivialLookAheadMatcher<M> {
    type Iter = M::Iter;

    fn new<F: Fst<W=W>>(fst: &'fst F, match_type: MatchType) -> Fallible<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
        })
    }

    fn iter(&self, state: usize, label: usize) -> Fallible<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst W>> {
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
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> LookaheadMatcher<'fst, W>
    for TrivialLookAheadMatcher<M>
{
    fn lookahead_fst<LF: Fst<W = W>>(&self, state: StateId, lfst: &LF) -> bool {
        true
    }

    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool> {
        Ok(true)
    }

    fn lookahead_prefix(&self) -> bool {
        false
    }

    fn lookahead_weight(&self) -> W {
        W::one()
    }
}
