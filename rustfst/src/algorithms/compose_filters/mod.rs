use std::fmt::Debug;
use std::rc::Rc;

use failure::Fallible;
use failure::_core::cell::RefCell;

pub use alt_sequence_compose_filter::AltSequenceComposeFilter;
pub use match_compose_filter::MatchComposeFilter;
pub use multi_eps_filter::MultiEpsFilter;
pub use no_match_compose_filter::NoMatchComposeFilter;
pub use null_compose_filter::NullComposeFilter;
pub use sequence_compose_filter::SequenceComposeFilter;
pub use trivial_compose_filter::TrivialComposeFilter;

use crate::algorithms::filter_states::FilterState;
use crate::algorithms::matchers::Matcher;
use crate::semirings::Semiring;
use crate::{Arc, StateId};

mod alt_sequence_compose_filter;
mod match_compose_filter;
mod multi_eps_filter;
mod no_match_compose_filter;
mod null_compose_filter;
mod sequence_compose_filter;
mod trivial_compose_filter;


/// Composition filters determine which matches are allowed to proceed. The
/// filter's state is represented by the type ComposeFilter::FS.
pub trait ComposeFilter<'fst, W: Semiring + 'fst>: Debug {
    type M1: Matcher<'fst, W>;
    type M2: Matcher<'fst, W>;
    type FS: FilterState;

    fn new<IM1: Into<Option<Self::M1>>, IM2: Into<Option<Self::M2>>>(
        fst1: &'fst <Self::M1 as Matcher<'fst, W>>::F,
        fst2: &'fst <Self::M2 as Matcher<'fst, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self>
    where
        Self: std::marker::Sized;

    fn start(&self) -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS);

    fn filter_arc(&self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS;

    fn filter_final(&self, w1: &mut W, w2: &mut W);

    fn matcher1(&self) -> Rc<RefCell<Self::M1>>;

    fn matcher2(&self) -> Rc<RefCell<Self::M2>>;
}
