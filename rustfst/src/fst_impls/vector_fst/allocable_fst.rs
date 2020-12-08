use crate::fst_impls::vector_fst::VectorFst;
use crate::fst_traits::AllocableFst;
use crate::semirings::Semiring;
use crate::StateId;
use anyhow::Result;
use std::sync::Arc;

impl<W: 'static + Semiring> AllocableFst<W> for VectorFst<W> {
    fn reserve_trs(&mut self, source: StateId, additional: usize) -> Result<()> {
        let trs = &mut self
            .states
            .get_mut(source as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs;

        Arc::make_mut(&mut trs.0).reserve(additional);
        Ok(())
    }

    #[inline]
    unsafe fn reserve_trs_unchecked(&mut self, source: StateId, additional: usize) {
        let trs = &mut self.states.get_unchecked_mut(source as usize).trs;
        Arc::make_mut(&mut trs.0).reserve(additional)
    }

    #[inline]
    fn reserve_states(&mut self, additional: usize) {
        self.states.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.states.shrink_to_fit();
        for state in self.states.iter_mut() {
            Arc::make_mut(&mut state.trs.0).shrink_to_fit();
        }
    }

    #[inline]
    fn shrink_to_fit_states(&mut self) {
        self.states.shrink_to_fit()
    }

    fn shrink_to_fit_trs(&mut self, source: StateId) -> Result<()> {
        let trs = &mut self
            .states
            .get_mut(source as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs;
        Arc::make_mut(&mut trs.0).shrink_to_fit();
        Ok(())
    }

    #[inline]
    unsafe fn shrink_to_fit_trs_unchecked(&mut self, source: StateId) {
        Arc::make_mut(&mut self.states.get_unchecked_mut(source as usize).trs.0).shrink_to_fit()
    }

    #[inline]
    fn states_capacity(&self) -> usize {
        self.states.capacity()
    }

    fn trs_capacity(&self, source: StateId) -> Result<usize> {
        Ok(self
            .states
            .get(source as usize)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .trs
            .0
            .capacity())
    }

    #[inline]
    unsafe fn trs_capacity_unchecked(&self, source: StateId) -> usize {
        self.states.get_unchecked(source as usize).trs.0.capacity()
    }
}
