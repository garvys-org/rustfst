use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
/// This filter requires epsilons on FST1 to be read before epsilons on FST2.
pub struct SequenceComposeFilter<
    'fst1,
    'fst2,
    W: Semiring + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
> {
    fst1: &'fst1 M1::F,
    fst2: &'fst2 M2::F,
    matcher1: Rc<RefCell<M1>>,
    matcher2: Rc<RefCell<M2>>,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: IntegerFilterState,
    /// Only epsilons (and non-final) leaving s1 ?
    alleps1: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
}

impl<'fst1, 'fst2, W: Semiring + 'fst2, M1: Matcher<'fst1, W>, M2: Matcher<'fst2, W>>
    ComposeFilter<'fst1, 'fst2, W> for SequenceComposeFilter<'fst1, 'fst2, W, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = IntegerFilterState;

    fn new<IM1: Into<Option<Self::M1>>, IM2: Into<Option<Self::M2>>>(
        fst1: &'fst1 M1::F,
        fst2: &'fst2 M2::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            fst1,
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
            fs: Self::FS::new(NO_STATE_ID),
            alleps1: false,
            noeps1: false,
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
            let na1 = self.fst1.num_arcs(self.s1).unwrap();
            let ne1 = self.fst1.num_output_epsilons(self.s1).unwrap();
            let fin1 = self.fst1.is_final(self.s1).unwrap();
            self.alleps1 = na1 == ne1 && !fin1;
            self.noeps1 = ne1 == 0;
        }
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS {
        if arc1.olabel == NO_LABEL {
            if self.alleps1 {
                Self::FS::new_no_state()
            } else if self.noeps1 {
                Self::FS::new(0)
            } else {
                Self::FS::new(1)
            }
        } else if arc2.ilabel == NO_LABEL {
            if self.fs != Self::FS::new(0) {
                Self::FS::new_no_state()
            } else {
                Self::FS::new(0)
            }
        } else {
            if arc1.olabel == EPS_LABEL {
                Self::FS::new_no_state()
            } else {
                Self::FS::new(0)
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
