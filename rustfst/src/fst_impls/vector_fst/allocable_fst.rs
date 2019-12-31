use crate::{StateId};
use crate::fst_impls::vector_fst::{VectorFst, VectorFstState};
use crate::fst_traits::AllocableFst;
use crate::semirings::Semiring;
use failure::Fallible;


impl<W: 'static + Semiring> AllocableFst for VectorFst<W> {

    fn reserve_arcs(&mut self, source: usize, additional: usize) -> Fallible<()> {
        self.states
            .get_mut(source)
            .ok_or_else(|| format_err!("State {:?} doesn't exist", source))?
            .arcs
            .reserve(additional);
        Ok(())
    }

    #[inline]
    unsafe fn reserve_arcs_unchecked(&mut self, source: usize, additional: usize) {
        self.states
            .get_unchecked_mut(source)
            .arcs
            .reserve(additional)
    }

    fn reserve_states(&mut self, additional: usize) {
        self.states.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {

    }

    fn shrink_to_fit_states(&mut self) {

    }

    fn shrink_to_fit_arcs(&mut self, source: StateId) -> Fallible<()> {
        Ok(())
    }

    unsafe fn shrink_to_fit_arcs_unchecked(&mut self, source: StateId) {

    }


    fn states_capacity(&self) -> usize {
        0
    }
    fn arcs_capacity(&self, source: StateId) -> Fallible<usize> {
        Ok(0)
    }
    unsafe fn arcs_capacity_unchecked(&self) -> usize {
        0
    }
}

