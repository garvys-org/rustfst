use std::collections::HashSet;

use failure::Fallible;

use crate::algorithms::connect;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use crate::semirings::Semiring;
use crate::Arc;

/// Removes final states that have epsilon-only input arcs.
pub fn rm_final_epsilon<F>(ifst: &mut F) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
{
    let mut visitors = SccVisitor::new(ifst, false, true);
    dfs_visit(ifst, &mut visitors, false);

    let mut finals = HashSet::new();

    for final_state in ifst.final_states_iter() {
        let final_state_id = final_state.state_id;
        let final_weight = final_state.final_weight;

        if final_weight.is_zero() {
            continue;
        }

        let mut future_coaccess = false;

        for arc in ifst.arcs_iter(final_state_id)? {
            if visitors.coaccess[arc.nextstate] {
                future_coaccess = true;
                break;
            }
        }

        if !future_coaccess {
            finals.insert(final_state_id);
        }
    }

    let states: Vec<_> = ifst.states_iter().collect();

    for state in states {
        let mut arcs = vec![];
        let mut weight = ifst.final_weight(state).unwrap_or_else(F::W::zero);

        for arc in ifst.arcs_iter(state).unwrap() {
            if finals.contains(&arc.nextstate) {
                if arc.ilabel == 0 && arc.olabel == 0 {
                    weight.plus_assign(
                        ifst.final_weight(arc.nextstate)
                            .unwrap()
                            .times(&arc.weight)?,
                    )?;
                } else {
                    arcs.push(arc);
                }
            } else {
                arcs.push(arc);
            }
        }

        if arcs.len() < ifst.num_arcs(state).unwrap() {
            let arcs_owned: Vec<Arc<F::W>> = arcs.into_iter().cloned().collect();
            ifst.delete_arcs(state)?;
            if !weight.is_zero() {
                ifst.set_final(state, weight)?;
            }
            for arc in arcs_owned.into_iter() {
                ifst.add_arc(state, arc)?;
            }
        }
    }

    connect(ifst)?;

    Ok(())
}
