use crate::fst_impls::const_fst::data_structure::ConstState;
use crate::fst_impls::{ConstFst, VectorFst};
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use std::sync::Arc;

impl<W: Semiring> From<VectorFst<W>> for ConstFst<W> {
    fn from(mut ifst: VectorFst<W>) -> Self {
        // Force the computation of all the properties as once stored, they won't be modified in the ConstFst.
        let properties = ifst.compute_and_update_properties_all().unwrap();
        let mut const_states = Vec::with_capacity(ifst.num_states());
        let mut const_trs = Vec::with_capacity(ifst.states.iter().map(|s| s.trs.len()).sum());
        let mut pos = 0;
        for mut s in ifst.states.into_iter() {
            const_states.push(ConstState {
                final_weight: s.final_weight,
                pos,
                ntrs: s.trs.len(),
                niepsilons: s.niepsilons,
                noepsilons: s.noepsilons,
            });

            pos += s.trs.len();

            const_trs.append(Arc::make_mut(&mut s.trs.0));
        }

        ConstFst {
            states: const_states,
            trs: Arc::new(const_trs),
            start: ifst.start_state,
            isymt: ifst.isymt,
            osymt: ifst.osymt,
            properties,
        }
    }
}
