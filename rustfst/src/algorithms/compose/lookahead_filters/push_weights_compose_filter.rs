use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, PairFilterState, WeightFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::{MatchTypeTrait, Selector};
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::MatcherFlags;
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::ExpandedFst;
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

#[derive(Debug, Clone)]
pub struct PushWeightsComposeFilterBuilder<W, CFB, SMT>
where
    W: Semiring,
    CFB: ComposeFilterBuilder<W>,
    CFB::CF: LookAheadComposeFilterTrait<W>,
    <CFB::CF as ComposeFilter<W>>::M1: LookaheadMatcher<W>,
    <CFB::CF as ComposeFilter<W>>::M2: LookaheadMatcher<W>,
{
    filter_builder: CFB,
    w: PhantomData<W>,
    smt: PhantomData<SMT>,
}

impl<W, F1, F2, M1, M2, CF, CFB, SMT> ComposeFilterBuilder<W>
    for PushWeightsComposeFilterBuilder<W, CFB, SMT>
where
    W: Semiring + WeaklyDivisibleSemiring + WeightQuantize,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    M1: Matcher<W, F = F1> + LookaheadMatcher<W>,
    M2: Matcher<W, F = F2> + LookaheadMatcher<W>,
    CF: ComposeFilter<W, M1 = M1, M2 = M2> + LookAheadComposeFilterTrait<W>,
    CFB: ComposeFilterBuilder<W, M1 = M1, M2 = M2, CF = CF>,
    SMT: MatchTypeTrait,
{
    type CF = PushWeightsComposeFilter<W, CF, SMT>;
    type M1 = M1;
    type M2 = M2;

    fn new(
        fst1: Arc<<<Self::CF as ComposeFilter<W>>::M1 as Matcher<W>>::F>,
        fst2: Arc<<<Self::CF as ComposeFilter<W>>::M2 as Matcher<W>>::F>,
        matcher1: Option<Self::M1>,
        matcher2: Option<Self::M2>,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            filter_builder: CFB::new(fst1, fst2, matcher1, matcher2)?,
            w: PhantomData,
            smt: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(PushWeightsComposeFilter::<W, CFB::CF, SMT> {
            filter: self.filter_builder.build()?,
            fs: FilterState::new_no_state(),
            smt: PhantomData,
        })
    }
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

    fn start(&self) -> Self::FS {
        Self::FS::new((self.filter.start(), WeightFilterState::new(W::one())))
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        self.fs = filter_state.clone();
        self.filter.set_state(s1, s2, filter_state.state1())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let fs1 = self.filter.filter_tr(arc1, arc2)?;
        // TODO: Find a way to avoid this unwrap. Should be safe as LaMatcherData has been computed above.
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
            // Unwrap should be safe because lookahead_tr is set to true. Find a better way!
            let la_matcher_data = self.filter.lookahead_matcher_data().unwrap();
            la_matcher_data.lookahead_weight.clone()
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

    fn matcher1(&self) -> &Self::M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &Self::M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<Self::M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<Self::M2> {
        self.filter.matcher2_shared()
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

    fn selector(&self) -> &Selector {
        self.filter.selector()
    }

    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>> {
        self.filter.lookahead_matcher_data()
    }
}
