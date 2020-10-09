use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::factor_weight::{
    factor_iterators::{
        GallicFactor, GallicFactorLeft, GallicFactorMin, GallicFactorRestrict, GallicFactorRight,
    },
    factor_weight, FactorWeightOptions, FactorWeightType,
};
use crate::algorithms::weight_convert;
use crate::algorithms::weight_converters::{FromGallicConverter, ToGallicConverter};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{
    GallicWeight, GallicWeightLeft, GallicWeightMin, GallicWeightRestrict, GallicWeightRight,
    SerializableSemiring, WeightQuantize,
};
use crate::tests_openfst::utils::test_eq_fst;
use crate::tests_openfst::FstTestData;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct FwGallicOperationResult {
    factor_final_weights: bool,
    factor_tr_weights: bool,
    gallic_type: String,
    result_path: String,
}

pub struct FwGallicTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub factor_final_weights: bool,
    pub factor_tr_weights: bool,
    pub gallic_type: String,
    pub result: F,
    w: PhantomData<W>,
}

impl FwGallicOperationResult {
    pub fn parse<W, F, P>(&self, dir_path: P) -> FwGallicTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
        P: AsRef<Path>,
    {
        FwGallicTestData {
            factor_final_weights: self.factor_final_weights,
            factor_tr_weights: self.factor_tr_weights,
            gallic_type: self.gallic_type.clone(),
            result: F::read(dir_path.as_ref().join(&self.result_path)).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_factor_weight_gallic<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    for data in &test_data.factor_weight_gallic {
        //        println!("test fwgallic");
        //        std::dbg!(data.factor_final_weights);
        //        std::dbg!(data.factor_tr_weights);
        let mode = FactorWeightType::from_bools(data.factor_final_weights, data.factor_tr_weights);
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
                    factor_weight::<_, VectorFst<_>, _, _, GallicFactorLeft<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_right" => {
                let fst_temp: VectorFst<GallicWeightRight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, VectorFst<_>, _, _, GallicFactorRight<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_restrict" => {
                let fst_temp: VectorFst<GallicWeightRestrict<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, VectorFst<_>, _, _, GallicFactorRestrict<_>>(
                        &fst_temp, opts,
                    )?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic_min" => {
                let fst_temp: VectorFst<GallicWeightMin<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<GallicWeightMin<W>> =
                    factor_weight::<_, VectorFst<GallicWeightMin<W>>, _, _, GallicFactorMin<_>>(
                        &fst_temp, opts,
                    )?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            "gallic" => {
                let fst_temp: VectorFst<GallicWeight<W>> =
                    weight_convert(&test_data.raw, &mut to_gallic)?;
                let fst_temp: VectorFst<_> =
                    factor_weight::<_, VectorFst<_>, _, _, GallicFactor<_>>(&fst_temp, opts)?;
                weight_convert(&fst_temp, &mut from_gallic)?
            }
            _ => bail!("Unexpected gallic_type={:?}", data.gallic_type),
        };

        test_eq_fst(&data.result, &fst_res,         format!(
            "Factor weight gallic failing with factor_final_weights={:?}, factor_tr_weights={:?} and gallic_type={:?}",
            data.factor_final_weights, data.factor_tr_weights, data.gallic_type
        ));
    }

    Ok(())
}
