use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::compose::lookahead_filters::lookahead_selector::Selector;
use crate::algorithms::compose::lookahead_filters::LookAheadComposeFilterTrait;
use crate::algorithms::compose::lookahead_matchers::{LookAheadMatcherData, LookaheadMatcher};
use crate::algorithms::compose::matchers::{MatchType, Matcher, MatcherFlags};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Clone, Debug)]
pub struct AltSequenceComposeFilter<W: Semiring, F1, F2, B1, B2, M1, M2>
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
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
    ghost: PhantomData<(W, F1, F2, B1, B2)>,
}

#[derive(Debug)]
pub struct AltSequenceComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2>: ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
    ghost: PhantomData<(W, F1, F2, B1, B2)>,
}

impl<W, F1, F2, B1, B2, M1, M2> Clone for AltSequenceComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2>: ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    fn clone(&self) -> Self {
        AltSequenceComposeFilterBuilder {
            matcher1: self.matcher1.clone(),
            matcher2: self.matcher2.clone(),
            ghost: PhantomData,
        }
    }
}

impl<W: Semiring, F1, F2, B1, B2, M1, M2> ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
    for AltSequenceComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    type IM1 = M1;
    type IM2 = M2;
    type CF = AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2>;

    fn new(fst1: B1, fst2: B2, matcher1: Option<M1>, matcher2: Option<M2>) -> Result<Self> {
        let matcher1 = matcher1.unwrap_or_else(|| M1::new(fst1, MatchType::MatchOutput).unwrap());
        let matcher2 = matcher2.unwrap_or_else(|| M2::new(fst2, MatchType::MatchInput).unwrap());
        Ok(Self {
            matcher1: Arc::new(matcher1),
            matcher2: Arc::new(matcher2),
            ghost: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(AltSequenceComposeFilter::<W, F1, F2, B1, B2, M1, M2> {
            matcher1: Arc::clone(&self.matcher1),
            matcher2: Arc::clone(&self.matcher2),
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: <AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2> as ComposeFilter<
                W,
                F1,
                F2,
                B1,
                B2,
                M1,
                M2,
            >>::FS::new(NO_STATE_ID),
            alleps2: false,
            noeps2: false,
            ghost: PhantomData,
        })
    }
}

impl<W: Semiring, F1, F2, B1, B2, M1, M2> ComposeFilter<W, F1, F2, B1, B2, M1, M2>
    for AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2>
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    type FS = IntegerFilterState;

    fn start(&self) -> Self::FS {
        Self::FS::new(0)
    }

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()> {
        if !(self.s1 == s1 && self.s2 == s2 && &self.fs == filter_state) {
            self.s1 = s1;
            self.s2 = s2;
            self.fs = filter_state.clone();
            // TODO: Could probably use unchecked here as the state should exist.
            let fst2 = self.matcher2().fst().borrow();
            let na2 = fst2.num_trs(self.s2)?;
            let ne2 = fst2.num_input_epsilons(self.s2)?;
            let fin2 = fst2.is_final(self.s2)?;
            self.alleps2 = na2 == ne2 && !fin2;
            self.noeps2 = ne2 == 0;
        }
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
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
        } else if arc1.olabel == EPS_LABEL {
            Self::FS::new_no_state()
        } else {
            Self::FS::new(0)
        };
        Ok(res)
    }

    fn filter_final(&self, _w1: &mut W, _w2: &mut W) -> Result<()> {
        Ok(())
    }

    fn matcher1(&self) -> &M1 {
        &self.matcher1
    }

    fn matcher2(&self) -> &M2 {
        &self.matcher2
    }

    fn matcher1_shared(&self) -> &Arc<M1> {
        &self.matcher1
    }

    fn matcher2_shared(&self) -> &Arc<M2> {
        &self.matcher2
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        inprops
    }
}

impl<W: Semiring + 'static, F1, F2, B1, B2, M1, M2>
    LookAheadComposeFilterTrait<W, F1, F2, B1, B2, M1, M2>
    for AltSequenceComposeFilter<W, F1, F2, B1, B2, M1, M2>
where
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: LookaheadMatcher<W, F1, B1>,
    M2: LookaheadMatcher<W, F2, B2>,
{
    fn lookahead_flags(&self) -> MatcherFlags {
        unreachable!()
    }

    fn lookahead_tr(&self) -> bool {
        unreachable!()
    }

    fn lookahead_type(&self) -> MatchType {
        unreachable!()
    }

    fn lookahead_output(&self) -> bool {
        unreachable!()
    }

    fn selector(&self) -> &Selector {
        unreachable!()
    }

    fn lookahead_matcher_data(&self) -> Option<&LookAheadMatcherData<W>> {
        unreachable!()
    }
}
