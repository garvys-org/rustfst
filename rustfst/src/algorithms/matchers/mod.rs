use failure::Fallible;

// pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::Arc;
use crate::{Label, StateId};

// mod sorted_matcher;

use bitflags::bitflags;

bitflags! {
    pub struct MatcherFlags: u32 {
        const REQUIRE_MATCH =  0b00001;
    }
}

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
pub trait Matcher<'iter, 'fst: 'iter, F: Fst + 'fst> {
    type Iter: Iterator<Item = &'iter Arc<F::W>>;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self>
    where
        Self: std::marker::Sized;
    fn iter(&mut self, state: StateId, label: Label) -> Fallible<Self::Iter>;
    fn final_weight(&self, state: StateId) -> Fallible<Option<&'iter F::W>>;
    fn match_type(&self) -> MatchType;
    fn flags(&self) -> MatcherFlags;
}
