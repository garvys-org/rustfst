use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{CharFilterState, FilterState};
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct AltSequenceComposeFilter<'fst, F2, M1, M2> {
    fst2: &'fst F2,
    matcher1: Rc<RefCell<M1>>,
    matcher2: Rc<RefCell<M2>>,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: CharFilterState,
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
}

impl<'fst, W: Semiring + 'fst, M1: Matcher<'fst, W>, M2: Matcher<'fst, W>> ComposeFilter<'fst, W>
    for AltSequenceComposeFilter<'fst, M2::F, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = CharFilterState;

    fn new<IM1: Into<Option<Self::M1>>, IM2: Into<Option<Self::M2>>>(
        fst1: &'fst <Self::M1 as Matcher<'fst, W>>::F,
        fst2: &'fst <Self::M2 as Matcher<'fst, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            fst2,
            matcher1: Rc::new(RefCell::new(
                m1.into()
                    .unwrap_or_else(|| Self::M1::new(fst1, MatchType::MatchOutput).unwrap()),
            )),
            matcher2: Rc::new(RefCell::new(
                m2.into()
                    .unwrap_or_else(|| Self::M2::new(fst2, MatchType::MatchInput).unwrap()),
            )),
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: Self::FS::default(),
            alleps2: false,
            noeps2: false,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();
            // TODO: Could probably use unchecked here as the state should exist.
            let na2 = self.fst2.num_arcs(self.s2).unwrap();
            let ne2 = self.fst2.num_input_epsilons(self.s2).unwrap();
            let fin2 = self.fst2.is_final(self.s2).unwrap();
            self.alleps2 = na2 == ne2 && !fin2;
            self.noeps2 = ne2 == 0;
        }
    }

    fn filter_arc(&self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Option<Self::FS> {
        if arc1.olabel == NO_LABEL {
            if self.alleps2 {
                None
            } else if self.noeps2 {
                Some(Self::FS::new(0))
            } else {
                Some(Self::FS::new(1))
            }
        } else if arc2.ilabel == NO_LABEL {
            if self.fs != Self::FS::new(0) {
                None
            } else {
                Some(Self::FS::new(0))
            }
        } else {
            if arc1.olabel == EPS_LABEL {
                None
            } else {
                Some(Self::FS::new(0))
            }
        }
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) {}

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        Rc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        Rc::clone(&self.matcher2)
    }
}
