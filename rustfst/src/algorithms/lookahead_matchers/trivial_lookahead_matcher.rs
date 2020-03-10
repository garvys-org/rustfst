use failure::Fallible;

use crate::algorithms::matchers::{Matcher, MatcherFlags, MatchType};
use crate::fst_traits::Fst;

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