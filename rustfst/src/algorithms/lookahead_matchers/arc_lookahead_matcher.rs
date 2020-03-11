use failure::Fallible;

use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::{Label, StateId};
use crate::semirings::Semiring;

#[derive(Debug)]
struct ArcLookAheadMatcher<M> {
    matcher: M,

    // Flags to customize the behaviour
    flags: MatcherFlags,
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> Matcher<'fst, W> for ArcLookAheadMatcher<M> {
    type F = M::F;
    type Iter = M::Iter;

    fn new(fst: &'fst Self::F, match_type: MatchType) -> Fallible<Self> {
        Ok(Self {
            matcher: M::new(fst, match_type)?,
            flags: MatcherFlags::LOOKAHEAD_NON_EPSILONS
                | MatcherFlags::LOOKAHEAD_EPSILONS
                | MatcherFlags::LOOKAHEAD_WEIGHT
                | MatcherFlags::LOOKAHEAD_PREFIX,
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
            | self.flags
    }
}

impl<'fst, W: Semiring + 'fst, M: Matcher<'fst, W>> LookaheadMatcher<'fst, W>
    for ArcLookAheadMatcher<M>
{
    fn lookahead_fst<LF: Fst<W = W>>(&self, state: StateId, lfst: &LF) -> bool {
        let mut result = false;
        unimplemented!()
    }

    fn lookahead_label(&self, state: StateId, label: Label) -> Fallible<bool> {
        let mut it = self.matcher.iter(state, label)?;
        Ok(it.next().is_some())
    }

    fn lookahead_prefix(&self) -> bool {
        unimplemented!()
    }

    fn lookahead_weight(&self) -> W {
        unimplemented!()
    }
}
