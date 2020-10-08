use std::borrow::Borrow;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

pub use alt_sequence_compose_filter::{AltSequenceComposeFilter, AltSequenceComposeFilterBuilder};
pub use match_compose_filter::{MatchComposeFilter, MatchComposeFilterBuilder};
pub use multi_eps_filter::MultiEpsFilter;
pub use no_match_compose_filter::{NoMatchComposeFilter, NoMatchComposeFilterBuilder};
pub use null_compose_filter::{NullComposeFilter, NullComposeFilterBuilder};
pub use sequence_compose_filter::{SequenceComposeFilter, SequenceComposeFilterBuilder};
pub use trivial_compose_filter::{TrivialComposeFilter, TrivialComposeFilterBuilder};

use crate::algorithms::compose::filter_states::FilterState;
use crate::algorithms::compose::matchers::Matcher;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr};

mod alt_sequence_compose_filter;
mod match_compose_filter;
mod multi_eps_filter;
mod no_match_compose_filter;
mod null_compose_filter;
mod sequence_compose_filter;
mod trivial_compose_filter;

pub trait ComposeFilterBuilder<W: Semiring, F1, F2, B1, B2, M1, M2>: Debug + Clone
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    type IM1: Matcher<W, F1, B1>;
    type IM2: Matcher<W, F2, B2>;

    type CF: ComposeFilter<W, F1, F2, B1, B2, Self::IM1, Self::IM2>;

    fn new(fst1: B1, fst2: B2, matcher1: Option<M1>, matcher2: Option<M2>) -> Result<Self>
    where
        Self: Sized;

    fn build(&self) -> Result<Self::CF>;
}

/// Composition filters determine which matches are allowed to proceed. The
/// filter's state is represented by the type ComposeFilter::FS.
pub trait ComposeFilter<W: Semiring, F1, F2, B1, B2, M1, M2>: Debug
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1>,
    B2: Borrow<F2>,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    type FS: FilterState;

    fn start(&self) -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()>;

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS>;

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()>;

    fn matcher1(&self) -> &M1;
    fn matcher2(&self) -> &M2;
    fn matcher1_shared(&self) -> &Arc<M1>;
    fn matcher2_shared(&self) -> &Arc<M2>;

    fn properties(&self, inprops: FstProperties) -> FstProperties;
}
