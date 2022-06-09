use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{AllocableFst, ExpandedFst, MutableFst};
use crate::semirings::Semiring;
use crate::{Tr, Trs, EPS_LABEL};
use unsafe_unwrap::UnsafeUnwrap;

/// The WeightConverter interfaces defines how a weight should be turned into another one.
/// Useful for changing the semiring of an FST.
pub trait WeightConverter<SI: Semiring, SO: Semiring> {
    fn tr_map(&mut self, tr: &Tr<SI>) -> Result<Tr<SO>>;
    fn final_tr_map(&mut self, final_tr: &FinalTr<SI>) -> Result<FinalTr<SO>>;
    fn final_action(&self) -> MapFinalAction;
    fn properties(&self, iprops: FstProperties) -> FstProperties;
}

/// Convert an FST in a given Semiring to another Semiring using a WeightConverter
/// to specify how the conversion should be performed.
pub fn weight_convert<W1, W2, F1, F2, M>(fst_in: &F1, mapper: &mut M) -> Result<F2>
where
    W1: Semiring,
    W2: Semiring,
    F1: ExpandedFst<W1>,
    F2: MutableFst<W2> + AllocableFst<W2>,
    M: WeightConverter<W1, W2>,
{
    let iprops = fst_in.properties();
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
        fst_out.set_final(superfinal.unwrap(), W2::one())?;
    }

    if let Some(start_state) = fst_in.start() {
        fst_out.set_start(start_state)?;
    }

    let states: Vec<_> = fst_in.states_iter().collect();
    for state in states {
        fst_out.reserve_trs(state, fst_in.num_trs(state)?)?;
        for tr in fst_in.get_trs(state)?.trs() {
            fst_out.add_tr(state, mapper.tr_map(tr)?)?;
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
                        bail!("TrMap: Non-zero tr labels for superfinal tr")
                    }

                    fst_out.set_final(state, mapped_final_tr.weight).unwrap();
                }
                MapFinalAction::MapAllowSuperfinal => {
                    if mapped_final_tr.ilabel != EPS_LABEL || mapped_final_tr.olabel != EPS_LABEL {
                        if superfinal.is_none() {
                            let superfinal_id = fst_out.add_state();
                            superfinal = Some(superfinal_id);
                            fst_out.set_final(superfinal_id, W2::one()).unwrap();
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

    let oprops = fst_out.properties();
    fst_out.set_properties_with_mask(
        mapper.properties(iprops) | oprops,
        FstProperties::all_properties(),
    );
    fst_out.set_symts_from_fst(fst_in);

    Ok(fst_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::weight_converters::SimpleWeightConverter;
    use crate::fst_traits::Fst;
    use crate::prelude::{TropicalWeight, VectorFst};
    use crate::SymbolTable;
    use proptest::prelude::any;
    use proptest::proptest;
    use std::sync::Arc;

    proptest! {
        #[test]
        fn test_proptest_weight_convert_keeps_symts(mut fst in any::<VectorFst::<TropicalWeight>>()) {
            let symt = Arc::new(SymbolTable::new());
            fst.set_input_symbols(Arc::clone(&symt));
            fst.set_output_symbols(Arc::clone(&symt));

            let mut weight_converter = SimpleWeightConverter{};
            let fst : VectorFst<TropicalWeight> = weight_convert(&fst, &mut weight_converter).unwrap();

            assert!(fst.input_symbols().is_some());
            assert!(fst.output_symbols().is_some());
        }
    }
}
