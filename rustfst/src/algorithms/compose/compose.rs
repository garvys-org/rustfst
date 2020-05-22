use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{
    AltSequenceComposeFilterBuilder, MatchComposeFilterBuilder, NoMatchComposeFilterBuilder,
    NullComposeFilterBuilder, SequenceComposeFilterBuilder, TrivialComposeFilterBuilder,
};
use crate::algorithms::compose::matchers::SortedMatcher;
use crate::algorithms::compose::ComposeFst;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;

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
pub struct ComposeConfig {
    pub compose_filter: ComposeFilterEnum,
    pub connect: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_filter: ComposeFilterEnum::AutoFilter,
            connect: true,
        }
    }
}

pub fn compose_with_config<
    W: Semiring,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    F3: MutableFst<W>,
>(
    fst1: Arc<F1>,
    fst2: Arc<F2>,
    config: ComposeConfig,
) -> Result<F3> {
    let mut ofst: F3 = match config.compose_filter {
        ComposeFilterEnum::AutoFilter => ComposeFst::new_auto(fst1, fst2)?.compute()?,
        ComposeFilterEnum::NullFilter => ComposeFst::<
            _,
            NullComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::SequenceFilter => ComposeFst::<
            _,
            SequenceComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::AltSequenceFilter => ComposeFst::<
            _,
            AltSequenceComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::MatchFilter => ComposeFst::<
            _,
            MatchComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::NoMatchFilter => ComposeFst::<
            _,
            NoMatchComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
        ComposeFilterEnum::TrivialFilter => ComposeFst::<
            _,
            TrivialComposeFilterBuilder<_, SortedMatcher<_, _>, SortedMatcher<_, _>>,
        >::new(fst1, fst2)?
        .compute()?,
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
/// let composed_fst : VectorFst<_> = compose(Arc::new(fst_1), Arc::new(fst_2))?;
/// assert_eq!(composed_fst, fst_ref);
/// # Ok(())
/// # }
/// ```
pub fn compose<W: Semiring, F1: ExpandedFst<W>, F2: ExpandedFst<W>, F3: MutableFst<W>>(
    fst1: Arc<F1>,
    fst2: Arc<F2>,
) -> Result<F3> {
    let config = ComposeConfig::default();
    compose_with_config(fst1, fst2, config)
}
