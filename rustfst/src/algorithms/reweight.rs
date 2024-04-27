use anyhow::Result;

use crate::fst_properties::mutable_properties::reweight_properties;
use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::{DivideType, WeaklyDivisibleSemiring};
use crate::{StateId, Tr, EPS_LABEL};

/// Different types of reweighting.
#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub enum ReweightType {
    /// Reweight toward initial state.
    ReweightToInitial,
    /// Reweight toward final states.
    ReweightToFinal,
}

/// Reweight an FST according to a vector of potentials in a given direction.
///
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

    for state in 0..(num_states as StateId) {
        // This handles elements past the end of the potentials array.
        if state as usize >= potentials.len() {
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

        let d_s = potentials.get(state as usize).unwrap_or(&zero);

        if d_s.is_zero() {
            continue;
        }

        unsafe {
            let mut it_tr = fst.tr_iter_unchecked_mut(state);
            for idx_tr in 0..it_tr.len() {
                let tr = it_tr.get_unchecked(idx_tr);
                let d_ns = potentials.get(tr.nextstate as usize).unwrap_or(&zero);

                if d_ns.is_zero() {
                    continue;
                }

                let weight = match reweight_type {
                    ReweightType::ReweightToInitial => {
                        tr.weight.times(d_ns)?.divide(d_s, DivideType::DivideLeft)?
                    }
                    ReweightType::ReweightToFinal => {
                        (d_s.times(&tr.weight)?).divide(d_ns, DivideType::DivideRight)?
                    }
                };

                it_tr.set_weight_unchecked(idx_tr, weight);
            }
        }
    }

    for state_id in 0..(fst.num_states() as StateId) {
        if let Some(mut final_weight) = unsafe { fst.final_weight_unchecked(state_id) } {
            let d_s = potentials.get(state_id as usize).unwrap_or(&zero);

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

            unsafe { fst.set_final_unchecked(state_id, final_weight) };
        }
    }

    // Handles potential of the start state
    if let Some(start_state) = fst.start() {
        let d_s = potentials.get(start_state as usize).unwrap_or(&zero);

        if !d_s.is_one() && !d_s.is_zero() {
            fst.compute_and_update_properties(FstProperties::INITIAL_ACYCLIC)?;
            if fst.properties().contains(FstProperties::INITIAL_ACYCLIC) {
                unsafe {
                    let mut it_tr = fst.tr_iter_unchecked_mut(start_state);
                    for idx_tr in 0..it_tr.len() {
                        let tr = it_tr.get_unchecked(idx_tr);
                        let weight = match reweight_type {
                            ReweightType::ReweightToInitial => d_s.times(&tr.weight)?,
                            ReweightType::ReweightToFinal => (W::one()
                                .divide(d_s, DivideType::DivideRight)?)
                            .times(&tr.weight)?,
                        };
                        it_tr.set_weight_unchecked(idx_tr, weight);
                    }
                }
                if let Some(final_weight) = fst.final_weight(start_state)? {
                    let new_weight = match reweight_type {
                        ReweightType::ReweightToInitial => d_s.times(final_weight)?,
                        ReweightType::ReweightToFinal => {
                            (W::one().divide(d_s, DivideType::DivideRight)?).times(final_weight)?
                        }
                    };

                    fst.set_final(start_state, new_weight)?;
                }
            } else {
                let s = fst.add_state();
                let weight = match reweight_type {
                    ReweightType::ReweightToInitial => d_s.clone(),
                    ReweightType::ReweightToFinal => {
                        W::one().divide(d_s, DivideType::DivideRight)?
                    }
                };
                fst.add_tr(s, Tr::new(EPS_LABEL, EPS_LABEL, weight, start_state))?;
                fst.set_start(s)?;
            }
        }
    }

    fst.set_properties_with_mask(
        reweight_properties(fst.properties()),
        FstProperties::all_properties(),
    );

    Ok(())
}
