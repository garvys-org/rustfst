use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use std::sync::Arc;

impl<W: Semiring + 'static> From<VectorFst<W>> for ConstFst<W> {
    fn from(ifst: VectorFst<W>) -> Self {
        let mut const_states = Vec::with_capacity(ifst.num_states());
        let mut const_trs = Vec::with_capacity(ifst.states.iter().map(|s| s.trs.len()).sum());
        let mut pos = 0;
        for mut s in ifst.states.into_iter() {
            let niepsilons = s.num_input_epsilons();
            let noepsilons = s.num_output_epsilons();
            const_states.push(ConstState {
                final_weight: s.final_weight,
                pos,
                ntrs: s.trs.len(),
                niepsilons,
                noepsilons,
            });

            pos += s.trs.len();

            const_trs.extend(Arc::make_mut(&mut s.trs.0).drain(..));
        }

        ConstFst {
            states: const_states,
            trs: Arc::new(const_trs),
            start: ifst.start_state,
            isymt: ifst.isymt,
            osymt: ifst.osymt,
        }
    }
}
