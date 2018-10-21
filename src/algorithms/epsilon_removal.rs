use algorithms::all_pairs_shortest_distance;
use arc::Arc;
use fst_traits::{ExpandedFst, FinalStatesIterator, MutableFst};
use semirings::{Semiring, StarSemiring};
use std::collections::HashMap;
use Result;
use EPS_LABEL;

// Compute the wFST derived from "fst" by keeping only the epsilon transitions
fn compute_fst_epsilon<W, F1, F2>(fst: &F1, keep_only_epsilon: bool) -> Result<F2>
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let mut fst_epsilon = F2::new();

    // Map old states id to new ones
    let mut mapping_states = HashMap::new();

    // First pass to add the necessary states
    for old_state_id in fst.states_iter() {
        let new_state_id = fst_epsilon.add_state();
        mapping_states.insert(old_state_id, new_state_id);
    }

    // Second pass to add the arcs
    for old_state_id in fst.states_iter() {
        for old_arc in fst.arcs_iter(&old_state_id)? {
            let a = keep_only_epsilon && old_arc.ilabel == EPS_LABEL && old_arc.olabel == EPS_LABEL;
            let b =
                !keep_only_epsilon && !(old_arc.ilabel == EPS_LABEL && old_arc.olabel == EPS_LABEL);

            if a || b {
                fst_epsilon.add_arc(
                    &mapping_states[&old_state_id],
                    Arc::new(
                        old_arc.ilabel,
                        old_arc.olabel,
                        old_arc.weight.clone(),
                        mapping_states[&old_arc.nextstate],
                    ),
                )?;
            }
        }
    }

    if let Some(start_state) = fst.start() {
        fst_epsilon.set_start(&mapping_states[&start_state])?;
    }

    for old_final_state in fst.final_states_iter() {
        fst_epsilon.set_final(
            &mapping_states[&old_final_state.state_id],
            old_final_state.final_weight,
        )?;
    }
    Ok(fst_epsilon)
}

pub fn epsilon_removal<W, F1, F2>(fst: &F1) -> Result<F2>
where
    W: StarSemiring,
    F1: ExpandedFst<W = W>,
    F2: MutableFst<W = W> + ExpandedFst<W = W>,
{
    let fst_epsilon: F2 = compute_fst_epsilon(fst, true)?;
    let dists_fst_epsilon = all_pairs_shortest_distance(&fst_epsilon)?;

    let mut eps_closures = vec![vec![]; fst_epsilon.num_states()];

    for p in fst_epsilon.states_iter() {
        for q in fst_epsilon.states_iter() {
            if p != q && dists_fst_epsilon[p][q] != W::zero() {
                eps_closures[p].push((q, &dists_fst_epsilon[p][q]));
            }
        }
    }

    let fst_no_epsilon: F2 = compute_fst_epsilon(fst, false)?;

    let mut output_fst = fst_no_epsilon.clone();

    for p in fst_no_epsilon.states_iter() {
        for (q, w_prime) in &eps_closures[p] {
            for arc in fst_no_epsilon.arcs_iter(q)? {
                output_fst.add_arc(
                    &p,
                    Arc::new(
                        arc.ilabel,
                        arc.olabel,
                        w_prime.times(&arc.weight),
                        arc.nextstate,
                    ),
                )?;
            }

            if fst_no_epsilon.is_final(q) {
                if !fst_no_epsilon.is_final(&p) {
                    output_fst.set_final(&p, W::zero())?;
                }
                let rho_prime_p = output_fst.final_weight(&p).unwrap();
                let rho_p = fst_no_epsilon.final_weight(&p).unwrap_or(W::zero());
                let new_weight = rho_prime_p.plus(&w_prime.times(&rho_p));
                output_fst.set_final(&p, new_weight)?;
            }
        }
    }

    Ok(output_fst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use counter::Counter;
    use failure::ResultExt;
    use fst_impls::VectorFst;
    use fst_traits::PathsIterator;
    use semirings::IntegerWeight;
    use test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_epsilon_removal_generic() {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let mut paths_ref: Counter<_> = fst.paths_iter().collect();

            let epsilon_removed_fst: VectorFst<IntegerWeight> = epsilon_removal(fst)
                .with_context(|_| {
                    format_err!(
                        "Error when performing epsilon removal operation for wFST {:?}",
                        &data.name,
                    )
                }).unwrap();
            let paths: Counter<_> = epsilon_removed_fst.paths_iter().collect();

            assert_eq!(
                paths, paths_ref,
                "Test failing for epsilon removal for wFST {:?}",
                &data.name
            );
        }
    }
}
