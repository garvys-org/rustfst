use anyhow::Result;
use std::ops::Deref;

use crate::fst_properties::FstProperties;
use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Tr;
use crate::{Label, StateId, EPS_LABEL};

/// Struct used to map final weights when performing a transition mapping.
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
    /// A final weight is mapped to a transition to the superfinal state when the result
    /// cannot be represented as a final weight. The superfinal state will be
    /// added only if it is needed.
    MapAllowSuperfinal,
    /// A final weight is mapped to a transition to the superfinal state unless the
    /// result can be represented as a final weight of weight Zero(). The
    /// superfinal state is always added (if the input is not the empty FST).
    MapRequireSuperfinal,
}

/// The TrMapper interfaces defines how trs and final weights are mapped.
/// This is useful for implementing operations that do not change the number of
/// trs.
pub trait TrMapper<S: Semiring> {
    /// How to modify the trs.
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()>;

    /// The mapper will be passed final weights as trs of the form
    /// `FinalTr(EPS_LABEL, EPS_LABEL, weight)`.
    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()>;

    /// Specifies final action the mapper requires (see above).
    fn final_action(&self) -> MapFinalAction;

    fn properties(&self, inprops: FstProperties) -> FstProperties;
}

impl<S: Semiring, T: TrMapper<S>, TP: Deref<Target = T>> TrMapper<S> for TP {
    fn tr_map(&self, tr: &mut Tr<S>) -> Result<()> {
        self.deref().tr_map(tr)
    }

    fn final_tr_map(&self, final_tr: &mut FinalTr<S>) -> Result<()> {
        self.deref().final_tr_map(final_tr)
    }

    fn final_action(&self) -> MapFinalAction {
        self.deref().final_action()
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        self.deref().properties(inprops)
    }
}

/// Maps every transition in the FST using an `TrMapper` object.
pub fn tr_map<W, F, M>(ifst: &mut F, mapper: &M) -> Result<()>
where
    W: Semiring,
    F: MutableFst<W>,
    M: TrMapper<W>,
{
    if ifst.start().is_none() {
        return Ok(());
    }

    let inprops = ifst.properties();

    let final_action = mapper.final_action();
    let mut superfinal: Option<StateId> = None;

    if final_action == MapFinalAction::MapRequireSuperfinal {
        let superfinal_id = ifst.add_state();
        superfinal = Some(superfinal_id);
        ifst.set_final(superfinal_id, W::one()).unwrap();
    }

    for state in 0..(ifst.num_states() as StateId) {
        unsafe {
            let mut it_tr = ifst.tr_iter_unchecked_mut(state);
            for idx_tr in 0..it_tr.len() {
                let mut tr = it_tr.get_unchecked(idx_tr).clone();
                mapper.tr_map(&mut tr)?;
                it_tr.set_tr_unchecked(idx_tr, tr);
            }
        }

        if let Some(w) = unsafe { ifst.final_weight_unchecked(state) } {
            let mut final_tr = FinalTr {
                ilabel: EPS_LABEL,
                olabel: EPS_LABEL,
                weight: w,
            };
            mapper.final_tr_map(&mut final_tr)?;
            match final_action {
                MapFinalAction::MapNoSuperfinal => {
                    if final_tr.ilabel != EPS_LABEL || final_tr.olabel != EPS_LABEL {
                        bail!("TrMap: Non-zero tr labels for superfinal tr")
                    }
                    unsafe {
                        ifst.set_final_unchecked(state, final_tr.weight);
                    }
                }
                MapFinalAction::MapAllowSuperfinal => {
                    if Some(state) != superfinal {
                        if final_tr.ilabel != EPS_LABEL || final_tr.olabel != EPS_LABEL {
                            if superfinal.is_none() {
                                let superfinal_id = ifst.add_state();
                                superfinal = Some(superfinal_id);
                                unsafe {
                                    // Checked because the state is created just above
                                    ifst.set_final_unchecked(superfinal_id, W::one());
                                }
                            }
                            unsafe {
                                // Checked
                                ifst.add_tr_unchecked(
                                    state,
                                    Tr::new(
                                        final_tr.ilabel,
                                        final_tr.olabel,
                                        final_tr.weight,
                                        superfinal.unwrap(), // Checked
                                    ),
                                );
                                ifst.delete_final_weight_unchecked(state);
                            }
                        } else {
                            unsafe {
                                // Checked
                                ifst.set_final_unchecked(state, final_tr.weight);
                            }
                        }
                    }
                }
                MapFinalAction::MapRequireSuperfinal => {
                    if Some(state) != superfinal
                        && (final_tr.ilabel != EPS_LABEL
                            || final_tr.olabel != EPS_LABEL
                            || !final_tr.weight.is_zero())
                    {
                        unsafe {
                            // checked
                            ifst.add_tr_unchecked(
                                state,
                                Tr::new(
                                    final_tr.ilabel,
                                    final_tr.olabel,
                                    final_tr.weight,
                                    superfinal.unwrap(),
                                ),
                            );
                            ifst.delete_final_weight_unchecked(state);
                        }
                    }
                }
            };
        }
    }

    ifst.set_properties_with_mask(mapper.properties(inprops), FstProperties::all_properties());

    Ok(())
}
