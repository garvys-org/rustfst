use crate::algorithms::matchers::{MatchType, Matcher};

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use failure::_core::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug)]
pub struct SMatchInput {}

#[derive(Debug)]
pub struct SMatchOutput {}

#[derive(Debug)]
pub struct SMatchBoth {}

#[derive(Debug)]
pub struct SMatchNone {}

#[derive(Debug)]
pub struct SMatchUnknown {}

pub trait MatchTypeTrait: Debug {
    fn match_type() -> MatchType;
}

impl MatchTypeTrait for SMatchInput {
    fn match_type() -> MatchType {
        MatchType::MatchInput
    }
}

impl MatchTypeTrait for SMatchOutput {
    fn match_type() -> MatchType {
        MatchType::MatchOutput
    }
}

impl MatchTypeTrait for SMatchBoth {
    fn match_type() -> MatchType {
        MatchType::MatchBoth
    }
}

impl MatchTypeTrait for SMatchNone {
    fn match_type() -> MatchType {
        MatchType::MatchNone
    }
}

impl MatchTypeTrait for SMatchUnknown {
    fn match_type() -> MatchType {
        MatchType::MatchUnknown
    }
}

pub struct LookAheadSelector<'fst, F, M> {
    pub fst: &'fst F,
    pub matcher: Rc<RefCell<M>>,
}

pub fn selector_match_input<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
) -> LookAheadSelector<'fst1, M1::F, M2> {
    LookAheadSelector {
        fst: lmatcher1.borrow().fst(),
        matcher: lmatcher2,
    }
}

pub fn selector_match_output<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
) -> LookAheadSelector<'fst2, M2::F, M1> {
    LookAheadSelector {
        fst: lmatcher2.borrow().fst(),
        matcher: lmatcher1,
    }
}

pub fn selector<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
    FN1,
    FN2,
    T,
>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
    match_type: MatchType,
    lookahead_type: MatchType,
    f1: FN1,
    f2: FN2,
) -> T
where
    FN1: Fn(LookAheadSelector<'fst1, M1::F, M2>) -> T,
    FN2: Fn(LookAheadSelector<'fst2, M2::F, M1>) -> T,
{
    match match_type {
        MatchType::MatchInput => f1(selector_match_input::<'fst1, 'fst2, W, M1, M2>(
            lmatcher1, lmatcher2,
        )),
        MatchType::MatchOutput => f2(selector_match_output(lmatcher1, lmatcher2)),
        _ => {
            if lookahead_type == MatchType::MatchOutput {
                f2(selector_match_output(lmatcher1, lmatcher2))
            } else {
                f1(selector_match_input(lmatcher1, lmatcher2))
            }
        }
    }
}
