use crate::fst_traits::MutableFst;
use crate::semirings::Semiring;
use crate::Arc;
use crate::{Label, Result, StateId, EPS_LABEL};

pub struct FinalArc<W: Semiring> {
    pub ilabel: Label,
    pub olabel: Label,
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

/// The ArcMapper interfaces defines how arcs and final weights are mapped.
/// This is useful for implementing operations that do not change the number of
/// arcs.
pub trait ArcMapper<S: Semiring> {
    /// How to modify the arcs.
    fn arc_map(&mut self, arc: &mut Arc<S>);

    /// How to modify the arcs.
    fn final_arc_map(&mut self, final_arc: &mut FinalArc<S>);

    /// Specifies final action the mapper requires (see above).
    ///The mapper will be passed final weights as arcs of the form
    /// Arc(0, 0, weight, kNoStateId).
    fn final_action(&self) -> MapFinalAction;
}

/// Maps an arc using a mapper function object. This function modifies its Fst input.
pub fn arc_map<F, M>(ifst: &mut F, mapper: &mut M) -> Result<()>
where
    F: MutableFst,
    M: ArcMapper<F::W>,
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

    let states: Vec<_> = ifst.states_iter().collect();
    for state in states {
        for arc in ifst.arcs_iter_mut(state).unwrap() {
            mapper.arc_map(arc);
        }

        if let Some(w) = ifst.final_weight_mut(state) {
            match final_action {
                MapFinalAction::MapNoSuperfinal => {
                    let mut final_arc = FinalArc {
                        ilabel: EPS_LABEL,
                        olabel: EPS_LABEL,
                        weight: w.clone(),
                    };
                    mapper.final_arc_map(&mut final_arc);

                    if final_arc.ilabel != EPS_LABEL || final_arc.olabel != EPS_LABEL {
                        bail!("ArcMap: Non-zero arc labels for superfinal arc")
                    }

                    ifst.set_final(state, final_arc.weight).unwrap();
                }
                MapFinalAction::MapAllowSuperfinal => unimplemented!(),
                MapFinalAction::MapRequireSuperfinal => {
                    if state != superfinal.unwrap() {
                        let mut final_arc = FinalArc {
                            ilabel: EPS_LABEL,
                            olabel: EPS_LABEL,
                            weight: w.clone(),
                        };
                        mapper.final_arc_map(&mut final_arc);

                        if !final_arc.weight.is_zero() {
                            ifst.add_arc(
                                state,
                                Arc::new(
                                    final_arc.ilabel,
                                    final_arc.olabel,
                                    final_arc.weight,
                                    superfinal.unwrap(),
                                ),
                            )
                            .unwrap();
                        }
                        ifst.delete_final_weight(state).unwrap();
                    }
                }
            };
        }
    }

    Ok(())
}
