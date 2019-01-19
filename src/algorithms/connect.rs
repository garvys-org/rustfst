use failure::Fallible;

use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::StateId;
use std::collections::HashSet;

pub(crate) fn dfs<F: Fst>(
    fst: &F,
    state_id_cour: StateId,
    accessible_states: &mut HashSet<StateId>,
    coaccessible_states: &mut HashSet<StateId>,
) -> Fallible<()> {
    accessible_states.insert(state_id_cour);
    let mut is_coaccessible = fst.is_final(state_id_cour);
    for arc in fst.arcs_iter(state_id_cour)? {
        let nextstate = arc.nextstate;

        if !accessible_states.contains(&nextstate) {
            dfs(fst, nextstate, accessible_states, coaccessible_states)?;
        }

        if coaccessible_states.contains(&nextstate) {
            is_coaccessible = true;
        }
    }

    if is_coaccessible {
        coaccessible_states.insert(state_id_cour);
    }

    Ok(())
}

/// This operation trims an FST, removing states and arcs that are not on successful paths.
///
/// # Example
/// ```
/// # #[macro_use] extern crate rustfst;
/// # use rustfst::utils::transducer;
/// # use rustfst::semirings::{Semiring, IntegerWeight};
/// # use rustfst::fst_impls::VectorFst;
/// # use rustfst::algorithms::connect;
/// # use rustfst::fst_traits::MutableFst;
/// let fst : VectorFst<IntegerWeight> = fst![2 => 3];
///
/// // Add a state not on a successful path
/// let mut no_connected_fst = fst.clone();
/// no_connected_fst.add_state();
///
/// let mut connected_fst = no_connected_fst.clone();
/// connect(&mut connected_fst);
///
/// assert_eq!(connected_fst, fst);
/// ```
pub fn connect<F: ExpandedFst + MutableFst>(fst: &mut F) -> Fallible<()> {
    let mut accessible_states = HashSet::new();
    let mut coaccessible_states = HashSet::new();

    if let Some(state_id) = fst.start() {
        dfs(
            fst,
            state_id,
            &mut accessible_states,
            &mut coaccessible_states,
        )?;
    }

    let mut to_delete = Vec::new();
    for i in 0..fst.num_states() {
        if !accessible_states.contains(&i) || !coaccessible_states.contains(&i) {
            to_delete.push(i);
        }
    }
    fst.del_states(to_delete)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::vector_fst::get_vector_fsts_for_tests;

    #[test]
    fn test_connect_generic() -> Fallible<()> {
        for data in get_vector_fsts_for_tests() {
            let fst = &data.fst;

            let mut connect_fst = fst.clone();
            connect(&mut connect_fst)?;

            assert_eq!(
                connect_fst, data.connected_fst,
                "Connect test fail for fst : {:?}",
                &data.name
            );
        }
        Ok(())
    }
}
