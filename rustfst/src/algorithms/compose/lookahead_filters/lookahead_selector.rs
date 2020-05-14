use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;

use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;

#[derive(Clone, Debug)]
pub struct SMatchInput {}

#[derive(Clone, Debug)]
pub struct SMatchOutput {}

#[derive(Clone, Debug)]
pub struct SMatchBoth {}

#[derive(Clone, Debug)]
pub struct SMatchNone {}

#[derive(Clone, Debug)]
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

// #[derive(Clone, Debug)]
// pub struct LookAheadSelector<F, M> {
//     pub fst: Arc<F>,
//     pub matcher: Arc<M>,
// }
//
// fn selector_match_input<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
//     lmatcher1: &Arc<M1>,
//     lmatcher2: &Arc<M2>,
// ) -> LookAheadSelector<M1::F, M2> {
//     LookAheadSelector {
//         fst: lmatcher1.fst(),
//         matcher: Arc::clone(lmatcher2),
//     }
// }
//
// fn selector_match_output<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
//     lmatcher1: &Arc<M1>,
//     lmatcher2: &Arc<M2>,
// ) -> LookAheadSelector<M2::F, M1> {
//     LookAheadSelector {
//         fst: lmatcher2.fst(),
//         matcher: Arc::clone(lmatcher1),
//     }
// }

#[derive(Clone, Debug)]
pub enum Selector {
    Fst1Matcher2,
    Fst2Matcher1,
}

pub(crate) fn selector<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
    match_type: MatchType,
    lookahead_type: MatchType,
) -> Selector {
    match match_type {
        MatchType::MatchInput => Selector::Fst1Matcher2,
        MatchType::MatchOutput => Selector::Fst2Matcher1,
        _ => {
            if lookahead_type == MatchType::MatchOutput {
                Selector::Fst2Matcher1
            } else {
                Selector::Fst1Matcher2
            }
        }
    }
}
