use arc::Arc;
use fst_traits::{CoreFst, MutableFst};
use semirings::Semiring;
use std::cmp;
use Label;

pub fn transducer<T: Iterator<Item = Label>, F: MutableFst>(
    labels_input: T,
    labels_output: T,
) -> F {
    let mut vec_labels_input: Vec<_> = labels_input.collect();
    let mut vec_labels_output: Vec<_> = labels_output.collect();

    let max_size = cmp::max(vec_labels_input.len(), vec_labels_output.len());

    vec_labels_input.resize(max_size, 0);
    vec_labels_output.resize(max_size, 0);

    let mut fst = F::new();
    let mut state_cour = fst.add_state();
    fst.set_start(&state_cour);

    for (i, o) in vec_labels_input.iter().zip(vec_labels_output.iter()) {
        let new_state = fst.add_state();
        fst.add_arc(
            &state_cour,
            Arc::new(*i, *o, <F as CoreFst>::W::one(), new_state),
        );
        state_cour = new_state;
    }

    fst.set_final(&state_cour, <F as CoreFst>::W::one());

    fst
}
