use crate::algorithms::matchers::{Matcher, MatchType, MatcherFlags};
use crate::fst_traits::Fst;

pub fn lookahead_match_type<'fst1, 'fst2, F1: Fst + 'fst1, F2: Fst + 'fst2, M1: Matcher<'fst1, F1>, M2: Matcher<'fst2, F2>>(m1: &M1, m2: &M2) -> MatchType {
    let type1 = m1.match_type();
    let type2 = m2.match_type();
    if type1 == MatchType::MatchOutput && m1.flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER) {
        MatchType::MatchOutput
    } else if type2 == MatchType::MatchInput && m2.flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER) {
        MatchType::MatchInput
    }  else {
        MatchType::MatchNone
    }
}