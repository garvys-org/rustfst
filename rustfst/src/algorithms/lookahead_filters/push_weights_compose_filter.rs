use std::marker::PhantomData;
use std::rc::Rc;

use failure::Fallible;
use failure::_core::cell::RefCell;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{FilterState, PairFilterState, WeightFilterState};
use crate::algorithms::lookahead_filters::lookahead_selector::{
    selector, LookAheadSelector, MatchTypeTrait, Selector,
};
use crate::algorithms::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::MatcherFlags;
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::semirings::{DivideType, Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Arc, KDELTA};

#[derive(Debug, Clone)]
pub struct PushWeightsComposeFilter<W, FS, CF, SMT> {
    filter: CF,
    fs: PairFilterState<FS, WeightFilterState<W>>,
    smt: PhantomData<SMT>,
}

impl<
        'fst1,
        'fst2,
        W: Semiring + WeaklyDivisibleSemiring + WeightQuantize + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > ComposeFilter<'fst1, 'fst2, W> for PushWeightsComposeFilter<W, CF::FS, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    type M1 = CF::M1;
    type M2 = CF::M2;
    type FS = PairFilterState<CF::FS, WeightFilterState<W>>;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self> {
        Ok(Self {
            filter: CF::new(fst1, fst2, m1, m2)?,
            fs: Self::FS::new_no_state(),
            smt: PhantomData,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new((self.filter.start(), WeightFilterState::new(W::one())))
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        self.fs = filter_state.clone();
        self.filter.set_state(s1, s2, filter_state.state1())
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS {
        let fs1 = self.filter.filter_arc(arc1, arc2);
        if fs1 == CF::FS::new_no_state() {
            return FilterState::new_no_state();
        }
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_WEIGHT)
        {
            return FilterState::new((fs1, FilterState::new(W::one())));
        }
        let lweight = if self.filter.lookahead_arc() {
            match self.selector() {
                Selector::MatchInput(s) => s.matcher.borrow().lookahead_weight().clone(),
                Selector::MatchOutput(s) => s.matcher.borrow().lookahead_weight().clone(),
            }
        } else {
            W::one()
        };

        let fs2 = self.fs.state2();
        let fweight = fs2.state();
        // Disallows zero() weight futures
        if lweight.is_zero() {
            return FilterState::new_no_state();
        }
        arc2.weight.times_assign(&lweight).unwrap();
        arc2.weight
            .divide_assign(fweight, DivideType::DivideAny)
            .unwrap();
        FilterState::new((fs1, FilterState::new(lweight.quantize(KDELTA).unwrap())))
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) {
        self.filter.filter_final(w1, w2);
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_WEIGHT)
            || w1.is_zero()
        {
            return;
        }
        let fs2 = self.fs.state2();
        let fweight = fs2.state();
        w1.divide_assign(fweight, DivideType::DivideAny).unwrap();
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        self.filter.matcher2()
    }
}

impl<
        'fst1,
        'fst2,
        W: Semiring + WeaklyDivisibleSemiring + WeightQuantize + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > LookAheadComposeFilterTrait<'fst1, 'fst2, W> for PushWeightsComposeFilter<W, CF::FS, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    fn lookahead_flags(&self) -> MatcherFlags {
        self.filter.lookahead_flags()
    }

    fn lookahead_arc(&self) -> bool {
        self.filter.lookahead_arc()
    }

    fn lookahead_type(&self) -> MatchType {
        self.filter.lookahead_type()
    }

    fn lookahead_output(&self) -> bool {
        self.filter.lookahead_output()
    }

    fn selector(&self) -> &Selector<'fst1, 'fst2, W, Self::M1, Self::M2> {
        self.filter.selector()
    }
}
