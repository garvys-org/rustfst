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
use crate::semirings::Semiring;
use crate::{StateId, Tr};

mod alt_sequence_compose_filter;
mod match_compose_filter;
mod multi_eps_filter;
mod no_match_compose_filter;
mod null_compose_filter;
mod sequence_compose_filter;
mod trivial_compose_filter;

pub trait ComposeFilterBuilder<W: Semiring>: Debug {
    type CF: ComposeFilter<W>;
    type M1: Matcher<W>;
    type M2: Matcher<W>;
    fn new(
        fst1: Arc<<<Self::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<Self::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        matcher1: Option<Self::M1>,
        matcher2: Option<Self::M2>,
    ) -> Result<Self>
    where
        Self: Sized;

    fn build(&self) -> Result<Self::CF>;
}

/// Composition filters determine which matches are allowed to proceed. The
/// filter's state is represented by the type ComposeFilter::FS.
pub trait ComposeFilter<W: Semiring>: Debug + Clone {
    type M1: Matcher<W>;
    type M2: Matcher<W>;
    type FS: FilterState;

    fn start(&self) -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()>;

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS>;

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()>;

    fn matcher1(&self) -> &Self::M1;
    fn matcher2(&self) -> &Self::M2;
    fn matcher1_shared(&self) -> &Arc<Self::M1>;
    fn matcher2_shared(&self) -> &Arc<Self::M2>;
    fn fst1(&self) -> &Arc<<Self::M1 as Matcher<W>>::F> {
        self.matcher1().fst()
    }
    fn fst2(&self) -> &Arc<<Self::M2 as Matcher<W>>::F> {
        self.matcher2().fst()
    }
}
