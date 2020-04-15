use crate::algorithms::{FinalArc, MapFinalAction, WeightConverter};
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    Semiring, StringWeightVariant,
};
use crate::{Arc, Label, EPS_LABEL};
use anyhow::Result;

/// Mapper from GallicWeight<W> to W.
pub struct FromGallicConverter {
    pub superfinal_label: Label,
}

macro_rules! impl_extract_gallic_weight {
    ($gallic: ident) => {{
        let w1 = $gallic.value1();
        let w2 = $gallic.value2();
        match w1.value() {
            StringWeightVariant::Infinity => bail!("Unexpected infinity"),
            StringWeightVariant::Labels(l) => {
                if l.len() > 1 {
                    bail!("Expected at most 1 element, {:?}", l);
                } else if l.len() == 1 {
                    return Ok((w2.clone(), l[0]));
                } else {
                    // l.len() == 0
                    Ok((w2.clone(), 0))
                }
            }
        }
    }};
}

fn extract_restrict<W: Semiring>(gw: &GallicWeightRestrict<W>) -> Result<(W, Label)> {
    impl_extract_gallic_weight!(gw)
}

fn extract_left<W: Semiring>(gw: &GallicWeightLeft<W>) -> Result<(W, Label)> {
    impl_extract_gallic_weight!(gw)
}

fn extract_right<W: Semiring>(gw: &GallicWeightRight<W>) -> Result<(W, Label)> {
    impl_extract_gallic_weight!(gw)
}

fn extract_min<W: Semiring>(gw: &GallicWeightMin<W>) -> Result<(W, Label)> {
    impl_extract_gallic_weight!(gw)
}

fn extract_gallic<W: Semiring>(gw: &GallicWeight<W>) -> Result<(W, Label)> {
    if gw.len() > 1 {
        bail!("error")
    }
    if gw.is_empty() {
        Ok((W::zero(), EPS_LABEL))
    } else {
        let back_w = gw.value();
        let v = back_w.last().unwrap();
        impl_extract_gallic_weight!(v)
    }
}

macro_rules! impl_weight_converter_gallic {
    ($gallic: ident, $fextract: ident) => {
        impl<W: Semiring> WeightConverter<$gallic<W>, W> for FromGallicConverter {
            fn arc_map(&mut self, arc: &Arc<$gallic<W>>) -> Result<Arc<W>> {
                let (extracted_w, extracted_l) = $fextract(&arc.weight)?;
                if arc.ilabel != arc.olabel {
                    bail!("Unrepresentable weight : {:?}", &arc);
                }

                let new_arc = Arc {
                    ilabel: arc.ilabel,
                    olabel: extracted_l,
                    weight: extracted_w,
                    nextstate: arc.nextstate,
                };
                Ok(new_arc)
            }

            fn final_arc_map(&mut self, final_arc: &FinalArc<$gallic<W>>) -> Result<FinalArc<W>> {
                let (extracted_w, extracted_l) = $fextract(&final_arc.weight).expect("Fail");
                if final_arc.ilabel != final_arc.olabel {
                    panic!("Unrepresentable weight : {:?}", &final_arc);
                }

                let new_final_arc = if final_arc.ilabel == EPS_LABEL && extracted_l != EPS_LABEL {
                    FinalArc {
                        ilabel: self.superfinal_label,
                        olabel: extracted_l,
                        weight: extracted_w,
                    }
                } else {
                    FinalArc {
                        ilabel: final_arc.ilabel,
                        olabel: extracted_l,
                        weight: extracted_w,
                    }
                };
                Ok(new_final_arc)
            }

            fn final_action(&self) -> MapFinalAction {
                MapFinalAction::MapAllowSuperfinal
            }
        }
    };
}

impl_weight_converter_gallic!(GallicWeightLeft, extract_left);
impl_weight_converter_gallic!(GallicWeightRight, extract_right);
impl_weight_converter_gallic!(GallicWeightMin, extract_min);
impl_weight_converter_gallic!(GallicWeightRestrict, extract_restrict);
impl_weight_converter_gallic!(GallicWeight, extract_gallic);
