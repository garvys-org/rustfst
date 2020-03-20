use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{compose, compose_with_config, ComposeConfig, ComposeFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::SerializableFst;
use crate::semirings::{SerializableSemiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::algorithms::dynamic_fst::compare_fst_static_dynamic;
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposeOperationResult {
    fst_2: String,
    result_static: String,
    result_dynamic: String,
    connect: bool,
    filter_name: String
}

pub struct ComposeTestData<F>
where
    F: SerializableFst,
    F::W: SerializableSemiring,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_dynamic: F,
    pub connect: bool,
    pub filter_name: String,
}

impl ComposeOperationResult {
    pub fn parse<F>(&self) -> ComposeTestData<F>
    where
        F: SerializableFst,
        F::W: SerializableSemiring,
    {
        ComposeTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
            connect: self.connect,
            filter_name: self.filter_name.clone()
        }
    }
}

pub fn test_compose<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for compose_test_data in &test_data.compose {
        let mut config = ComposeConfig::default();
        config.connect = compose_test_data.connect;
        let fst_res_static: VectorFst<_> =
            compose_with_config(&test_data.raw, &compose_test_data.fst_2, config)?;

        assert_eq!(
            compose_test_data.result_static,
            fst_res_static,
            "{}",
            error_message_fst!(
                compose_test_data.result_static,
                fst_res_static,
                format!("Compose failed : connect = {} filter_name = {}", compose_test_data.connect, compose_test_data.filter_name)
            )
        );
    }
    Ok(())
}

pub fn test_compose_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: SerializableSemiring + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    unimplemented!()
    // for compose_test_data in &test_data.compose {
    //     let compose_dynamic_fst =
    //         ComposeFst::new_auto(&test_data.raw, &compose_test_data.fst_2)?;
    //
    //     compare_fst_static_dynamic(&compose_test_data.result_dynamic, &compose_dynamic_fst)?;
    // }
    // Ok(())
}
