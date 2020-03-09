use std::rc::Rc;

use failure::Fallible;

// pub use alt_sequence_compose_filter::AltSequenceComposeFilter;
// pub use match_compose_filter::MatchComposeFilter;
// pub use multi_eps_filter::MultiEpsFilter;
// pub use no_match_compose_filter::NoMatchComposeFilter;
// pub use null_compose_filter::NullComposeFilter;
// pub use sequence_compose_filter::SequenceComposeFilter;
pub use trivial_compose_filter::TrivialComposeFilter;

use crate::algorithms::filter_states::FilterState;
use crate::algorithms::matchers::Matcher;
use crate::fst_traits::Fst;
use crate::{Arc, StateId};
use failure::_core::cell::RefCell;
use std::fmt::Debug;

// mod alt_sequence_compose_filter;
// mod match_compose_filter;
// mod multi_eps_filter;
// mod no_match_compose_filter;
// mod null_compose_filter;
// mod sequence_compose_filter;
mod trivial_compose_filter;

pub trait ComposeFilter<'iter, 'fst: 'iter, F1: Fst + 'fst, F2: Fst<W = F1::W> + 'fst>:
    Debug + PartialEq
{
    type M1: Matcher<'iter, 'fst, F1>;
    type M2: Matcher<'iter, 'fst, F2>;
    type FS: FilterState;

    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self>
    where
        Self: std::marker::Sized;

    fn start(&self) -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS);

    fn filter_arc(&self, arc1: &mut Arc<F1::W>, arc2: &mut Arc<F2::W>) -> Option<Self::FS>;

    fn filter_final(&self, w1: &mut F1::W, w2: &mut F2::W);

    fn matcher1(&self) -> Rc<RefCell<Self::M1>>;

    fn matcher2(&self) -> Rc<RefCell<Self::M2>>;
}
