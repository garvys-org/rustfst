use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, PairFilterState, WeightFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::{MatchTypeTrait, Selector};
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{MatchType, MatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst};
use crate::semirings::{
    DivideType, Semiring, SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::{StateId, Tr, KDELTA};

#[derive(Debug, Clone)]
pub struct PushWeightsComposeFilter<W: Semiring, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    filter: CF,
    fs: PairFilterState<CF::FS, WeightFilterState<W>>,
    smt: PhantomData<SMT>,
}

#[derive(Debug)]
pub struct PushWeightsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    filter_builder: CFB,
    #[allow(clippy::type_complexity)]
    ghost: PhantomData<(W, F1, F2, B1, B2, M1, M2, SMT)>,
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> Clone
    for PushWeightsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    fn clone(&self) -> Self {
        Self {
            filter_builder: self.filter_builder.clone(),
            ghost: PhantomData,
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CFB, SMT> ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
    for PushWeightsComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2, CFB, SMT>
where
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    F1: ExpandedFst<W>,
    F2: ExpandedFst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CFB: ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>,
    CFB::CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    type IM1 = M1;
    type IM2 = M2;
    type CF = PushWeightsComposeFilter<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT>;

    fn new(fst1: B1, fst2: B2, matcher1: Option<M1>, matcher2: Option<M2>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            filter_builder: CFB::new(fst1, fst2, matcher1, matcher2)?,
            ghost: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(
            PushWeightsComposeFilter::<W, F1, F2, B1, B2, M1, M2, CFB::CF, SMT> {
                filter: self.filter_builder.build()?,
                fs: FilterState::new_no_state(),
                smt: PhantomData,
            },
        )
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> ComposeFilter<W, F1, F2, B1, B2, M1, M2>
    for PushWeightsComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
{
    type FS = PairFilterState<CF::FS, WeightFilterState<W>>;

    fn start(&self) -> Self::FS {
        Self::FS::new((self.filter.start(), WeightFilterState::new(W::one())))
    }

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()> {
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

    fn matcher1(&self) -> &M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<M2> {
        self.filter.matcher2_shared()
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        self.filter.properties(inprops) & FstProperties::weight_invariant_properties()
    }
}

impl<W, F1, F2, B1, B2, M1, M2, CF, SMT> LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>
    for PushWeightsComposeFilter<W, F1, F2, B1, B2, M1, M2, CF, SMT>
where
    W: SerializableSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
    CF: LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>,
    SMT: MatchTypeTrait,
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
