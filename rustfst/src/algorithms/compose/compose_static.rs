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
use crate::fst_traits::{AllocableFst, ExpandedFst, Fst, MutableFst};
use crate::prelude::compose::matchers::{MatchType, MatcherRewriteMode};
use crate::prelude::compose::ComposeFstOpOptions;
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

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub struct SigmaMatcherConfig {
    pub sigma_label: Label,
    pub rewrite_mode: MatcherRewriteMode,
    pub sigma_allowed_matches: Option<Vec<Label>>,
}

#[derive(Default, PartialEq, PartialOrd, Debug, Clone)]
pub struct MatcherConfig {
    pub sigma_matcher_config: Option<SigmaMatcherConfig>,
}

impl MatcherConfig {
    pub fn empty(&self) -> bool {
        self.sigma_matcher_config.is_none()
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone)]
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

#[derive(Clone)]
pub enum MatcherEnum<W, F, B>
where
    W: Semiring,
    F: Fst<W>,
    B: Borrow<F> + Debug,
{
    SortedMatcher(SortedMatcher<W, F, B>),
    SigmaMatcher(SigmaMatcher<W, F, B, SortedMatcher<W, F, B>>),
}

impl MatcherConfig {
    pub fn create_matcher<W, F, B>(
        &self,
        fst: B,
        match_type: MatchType,
    ) -> Result<MatcherEnum<W, F, B>>
    where
        W: Semiring,
        F: Fst<W>,
        B: Borrow<F> + Debug,
    {
        if self.sigma_matcher_config.is_none() {
            let matcher = SortedMatcher::new(fst, match_type)?;

            Ok(MatcherEnum::SortedMatcher(matcher))
        } else {
            let sigma_config = self.sigma_matcher_config.clone().unwrap();
            let matcher = SortedMatcher::new(fst, match_type)?;
            let matcher = SigmaMatcher::new(
                match_type,
                sigma_config.sigma_label,
                sigma_config.rewrite_mode,
                Arc::new(matcher),
                sigma_config
                    .sigma_allowed_matches
                    .map(|e| e.iter().cloned().collect()),
            )?;

            Ok(MatcherEnum::SigmaMatcher(matcher))
        }
    }
}

macro_rules! run_compose {
    (
        $fst1: expr, $fst2: expr,
        $f1: ty, $f2: ty,
        $builder: tt,
        $matcher1: expr, $matcher1_ty: ty,
        $matcher2: expr, $matcher2_ty: ty
    ) => {{
        let compose_fst_op_opts = ComposeFstOpOptions::new($matcher1, $matcher2, None, None);
        ComposeFst::<
            _,
            $f1,
            $f2,
            &$f1,
            &$f2,
            _,
            _,
            $builder<_, _, _, _, _, $matcher1_ty, $matcher2_ty>,
        >::new_with_options($fst1, $fst2, compose_fst_op_opts)?
        .compute()?
    }};
}

macro_rules! compose_generate_matchers {
    (
        $fst1: expr, $fst2: expr, $f1: ty, $f2: ty,
        $builder: tt, $matcher1_enum: expr, $matcher2_enum: expr
    ) => {
        {
            match ($matcher1_enum, $matcher2_enum) {
                (MatcherEnum::SortedMatcher(m1), MatcherEnum::SortedMatcher(m2)) => {
                    run_compose!(
                        $fst1.borrow(), $fst2.borrow(), $f1, $f2, $builder, Some(m1), SortedMatcher<_, _, _>, Some(m2), SortedMatcher<_,_,_>
                    )
                },
                (MatcherEnum::SigmaMatcher(m1), MatcherEnum::SortedMatcher(m2)) => {
                    run_compose!(
                        $fst1.borrow(), $fst2.borrow(), $f1, $f2, $builder, Some(m1), SigmaMatcher<_, _, _, _>, Some(m2), SortedMatcher<_,_,_>
                    )
                },
                (MatcherEnum::SortedMatcher(m1), MatcherEnum::SigmaMatcher(m2)) => {
                    run_compose!(
                        $fst1.borrow(), $fst2.borrow(), $f1, $f2, $builder, Some(m1), SortedMatcher<_, _, _>, Some(m2), SigmaMatcher<_,_,_,_>
                    )
                },
                (MatcherEnum::SigmaMatcher(m1), MatcherEnum::SigmaMatcher(m2)) => {
                    run_compose!(
                        $fst1.borrow(), $fst2.borrow(), $f1, $f2, $builder, Some(m1), SigmaMatcher<_, _, _, _>, Some(m2), SigmaMatcher<_,_,_,_>
                    )
                }
            }
        }
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
    let matcher1 = config
        .matcher1_config
        .create_matcher(fst1.borrow(), MatchType::MatchOutput)?;
    let matcher2 = config
        .matcher2_config
        .create_matcher(fst2.borrow(), MatchType::MatchInput)?;

    let mut ofst: F3 = match config.compose_filter {
        ComposeFilterEnum::AutoFilter => {
            if config.matcher1_config.empty() && config.matcher2_config.empty() {
                ComposeFst::new_auto(fst1, fst2)?.compute()?
            } else {
                bail!("Custom MatcherConfig not supported with AutoFilter")
            }
        }
        ComposeFilterEnum::NullFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                NullComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
        ComposeFilterEnum::SequenceFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                SequenceComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
        ComposeFilterEnum::AltSequenceFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                AltSequenceComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
        ComposeFilterEnum::MatchFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                MatchComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
        ComposeFilterEnum::NoMatchFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                NoMatchComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
        ComposeFilterEnum::TrivialFilter => {
            compose_generate_matchers!(
                fst1,
                fst2,
                F1,
                F2,
                TrivialComposeFilterBuilder,
                matcher1,
                matcher2
            )
        }
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
