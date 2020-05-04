use anyhow::Result;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::filter_states::{FilterState, IntegerFilterState};
use crate::algorithms::compose::matchers::{MatchType, Matcher};
use crate::fst_traits::{CoreFst, Fst};
use crate::semirings::Semiring;
use crate::{StateId, Tr, EPS_LABEL, NO_LABEL, NO_STATE_ID};
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Debug)]
/// This filter requires epsilons on FST1 to be read before epsilons on FST2.
pub struct SequenceComposeFilter<W: Semiring, M1: Matcher<W>, M2: Matcher<W>> {
    fst1: Arc<M1::F>,
    fst2: Arc<M2::F>,
    matcher1: Arc<RefCell<M1>>,
    matcher2: Arc<RefCell<M2>>,
    /// Current fst1 state
    s1: StateId,
    /// Current fst2 state
    s2: StateId,
    /// Current filter state
    fs: IntegerFilterState,
    /// Only epsilons (and non-final) leaving s1 ?
    alleps1: bool,
    /// No epsilons leaving s1 ?
    noeps1: bool,
}

impl<W: Semiring + 'static, M1: Matcher<W>, M2: Matcher<W>> ComposeFilter<W>
    for SequenceComposeFilter<W, M1, M2>
{
    type M1 = M1;
    type M2 = M2;
    type FS = IntegerFilterState;

    fn new<IM1: Into<Option<Arc<RefCell<Self::M1>>>>, IM2: Into<Option<Arc<RefCell<Self::M2>>>>>(
        fst1: Arc<M1::F>,
        fst2: Arc<M2::F>,
        m1: IM1,
        m2: IM2,
    ) -> Result<Self> {
        Ok(Self {
            fst1: Arc::clone(&fst1),
            fst2: Arc::clone(&fst2),
            matcher1: m1.into().unwrap_or_else(|| {
                Arc::new(RefCell::new(
                    Self::M1::new(fst1, MatchType::MatchOutput).unwrap(),
                ))
            }),
            matcher2: m2.into().unwrap_or_else(|| {
                Arc::new(RefCell::new(
                    Self::M2::new(fst2, MatchType::MatchInput).unwrap(),
                ))
            }),
            s1: NO_STATE_ID,
            s2: NO_STATE_ID,
            fs: Self::FS::new(NO_STATE_ID),
            alleps1: false,
            noeps1: false,
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
            let na1 = self.fst1.num_trs(self.s1)?;
            let ne1 = self.fst1.num_output_epsilons(self.s1)?;
            let fin1 = self.fst1.is_final(self.s1)?;
            self.alleps1 = na1 == ne1 && !fin1;
            self.noeps1 = ne1 == 0;
        }
        Ok(())
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let res = if arc1.olabel == NO_LABEL {
            if self.alleps1 {
                Self::FS::new_no_state()
            } else if self.noeps1 {
                Self::FS::new(0)
            } else {
                Self::FS::new(1)
            }
        } else if arc2.ilabel == NO_LABEL {
            if self.fs != Self::FS::new(0) {
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

    fn matcher1(&self) -> Arc<RefCell<Self::M1>> {
        Arc::clone(&self.matcher1)
    }

    fn matcher2(&self) -> Arc<RefCell<Self::M2>> {
        Arc::clone(&self.matcher2)
    }
}
