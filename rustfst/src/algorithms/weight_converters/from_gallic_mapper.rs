use crate::algorithms::{FinalTr, MapFinalAction, WeightConverter};
use crate::fst_properties::FstProperties;
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    Semiring, StringWeightVariant,
};
use crate::{Label, Tr, EPS_LABEL};
use anyhow::Result;

/// Mapper from `GallicWeight<W>` to `W`.
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
            fn tr_map(&mut self, tr: &Tr<$gallic<W>>) -> Result<Tr<W>> {
                let (extracted_w, extracted_l) = $fextract(&tr.weight)?;
                if tr.ilabel != tr.olabel {
                    bail!("Unrepresentable weight : {:?}", &tr);
                }

                let new_tr = Tr {
                    ilabel: tr.ilabel,
                    olabel: extracted_l,
                    weight: extracted_w,
                    nextstate: tr.nextstate,
                };
                Ok(new_tr)
            }

            fn final_tr_map(&mut self, final_tr: &FinalTr<$gallic<W>>) -> Result<FinalTr<W>> {
                let (extracted_w, extracted_l) = $fextract(&final_tr.weight).expect("Fail");
                if final_tr.ilabel != final_tr.olabel {
                    panic!("Unrepresentable weight : {:?}", &final_tr);
                }

                let new_final_tr = if final_tr.ilabel == EPS_LABEL && extracted_l != EPS_LABEL {
                    FinalTr {
                        ilabel: self.superfinal_label,
                        olabel: extracted_l,
                        weight: extracted_w,
                    }
                } else {
                    FinalTr {
                        ilabel: final_tr.ilabel,
                        olabel: extracted_l,
                        weight: extracted_w,
                    }
                };
                Ok(new_final_tr)
            }

            fn final_action(&self) -> MapFinalAction {
                MapFinalAction::MapAllowSuperfinal
            }

            fn properties(&self, inprops: FstProperties) -> FstProperties {
                inprops
                    & FstProperties::o_label_invariant_properties()
                    & FstProperties::weight_invariant_properties()
                    & FstProperties::add_super_final_properties()
            }
        }
    };
}

impl_weight_converter_gallic!(GallicWeightLeft, extract_left);
impl_weight_converter_gallic!(GallicWeightRight, extract_right);
impl_weight_converter_gallic!(GallicWeightMin, extract_min);
impl_weight_converter_gallic!(GallicWeightRestrict, extract_restrict);
impl_weight_converter_gallic!(GallicWeight, extract_gallic);
