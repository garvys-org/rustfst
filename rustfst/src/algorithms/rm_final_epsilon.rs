use std::collections::HashSet;

use failure::Fallible;
use unsafe_unwrap::UnsafeUnwrap;

use crate::algorithms::connect;
use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::visitors::SccVisitor;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::EPS_LABEL;

/// Removes final states that have epsilon-only input arcs.
pub fn rm_final_epsilon<F>(ifst: &mut F) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
{
    let mut visitors = SccVisitor::new(ifst, false, true);
    dfs_visit(ifst, &mut visitors, false);

    let mut finals = HashSet::new();

    for s in 0..ifst.num_states() {
        if unsafe { ifst.is_final_unchecked(s) } {
            let mut future_coaccess = false;

            for arc in unsafe { ifst.arcs_iter_unchecked(s) } {
                if visitors.coaccess[arc.nextstate] {
                    future_coaccess = true;
                    break;
                }
            }

            if !future_coaccess {
                finals.insert(s);
            }
        }
    }

    let mut arcs_to_del = vec![];
    for state in 0..ifst.num_states() {
        let mut weight = None;
        arcs_to_del.clear();

        for (idx, arc) in unsafe { ifst.arcs_iter_unchecked(state).enumerate() } {
            if finals.contains(&arc.nextstate) && arc.ilabel == EPS_LABEL && arc.olabel == EPS_LABEL
            {
                unsafe {
                    if weight.is_none() {
                        weight = Some(
                            ifst.final_weight_unchecked(state)
                                .cloned()
                                .unwrap_or_else(F::W::zero),
                        );
                    }
                    weight.as_mut().unsafe_unwrap().plus_assign(
                        ifst.final_weight_unchecked(arc.nextstate)
                            .unsafe_unwrap()
                            .times(&arc.weight)?,
                    )?
                };
                arcs_to_del.push(idx);
            }
        }

        if !arcs_to_del.is_empty() {
            let w = unsafe { weight.unsafe_unwrap() };
            if !w.is_zero() {
                unsafe { ifst.set_final_unchecked(state, w) };
            }
            unsafe { ifst.del_arcs_id_sorted_unchecked(state, &arcs_to_del) };
        }
    }

    connect(ifst)?;

    Ok(())
}
