use fst::ExpandedFst;
use semirings::Semiring;

pub fn project<W: Semiring, F: ExpandedFst<W>>(fst: &mut F, project_input: bool) {
    for state_id in 0..fst.num_states() {
        for mut arc in fst.arc_iter(&state_id) {
            if project_input {
                arc.olabel = arc.ilabel;
            } else {
                arc.ilabel = arc.olabel;
            }
        }
    }
}
