use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

pub use alt_sequence_compose_filter::AltSequenceComposeFilter;
pub use match_compose_filter::MatchComposeFilter;
pub use multi_eps_filter::MultiEpsFilter;
pub use no_match_compose_filter::NoMatchComposeFilter;
pub use null_compose_filter::NullComposeFilter;
pub use sequence_compose_filter::SequenceComposeFilter;
pub use trivial_compose_filter::TrivialComposeFilter;

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

#[derive(Debug)]
pub struct ComposeFilterBuilder<W: Semiring, CF: ComposeFilter<W>> {
    fst1: Arc<<CF::M1 as Matcher<W>>::F>,
    fst2: Arc<<CF::M2 as Matcher<W>>::F>,
    matcher1: Option<Arc<RefCell<CF::M1>>>,
    matcher2: Option<Arc<RefCell<CF::M2>>>,
}

impl<W: Semiring, CF: ComposeFilter<W>> ComposeFilterBuilder<W, CF> {
    pub fn new(
        fst1: Arc<<CF::M1 as Matcher<W>>::F>,
        fst2: Arc<<CF::M2 as Matcher<W>>::F>,
        matcher1: Option<Arc<RefCell<CF::M1>>>,
        matcher2: Option<Arc<RefCell<CF::M2>>>,
    ) -> Self {
        Self {fst1, fst2, matcher1, matcher2}
    }

    pub fn build(&self) -> Result<CF> {
        let fst1 = Arc::clone(&self.fst1);
        let fst2 = Arc::clone(&self.fst2);
        let matcher1 = if let Some(m1) = &self.matcher1 {
            Some(Arc::clone(m1))
        } else {
            None
        };
        let matcher2 = if let Some(m2) = &self.matcher2 {
            Some(Arc::clone(m2))
        } else {
            None
        };
        CF::new(
            fst1, fst2, matcher1, matcher2
        )
    }
}


/// Composition filters determine which matches are allowed to proceed. The
/// filter's state is represented by the type ComposeFilter::FS.
pub trait ComposeFilter<W: Semiring>: Debug {
    type M1: Matcher<W>;
    type M2: Matcher<W>;
    type FS: FilterState;

    fn new<IM1: Into<Option<Arc<RefCell<Self::M1>>>>, IM2: Into<Option<Arc<RefCell<Self::M2>>>>>(
        fst1: Arc<<Self::M1 as Matcher<W>>::F>,
        fst2: Arc<<Self::M2 as Matcher<W>>::F>,
        m1: IM1,
        m2: IM2,
    ) -> Result<Self>
        where
            Self: std::marker::Sized;

    fn start(&self) -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()>;

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS>;

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()>;

    fn matcher1(&self) -> Arc<RefCell<Self::M1>>;

    fn matcher2(&self) -> Arc<RefCell<Self::M2>>;
}
