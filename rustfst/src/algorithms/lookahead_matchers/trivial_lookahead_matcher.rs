use failure::Fallible;

use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{Matcher, MatcherFlags, MatchType};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Label};

#[derive(Debug)]
struct TrivialLookAheadMatcher<M> {
    matcher: M
}

impl<'fst, F: Fst + 'fst, M: Matcher<'fst, F>> Matcher<'fst, F> for TrivialLookAheadMatcher<M> {
    type Iter = M::Iter;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self>{
        Ok(Self {
            matcher: M::new(fst, match_type)?
        })
    }

    fn iter(&self, state: usize, label: usize) -> Fallible<Self::Iter> {
        self.matcher.iter(state, label)
    }

    fn final_weight(&self, state: usize) -> Fallible<Option<&'fst F::W>> {
        self.matcher.final_weight(state)
    }

    fn match_type(&self) -> MatchType {
        self.matcher.match_type()
    }

    fn flags(&self) -> MatcherFlags {
        self.matcher.flags() | MatcherFlags::INPUT_LOOKAHEAD_MATCHER | MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER
    }
}

impl<'fst, F: Fst + 'fst, M: Matcher<'fst, F>> LookaheadMatcher<'fst, F> for TrivialLookAheadMatcher<M> {
    fn lookahead_fst<LF: Fst<W=F::W>>(&self, state: StateId, lfst: &LF) -> bool {
        true
    }

    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool> {
        Ok(true)
    }

    fn lookahead_prefix(&self) -> bool {
        false
    }

    fn lookahead_weight(&self) -> F::W {
        F::W::one()
    }
}