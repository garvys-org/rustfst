use anyhow::Result;

use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Tr;
use crate::{Label, StateId, EPS_LABEL};

/// Struct used to map final weights when performing an arc mapping.
/// It will always be of the form `(EPS_LABEL, EPS_LABEL, final_weight)`
/// where `final_weight` is the `final_weight` of the current state.
///
/// If the mapper modifies the input label or output one,
/// a super final state will need to be created.
#[derive(Clone, Debug)]
pub struct FinalTr<W: Semiring> {
    /// Input label. Default to `EPS_LABEL`.
    pub ilabel: Label,
    /// Output label. Default to `EPS_LABEL`.
    pub olabel: Label,
    /// Weight. Default to the final weight of the current state.
    pub weight: W,
}

/// Determines how final weights are mapped.
#[derive(PartialEq)]
pub enum MapFinalAction {
    /// A final weight is mapped into a final weight. An error is raised if this
    /// is not possible.
    MapNoSuperfinal,
    /// A final weight is mapped to an arc to the superfinal state when the result
    /// cannot be represented as a final weight. The superfinal state will be
    /// added only if it is needed.
    MapAllowSuperfinal,
    /// A final weight is mapped to an arc to the superfinal state unless the
    /// result can be represented as a final weight of weight Zero(). The
    /// superfinal state is always added (if the input is not the empty FST).
    MapRequireSuperfinal,
}

/// The TrMapper interfaces defines how arcs and final weights are mapped.
/// This is useful for implementing operations that do not change the number of
/// arcs.
pub trait TrMapper<S: Semiring> {
    /// How to modify the arcs.
    fn arc_map(&self, arc: &mut Tr<S>) -> Result<()>;

    /// The mapper will be passed final weights as arcs of the form
    /// `FinalTr(EPS_LABEL, EPS_LABEL, weight)`.
    fn final_arc_map(&self, final_arc: &mut FinalTr<S>) -> Result<()>;

    /// Specifies final action the mapper requires (see above).
    fn final_action(&self) -> MapFinalAction;
}

/// Maps every arc in the FST using an `TrMapper` object.
pub fn arc_map<F, M>(ifst: &mut F, mapper: &M) -> Result<()>
where
    F: MutableFst,
    M: TrMapper<F::W>,
{
    if ifst.start().is_none() {
        return Ok(());
    }

    let final_action = mapper.final_action();
    let mut superfinal: Option<StateId> = None;

    if final_action == MapFinalAction::MapRequireSuperfinal {
        let superfinal_id = ifst.add_state();
        superfinal = Some(superfinal_id);
        ifst.set_final(superfinal_id, F::W::one()).unwrap();
    }

    // TODO: Remove this collect
    let states: Vec<_> = ifst.states_iter().collect();
    for state in states {
        for arc in unsafe { ifst.arcs_iter_unchecked_mut(state) } {
            mapper.arc_map(arc)?;
        }

        if let Some(w) = unsafe { ifst.final_weight_unchecked_mut(state) } {
            let mut final_arc = FinalTr {
                ilabel: EPS_LABEL,
                olabel: EPS_LABEL,
                weight: w.clone(),
            };
            mapper.final_arc_map(&mut final_arc)?;
            match final_action {
                MapFinalAction::MapNoSuperfinal => {
                    if final_arc.ilabel != EPS_LABEL || final_arc.olabel != EPS_LABEL {
                        bail!("TrMap: Non-zero arc labels for superfinal arc")
                    }
                    unsafe {
                        ifst.set_final_unchecked(state, final_arc.weight);
                    }
                }
                MapFinalAction::MapAllowSuperfinal => {
                    if Some(state) != superfinal {
                        if final_arc.ilabel != EPS_LABEL || final_arc.olabel != EPS_LABEL {
                            if superfinal.is_none() {
                                let superfinal_id = ifst.add_state();
                                superfinal = Some(superfinal_id);
                                unsafe {
                                    // Checked because the state is created just above
                                    ifst.set_final_unchecked(superfinal_id, F::W::one());
                                }
                            }
                            unsafe {
                                // Checked
                                ifst.add_arc_unchecked(
                                    state,
                                    Tr::new(
                                        final_arc.ilabel,
                                        final_arc.olabel,
                                        final_arc.weight,
                                        superfinal.unwrap(), // Checked
                                    ),
                                );
                                ifst.delete_final_weight_unchecked(state);
                            }
                        } else {
                            unsafe {
                                // Checked
                                ifst.set_final_unchecked(state, final_arc.weight);
                            }
                        }
                    }
                }
                MapFinalAction::MapRequireSuperfinal => {
                    if Some(state) != superfinal {
                        if final_arc.ilabel != EPS_LABEL
                            || final_arc.olabel != EPS_LABEL
                            || !final_arc.weight.is_zero()
                        {
                            unsafe {
                                // checked
                                ifst.add_arc_unchecked(
                                    state,
                                    Tr::new(
                                        final_arc.ilabel,
                                        final_arc.olabel,
                                        final_arc.weight,
                                        superfinal.unwrap(),
                                    ),
                                );
                                ifst.delete_final_weight_unchecked(state);
                            }
                        }
                    }
                }
            };
        }
    }

    Ok(())
}
