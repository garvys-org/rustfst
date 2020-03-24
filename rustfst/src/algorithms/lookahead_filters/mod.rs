use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use std::cell::RefCell;
use std::rc::Rc;

pub mod lookahead_compose_filter;
pub mod lookahead_selector;

pub fn lookahead_match_type<
    'fst1, 'fst2,
    W: Semiring + 'fst1 + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
>(
    m1: Rc<RefCell<M1>>,
    m2: Rc<RefCell<M2>>,
) -> MatchType {
    let type1 = m1.borrow().match_type();
    let type2 = m2.borrow().match_type();
    if type1 == MatchType::MatchOutput
        && m1.borrow().flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER)
    {
        MatchType::MatchOutput
    } else if type2 == MatchType::MatchInput
        && m2.borrow().flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER)
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
