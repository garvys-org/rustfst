use algorithms::closure_plus;
use arc::Arc;
use fst_traits::{CoreFst, ExpandedFst, MutableFst};
use semirings::Semiring;
use Result;
use EPS_LABEL;

pub fn closure_star<F>(fst: &mut F) -> Result<()>
where
    F: ExpandedFst + MutableFst,
{
    closure_plus(fst)?;

    // Add a new start state to allow empty path
    let start_state = fst.start();
    if let Some(start_state_id) = start_state {
        let new_start_state_id = fst.add_state();
        fst.set_start(&new_start_state_id)?;
        fst.add_arc(
            &new_start_state_id,
            Arc::new(
                EPS_LABEL,
                EPS_LABEL,
                <F as CoreFst>::W::one(),
                start_state_id,
            ),
        )?;
    }
    Ok(())
}
