use std::cell::RefCell;
use std::rc::Rc;

use failure::Fallible;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::{Arc, NO_LABEL};

#[derive(Debug)]
pub struct NullComposeFilter<M1, M2> {
    matcher1: Rc<RefCell<M1>>,
    matcher2: Rc<RefCell<M2>>,
}

impl<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> ComposeFilter<W> for NullComposeFilter<M1, M2> {
    type M1 = M1;
    type M2 = M2;
    type FS = TrivialFilterState;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: Rc<<Self::M1 as Matcher<W>>::F>,
        fst2: Rc<<Self::M2 as Matcher<W>>::F>,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            matcher1: m1.into().unwrap_or_else(|| {
                Rc::new(RefCell::new(M1::new(fst1, MatchType::MatchOutput).unwrap()))
            }),
            matcher2: m2.into().unwrap_or_else(|| {
                Rc::new(RefCell::new(M2::new(fst2, MatchType::MatchInput).unwrap()))
            }),
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(true)
    }

    fn set_state(&mut self, _s1: usize, _s2: usize, _filter_state: &Self::FS) -> Fallible<()> {
        Ok(())
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Fallible<Self::FS> {
        let res = if arc1.olabel == NO_LABEL || arc2.ilabel == NO_LABEL {
            Self::FS::new_no_state()
        } else {
            Self::FS::new(true)
        };
        Ok(res)
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) -> Fallible<()> {
        Ok(())
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        Rc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        Rc::clone(&self.matcher2)
    }
}
