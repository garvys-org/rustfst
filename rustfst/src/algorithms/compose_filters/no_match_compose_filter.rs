use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::{Arc, EPS_LABEL};

pub struct NoMatchComposeFilter<M1, M2> {
    matcher1: M1,
    matcher2: M2,
}

impl<
        'matcher,
        'fst: 'matcher,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        M1: Matcher<'matcher, 'fst, F1>,
        M2: Matcher<'matcher, 'fst, F2>,
    > ComposeFilter<'matcher, 'fst, F1, F2> for NoMatchComposeFilter<M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = TrivialFilterState;

    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self> {
        Ok(Self {
            matcher1: Self::M1::new(fst1, MatchType::MatchOutput)?,
            matcher2: Self::M2::new(fst2, MatchType::MatchInput)?,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(true)
    }

    fn set_state(&mut self, _s1: usize, _s2: usize, _filter_state: &Self::FS) {}

    fn filter_arc(
        &self,
        arc1: &mut Arc<<F1 as CoreFst>::W>,
        arc2: &mut Arc<<F2 as CoreFst>::W>,
    ) -> Option<Self::FS> {
        Some(Self::FS::new(
            arc1.olabel != EPS_LABEL || arc2.ilabel != EPS_LABEL,
        ))
    }

    fn filter_final(&self, _w1: &mut <F1 as CoreFst>::W, _w2: &mut <F2 as CoreFst>::W) {}

    fn matcher1(&mut self) -> &mut Self::M1 {
        &mut self.matcher1
    }

    fn matcher2(&mut self) -> &mut Self::M2 {
        &mut self.matcher2
    }
}
