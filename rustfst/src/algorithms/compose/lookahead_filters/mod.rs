use std::borrow::Borrow;

use anyhow::Result;

pub use lookahead_compose_filter::{LookAheadComposeFilter, LookAheadComposeFilterBuilder};
pub use lookahead_selector::{SMatchBoth, SMatchInput, SMatchNone, SMatchOutput, SMatchUnknown};
pub use push_labels_compose_filter::{PushLabelsComposeFilter, PushLabelsComposeFilterBuilder};
pub use push_weights_compose_filter::{PushWeightsComposeFilter, PushWeightsComposeFilterBuilder};

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::Fst;
use crate::semirings::Semiring;

mod lookahead_compose_filter;
pub mod lookahead_selector;
mod push_labels_compose_filter;
mod push_weights_compose_filter;

pub fn lookahead_match_type<
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1>,
    B2: Borrow<F2>,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
>(
    m1: &M1,
    m2: &M2,
) -> Result<MatchType> {
    let type1 = m1.match_type(false)?;
    let type2 = m2.match_type(false)?;
    if type1 == MatchType::MatchOutput
        && m1.flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER)
    {
        Ok(MatchType::MatchOutput)
    } else if type2 == MatchType::MatchInput
        && m2.flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER)
    {
        Ok(MatchType::MatchInput)
    } else if m1.flags().contains(MatcherFlags::OUTPUT_LOOKAHEAD_MATCHER)
        && m1.match_type(true)? == MatchType::MatchOutput
    {
        Ok(MatchType::MatchOutput)
    } else if m2.flags().contains(MatcherFlags::INPUT_LOOKAHEAD_MATCHER)
        && m2.match_type(true)? == MatchType::MatchInput
    {
        Ok(MatchType::MatchInput)
    } else {
        Ok(MatchType::MatchNone)
    }
}

pub fn lookahead_match_type_2<'fst, W: Semiring + 'fst, F1: Fst<W> + 'fst, F2: Fst<W> + 'fst>(
) -> MatchType {
    unimplemented!()
}

pub trait LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>:
    ComposeFilter<W, F1, F2, B1, B2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1>,
    B2: Borrow<F2>,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
{
    fn lookahead_flags(&self) -> MatcherFlags;
    fn lookahead_tr(&self) -> bool;
    fn lookahead_type(&self) -> MatchType;
    fn lookahead_output(&self) -> bool;
    fn selector(&self) -> &Selector;
    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>>;
}
