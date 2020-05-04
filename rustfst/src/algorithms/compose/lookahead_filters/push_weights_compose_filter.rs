use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::filter_states::{FilterState, PairFilterState, WeightFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::{MatchTypeTrait, Selector};
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::semirings::{DivideType, Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::{Tr, KDELTA};

#[derive(Debug, Clone)]
pub struct PushWeightsComposeFilter<W: Semiring, CF: LookAheadComposeFilterTrait<W>, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    filter: CF,
    fs: PairFilterState<CF::FS, WeightFilterState<W>>,
    smt: PhantomData<SMT>,
}

impl<
        W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
        CF: LookAheadComposeFilterTrait<W>,
        SMT: MatchTypeTrait,
    > ComposeFilter<W> for PushWeightsComposeFilter<W, CF, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    type M1 = CF::M1;
    type M2 = CF::M2;
    type FS = PairFilterState<CF::FS, WeightFilterState<W>>;

    fn new<IM1: Into<Option<Rc<RefCell<Self::M1>>>>, IM2: Into<Option<Rc<RefCell<Self::M2>>>>>(
        fst1: Rc<<Self::M1 as Matcher<W>>::F>,
        fst2: Rc<<Self::M2 as Matcher<W>>::F>,
        m1: IM1,
        m2: IM2,
    ) -> Result<Self> {
        Ok(Self {
            filter: CF::new(fst1, fst2, m1, m2)?,
            fs: Self::FS::new_no_state(),
            smt: PhantomData,
        })
    }

    fn start(&self) -> Self::FS {
        Self::FS::new((self.filter.start(), WeightFilterState::new(W::one())))
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        self.fs = filter_state.clone();
        self.filter.set_state(s1, s2, filter_state.state1())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let fs1 = self.filter.filter_tr(arc1, arc2)?;
        if fs1 == CF::FS::new_no_state() {
            return Ok(FilterState::new_no_state());
        }
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_WEIGHT)
        {
            return Ok(FilterState::new((fs1, FilterState::new(W::one()))));
        }
        let lweight = if self.filter.lookahead_tr() {
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
            return Ok(FilterState::new_no_state());
        }
        arc2.weight.times_assign(&lweight)?;
        arc2.weight.divide_assign(fweight, DivideType::DivideAny)?;
        Ok(FilterState::new((
            fs1,
            FilterState::new(lweight.quantize(KDELTA)?),
        )))
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()> {
        self.filter.filter_final(w1, w2)?;
        if !self
            .lookahead_flags()
            .contains(MatcherFlags::LOOKAHEAD_WEIGHT)
            || w1.is_zero()
        {
            return Ok(());
        }
        let fs2 = self.fs.state2();
        let fweight = fs2.state();
        w1.divide_assign(fweight, DivideType::DivideAny)
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        self.filter.matcher2()
    }
}

impl<
        W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
        CF: LookAheadComposeFilterTrait<W>,
        SMT: MatchTypeTrait,
    > LookAheadComposeFilterTrait<W> for PushWeightsComposeFilter<W, CF, SMT>
where
    CF::M1: LookaheadMatcher<W>,
    CF::M2: LookaheadMatcher<W>,
{
    fn lookahead_flags(&self) -> MatcherFlags {
        self.filter.lookahead_flags()
    }

    fn lookahead_tr(&self) -> bool {
        self.filter.lookahead_tr()
    }

    fn lookahead_type(&self) -> MatchType {
        self.filter.lookahead_type()
    }

    fn lookahead_output(&self) -> bool {
        self.filter.lookahead_output()
    }

    fn selector(&self) -> &Selector<W, Self::M1, Self::M2> {
        self.filter.selector()
    }
}
