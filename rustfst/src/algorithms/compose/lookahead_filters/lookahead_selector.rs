use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;

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

#[derive(Debug)]
pub struct LookAheadSelector<F, M> {
    pub fst: Rc<F>,
    pub matcher: Rc<RefCell<M>>,
}

fn selector_match_input<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
) -> LookAheadSelector<M1::F, M2> {
    LookAheadSelector {
        fst: lmatcher1.borrow().fst(),
        matcher: lmatcher2,
    }
}

fn selector_match_output<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
) -> LookAheadSelector<M2::F, M1> {
    LookAheadSelector {
        fst: lmatcher2.borrow().fst(),
        matcher: lmatcher1,
    }
}

#[derive(Debug)]
pub enum Selector<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    MatchInput(LookAheadSelector<M1::F, M2>),
    MatchOutput(LookAheadSelector<M2::F, M1>),
}

pub(crate) fn selector<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
    lmatcher1: Rc<RefCell<M1>>,
    lmatcher2: Rc<RefCell<M2>>,
    match_type: MatchType,
    lookahead_type: MatchType,
) -> Selector<W, M1, M2> {
    match match_type {
        MatchType::MatchInput => {
            Selector::MatchInput(selector_match_input::<W, M1, M2>(lmatcher1, lmatcher2))
        }
        MatchType::MatchOutput => {
            Selector::MatchOutput(selector_match_output(lmatcher1, lmatcher2))
        }
        _ => {
            if lookahead_type == MatchType::MatchOutput {
                Selector::MatchOutput(selector_match_output(lmatcher1, lmatcher2))
            } else {
                Selector::MatchInput(selector_match_input(lmatcher1, lmatcher2))
            }
        }
    }
}
