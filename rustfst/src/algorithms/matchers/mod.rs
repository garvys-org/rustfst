use failure::Fallible;

pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::Arc;
use crate::{Label, StateId};

mod sorted_matcher;

#[derive(Copy, Debug, PartialOrd, PartialEq, Clone)]
/// Specifies matcher action
pub enum MatchType {
    /// Match input label
    MatchInput,
    /// Match output label
    MatchOutput,
    /// Match input or output label
    MatchBoth,
    /// Match anything
    MatchNone,
    /// Otherwise, match unknown
    MatchUnknown,
}

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<'matcher, 'fst: 'matcher, F: Fst + 'fst> {
    type W: Semiring + 'fst;
    type Iter: Iterator<Item = &'matcher Arc<Self::W>>;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self>
    where
        Self: std::marker::Sized;
    fn iter(&'matcher mut self, state: StateId, label: Label) -> Fallible<Self::Iter>;
    fn final_weight(&self, state: StateId) -> Fallible<Option<&'matcher F::W>>;
}
