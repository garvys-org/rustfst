use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::weight_convert;
use crate::algorithms::weight_converters::FromGallicConverter;
use crate::algorithms::weight_converters::ToGallicConverter;
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::GallicWeightMin;
use crate::semirings::GallicWeightRestrict;
use crate::semirings::GallicWeightRight;
use crate::semirings::{GallicWeight, SerializableSemiring};
use crate::semirings::{GallicWeightLeft, WeightQuantize};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct GallicOperationResult {
    gallic_type: String,
    result_path: String,
}

pub struct GallicTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub gallic_type: String,
    pub result: F,
    w: PhantomData<W>,
}

impl GallicOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> GallicTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        GallicTestData {
            gallic_type: self.gallic_type.clone(),
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_gallic_encode_decode<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    for data in &test_data.gallic_encode_decode {
        let mut to_gallic = ToGallicConverter {};
        let mut from_gallic = FromGallicConverter {
            superfinal_label: 0,
        };

        let fst_res: VectorFst<W> = match data.gallic_type.as_str() {
            "gallic_left" => {
                let fst_temp: VectorFst<GallicWeightLeft<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_right" => {
                let fst_temp: VectorFst<GallicWeightRight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_restrict" => {
                let fst_temp: VectorFst<GallicWeightRestrict<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_min" => {
                let fst_temp: VectorFst<GallicWeightMin<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic" => {
                let fst_temp: VectorFst<GallicWeight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            _ => bail!("Unexpected gallic_type={:?}", data.gallic_type),
        };

        test_eq_fst(
            &data.result,
            &fst_res,
            format!(
                "Gallic encode decode with failling with gallic_type={:?}",
                data.gallic_type
            ),
        );
    }
    Ok(())
}
