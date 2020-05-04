use anyhow::Result;

use crate::algorithms::{FinalTr, MapFinalAction, WeightConverter};
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    Semiring, StringWeightLeft, StringWeightRestrict, StringWeightRight,
};
use crate::{Tr, EPS_LABEL};

/// Mapper from W to GallicTr<W>.
pub struct ToGallicConverter {}

macro_rules! impl_to_gallic_converter {
    ($gallic: ident, $string_weight: ident) => {
        impl<W> WeightConverter<W, $gallic<W>> for ToGallicConverter
        where
            W: Semiring,
        {
            fn arc_map(&mut self, arc: &Tr<W>) -> Result<Tr<$gallic<W>>> {
                let new_arc = if arc.olabel == EPS_LABEL {
                    let w = ($string_weight::one(), arc.weight.clone());
                    Tr::new(arc.ilabel, arc.ilabel, w, arc.nextstate)
                } else {
                    let w = (arc.olabel, arc.weight.clone());
                    Tr::new(arc.ilabel, arc.ilabel, w, arc.nextstate)
                };
                Ok(new_arc)
            }

            fn final_arc_map(&mut self, final_arc: &FinalTr<W>) -> Result<FinalTr<$gallic<W>>> {
                if final_arc.weight.is_zero() {
                    bail!("Shouldn't happen")
                }
                let w = ($string_weight::one(), final_arc.weight.clone());
                Ok(FinalTr {
                    ilabel: EPS_LABEL,
                    olabel: EPS_LABEL,
                    weight: w.into(),
                })
            }

            fn final_action(&self) -> MapFinalAction {
                MapFinalAction::MapNoSuperfinal
            }
        }
    };
}

impl_to_gallic_converter!(GallicWeightLeft, StringWeightLeft);
impl_to_gallic_converter!(GallicWeightRight, StringWeightRight);
impl_to_gallic_converter!(GallicWeightRestrict, StringWeightRestrict);
impl_to_gallic_converter!(GallicWeightMin, StringWeightRestrict);
impl_to_gallic_converter!(GallicWeight, StringWeightRestrict);
