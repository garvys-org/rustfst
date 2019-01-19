use failure::Fallible;

use crate::fst_traits::{ExpandedFst, FinalStatesIterator, Fst, MutableFst};
use crate::semirings::{Semiring, WeaklyDivisibleSemiring};

/// Different types of reweighting.
pub enum ReweightType {
    /// Reweight toward initial state.
    ReweightToInitial,
    /// Reweight toward final states.
    ReweightToFinal,
}

macro_rules! state_to_dist {
    ($state: expr, $dist: expr) => {
        $dist
            .get($state)
            .ok_or_else(|| format_err!("State {} not in dists array", $state))?;
    };
}

/// Reweights an FST according to a vector of potentials in a given direction.
/// The weight must be left distributive when reweighting towards the initial
/// state and right distributive when reweighting towards the final states.
///
/// An arc of weight w, with an origin state of potential p and destination state
/// of potential q, is reweighted by p^-1 \otimes (w \otimes q) when reweighting
/// torwards the initial state, and by (p \otimes w) \otimes q^-1 when
/// reweighting towards the final states.
pub fn reweight<F>(fst: &mut F, potentials: &[F::W], reweight_type: ReweightType) -> Fallible<()>
where
    F: Fst + ExpandedFst + MutableFst,
    F::W: WeaklyDivisibleSemiring,
{
    let num_states = fst.num_states();

    if num_states == 0 {
        return Ok(());
    }

    for state in 0..num_states {
        // This handles elements past the end of the potentials array.
        if state >= potentials.len() {
            match reweight_type {
                ReweightType::ReweightToInitial => {}
                ReweightType::ReweightToFinal => {
                    if let Some(final_weight) = fst.final_weight(state) {
                        let new_weight = F::W::zero().times(&final_weight);
                        fst.set_final(state, new_weight)?;
                    }
                }
            };
            continue;
        }

        let d_s = state_to_dist!(state, potentials);

        if d_s.is_zero() {
            continue;
        }

        for arc in fst.arcs_iter_mut(state)? {
            let d_ns = state_to_dist!(arc.nextstate, potentials);

            if d_ns.is_zero() {
                continue;
            }

            arc.weight = match reweight_type {
                ReweightType::ReweightToInitial => d_s.inverse().times(&arc.weight.times(d_ns)),
                ReweightType::ReweightToFinal => (d_s.times(&arc.weight)).times(&d_ns.inverse()),
            };
        }
    }

    let final_states: Vec<_> = fst.final_states_iter().collect();

    for final_state in final_states {
        let d_s = state_to_dist!(final_state.state_id, potentials);

        match reweight_type {
            ReweightType::ReweightToFinal => {
                let new_weight = d_s.times(&final_state.final_weight);
                fst.set_final(final_state.state_id, new_weight)?;
            }
            ReweightType::ReweightToInitial => {
                if d_s.is_zero() {
                    continue;
                }
                let new_weight = d_s.inverse().times(&final_state.final_weight);
                fst.set_final(final_state.state_id, new_weight)?;
            }
        };
    }

    // Handles potential of the start state
    if let Some(start_state) = fst.start() {
        let d_s = state_to_dist!(start_state, potentials);

        if !d_s.is_one() && !d_s.is_zero() {
            for arc in fst.arcs_iter_mut(start_state)? {
                arc.weight = match reweight_type {
                    ReweightType::ReweightToInitial => d_s.times(&arc.weight),
                    ReweightType::ReweightToFinal => {
                        (F::W::one().times(&d_s.inverse())).times(&arc.weight)
                    }
                };
            }

            if let Some(final_weight) = fst.final_weight(start_state) {
                let new_weight = match reweight_type {
                    ReweightType::ReweightToInitial => d_s.times(&final_weight),
                    ReweightType::ReweightToFinal => {
                        (F::W::one().times(&d_s.inverse())).times(&final_weight)
                    }
                };

                fst.set_final(start_state, new_weight)?;
            }
        }
    }

    Ok(())
}
