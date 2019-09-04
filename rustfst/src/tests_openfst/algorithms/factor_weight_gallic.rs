use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::factor_iterators::GallicFactor;
use crate::algorithms::factor_iterators::GallicFactorLeft;
use crate::algorithms::factor_iterators::GallicFactorMin;
use crate::algorithms::factor_iterators::GallicFactorRestrict;
use crate::algorithms::factor_iterators::GallicFactorRight;
use crate::algorithms::factor_weight;
use crate::algorithms::weight_converters::FromGallicConverter;
use crate::algorithms::weight_converters::ToGallicConverter;
use crate::algorithms::{weight_convert, FactorWeightOptions, FactorWeightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::GallicWeight;
use crate::semirings::GallicWeightLeft;
use crate::semirings::GallicWeightMin;
use crate::semirings::GallicWeightRestrict;
use crate::semirings::GallicWeightRight;
use crate::semirings::Semiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct FwGallicOperationResult {
    factor_final_weights: bool,
    factor_arc_weights: bool,
    gallic_type: String,
    result: String,
}

pub struct FwGallicTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub factor_final_weights: bool,
    pub factor_arc_weights: bool,
    pub gallic_type: String,
    pub result: F,
}

impl FwGallicOperationResult {
    pub fn parse<F>(&self) -> FwGallicTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        FwGallicTestData {
            factor_final_weights: self.factor_final_weights,
            factor_arc_weights: self.factor_arc_weights,
            gallic_type: self.gallic_type.clone(),
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_factor_weight_gallic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + 'static,
{
    for data in &test_data.factor_weight_gallic {
        //        println!("test fwgallic");
        //        std::dbg!(data.factor_final_weights);
        //        std::dbg!(data.factor_arc_weights);
        let mode = FactorWeightType::from_bools(data.factor_final_weights, data.factor_arc_weights);
        let opts = FactorWeightOptions::new(mode);

        let mut to_gallic = ToGallicConverter {};
        let mut from_gallic = FromGallicConverter {
            superfinal_label: 0,
        };

        let fst_res: VectorFst<W> = match data.gallic_type.as_str() {
            "gallic_left" => {
                let fst_temp: VectorFst<GallicWeightLeft<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, _, GallicFactorLeft<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_right" => {
                let fst_temp: VectorFst<GallicWeightRight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, _, GallicFactorRight<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_restrict" => {
                let fst_temp: VectorFst<GallicWeightRestrict<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, _, GallicFactorRestrict<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_min" => {
                let fst_temp: VectorFst<GallicWeightMin<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, _, GallicFactorMin<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic" => {
                let fst_temp: VectorFst<GallicWeight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, _, GallicFactor<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            _ => bail!("Unexpected gallic_type={:?}", data.gallic_type),
        };

        assert_eq_fst!(
        data.result,
        fst_res,
        format!(
            "Factor weight gallic failing with factor_final_weights={:?}, factor_arc_weights={:?} and gallic_type={:?}",
            data.factor_final_weights, data.factor_arc_weights, data.gallic_type
        )
    );
    }

    Ok(())
}
