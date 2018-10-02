use arc::Arc;
use fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use Result;

pub fn concat<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> Result<F3>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    let mapping_states_fst_1 = fst_out.add_fst(fst_1)?;
    let mapping_states_fst_2 = fst_out.add_fst(fst_2)?;

    // Start state is the start state of the first fst
    let old_start_state = fst_1.start().ok_or_else(|| format_err!("Fst doesn't have a start state"))?;
    fst_out.set_start(&mapping_states_fst_1[&old_start_state])?;

    // Final states of the first epsilon are connected to the start state of the second one
    let old_start_state_2 = fst_2.start().ok_or_else(|| format_err!("Fst doesn't have a start state"))?;
    let start_state_2 = &mapping_states_fst_2[&old_start_state_2];
    for old_final_state_1 in fst_1.final_states_iter() {
        let final_state_1 = &mapping_states_fst_1[&old_final_state_1];
        fst_out.add_arc(start_state_2, Arc::new(0, 0, W::one(), *final_state_1))?;
    }

    // Final states are final states of the second fst
    for old_final_state in fst_2.final_states_iter() {
        let final_state = &mapping_states_fst_2[&old_final_state];
        let final_weight = fst_out.final_weight(&old_final_state).ok_or_else(|| format_err!("State {:?} is not final", old_final_state))?;
        fst_out.set_final(final_state, final_weight)?;
    }

    // FINISH

    Ok(fst_out)
}
