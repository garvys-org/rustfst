use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;

pub mod lookahead_compose_filter;
pub mod lookahead_selector;

pub fn lookahead_match_type<
    'fst,
    W: Semiring + 'fst,
    M1: Matcher<'fst, W>,
    M2: Matcher<'fst, W>,
>(
    m1: &M1,
    m2: &M2,
) -> MatchType {
    let type1 = m1.match_type();
    let type2 = m2.match_type();
    if type1 == MatchType::MatchOutput
        && m1.flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER)
    {
        MatchType::MatchOutput
    } else if type2 == MatchType::MatchInput
        && m2.flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER)
    {
        MatchType::MatchInput
    } else {
        MatchType::MatchNone
    }
}

pub fn lookahead_match_type_2<
    'fst,
    W: Semiring + 'fst,
    F1: Fst<W = W> + 'fst,
    F2: Fst<W = W> + 'fst,
>() -> MatchType {
    unimplemented!()
}
