use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{Arc, StateId, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Debug)]
pub struct AltSequenceComposeFilter<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    M1: Matcher<'fst1, W>,
    M2: Matcher<'fst2, W>,
> {
    fst1: PhantomData<&'fst1 M1::F>,
    fst2: &'fst2 M2::F,
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

impl<'fst1, 'fst2, W: Semiring + 'fst1, M1: Matcher<'fst1, W>, M2: Matcher<'fst2, W>>
    ComposeFilter<'fst1, 'fst2, W> for AltSequenceComposeFilter<'fst1, 'fst2, W, M1, M2>
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
            alleps2: false,
            noeps2: false,
            fst1: PhantomData,
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

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Fallible<Self::FS> {
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

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        M1: LookaheadMatcher<'fst1, W>,
        M2: LookaheadMatcher<'fst2, W>,
    > LookAheadComposeFilterTrait<'fst1, 'fst2, W>
    for AltSequenceComposeFilter<'fst1, 'fst2, W, M1, M2>
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

    fn selector(&self) -> &Selector<'fst1, 'fst2, W, Self::M1, Self::M2> {
        unreachable!()
    }
}
