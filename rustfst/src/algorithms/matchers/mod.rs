use std::fmt::Debug;

use failure::Fallible;

use bitflags::bitflags;
pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Arc, EPS_LABEL, NO_LABEL};
use crate::{Label, StateId};

mod sorted_matcher;

bitflags! {
    pub struct MatcherFlags: u32 {
        const REQUIRE_MATCH =  1u32 << 0;
        const INPUT_LOOKAHEAD_MATCHER =  1u32 << 1;
        const OUTPUT_LOOKAHEAD_MATCHER =  1u32 << 2;
        const LOOKAHEAD_WEIGHT =  1u32 << 3;
        const LOOKAHEAD_PREFIX =  1u32 << 4;
        const LOOKAHEAD_NON_EPSILONS =  1u32 << 5;
        const LOOKAHEAD_EPSILONS =  1u32 << 6;
        const LOOKAHEAD_NON_EPSILON_PREFIX =  1u32 << 7;
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

// Use this to avoid autoref
pub enum IterItemMatcher<'a, W: Semiring> {
    Arc(&'a Arc<W>),
    EpsLoop,
}

impl<'a, W: Semiring> IterItemMatcher<'a, W> {
    pub fn into_arc(self, state: StateId, match_type: MatchType) -> Fallible<Arc<W>> {
        match self {
            IterItemMatcher::Arc(arc) => Ok(arc.clone()),
            IterItemMatcher::EpsLoop => eps_loop(state, match_type),
        }
    }
}

pub fn eps_loop<W: Semiring>(state: StateId, match_type: MatchType) -> Fallible<Arc<W>> {
    let arc = match match_type {
        MatchType::MatchInput => Arc::new(NO_LABEL, EPS_LABEL, W::one(), state),
        MatchType::MatchOutput => Arc::new(EPS_LABEL, NO_LABEL, W::one(), state),
        _ => bail!("Unsupported match_type : {:?}", match_type),
    };
    Ok(arc)
}

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<'fst, F: Fst + 'fst>: Debug {
    type Iter: Iterator<Item = IterItemMatcher<'fst, F::W>>;

    fn new(fst: &'fst F, match_type: MatchType) -> Fallible<Self>
    where
        Self: std::marker::Sized;
    fn iter(&self, state: StateId, label: Label) -> Fallible<Self::Iter>;
    fn final_weight(&self, state: StateId) -> Fallible<Option<&'fst F::W>>;
    fn match_type(&self) -> MatchType;
    fn flags(&self) -> MatcherFlags;
}
