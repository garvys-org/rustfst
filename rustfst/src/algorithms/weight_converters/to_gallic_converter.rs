use anyhow::Result;

use crate::algorithms::ProjectType;
use crate::algorithms::{FinalTr, MapFinalAction, WeightConverter};
use crate::fst_properties::mutable_properties::project_properties;
use crate::fst_properties::FstProperties;
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    Semiring, StringWeightLeft, StringWeightRestrict, StringWeightRight,
};
use crate::{Tr, EPS_LABEL};

/// Mapper from `W` to `GallicTr<W>`.
pub struct ToGallicConverter {}

macro_rules! impl_to_gallic_converter {
    ($gallic: ident, $string_weight: ident) => {
        impl<W> WeightConverter<W, $gallic<W>> for ToGallicConverter
        where
            W: Semiring,
        {
            fn tr_map(&mut self, tr: &Tr<W>) -> Result<Tr<$gallic<W>>> {
                let new_tr = if tr.olabel == EPS_LABEL {
                    let w = ($string_weight::one(), tr.weight.clone());
                    Tr::new(tr.ilabel, tr.ilabel, w, tr.nextstate)
                } else {
                    let w = (tr.olabel, tr.weight.clone());
                    Tr::new(tr.ilabel, tr.ilabel, w, tr.nextstate)
                };
                Ok(new_tr)
            }

            fn final_tr_map(&mut self, final_tr: &FinalTr<W>) -> Result<FinalTr<$gallic<W>>> {
                if final_tr.weight.is_zero() {
                    bail!("Shouldn't happen")
                }
                let w = ($string_weight::one(), final_tr.weight.clone());
                Ok(FinalTr {
                    ilabel: EPS_LABEL,
                    olabel: EPS_LABEL,
                    weight: w.into(),
                })
            }

            fn final_action(&self) -> MapFinalAction {
                MapFinalAction::MapNoSuperfinal
            }

            fn properties(&self, inprops: FstProperties) -> FstProperties {
                inprops
                    & project_properties(inprops, ProjectType::ProjectInput)
                    & FstProperties::weight_invariant_properties()
            }
        }
    };
}

impl_to_gallic_converter!(GallicWeightLeft, StringWeightLeft);
impl_to_gallic_converter!(GallicWeightRight, StringWeightRight);
impl_to_gallic_converter!(GallicWeightRestrict, StringWeightRestrict);
impl_to_gallic_converter!(GallicWeightMin, StringWeightRestrict);
impl_to_gallic_converter!(GallicWeight, StringWeightRestrict);
