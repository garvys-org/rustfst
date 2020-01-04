use counter::Counter;

use failure::Fallible;
use serde_derive::{Deserialize, Serialize};

use crate::algorithms::{replace, ReplaceFst};
use crate::fst_impls::VectorFst;
use crate::fst_traits::{CoreFst, ExpandedFst, StateIterator, TextParser, ArcIterator};
use crate::semirings::{Semiring, WeaklyDivisibleSemiring, WeightQuantize};
use crate::tests_openfst::FstTestData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplaceOperationResult {
    root: usize,
    label_fst_pairs: Vec<(usize, String)>,
    epsilon_on_replace: bool,
    result: String,
}

pub struct ReplaceTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub root: usize,
    pub label_fst_pairs: Vec<(usize, F)>,
    pub epsilon_on_replace: bool,
    pub result: F,
}

impl ReplaceOperationResult {
    pub fn parse<F>(&self) -> ReplaceTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        ReplaceTestData {
            root: self.root,
            label_fst_pairs: self
                .label_fst_pairs
                .iter()
                .map(|v| (v.0, F::from_text_string(v.1.as_str()).unwrap()))
                .collect(),
            epsilon_on_replace: self.epsilon_on_replace,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

pub fn test_replace<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for replace_test_data in &test_data.replace {
        let mut fst_list = vec![];
        fst_list.push((replace_test_data.root, test_data.raw.clone()));
        fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
        let replaced_fst: VectorFst<_> = replace(
            fst_list.clone(),
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        assert_eq!(
            replace_test_data.result,
            replaced_fst,
            "{}",
            error_message_fst!(
                replace_test_data.result,
                replaced_fst,
                format!(
                    "Replace failed with parameters root={:?} epsilon_on_replace={:?}",
                    replace_test_data.root, replace_test_data.epsilon_on_replace
                )
            )
        );
    }
    Ok(())
}

pub fn test_replace_dynamic<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring + 'static,
    W::ReverseWeight: 'static,
{
    for replace_test_data in &test_data.replace {
        let mut fst_list = vec![];
        fst_list.push((replace_test_data.root, test_data.raw.clone()));
        fst_list.extend_from_slice(replace_test_data.label_fst_pairs.as_slice());
        let replaced_static_fst: VectorFst<_> = replace(
            fst_list.clone(),
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        let replaced_dynamic_fst = ReplaceFst::new(
            fst_list,
            replace_test_data.root,
            replace_test_data.epsilon_on_replace,
        )?;

        assert_eq!(
            replaced_dynamic_fst.states_iter().count(),
            replaced_static_fst.num_states()
        );

        assert_eq!(replaced_dynamic_fst.start(), replaced_static_fst.start());

        for i in 0..replaced_static_fst.num_states() {
            assert_eq!(
                replaced_dynamic_fst.final_weight(i)?,
                replaced_static_fst.final_weight(i)?
            );
            unsafe {
                assert_eq!(
                    replaced_dynamic_fst.final_weight_unchecked(i),
                    replaced_static_fst.final_weight_unchecked(i)
                )
            };
            assert_eq!(
                replaced_dynamic_fst.num_arcs(i)?,
                replaced_static_fst.num_arcs(i)?
            );
            unsafe {
                assert_eq!(
                    replaced_dynamic_fst.num_arcs_unchecked(i),
                    replaced_static_fst.num_arcs_unchecked(i)
                )
            };

            let mut arcs_dynamic : Counter<_, usize> = Counter::new();
            arcs_dynamic.update(replaced_dynamic_fst.arcs_iter(i)?.cloned());

            let mut arcs_static : Counter<_, usize> = Counter::new();
            arcs_static.update(replaced_static_fst.arcs_iter(i)?.cloned());

            assert_eq!(arcs_dynamic, arcs_static);
        }
    }
    Ok(())
}
