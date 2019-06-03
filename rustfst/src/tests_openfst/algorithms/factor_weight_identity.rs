use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::factor_iterators::IdentityFactor;
use crate::algorithms::factor_weight;
use crate::algorithms::{FactorWeightOptions, FactorWeightType};
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::Semiring;
use crate::semirings::WeightQuantize;
use crate::tests_openfst::TestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct FwIdentityOperationResult {
    factor_final_weights: bool,
    factor_arc_weights: bool,
    result: String,
}

pub struct FwIdentityTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub factor_final_weights: bool,
    pub factor_arc_weights: bool,
    pub result: F,
}

impl FwIdentityOperationResult {
    pub fn parse<F>(&self) -> FwIdentityTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        FwIdentityTestData {
            factor_final_weights: self.factor_final_weights,
            factor_arc_weights: self.factor_arc_weights,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_factor_weight_identity<W>(test_data: &TestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + 'static,
{
    for data in &test_data.factor_weight_identity {
        //        println!("test fwinentity");
        //        std::dbg!(data.factor_final_weights);
        //        std::dbg!(data.factor_arc_weights);
        let mode = FactorWeightType::from_bools(data.factor_final_weights, data.factor_arc_weights);
        let opts = FactorWeightOptions::new(mode);

        let fst_res: VectorFst<_> = factor_weight::<_, _, IdentityFactor<_>>(&test_data.raw, opts)?;

        assert_eq_fst!(
        data.result,
        fst_res,
        format!(
            "Factor weight identity failing with factor_final_weights={:?} and factor_arc_weights={:?}",
            data.factor_final_weights, data.factor_arc_weights
        )
    );
    }

    Ok(())
}
