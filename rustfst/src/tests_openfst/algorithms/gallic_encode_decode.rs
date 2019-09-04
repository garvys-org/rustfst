use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::weight_convert;
use crate::algorithms::weight_converters::FromGallicConverter;
use crate::algorithms::weight_converters::ToGallicConverter;
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::GallicWeight;
use crate::semirings::GallicWeightLeft;
use crate::semirings::GallicWeightMin;
use crate::semirings::GallicWeightRestrict;
use crate::semirings::GallicWeightRight;
use crate::semirings::Semiring;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct GallicOperationResult {
    gallic_type: String,
    result: String,
}

pub struct GallicTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub gallic_type: String,
    pub result: F,
}

impl GallicOperationResult {
    pub fn parse<F>(&self) -> GallicTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        GallicTestData {
            gallic_type: self.gallic_type.clone(),
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_gallic_encode_decode<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + 'static,
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

        assert_eq_fst!(
            data.result,
            fst_res,
            format!(
                "Gallic encode decode with failling with gallic_type={:?}",
                data.gallic_type
            )
        );
    }
    Ok(())
}
