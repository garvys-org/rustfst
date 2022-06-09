use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL, NO_LABEL, NO_STATE_ID};

#[derive(Debug, Clone)]
pub struct MatchComposeFilter<W: Semiring, F1, F2, B1, B2, M1, M2>
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
    /// Only epsilons (and non-final) leaving s1 ?
    alleps1: bool,
    /// Only epsilons (and non-final) leaving s2 ?
    alleps2: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
    /// No epsilons leaving s2 ?
    noeps2: bool,
    ghost: PhantomData<(W, F1, F2, B1, B2)>,
}

#[derive(Debug)]
pub struct MatchComposeFilterBuilder<W: Semiring, F1, F2, B1, B2, M1, M2>
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
    ghost: PhantomData<(W, F1, F2, M1, M2, B1, B2)>,
}

impl<W, F1, F2, B1, B2, M1, M2> Clone for MatchComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    fn clone(&self) -> Self {
        MatchComposeFilterBuilder {
            matcher1: self.matcher1.clone(),
            matcher2: self.matcher2.clone(),
            ghost: PhantomData,
        }
    }
}

impl<W, F1, F2, B1, B2, M1, M2> ComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
    for MatchComposeFilterBuilder<W, F1, F2, B1, B2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
{
    type IM1 = M1;
    type IM2 = M2;
    type CF = MatchComposeFilter<W, F1, F2, B1, B2, M1, M2>;

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
        Ok(MatchComposeFilter::<W, F1, F2, B1, B2, M1, M2> {
            matcher1: Arc::clone(&self.matcher1),
            matcher2: Arc::clone(&self.matcher2),
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: <Self::CF as ComposeFilter<W, F1, F2, B1, B2, M1, M2>>::FS::new(NO_STATE_ID),
            alleps1: false,
            alleps2: false,
            noeps1: false,
            noeps2: false,
            ghost: PhantomData,
        })
    }
}

impl<W: Semiring, F1, F2, B1, B2, M1, M2> ComposeFilter<W, F1, F2, B1, B2, M1, M2>
    for MatchComposeFilter<W, F1, F2, B1, B2, M1, M2>
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

            let fst1 = self.matcher1().fst().borrow();
            let fst2 = self.matcher2().fst().borrow();

            let na1 = fst1.num_trs(s1)?;
            let na2 = fst2.num_trs(s2)?;

            let ne1 = fst1.num_output_epsilons(s1)?;
            let ne2 = fst2.num_input_epsilons(s2)?;

            let f1 = fst1.is_final(s1)?;
            let f2 = fst2.is_final(s2)?;

            self.alleps1 = na1 == ne1 && !f1;
            self.alleps2 = na2 == ne2 && !f2;

            self.noeps1 = ne1 == 0;
            self.noeps2 = ne2 == 0;
        }
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
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
            } else if self.fs == Self::FS::new(1) {
                Self::FS::new(1)
            } else {
                Self::FS::new_no_state()
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
            } else if self.fs == Self::FS::new(2) {
                Self::FS::new(2)
            } else {
                Self::FS::new_no_state()
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
