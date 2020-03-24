use crate::algorithms::compose_filters::ComposeFilter;
use crate::algorithms::filter_states::{IntegerFilterState, PairFilterState};
use crate::algorithms::lookahead_filters::lookahead_selector::MatchTypeTrait;
use crate::algorithms::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::lookahead_matchers::LookaheadMatcher;
use crate::algorithms::matchers::MatcherFlags;
use crate::algorithms::matchers::{MatchType, Matcher};
use crate::semirings::Semiring;
use crate::Arc;
use failure::Fallible;
use failure::_core::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
#[derive(Debug, Clone)]
pub struct PushLabelsComposeFilter<
    'fst1,
    'fst2,
    W: Semiring + 'fst1 + 'fst2,
    CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
    SMT: MatchTypeTrait,
> where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    fst1: &'fst1 <CF::M1 as Matcher<'fst1, W>>::F,
    fst2: &'fst2 <CF::M2 as Matcher<'fst2, W>>::F,
    matcher1: Rc<RefCell<CF::M1>>,
    matcher2: Rc<RefCell<CF::M2>>,
    filter: CF,
    fs: PairFilterState<CF::FS, IntegerFilterState>,
    smt: PhantomData<SMT>,
    narcsa: usize,
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > ComposeFilter<'fst1, 'fst2, W> for PushLabelsComposeFilter<'fst1, 'fst2, W, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    type M1 = ();
    type M2 = ();
    type FS = PairFilterState<CF::FS, IntegerFilterState>;

    fn new<IM1: Into<Option<Self::M1>>, IM2: Into<Option<Self::M2>>>(
        fst1: &'fst1 <Self::M1 as Matcher<'fst1, W>>::F,
        fst2: &'fst2 <Self::M2 as Matcher<'fst2, W>>::F,
        m1: IM1,
        m2: IM2,
    ) -> Fallible<Self>
    where
        Self: std::marker::Sized,
    {
        unimplemented!()
    }

    fn start(&self) -> Self::FS {
        unimplemented!()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        unimplemented!()
    }

    fn filter_arc(&mut self, arc1: &mut Arc<W>, arc2: &mut Arc<W>) -> Self::FS {
        unimplemented!()
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) {
        unimplemented!()
    }

    fn matcher1(&self) -> Rc<RefCell<Self::M1>> {
        unimplemented!()
    }

    fn matcher2(&self) -> Rc<RefCell<Self::M2>> {
        unimplemented!()
    }
}

impl<
        'fst1,
        'fst2,
        W: Semiring + 'fst1 + 'fst2,
        CF: LookAheadComposeFilterTrait<'fst1, 'fst2, W>,
        SMT: MatchTypeTrait,
    > LookAheadComposeFilterTrait<'fst1, 'fst2, W>
    for PushLabelsComposeFilter<'fst1, 'fst2, W, CF, SMT>
where
    CF::M1: LookaheadMatcher<'fst1, W>,
    CF::M2: LookaheadMatcher<'fst2, W>,
{
    fn lookahead_flags(&self) -> MatcherFlags {
        unimplemented!()
    }

    fn lookahead_arc(&self) -> bool {
        unimplemented!()
    }

    fn lookahead_type(&self) -> MatchType {
        unimplemented!()
    }
}
