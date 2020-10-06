use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::{ComposeFilter, ComposeFilterBuilder};
use crate::algorithms::compose::filter_states::{FilterState, TrivialFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{Tr, EPS_LABEL};

#[derive(Debug, Clone)]
pub struct NoMatchComposeFilter<W, F1, F2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    M1: Matcher<W, F1>,
    M2: Matcher<W, F2>,
{
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
    ghost: PhantomData<(W, F1, F2)>,
}

#[derive(Debug)]
pub struct NoMatchComposeFilterBuilder<W, F1, F2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    M1: Matcher<W, F1>,
    M2: Matcher<W, F2>,
{
    matcher1: Arc<M1>,
    matcher2: Arc<M2>,
    ghost: PhantomData<(W, F1, F2)>,
}

impl<W, F1, F2, M1, M2> Clone for NoMatchComposeFilterBuilder<W, F1, F2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    M1: Matcher<W, F1>,
    M2: Matcher<W, F2>,
{
    fn clone(&self) -> Self {
        NoMatchComposeFilterBuilder {
            matcher1: self.matcher1.clone(),
            matcher2: self.matcher2.clone(),
            ghost: PhantomData,
        }
    }
}

impl<W: Semiring, F1, F2, M1, M2> ComposeFilterBuilder<W, F1, F2, M1, M2>
    for NoMatchComposeFilterBuilder<W, F1, F2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    M1: Matcher<W, F1>,
    M2: Matcher<W, F2>,
{
    type IM1 = M1;
    type IM2 = M2;
    type CF = NoMatchComposeFilter<W, F1, F2, M1, M2>;

    fn new(
        fst1: Arc<F1>,
        fst2: Arc<F2>,
        matcher1: Option<M1>,
        matcher2: Option<M2>,
    ) -> Result<Self> {
        let matcher1 =
            matcher1.unwrap_or_else(|| M1::new(Arc::clone(&fst1), MatchType::MatchOutput).unwrap());
        let matcher2 =
            matcher2.unwrap_or_else(|| M2::new(Arc::clone(&fst2), MatchType::MatchInput).unwrap());
        Ok(Self {
            matcher1: Arc::new(matcher1),
            matcher2: Arc::new(matcher2),
            ghost: PhantomData,
        })
    }

    fn build(&self) -> Result<Self::CF> {
        Ok(NoMatchComposeFilter::<W, F1, F2, M1, M2> {
            matcher1: Arc::clone(&self.matcher1),
            matcher2: Arc::clone(&self.matcher2),
            ghost: PhantomData,
        })
    }
}

impl<W, F1, F2, M1, M2> ComposeFilter<W, F1, F2, M1, M2>
    for NoMatchComposeFilter<W, F1, F2, M1, M2>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    M1: Matcher<W, F1>,
    M2: Matcher<W, F2>,
{
    type FS = TrivialFilterState;

    fn start(&self) -> Self::FS {
        Self::FS::new(true)
    }

    fn set_state(&mut self, _s1: usize, _s2: usize, _filter_state: &Self::FS) -> Result<()> {
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        Ok(Self::FS::new(
            arc1.olabel != EPS_LABEL || arc2.ilabel != EPS_LABEL,
        ))
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
