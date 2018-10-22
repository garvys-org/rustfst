use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::Semiring;
use Result;
use EPS_LABEL;

pub fn closure_plus<F>(fst: &mut F) -> Result<()>
where
    F: ExpandedFst + MutableFst,
{
    // Add an epsilon arc from each final states to the start state
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst.final_states_iter().map(|u| u.state_id).collect();
        for final_state_id in final_states_id {
            fst.add_arc(
                &final_state_id,
                Arc::new(EPS_LABEL, EPS_LABEL, <F as CoreFst>::W::one(), start_state),
            )?;
        }
    }
    Ok(())
}
