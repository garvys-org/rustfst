use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::union;
use crate::fst_impls::VectorFst;
use crate::fst_traits::TextParser;
use crate::semirings::{Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnionOperationResult {
    fst_2: String,
    result_static: String,
    result_dynamic: String,
}

pub struct UnionTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub fst_2: F,
    pub result_static: F,
    pub result_dynamic: F,
}

impl UnionOperationResult {
    pub fn parse<F>(&self) -> UnionTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        UnionTestData {
            fst_2: F::from_text_string(self.fst_2.as_str()).unwrap(),
            result_static: F::from_text_string(self.result_static.as_str()).unwrap(),
            result_dynamic: F::from_text_string(self.result_dynamic.as_str()).unwrap(),
        }
    }
}

pub fn test_union<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for union_test_data in &test_data.union {
        let mut fst_res_static = test_data.raw.clone();
         union(&mut fst_res_static, &union_test_data.fst_2)?;

        assert_eq!(
            union_test_data.result_static,
            fst_res_static,
            "{}",
            error_message_fst!(
                union_test_data.result_static,
                fst_res_static,
                format!("Union failed")
            )
        );
    }
    Ok(())
}
//
//pub fn test_replace_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
//    where
//        W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
//        W::ReverseWeight: 'static,
//{
//    for replace_test_data in &test_data.replace {
//        let mut fst_list = vec![];
//        fst_list.push((replace_test_data.root, test_data.raw.clone()));
//        fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
//
//        let fst_list_2: Vec<(usize, &VectorFst<W>)> =
//            fst_list.iter().map(|v| (v.0, &v.1)).collect();
//        let replaced_static_fst: VectorFst<_> = replace(
//            fst_list_2,
//            replace_test_data.root,
//            replace_test_data.epsilon_on_replace,
//        )?;
//
//        let replaced_dynamic_fst = ReplaceFst::new(
//            fst_list,
//            replace_test_data.root,
//            replace_test_data.epsilon_on_replace,
//        )?;
//
//        compare_fst_static_dynamic(&replaced_static_fst, &replaced_dynamic_fst)?;
//    }
//    Ok(())
//}
