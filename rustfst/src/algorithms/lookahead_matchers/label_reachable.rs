use crate::fst_impls::VectorFst;
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::Label;
use std::collections::HashMap;

pub struct LabelReachableData {
    reach_input: bool,
    final_label: Label,
    label2index: HashMap<Label, Label>,
}

pub struct LabelReachable<W: Semiring> {
    fst: VectorFst<W>,
    reach_input: bool,
}

impl<W: Semiring + 'static> LabelReachable<W> {
    pub fn new(fst: VectorFst<W>, reach_input: bool) -> Self {
        let mut label_reachable = Self { fst, reach_input };

        let nstates = label_reachable.fst.num_states();
        label_reachable.transform_fst();
        // label_reachable.find_intervals(nstates);

        label_reachable
    }

    // Redirects labeled arcs (input or output labels determined by ReachInput())
    // to new label-specific final states. Each original final state is
    // redirected via a transition labeled with kNoLabel to a new
    // kNoLabel-specific final state. Creates super-initial state for all states
    // with zero in-degree.
    fn transform_fst(&mut self) {}
}
