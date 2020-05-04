use crate::fst_impls::vector_fst::VectorFst;
use crate::fst_traits::AllocableFst;
use crate::semirings::Semiring;
use crate::StateId;
use anyhow::Result;

impl<W: 'static + Semiring> AllocableFst for VectorFst<W> {
    fn reserve_trs(&mut self, source: usize, additional: usize) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .reserve(additional);
        Ok(())
    }

    #[inline]
    unsafe fn reserve_trs_unchecked(&mut self, source: usize, additional: usize) {
        self.states
            .get_unchecked_mut(source)
            .arcs
            .reserve(additional)
    }

    #[inline]
    fn reserve_states(&mut self, additional: usize) {
        self.states.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.states.shrink_to_fit();
        for state in self.states.iter_mut() {
            state.arcs.shrink_to_fit();
        }
    }

    #[inline]
    fn shrink_to_fit_states(&mut self) {
        self.states.shrink_to_fit()
    }

    fn shrink_to_fit_trs(&mut self, source: StateId) -> Result<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .shrink_to_fit();
        Ok(())
    }

    #[inline]
    unsafe fn shrink_to_fit_trs_unchecked(&mut self, source: StateId) {
        self.states.get_unchecked_mut(source).arcs.shrink_to_fit()
    }

    #[inline]
    fn states_capacity(&self) -> usize {
        self.states.capacity()
    }

    fn arcs_capacity(&self, source: StateId) -> Result<usize> {
        Ok(self
            .states
            .get(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .capacity())
    }

    #[inline]
    unsafe fn arcs_capacity_unchecked(&self, source: StateId) -> usize {
        self.states.get_unchecked(source).arcs.capacity()
    }
}
