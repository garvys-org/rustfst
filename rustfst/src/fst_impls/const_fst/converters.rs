use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;

impl<W: Semiring + 'static> From<VectorFst<W>> for ConstFst<W> {
    fn from(ifst: VectorFst<W>) -> Self {
        let mut const_states = Vec::with_capacity(ifst.num_states());
        let mut const_arcs = Vec::with_capacity(ifst.states.iter().map(|s| s.arcs.len()).sum());
        let mut pos = 0;
        for s in ifst.states.into_iter() {
            let niepsilons = s.num_input_epsilons();
            let noepsilons = s.num_output_epsilons();
            const_states.push(ConstState {
                final_weight: s.final_weight,
                pos,
                narcs: s.arcs.len(),
                niepsilons,
                noepsilons,
            });

            pos += s.arcs.len();

            const_arcs.extend(s.arcs.into_iter());
        }

        ConstFst {
            states: const_states,
            arcs: const_arcs,
            start: ifst.start_state,
            isymt: ifst.isymt,
            osymt: ifst.osymt,
        }
    }
}
