use crate::algorithms::matchers::Matcher;
use crate::fst_traits::Fst;

mod trivial_lookahead_matcher;

pub trait LookaheadMatcher<'fst, F: Fst + 'fst>: Matcher<'fst, F> {}
