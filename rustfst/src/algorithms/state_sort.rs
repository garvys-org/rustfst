use std::mem::swap;

use anyhow::{ensure, Result};

use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::StateId;

/// Sorts the input states of an FST. order[i] gives the the state ID after
/// sorting that corresponds to the state ID i before sorting; it must
/// therefore be a permutation of the input FST's states ID sequence.
pub fn state_sort<F>(fst: &mut F, order: &[StateId]) -> Result<()>
where
    F: MutableFst + ExpandedFst,
{
    ensure!(
        order.len() == fst.num_states(),
        "StateSort : Bad order vector size : {}. Expected {}",
        order.len(),
        fst.num_states()
    );
    if fst.start().is_none() {
        return Ok(());
    }
    let start_state = fst.start().unwrap();

    let mut done = vec![false; order.len()];

    if cfg!(debug_assertions) {
        assert!(start_state < order.len());
        assert!(order[start_state] < fst.num_states());
    }

    fst.set_start(order[start_state])?;

    let states: Vec<_> = fst.states_iter().collect();
    for mut s1 in states {
        if done[s1] {
            continue;
        }
        let mut final1 = unsafe { fst.final_weight_unchecked(s1) }.cloned();
        let mut final2 = None;
        let mut arcsa: Vec<_> = fst.arcs_iter(s1)?.cloned().collect();
        let mut arcsb = vec![];
        while !done[s1] {
            let s2 = order[s1];
            if !done[s2] {
                final2 = unsafe { fst.final_weight_unchecked(s2) }.cloned();
                arcsb = fst.arcs_iter(s2)?.cloned().collect();
            }
            match final1 {
                None => fst.delete_final_weight(s2)?,
                Some(v) => fst.set_final(s2, v.clone())?,
            };
            fst.delete_trs(s2)?;
            for arc in arcsa.iter() {
                let mut arc = arc.clone();
                arc.nextstate = order[arc.nextstate];
                fst.add_tr(s2, arc)?;
            }
            done[s1] = true;

            // next
            swap(&mut arcsa, &mut arcsb);
            final1 = final2.clone();
            s1 = s2;
        }
    }

    Ok(())
}
