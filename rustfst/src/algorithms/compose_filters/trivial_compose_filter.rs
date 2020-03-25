use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::Arc;
use failure::_core::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct TrivialComposeFilter<M1, M2> {
    matcher1: Rc<RefCell<M1>>,
    matcher2: Rc<RefCell<M2>>,
}

impl<'fst1, 'fst2, W: Semiring + 'fst1 + 'fst2, M1: Matcher<'fst1, W>, M2: Matcher<'fst2, W>>
    ComposeFilter<'fst1, 'fst2, W> for TrivialComposeFilter<M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = TrivialFilterState;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            matcher1: m1.into().unwrap_or_else(|| {
                Rc::new(RefCell::new(
                    Self::M1::new(fst1, MatchType::MatchOutput).unwrap(),
                ))
            }),
            matcher2: m2.into().unwrap_or_else(|| {
                Rc::new(RefCell::new(
                    Self::M2::new(fst2, MatchType::MatchInput).unwrap(),
                ))
            }),
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(true)
    }

    fn set_state(&mut self, _s1: usize, _s2: usize, _filter_state: &Self::FS) {}

    fn filter_arc(&mut self, _arc1: &mut Arc<W>, _arc2: &mut Arc<W>) -> Self::FS {
        Self::FS::new(true)
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) {}

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        Rc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        Rc::clone(&self.matcher2)
    }
}
