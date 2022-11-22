use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{
    AltSequenceComposeFilterBuilder, MatchComposeFilterBuilder, NoMatchComposeFilterBuilder,
    NullComposeFilterBuilder, SequenceComposeFilterBuilder, TrivialComposeFilterBuilder,
};
use crate::algorithms::compose::matchers::{Matcher, SigmaMatcher, SortedMatcher};
use crate::algorithms::compose::ComposeFst;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::prelude::compose::matchers::{MatchType, MatcherRewriteMode};
use crate::semirings::Semiring;
use crate::Label;

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum ComposeFilterEnum {
    AutoFilter,
    NullFilter,
    TrivialFilter,
    SequenceFilter,
    AltSequenceFilter,
    MatchFilter,
    NoMatchFilter,
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum MatcherEnum {
    SortedMatcher,
    SigmaMatcher,
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct SigmaMatcherConfig {
    pub sigma_label: Label,
    pub rewrite_mode: MatcherRewriteMode,
}

#[derive(Default, PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct MatcherConfig {
    sigma_matcher_config: Option<SigmaMatcherConfig>,
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub struct ComposeConfig {
    pub compose_filter: ComposeFilterEnum,
    pub matcher1_config: MatcherConfig,
    pub matcher2_config: MatcherConfig,
    pub connect: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_filter: ComposeFilterEnum::AutoFilter,
            matcher1_config: MatcherConfig::default(),
            matcher2_config: MatcherConfig::default(),
            connect: true,
        }
    }
}

macro_rules! macro_compose {
    ($fst1: expr, $fst2: expr, $lol: tt) => {
        ComposeFst::<
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            $lol<_, _, _, _, _, SortedMatcher<_, _, _>, SortedMatcher<_, _, _>>,
        >::new($fst1, $fst2)?
        .compute()?
    };
}

pub fn compose_with_config<
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
    F3: MutableFst<W> + AllocableFst<W>,
>(
    fst1: B1,
    fst2: B2,
    config: ComposeConfig,
) -> Result<F3> {
    // let matcher2 = SortedMatcher::new(fst2.borrow(), MatchType::MatchInput)?;
    // if let Some(sigma_config) = config.matcher2_config.sigma_matcher_config {
    //     let matcher2 = SigmaMatcher::new(
    //         fst2.borrow(), MatchType::MatchInput,
    //         sigma_config.sigma_label, sigma_config.rewrite_mode,
    //         Arc::new(matcher2)
    //     )?;
    // }
    let mut ofst: F3 = match config.compose_filter {
        ComposeFilterEnum::AutoFilter => ComposeFst::new_auto(fst1, fst2)?.compute()?,
        ComposeFilterEnum::NullFilter => macro_compose!(fst1, fst2, NullComposeFilterBuilder),
        ComposeFilterEnum::SequenceFilter => {
            macro_compose!(fst1, fst2, SequenceComposeFilterBuilder)
        }
        ComposeFilterEnum::AltSequenceFilter => {
            macro_compose!(fst1, fst2, AltSequenceComposeFilterBuilder)
        }
        ComposeFilterEnum::MatchFilter => macro_compose!(fst1, fst2, MatchComposeFilterBuilder),
        ComposeFilterEnum::NoMatchFilter => macro_compose!(fst1, fst2, NoMatchComposeFilterBuilder),
        ComposeFilterEnum::TrivialFilter => macro_compose!(fst1, fst2, TrivialComposeFilterBuilder),
    };

    if config.connect {
        crate::algorithms::connect(&mut ofst)?;
    }

    Ok(ofst)
}

/// This operation computes the composition of two transducers.
/// If `A` transduces string `x` to `y` with weight `a` and `B` transduces `y` to `z`
/// with weight `b`, then their composition transduces string `x` to `z` with weight `a ⊗ b`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use anyhow::Result;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::compose::compose;
/// # use std::sync::Arc;
/// # fn main() -> Result<()> {
/// let fst_1 : VectorFst<IntegerWeight> = fst![1,2 => 2,3];
///
/// let fst_2 : VectorFst<IntegerWeight> = fst![2,3 => 3,4];
///
/// let fst_ref : VectorFst<IntegerWeight> = fst![1,2 => 3,4];
///
/// let composed_fst : VectorFst<_> = compose(fst_1, fst_2)?;
/// assert_eq!(composed_fst, fst_ref);
/// # Ok(())
/// # }
/// ```
pub fn compose<
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    F3: MutableFst<W> + AllocableFst<W>,
    B1: Borrow<F1> + Debug + Clone,
    B2: Borrow<F2> + Debug + Clone,
>(
    fst1: B1,
    fst2: B2,
) -> Result<F3> {
    let config = ComposeConfig::default();
    compose_with_config(fst1, fst2, config)
}
