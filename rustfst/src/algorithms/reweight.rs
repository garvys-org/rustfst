use anyhow::Result;

use crate::fst_traits::MutableFst;
use crate::semirings::{DivideType, WeaklyDivisibleSemiring};

/// Different types of reweighting.
#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum ReweightType {
    /// Reweight toward initial state.
    ReweightToInitial,
    /// Reweight toward final states.
    ReweightToFinal,
}

/// Reweights an FST according to a vector of potentials in a given direction.
/// The weight must be left distributive when reweighting towards the initial
/// state and right distributive when reweighting towards the final states.
///
/// A transition of weight w, with an origin state of potential p and destination state
/// of potential q, is reweighted by p^-1 \otimes (w \otimes q) when reweighting
/// torwards the initial state, and by (p \otimes w) \otimes q^-1 when
/// reweighting towards the final states.
pub fn reweight<W, F>(fst: &mut F, potentials: &[W], reweight_type: ReweightType) -> Result<()>
where
    F: MutableFst<W>,
    W: WeaklyDivisibleSemiring,
{
    let zero = W::zero();
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
                    if let Some(final_weight) = fst.final_weight(state)? {
                        let new_weight = W::zero().times(final_weight)?;
                        fst.set_final(state, new_weight)?;
                    }
                }
            };
            continue;
        }

        let d_s = potentials.get(state).unwrap_or(&zero);

        if d_s.is_zero() {
            continue;
        }

        for tr in fst.tr_iter_mut(state)? {
            let d_ns = potentials.get(tr.nextstate).unwrap_or(&zero);

            if d_ns.is_zero() {
                continue;
            }

            tr.weight = match reweight_type {
                ReweightType::ReweightToInitial => {
                    (&tr.weight.times(d_ns)?).divide(d_s, DivideType::DivideLeft)?
                }
                ReweightType::ReweightToFinal => {
                    (d_s.times(&tr.weight)?).divide(&d_ns, DivideType::DivideRight)?
                }
            };
        }
    }

    for state_id in 0..fst.num_states() {
        if let Some(final_weight) = unsafe { fst.final_weight_unchecked_mut(state_id) } {
            let d_s = potentials.get(state_id).unwrap_or(&zero);

            match reweight_type {
                ReweightType::ReweightToFinal => {
                    final_weight.times_assign(d_s)?;
                }
                ReweightType::ReweightToInitial => {
                    if d_s.is_zero() {
                        continue;
                    }
                    final_weight.divide_assign(d_s, DivideType::DivideLeft)?;
                }
            };
        }
    }

    // Handles potential of the start state
    if let Some(start_state) = fst.start() {
        let d_s = potentials.get(start_state).unwrap_or(&zero);

        if !d_s.is_one() && !d_s.is_zero() {
            for tr in fst.tr_iter_mut(start_state)? {
                tr.weight = match reweight_type {
                    ReweightType::ReweightToInitial => d_s.times(&tr.weight)?,
                    ReweightType::ReweightToFinal => {
                        (W::one().divide(&d_s, DivideType::DivideRight)?).times(&tr.weight)?
                    }
                };
            }

            if let Some(final_weight) = fst.final_weight(start_state)? {
                let new_weight = match reweight_type {
                    ReweightType::ReweightToInitial => d_s.times(final_weight)?,
                    ReweightType::ReweightToFinal => {
                        (W::one().divide(&d_s, DivideType::DivideRight)?).times(final_weight)?
                    }
                };

                fst.set_final(start_state, new_weight)?;
            }
        }
    }

    Ok(())
}
