use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Tr, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Clone, Debug)]
pub struct AltSequenceComposeFilter<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    fst1: PhantomData<M1::F>,
    fst2: Rc<M2::F>,
    matcher1: Rc<RefCell<M1>>,
    matcher2: Rc<RefCell<M2>>,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: IntegerFilterState,
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
}

impl<W: Semiring + 'static, M1: Matcher<W>, M2: Matcher<W>> ComposeFilter<W>
    for AltSequenceComposeFilter<W, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = IntegerFilterState;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: Rc<<Self::M1 as Matcher<W>>::F>,
        fst2: Rc<<Self::M2 as Matcher<W>>::F>,
        m1: IM1,
        m2: IM2,
    ) -> Result<Self> {
        Ok(Self {
            fst2: Rc::clone(&fst2),
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
            alleps2: false,
            noeps2: false,
            fst1: PhantomData,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();
            // TODO: Could probably use unchecked here as the state should exist.
            let na2 = self.fst2.num_arcs(self.s2)?;
            let ne2 = self.fst2.num_input_epsilons(self.s2)?;
            let fin2 = self.fst2.is_final(self.s2)?;
            self.alleps2 = na2 == ne2 && !fin2;
            self.noeps2 = ne2 == 0;
        }
        Ok(())
    }

    fn filter_arc(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let res = if arc2.ilabel == NO_LABEL {
            if self.alleps2 {
                Self::FS::new_no_state()
            } else if self.noeps2 {
                Self::FS::new(0)
            } else {
                Self::FS::new(1)
            }
        } else if arc1.olabel == NO_LABEL {
            if self.fs == Self::FS::new(1) {
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
        };
        Ok(res)
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) -> Result<()> {
        Ok(())
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        Rc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        Rc::clone(&self.matcher2)
    }
}

impl<W: Semiring + 'static, M1: LookaheadMatcher<W>, M2: LookaheadMatcher<W>>
    LookAheadComposeFilterTrait<W> for AltSequenceComposeFilter<W, M1, M2>
{
    fn lookahead_flags(&self) -> MatcherFlags {
        unreachable!()
    }

    fn lookahead_arc(&self) -> bool {
        unreachable!()
    }

    fn lookahead_type(&self) -> MatchType {
        unreachable!()
    }

    fn lookahead_output(&self) -> bool {
        unreachable!()
    }

    fn selector(&self) -> &Selector<W, Self::M1, Self::M2> {
        unreachable!()
    }
}
