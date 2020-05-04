use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction};
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::{Tr, EPS_LABEL};
use unsafe_unwrap::UnsafeUnwrap;

/// The WeightConverter interfaces defines how a weight should be turned into another one.
/// Useful for changing the semiring of an FST.
pub trait WeightConverter<SI: Semiring, SO: Semiring> {
    fn tr_map(&mut self, arc: &Tr<SI>) -> Result<Tr<SO>>;
    fn final_tr_map(&mut self, final_tr: &FinalTr<SI>) -> Result<FinalTr<SO>>;
    fn final_action(&self) -> MapFinalAction;
}

/// Convert an FST in a given Semiring to another Semiring using a WeightConverter
/// to specify how the conversion should be performed.
pub fn weight_convert<F1, F2, M>(fst_in: &F1, mapper: &mut M) -> Result<F2>
where
    F1: ExpandedFst,
    F2: MutableFst + AllocableFst,
    M: WeightConverter<F1::W, F2::W>,
{
    let mut fst_out = F2::new();
    let final_action = mapper.final_action();

    // Empty FST.
    if fst_in.start().is_none() {
        return Ok(fst_out);
    }

    // Reserve enough space for all the states to avoid re-allocations.
    let mut num_states_needed = fst_in.num_states();
    if !(final_action == MapFinalAction::MapNoSuperfinal) {
        num_states_needed += 1;
    }
    fst_out.reserve_states(num_states_needed);

    // Add all the states from the input fst to the output fst.
    for _ in fst_in.states_iter() {
        fst_out.add_state();
    }

    // Set superfinal states as final.
    let mut superfinal = None;
    if final_action == MapFinalAction::MapRequireSuperfinal {
        superfinal = Some(fst_out.add_state());
        fst_out.set_final(superfinal.unwrap(), F2::W::one())?;
    }

    if let Some(start_state) = fst_in.start() {
        fst_out.set_start(start_state)?;
    }

    let states: Vec<_> = fst_in.states_iter().collect();
    for state in states {
        fst_out.reserve_trs(state, fst_in.num_trs(state)?)?;
        for arc in fst_in.arcs_iter(state)? {
            fst_out.add_tr(state, mapper.tr_map(arc)?)?;
        }
        if let Some(w) = unsafe { fst_in.final_weight_unchecked(state) } {
            let final_tr = FinalTr {
                ilabel: EPS_LABEL,
                olabel: EPS_LABEL,
                weight: w.clone(),
            };
            let mapped_final_tr = mapper.final_tr_map(&final_tr)?;
            match final_action {
                MapFinalAction::MapNoSuperfinal => {
                    if mapped_final_tr.ilabel != EPS_LABEL || mapped_final_tr.olabel != EPS_LABEL {
                        bail!("TrMap: Non-zero arc labels for superfinal arc")
                    }

                    fst_out.set_final(state, mapped_final_tr.weight).unwrap();
                }
                MapFinalAction::MapAllowSuperfinal => {
                    if mapped_final_tr.ilabel != EPS_LABEL || mapped_final_tr.olabel != EPS_LABEL {
                        if superfinal.is_none() {
                            let superfinal_id = fst_out.add_state();
                            superfinal = Some(superfinal_id);
                            fst_out.set_final(superfinal_id, F2::W::one()).unwrap();
                        }

                        fst_out.add_tr(
                            state,
                            Tr::new(
                                mapped_final_tr.ilabel,
                                mapped_final_tr.olabel,
                                mapped_final_tr.weight,
                                unsafe { superfinal.unsafe_unwrap() },
                            ),
                        )?;

                        fst_out.delete_final_weight(state)?;
                    } else {
                        fst_out.set_final(state, mapped_final_tr.weight)?;
                    }
                }
                MapFinalAction::MapRequireSuperfinal => {
                    if mapped_final_tr.ilabel != EPS_LABEL
                        || mapped_final_tr.olabel != EPS_LABEL
                        || !mapped_final_tr.weight.is_zero()
                    {
                        fst_out
                            .add_tr(
                                state,
                                Tr::new(
                                    mapped_final_tr.ilabel,
                                    mapped_final_tr.olabel,
                                    mapped_final_tr.weight,
                                    superfinal.unwrap(),
                                ),
                            )
                            .unwrap();
                    }
                    fst_out.delete_final_weight(state).unwrap();
                }
            }
        }
    }

    Ok(fst_out)
}
