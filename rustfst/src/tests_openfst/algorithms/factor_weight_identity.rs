use std::marker::PhantomData;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::algorithms::factor_weight::factor_iterators::IdentityFactor;
use crate::algorithms::factor_weight::factor_weight;
use crate::algorithms::factor_weight::{FactorWeightFst, FactorWeightOptions, FactorWeightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::SerializableSemiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::FstTestData;

use super::lazy_fst::compare_fst_static_lazy;

#[derive(Serialize, Deserialize, Debug)]
pub struct FwIdentityOperationResult {
    factor_final_weights: bool,
    factor_tr_weights: bool,
    result: String,
}

pub struct FwIdentityTestData<W, F>
where
    F: SerializableFst<W>,
    W: SerializableSemiring,
{
    pub factor_final_weights: bool,
    pub factor_tr_weights: bool,
    pub result: F,
    w: PhantomData<W>,
}

impl FwIdentityOperationResult {
    pub fn parse<W, F>(&self) -> FwIdentityTestData<W, F>
    where
        F: SerializableFst<W>,
        W: SerializableSemiring,
    {
        FwIdentityTestData {
            factor_final_weights: self.factor_final_weights,
            factor_tr_weights: self.factor_tr_weights,
            result: F::from_text_string(self.result.as_str()).unwrap(),
            w: PhantomData,
        }
    }
}

pub fn test_factor_weight_identity<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    for data in &test_data.factor_weight_identity {
        let mode = FactorWeightType::from_bools(data.factor_final_weights, data.factor_tr_weights);
        let opts = FactorWeightOptions::new(mode);

        let fst_res: VectorFst<_> =
            factor_weight::<_, VectorFst<_>, _, _, IdentityFactor<_>>(&test_data.raw, opts)?;

        assert_eq_fst!(
        data.result,
        fst_res,
        format!(
            "Factor weight identity failing with factor_final_weights={:?} and factor_tr_weights={:?}",
            data.factor_final_weights, data.factor_tr_weights
        )
    );
    }

    Ok(())
}

pub fn test_factor_weight_identity_lazy<W>(test_data: &FstTestData<W, VectorFst<W>>) -> Result<()>
where
    W: SerializableSemiring + WeightQuantize,
{
    for data in &test_data.factor_weight_identity {
        let mode = FactorWeightType::from_bools(data.factor_final_weights, data.factor_tr_weights);
        let opts = FactorWeightOptions::new(mode);

        let fst_res_static: VectorFst<_> = factor_weight::<_, VectorFst<_>, _, _, IdentityFactor<_>>(
            &test_data.raw,
            opts.clone(),
        )?;

        let fst_res_lazy =
            FactorWeightFst::<_, _, _, IdentityFactor<_>>::new(test_data.raw.clone(), opts)?;

        compare_fst_static_lazy(&fst_res_static, &fst_res_lazy)?;
    }

    Ok(())
}
