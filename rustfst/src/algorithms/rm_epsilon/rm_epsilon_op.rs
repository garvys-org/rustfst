use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::cache::{CacheImpl, FstImpl};
use crate::algorithms::queues::FifoQueue;
use crate::algorithms::rm_epsilon::{RmEpsilonConfig, RmEpsilonState};
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;

#[derive(Clone, Eq)]
pub struct RmEpsilonImpl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> {
    rmeps_state: RmEpsilonState<W, F, B, FifoQueue>,
    cache_impl: CacheImpl<W>,
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> std::fmt::Debug for RmEpsilonImpl<W, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RmEpsilonImpl {{ rmeps_state : {:?}, cache_impl : {:?} }}",
            self.rmeps_state, self.cache_impl
        )
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> PartialEq for RmEpsilonImpl<W, F, B> {
    fn eq(&self, other: &Self) -> bool {
        self.rmeps_state.eq(&other.rmeps_state) && self.cache_impl.eq(&other.cache_impl)
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> RmEpsilonImpl<W, F, B> {
    fn new(fst: B) -> Self {
        Self {
            cache_impl: CacheImpl::new(),
            rmeps_state: RmEpsilonState::new(
                fst,
                RmEpsilonConfig::new_with_default(FifoQueue::default()),
            ),
        }
    }
}

impl<W: Semiring, F: MutableFst<W>, B: Borrow<F>> FstImpl for RmEpsilonImpl<W, F, B> {
    type W = W;

    fn cache_impl_mut(&mut self) -> &mut CacheImpl<Self::W> {
        &mut self.cache_impl
    }

    fn cache_impl_ref(&self) -> &CacheImpl<Self::W> {
        &self.cache_impl
    }

    fn expand(&mut self, state: usize) -> Result<()> {
        let (trs, final_weight) = self.rmeps_state.expand(state)?;
        let zero = W::zero();

        for tr in trs.into_iter().rev() {
            self.cache_impl.push_tr(state, tr)?;
        }
        if final_weight != zero {
            self.cache_impl
                .set_final_weight(state, Some(final_weight))?;
        } else {
            self.cache_impl.set_final_weight(state, None)?;
        }

        Ok(())
    }

    fn compute_start(&mut self) -> Result<Option<usize>> {
        Ok(self.rmeps_state.sd_state.fst.borrow().start())
    }

    fn compute_final(&mut self, state: usize) -> Result<Option<Self::W>> {
        // A bit hacky as the final weight is computed inside the expand function.
        // Should in theory never be called
        self.expand(state)?;
        let weight = self.cache_impl.final_weight(state)?;
        Ok(weight.cloned())
    }
}
