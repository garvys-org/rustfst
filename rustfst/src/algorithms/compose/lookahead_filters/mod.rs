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

pub fn lookahead_match_type<W: Semiring, M1: Matcher<W>, M2: Matcher<W>>(
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

pub fn lookahead_match_type_2<'fst, W: Semiring + 'fst, F1: Fst<W> + 'fst, F2: Fst<W> + 'fst>(
) -> MatchType {
    unimplemented!()
}

pub trait LookAheadComposeFilterTrait<W: Semiring>: ComposeFilter<W>
where
    Self::M1: LookaheadMatcher<W>,
    Self::M2: LookaheadMatcher<W>,
{
    fn lookahead_flags(&self) -> MatcherFlags;
    fn lookahead_tr(&self) -> bool;
    fn lookahead_type(&self) -> MatchType;
    fn lookahead_output(&self) -> bool;
    fn selector(&self) -> &Selector;
    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>>;
}
