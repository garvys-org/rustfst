use std::fmt::Debug;

use anyhow::Result;

use bitflags::bitflags;
pub use generic_matcher::GenericMatcher;
pub use multi_eps_matcher::{MultiEpsMatcher, MultiEpsMatcherFlags};
pub use sigma_matcher::SigmaMatcher;
pub use sorted_matcher::SortedMatcher;

use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Label, StateId};
use crate::{Tr, EPS_LABEL, NO_LABEL};
use std::borrow::Borrow;

mod generic_matcher;
mod multi_eps_matcher;
mod sigma_matcher;
mod sorted_matcher;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MatcherFlags: u32 {
        const REQUIRE_MATCH =  1u32;
        const INPUT_LOOKAHEAD_MATCHER =  1u32 << 4;
        const OUTPUT_LOOKAHEAD_MATCHER =  1u32 << 5;
        const LOOKAHEAD_WEIGHT =  1u32 << 6;
        const LOOKAHEAD_PREFIX =  1u32 << 7;
        const LOOKAHEAD_NON_EPSILONS =  1u32 << 8;
        const LOOKAHEAD_EPSILONS =  1u32 << 9;
        const LOOKAHEAD_NON_EPSILON_PREFIX =  1u32 << 10;

        const LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits() |
            Self::OUTPUT_LOOKAHEAD_MATCHER.bits() |
            Self::LOOKAHEAD_WEIGHT.bits() |
            Self::LOOKAHEAD_PREFIX.bits() |
            Self::LOOKAHEAD_NON_EPSILONS.bits() |
            Self::LOOKAHEAD_EPSILONS.bits() |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits();

        const ILABEL_LOOKAHEAD_FLAGS = Self::INPUT_LOOKAHEAD_MATCHER.bits() |
            Self::LOOKAHEAD_WEIGHT.bits() |
            Self::LOOKAHEAD_PREFIX.bits() |
            Self::LOOKAHEAD_EPSILONS.bits() |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits();

        const OLABEL_LOOKAHEAD_FLAGS = Self::OUTPUT_LOOKAHEAD_MATCHER.bits() |
            Self::LOOKAHEAD_WEIGHT.bits() |
            Self::LOOKAHEAD_PREFIX.bits() |
            Self::LOOKAHEAD_EPSILONS.bits() |
            Self::LOOKAHEAD_NON_EPSILON_PREFIX.bits();
    }
}

pub static REQUIRE_PRIORITY: usize = std::usize::MAX;

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

#[derive(Copy, Debug, PartialOrd, PartialEq, Clone)]
/// Specifies whether we rewrite both the input and output sides during matching.
pub enum MatcherRewriteMode {
    /// Rewrites both sides iff acceptor.
    MatcherRewriteAuto,
    MatcherRewriteAlways,
    MatcherRewriteNever,
}

// Use this to avoid autoref
#[derive(Clone, Debug)]
pub enum IterItemMatcher<W: Semiring> {
    Tr(Tr<W>),
    EpsLoop,
}

impl<W: Semiring> IterItemMatcher<W> {
    pub fn into_tr(self, state: StateId, match_type: MatchType) -> Result<Tr<W>> {
        match self {
            IterItemMatcher::Tr(tr) => Ok(tr),
            IterItemMatcher::EpsLoop => eps_loop(state, match_type),
        }
    }
}

pub fn eps_loop<W: Semiring>(state: StateId, match_type: MatchType) -> Result<Tr<W>> {
    let tr = match match_type {
        MatchType::MatchInput => Tr::new(NO_LABEL, EPS_LABEL, W::one(), state),
        MatchType::MatchOutput => Tr::new(EPS_LABEL, NO_LABEL, W::one(), state),
        _ => bail!("Unsupported match_type : {:?}", match_type),
    };
    Ok(tr)
}

/// Matchers find and iterate through requested labels at FST states. In the
/// simplest form, these are just some associative map or search keyed on labels.
/// More generally, they may implement matching special labels that represent
/// sets of labels such as sigma (all), rho (rest), or phi (fail).
pub trait Matcher<W: Semiring, F: Fst<W>, B: Borrow<F>>: Debug {
    type Iter: Iterator<Item = IterItemMatcher<W>>;

    fn new(fst: B, match_type: MatchType) -> Result<Self>
    where
        Self: std::marker::Sized;
    fn iter(&self, state: StateId, label: Label) -> Result<Self::Iter>;
    fn final_weight(&self, state: StateId) -> Result<Option<W>>;
    fn match_type(&self, test: bool) -> Result<MatchType>;
    fn flags(&self) -> MatcherFlags;

    /// Indicates preference for being the side used for matching in
    /// composition. If the value is kRequirePriority, then it is
    /// mandatory that it be used. Calling this method without passing the
    /// current state of the matcher invalidates the state of the matcher.
    fn priority(&self, state: StateId) -> Result<usize>;

    fn fst(&self) -> &B;
}
