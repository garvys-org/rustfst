use std::fmt::Debug;

use crate::algorithms::compose::matchers::MatchType;

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

pub trait MatchTypeTrait: Debug + Clone {
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

#[derive(Clone, Debug, Copy)]
pub enum Selector {
    Fst1Matcher2,
    Fst2Matcher1,
}

pub(crate) fn selector(match_type: MatchType, lookahead_type: MatchType) -> Selector {
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
