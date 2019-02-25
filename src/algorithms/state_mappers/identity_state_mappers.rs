use crate::algorithms::StateMapper;
use crate::fst_traits::MutableFst;

/// Mapper that returns its input.
pub struct IdentityStateMapper {}

impl<F: MutableFst> StateMapper<F> for IdentityStateMapper {
    fn map_final_weight(&self, _weight: Option<&mut F::W>) {}

    fn map_arcs(&self, _fst: &mut F, _state: usize) {}
}
