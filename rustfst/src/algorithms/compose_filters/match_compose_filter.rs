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
pub struct MatchComposeFilter<'fst1, 'fst2, F1, F2, M1, M2> {
    fst1: &'fst1 F1,
    fst2: &'fst2 F2,
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
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
}

impl<'fst1, 'fst2, W: Semiring, M1: Matcher<'fst1, W>, M2: Matcher<'fst2, W>>
    ComposeFilter<'fst1, 'fst2, W> for MatchComposeFilter<'fst1, 'fst2, M1::F, M2::F, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = IntegerFilterState;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            fst1,
            fst2,
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
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: Self::FS::new(NO_STATE_ID),
            alleps1: false,
            alleps2: false,
            noeps1: false,
            noeps2: false,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Fallible<()> {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();

            let na1 = self.fst1.num_arcs(s1)?;
            let na2 = self.fst2.num_arcs(s2)?;

            let ne1 = self.fst1.num_output_epsilons(s1)?;
            let ne2 = self.fst2.num_input_epsilons(s2)?;

            let f1 = self.fst1.is_final(s1)?;
            let f2 = self.fst2.is_final(s2)?;

            self.alleps1 = na1 == ne1 && !f1;
            self.alleps2 = na2 == ne2 && !f2;

            self.noeps1 = ne1 == 0;
            self.noeps2 = ne2 == 0;
        }
        Ok(())
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Fallible<Self::FS> {
        let res = if arc2.ilabel == NO_LABEL {
            // EPSILON in FST1
            if self.fs == Self::FS::new(0) {
                if self.noeps2 {
                    Self::FS::new(0)
                } else if self.alleps2 {
                    Self::FS::new_no_state()
                } else {
                    Self::FS::new(1)
                }
            } else {
                if self.fs == Self::FS::new(1) {
                    Self::FS::new(1)
                } else {
                    Self::FS::new_no_state()
                }
            }
        } else if arc1.olabel == NO_LABEL {
            // Epsilon in FST2
            if self.fs == Self::FS::new(0) {
                if self.noeps1 {
                    Self::FS::new(0)
                } else if self.alleps1 {
                    Self::FS::new_no_state()
                } else {
                    Self::FS::new(2)
                }
            } else {
                if self.fs == Self::FS::new(2) {
                    Self::FS::new(2)
                } else {
                    Self::FS::new_no_state()
                }
            }
        } else if arc1.olabel == EPS_LABEL {
            // Epsilon in both
            if self.fs == Self::FS::new(0) {
                Self::FS::new(0)
            } else {
                Self::FS::new_no_state()
            }
        } else {
            // Both are non-epsilons
            Self::FS::new(0)
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
