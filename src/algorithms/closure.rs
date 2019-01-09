use crate::arc::Arc;
use crate::fst_traits::{CoreFst, FinalStatesIterator, MutableFst};
use crate::semirings::Semiring;
use crate::EPS_LABEL;

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
pub fn closure_plus<F>(fst: &mut F)
where
    F: MutableFst,
{
    // Add an epsilon arc from each final states to the start state
    if let Some(start_state) = fst.start() {
        let final_states_id: Vec<_> = fst.final_states_iter().map(|u| u.state_id).collect();
        for final_state_id in final_states_id {
            fst.add_arc(
                final_state_id,
                Arc::new(EPS_LABEL, EPS_LABEL, <F as CoreFst>::W::ONE, start_state),
            )
            .unwrap();
        }
    }
}

/// This operation computes the concatenative closure.
/// If A transduces string `x` to `y` with weight `a`,
/// then the closure transduces `x` to `y` with weight `a`,
/// `xx` to `yy` with weight `a ⊗ a`, `xxx` to `yyy` with weight `a ⊗ a ⊗ a`, etc.
/// The empty string is transduced to itself with weight `1` as well.
pub fn closure_star<F>(fst: &mut F)
where
    F: MutableFst,
{
    closure_plus(fst);

    // Add a new start state to allow empty path
    let start_state = fst.start();
    if let Some(start_state_id) = start_state {
        let new_start_state_id = fst.add_state();
        fst.set_start(new_start_state_id).unwrap();
        fst.add_arc(
            new_start_state_id,
            Arc::new(EPS_LABEL, EPS_LABEL, <F as CoreFst>::W::ONE, start_state_id),
        )
        .unwrap();
    }
}
