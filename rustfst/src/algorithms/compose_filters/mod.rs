use failure::Fallible;

pub use alt_sequence_compose_filter::AltSequenceComposeFilter;
pub use null_compose_filter::NullComposeFilter;
pub use sequence_compose_filter::SequenceComposeFilter;
pub use trivial_compose_filter::TrivialComposeFilter;

use crate::algorithms::filter_states::FilterState;
use crate::algorithms::matchers::Matcher;
use crate::fst_traits::Fst;
use crate::{Arc, StateId};

mod alt_sequence_compose_filter;
mod null_compose_filter;
mod sequence_compose_filter;
mod trivial_compose_filter;

pub trait ComposeFilter<'matcher, 'fst: 'matcher, F1: Fst + 'fst, F2: Fst<W = F1::W> + 'fst> {
    type M1: Matcher<'matcher, 'fst, F1>;
    type M2: Matcher<'matcher, 'fst, F2>;
    type FS: FilterState;

    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self>
    where
        Self: std::marker::Sized;

    fn start() -> Self::FS;

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS);

    fn filter_arc(&self, arc1: &Arc<F1::W>, arc2: &Arc<F2::W>) -> Option<Self::FS>;

    fn filter_final(&self, w1: &mut F1::W, w2: &mut F2::W);

    fn matcher1(&mut self) -> &mut Self::M1;

    fn matcher2(&mut self) -> &mut Self::M2;
}
