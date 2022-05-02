#![cfg(test)]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;

use anyhow::{bail, Result};
use path_abs::PathAbs;
use path_abs::PathInfo;
use path_abs::PathMut;
use serde::{Deserialize, Serialize};

use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::SerializableFst;
use crate::semirings::{LogWeight, ProductWeight, SerializableSemiring, TropicalWeight};
use crate::tests_openfst::algorithms::closure::{
    test_closure_plus, test_closure_plus_lazy, test_closure_star, test_closure_star_lazy,
    SimpleStaticLazyOperationResult, SimpleStaticLazyTestData,
};
use crate::tests_openfst::algorithms::compose::{ComposeOperationResult, ComposeTestData};
use crate::tests_openfst::algorithms::concat::{
    test_concat, test_concat_lazy, ConcatOperationResult, ConcatTestData,
};
use crate::tests_openfst::algorithms::condense::{
    test_condense, CondenseOperationResult, CondenseTestData,
};
use crate::tests_openfst::algorithms::factor_weight_gallic::test_factor_weight_gallic;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicOperationResult;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityOperationResult;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::{
    test_factor_weight_identity, test_factor_weight_identity_lazy,
};
use crate::tests_openfst::algorithms::fst_convert::test_fst_convert;
use crate::tests_openfst::algorithms::gallic_encode_decode::test_gallic_encode_decode;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicOperationResult;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicTestData;
use crate::tests_openfst::algorithms::optimize::test_optimize;
// use crate::tests_openfst::algorithms::matcher::test_sorted_matcher;
// use crate::tests_openfst::algorithms::matcher::{MatcherOperationResult, MatcherTestData};
use crate::tests_openfst::algorithms::state_reachable::{
    test_state_reachable, StateReachableOperationResult, StateReachableTestData,
};
use crate::tests_openfst::algorithms::union::{test_union, test_union_lazy};
use crate::tests_openfst::io::const_fst_bin_deserializer::{
    test_const_fst_aligned_bin_deserializer, test_const_fst_aligned_bin_deserializer_as_vector,
    test_const_fst_bin_deserializer, test_const_fst_bin_deserializer_as_vector,
};
use crate::tests_openfst::io::const_fst_bin_serializer::test_const_fst_bin_serializer;
use crate::tests_openfst::io::const_fst_bin_serializer::test_const_fst_bin_serializer_with_symt;
use crate::tests_openfst::io::const_fst_text_deserialization::test_const_fst_text_deserialization;
use crate::tests_openfst::io::const_fst_text_serialization::test_const_fst_text_serialization;
use crate::tests_openfst::io::const_fst_text_serialization::test_const_fst_text_serialization_with_symt;
use crate::tests_openfst::io::vector_fst_bin_deserializer::test_vector_fst_bin_deserializer;
use crate::tests_openfst::io::vector_fst_bin_deserializer::test_vector_fst_bin_with_symt_deserializer;
use crate::tests_openfst::io::vector_fst_bin_serializer::{
    test_vector_fst_bin_serializer, test_vector_fst_bin_serializer_with_symt,
};
use crate::tests_openfst::io::vector_fst_text_deserialization::test_vector_fst_text_deserialization;
use crate::tests_openfst::io::vector_fst_text_serialization::{
    test_vector_fst_text_serialization, test_vector_fst_text_serialization_with_symt,
};

use self::algorithms::{
    compose::test_compose,
    connect::test_connect,
    determinize::{test_determinize, DeterminizeOperationResult, DeterminizeTestData},
    encode::{test_encode, test_encode_decode, EncodeOperationResult, EncodeTestData},
    inverse::test_invert,
    minimize::{test_minimize, MinimizeOperationResult, MinimizeTestData},
    project::{test_project_input, test_project_output},
    properties::{parse_fst_properties, test_fst_properties},
    push::{test_push, PushOperationResult, PushTestData},
    replace::{test_replace, test_replace_lazy, ReplaceOperationResult, ReplaceTestData},
    reverse::test_reverse,
    rm_epsilon::{test_rmepsilon, test_rmepsilon_lazy},
    shortest_distance::{
        test_shortest_distance, ShorestDistanceOperationResult, ShortestDistanceTestData,
    },
    shortest_path::{test_shortest_path, ShorestPathOperationResult, ShortestPathTestData},
    state_map::{test_state_map_tr_sum, test_state_map_tr_unique},
    topsort::test_topsort,
    tr_map::{
        test_tr_map_identity, test_tr_map_input_epsilon, test_tr_map_invert,
        test_tr_map_output_epsilon, test_tr_map_plus, test_tr_map_quantize, test_tr_map_rmweight,
        test_tr_map_times, TrMapWithWeightOperationResult, TrMapWithWeightTestData,
    },
    tr_sort::{test_trsort_ilabel, test_trsort_olabel},
    union::{UnionOperationResult, UnionTestData},
    weight_pushing::{test_weight_pushing_final, test_weight_pushing_initial},
};
use self::fst_impls::const_fst::test_const_fst_convert_convert;
use self::fst_impls::test_fst_into_iterator::{
    test_fst_into_iterator_const, test_fst_into_iterator_vector,
};
use self::misc::test_del_all_states;
use crate::tests_openfst::algorithms::queue::{test_queue, QueueOperationResult};

mod utils;

mod algorithms;
mod fst_impls;
mod io;
mod misc;
mod test_symt;
mod test_weights;

#[derive(Serialize, Deserialize, Debug)]
struct FstOperationResult {
    result_path: String,
}

impl FstOperationResult {
    fn parse<W, F, P: AsRef<Path>>(&self, dir_path: P) -> F
    where
        W: SerializableSemiring,
        F: SerializableFst<W>,
    {
        let path = dir_path.as_ref().join(&self.result_path);
        F::read(path).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedFstTestData {
    rmepsilon: SimpleStaticLazyOperationResult,
    name: String,
    invert: FstOperationResult,
    weight_type: String,
    raw: FstOperationResult,
    raw_text: String,
    project_output: FstOperationResult,
    connect: FstOperationResult,
    condense: CondenseOperationResult,
    weight_pushing_initial: FstOperationResult,
    weight_pushing_final: FstOperationResult,
    project_input: FstOperationResult,
    reverse: FstOperationResult,
    tr_map_identity: FstOperationResult,
    tr_map_rmweight: FstOperationResult,
    tr_map_invert: FstOperationResult,
    tr_map_input_epsilon: FstOperationResult,
    tr_map_output_epsilon: FstOperationResult,
    tr_map_plus: TrMapWithWeightOperationResult,
    tr_map_times: TrMapWithWeightOperationResult,
    tr_map_quantize: FstOperationResult,
    encode: Vec<EncodeOperationResult>,
    encode_decode: Vec<EncodeOperationResult>,
    state_map_tr_sum: FstOperationResult,
    state_map_tr_unique: FstOperationResult,
    determinize: Vec<DeterminizeOperationResult>,
    minimize: Vec<MinimizeOperationResult>,
    tr_sort_ilabel: FstOperationResult,
    tr_sort_olabel: FstOperationResult,
    topsort: FstOperationResult,
    fst_properties: HashMap<String, bool>,
    raw_vector_bin_path: String,
    raw_const_bin_path: String,
    raw_const_aligned_bin_path: String,
    shortest_distance: Vec<ShorestDistanceOperationResult>,
    shortest_path: Vec<ShorestPathOperationResult>,
    gallic_encode_decode: Vec<GallicOperationResult>,
    factor_weight_identity: Vec<FwIdentityOperationResult>,
    factor_weight_gallic: Vec<FwGallicOperationResult>,
    push: Vec<PushOperationResult>,
    replace: Vec<ReplaceOperationResult>,
    union: Vec<UnionOperationResult>,
    concat: Vec<ConcatOperationResult>,
    closure_plus: SimpleStaticLazyOperationResult,
    closure_star: SimpleStaticLazyOperationResult,
    raw_vector_with_symt_bin_path: String,
    // matcher: Vec<MatcherOperationResult>,
    compose: Vec<ComposeOperationResult>,
    state_reachable: StateReachableOperationResult,
    queue: QueueOperationResult,
    optimize: FstOperationResult,
}

pub struct FstTestData<W, F: SerializableFst<W>>
where
    W: SerializableSemiring,
{
    pub rmepsilon: SimpleStaticLazyTestData<W, F>,
    #[allow(unused)]
    pub name: String,
    pub invert: F,
    pub raw: F,
    pub raw_text: String,
    pub project_output: F,
    pub connect: F,
    pub condense: CondenseTestData<W, F>,
    pub weight_pushing_initial: F,
    pub weight_pushing_final: F,
    pub project_input: F,
    pub reverse: F,
    pub tr_map_identity: F,
    pub tr_map_rmweight: F,
    pub tr_map_invert: F,
    pub tr_map_input_epsilon: F,
    pub tr_map_output_epsilon: F,
    pub tr_map_plus: TrMapWithWeightTestData<W, F>,
    pub tr_map_times: TrMapWithWeightTestData<W, F>,
    pub tr_map_quantize: F,
    pub encode: Vec<EncodeTestData<W, F>>,
    pub encode_decode: Vec<EncodeTestData<W, F>>,
    pub state_map_tr_sum: F,
    pub state_map_tr_unique: F,
    pub determinize: Vec<DeterminizeTestData<W, F>>,
    pub minimize: Vec<MinimizeTestData<W, F>>,
    pub tr_sort_ilabel: F,
    pub tr_sort_olabel: F,
    pub topsort: F,
    pub fst_properties: FstProperties,
    pub raw_vector_bin_path: PathBuf,
    pub raw_const_bin_path: PathBuf,
    pub raw_const_aligned_bin_path: PathBuf,
    pub shortest_distance: Vec<ShortestDistanceTestData<W>>,
    pub shortest_path: Vec<ShortestPathTestData<W, F>>,
    pub gallic_encode_decode: Vec<GallicTestData<W, F>>,
    pub factor_weight_identity: Vec<FwIdentityTestData<W, F>>,
    pub factor_weight_gallic: Vec<FwGallicTestData<W, F>>,
    pub push: Vec<PushTestData<W, F>>,
    pub replace: Vec<ReplaceTestData<W, F>>,
    pub union: Vec<UnionTestData<W, F>>,
    pub concat: Vec<ConcatTestData<W, F>>,
    pub closure_plus: SimpleStaticLazyTestData<W, F>,
    pub closure_star: SimpleStaticLazyTestData<W, F>,
    pub raw_vector_with_symt_bin_path: PathBuf,
    // pub matcher: Vec<MatcherTestData<F>>,
    pub compose: Vec<ComposeTestData<W, F>>,
    pub state_reachable: StateReachableTestData,
    pub queue: QueueOperationResult,
    pub optimize: F,
}

impl<W, F> FstTestData<W, F>
where
    W: SerializableSemiring,
    F: SerializableFst<W>,
{
    pub fn new(data: &ParsedFstTestData, absolute_path_folder: &Path) -> Self {
        Self {
            rmepsilon: data.rmepsilon.parse(absolute_path_folder),
            name: data.name.clone(),
            invert: data.invert.parse(absolute_path_folder),
            raw: data.raw.parse(absolute_path_folder),
            raw_text: data.raw_text.clone(),
            project_output: data.project_output.parse(absolute_path_folder),
            connect: data.connect.parse(absolute_path_folder),
            condense: data.condense.parse(absolute_path_folder),
            weight_pushing_initial: data.weight_pushing_initial.parse(absolute_path_folder),
            weight_pushing_final: data.weight_pushing_final.parse(absolute_path_folder),
            project_input: data.project_input.parse(absolute_path_folder),
            reverse: data.reverse.parse(absolute_path_folder),
            tr_map_identity: data.tr_map_identity.parse(absolute_path_folder),
            tr_map_rmweight: data.tr_map_rmweight.parse(absolute_path_folder),
            tr_map_invert: data.tr_map_invert.parse(absolute_path_folder),
            tr_map_input_epsilon: data.tr_map_input_epsilon.parse(absolute_path_folder),
            tr_map_output_epsilon: data.tr_map_output_epsilon.parse(absolute_path_folder),
            tr_map_plus: data.tr_map_plus.parse(absolute_path_folder),
            tr_map_times: data.tr_map_times.parse(absolute_path_folder),
            tr_map_quantize: data.tr_map_quantize.parse(absolute_path_folder),
            encode: data
                .encode
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            encode_decode: data
                .encode_decode
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            state_map_tr_sum: data.state_map_tr_sum.parse(absolute_path_folder),
            state_map_tr_unique: data.state_map_tr_unique.parse(absolute_path_folder),
            determinize: data
                .determinize
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            minimize: data
                .minimize
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            tr_sort_ilabel: data.tr_sort_ilabel.parse(absolute_path_folder),
            tr_sort_olabel: data.tr_sort_olabel.parse(absolute_path_folder),
            topsort: data.topsort.parse(absolute_path_folder),
            fst_properties: parse_fst_properties(&data.fst_properties),
            raw_vector_bin_path: absolute_path_folder
                .join(&data.raw_vector_bin_path)
                .to_path_buf(),
            raw_const_bin_path: absolute_path_folder
                .join(&data.raw_const_bin_path)
                .to_path_buf(),
            raw_const_aligned_bin_path: absolute_path_folder
                .join(&data.raw_const_aligned_bin_path)
                .to_path_buf(),
            shortest_distance: data.shortest_distance.iter().map(|v| v.parse()).collect(),
            shortest_path: data
                .shortest_path
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            gallic_encode_decode: data
                .gallic_encode_decode
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            factor_weight_identity: data
                .factor_weight_identity
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            factor_weight_gallic: data
                .factor_weight_gallic
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            push: data
                .push
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            replace: data
                .replace
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            union: data
                .union
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            concat: data
                .concat
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            closure_plus: data.closure_plus.parse(absolute_path_folder),
            closure_star: data.closure_star.parse(absolute_path_folder),
            raw_vector_with_symt_bin_path: absolute_path_folder
                .join(&data.raw_vector_with_symt_bin_path)
                .to_path_buf(),
            // matcher: data.matcher.iter().map(|v| v.parse()).collect(),
            compose: data
                .compose
                .iter()
                .map(|v| v.parse(absolute_path_folder))
                .collect(),
            state_reachable: data.state_reachable.parse(),
            queue: data.queue.clone(),
            optimize: data.optimize.parse(absolute_path_folder),
        }
    }
}

pub(crate) fn get_path_folder(test_name: &str) -> Result<PathBuf> {
    let mut path_repo = PathAbs::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
    path_repo.append("rustfst-tests-data")?;
    path_repo.append(test_name)?;
    Ok(path_repo.as_path().to_path_buf())
}

macro_rules! do_run {
    ($f: ident, $fst_name: expr) => {
        let absolute_path_folder = get_path_folder($fst_name)?;
        let mut path_metadata = absolute_path_folder.clone();
        path_metadata.push("metadata.json");

        let string = read_to_string(&path_metadata)
            .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
        let parsed_test_data: ParsedFstTestData = serde_json::from_str(&string).unwrap();

        match parsed_test_data.weight_type.as_str() {
            "tropical" | "standard" => {
                let test_data: FstTestData<TropicalWeight, VectorFst<TropicalWeight>> =
                    FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            "log" => {
                let test_data: FstTestData<LogWeight, VectorFst<LogWeight>> =
                    FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            "tropical_X_log" => {
                let test_data: FstTestData<
                    ProductWeight<TropicalWeight, LogWeight>,
                    VectorFst<ProductWeight<TropicalWeight, LogWeight>>,
                > = FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
                $f(&test_data)?;
            }
            _ => bail!("Weight type unknown : {:?}", parsed_test_data.weight_type),
        };
    };
}

macro_rules! test_fst {
    ($namespace: tt, $fst_name: expr) => {
        mod $namespace {
            use super::*;

            #[test]
            fn test_union_openfst() -> Result<()> {
                do_run!(test_union, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_identity_openfst() -> Result<()> {
                do_run!(test_tr_map_identity, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_invert_openfst() -> Result<()> {
                do_run!(test_tr_map_invert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_input_epsilon_openfst() -> Result<()> {
                do_run!(test_tr_map_input_epsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_output_epsilon_openfst() -> Result<()> {
                do_run!(test_tr_map_output_epsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_plus_openfst() -> Result<()> {
                do_run!(test_tr_map_plus, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_times_openfst() -> Result<()> {
                do_run!(test_tr_map_times, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_quantize_openfst() -> Result<()> {
                do_run!(test_tr_map_quantize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_tr_map_rmweight_openfst() -> Result<()> {
                do_run!(test_tr_map_rmweight, $fst_name);
                Ok(())
            }

            #[test]
            fn test_trsort_ilabel_openfst() -> Result<()> {
                do_run!(test_trsort_ilabel, $fst_name);
                Ok(())
            }

            #[test]
            fn test_trsort_olabel_openfst() -> Result<()> {
                do_run!(test_trsort_olabel, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_plus_openfst() -> Result<()> {
                do_run!(test_closure_plus, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_star_openfst() -> Result<()> {
                do_run!(test_closure_star, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_plus_lazy_openfst() -> Result<()> {
                do_run!(test_closure_plus_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_closure_star_lazy_openfst() -> Result<()> {
                do_run!(test_closure_star_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_concat_openfst() -> Result<()> {
                do_run!(test_concat, $fst_name);
                Ok(())
            }

            #[test]
            fn test_concat_lazy_openfst() -> Result<()> {
                do_run!(test_concat_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_connect_openfst() -> Result<()> {
                do_run!(test_connect, $fst_name);
                Ok(())
            }

            #[test]
            fn test_optimize_openfst() -> Result<()> {
                do_run!(test_optimize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_identity_openfst() -> Result<()> {
                do_run!(test_factor_weight_identity, $fst_name);
                Ok(())
            }

            #[test]
            fn test_determinize_openfst() -> Result<()> {
                do_run!(test_determinize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_encode_decode_openfst() -> Result<()> {
                do_run!(test_encode_decode, $fst_name);
                Ok(())
            }

            #[test]
            #[ignore]
            fn test_encode_openfst() -> Result<()> {
                do_run!(test_encode, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_gallic_openfst() -> Result<()> {
                do_run!(test_factor_weight_gallic, $fst_name);
                Ok(())
            }

            #[test]
            fn test_factor_weight_identity_lazy_openfst() -> Result<()> {
                do_run!(test_factor_weight_identity_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_gallic_encode_decode_openfst() -> Result<()> {
                do_run!(test_gallic_encode_decode, $fst_name);
                Ok(())
            }

            #[test]
            fn test_invert_openfst() -> Result<()> {
                do_run!(test_invert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_minimize_openfst() -> Result<()> {
                do_run!(test_minimize, $fst_name);
                Ok(())
            }

            #[test]
            fn test_project_output_openfst() -> Result<()> {
                do_run!(test_project_output, $fst_name);
                Ok(())
            }

            #[test]
            fn test_project_input_openfst() -> Result<()> {
                do_run!(test_project_input, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_properties_openfst() -> Result<()> {
                do_run!(test_fst_properties, $fst_name);
                Ok(())
            }

            #[test]
            fn test_push_openfst() -> Result<()> {
                do_run!(test_push, $fst_name);
                Ok(())
            }

            #[test]
            fn test_replace_openfst() -> Result<()> {
                do_run!(test_replace, $fst_name);
                Ok(())
            }

            #[test]
            fn test_replace_lazy_openfst() -> Result<()> {
                do_run!(test_replace_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_reverse_openfst() -> Result<()> {
                do_run!(test_reverse, $fst_name);
                Ok(())
            }

            #[test]
            fn test_shortest_distance_openfst() -> Result<()> {
                do_run!(test_shortest_distance, $fst_name);
                Ok(())
            }

            #[test]
            fn test_state_map_tr_unique_openfst() -> Result<()> {
                do_run!(test_state_map_tr_unique, $fst_name);
                Ok(())
            }

            #[test]
            fn test_state_map_tr_sum_openfst() -> Result<()> {
                do_run!(test_state_map_tr_sum, $fst_name);
                Ok(())
            }

            #[test]
            fn test_shortest_path_openfst() -> Result<()> {
                do_run!(test_shortest_path, $fst_name);
                Ok(())
            }

            #[test]
            fn test_topsort_openfst() -> Result<()> {
                do_run!(test_topsort, $fst_name);
                Ok(())
            }

            #[test]
            fn test_union_lazy_openfst() -> Result<()> {
                do_run!(test_union_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_weight_pushing_initial_openfst() -> Result<()> {
                do_run!(test_weight_pushing_initial, $fst_name);
                Ok(())
            }

            #[test]
            fn test_del_all_states_openfst() -> Result<()> {
                do_run!(test_del_all_states, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_text_serialization_openfst() -> Result<()> {
                do_run!(test_vector_fst_text_serialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_text_serialization_with_symt_openfst() -> Result<()> {
                do_run!(test_vector_fst_text_serialization_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_serializer_openfst() -> Result<()> {
                do_run!(test_vector_fst_bin_serializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_serializer_with_symt_openfst() -> Result<()> {
                do_run!(test_vector_fst_bin_serializer_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_with_symt_deserializer_openfst() -> Result<()> {
                do_run!(test_vector_fst_bin_with_symt_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_bin_deserializer_openfst() -> Result<()> {
                do_run!(test_vector_fst_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_weight_pushing_final_openfst() -> Result<()> {
                do_run!(test_weight_pushing_final, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_convert_convert_openfst() -> Result<()> {
                do_run!(test_const_fst_convert_convert, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_deserializer_openfst() -> Result<()> {
                do_run!(test_const_fst_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_aligned_bin_deserializer_openfst() -> Result<()> {
                do_run!(test_const_fst_aligned_bin_deserializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_serializer_openfst() -> Result<()> {
                do_run!(test_const_fst_bin_serializer, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_serializer_with_symt_openfst() -> Result<()> {
                do_run!(test_const_fst_bin_serializer_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_text_serialization_openfst() -> Result<()> {
                do_run!(test_const_fst_text_serialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_text_serialization_with_symt_openfst() -> Result<()> {
                do_run!(test_const_fst_text_serialization_with_symt, $fst_name);
                Ok(())
            }

            #[test]
            fn test_rmepsilon_openfst() -> Result<()> {
                do_run!(test_rmepsilon, $fst_name);
                Ok(())
            }

            #[test]
            fn test_rmepsilon_lazy_openfst() -> Result<()> {
                do_run!(test_rmepsilon_lazy, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_into_iterator_const_openfst() -> Result<()> {
                do_run!(test_fst_into_iterator_const, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_into_iterator_vector_openfst() -> Result<()> {
                do_run!(test_fst_into_iterator_vector, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_convert_openfst() -> Result<()> {
                do_run!(test_fst_convert, $fst_name);
                Ok(())
            }

            // #[test]
            // #[ignore]
            // fn test_fst_sorted_matcher_openfst() -> Result<()> {
            //     do_run!(test_sorted_matcher, $fst_name);
            //     Ok(())
            // }

            #[test]
            fn test_fst_compose_openfst() -> Result<()> {
                do_run!(test_compose, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_condense_openfst() -> Result<()> {
                do_run!(test_condense, $fst_name);
                Ok(())
            }

            #[test]
            fn test_fst_state_reachable_openfst() -> Result<()> {
                do_run!(test_state_reachable, $fst_name);
                Ok(())
            }

            #[test]
            fn test_queue_openfst() -> Result<()> {
                do_run!(test_queue, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_text_deserialization_openfst() -> Result<()> {
                do_run!(test_const_fst_text_deserialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_bin_deserializer_as_vector_openfst() -> Result<()> {
                do_run!(test_const_fst_bin_deserializer_as_vector, $fst_name);
                Ok(())
            }

            #[test]
            fn test_vector_fst_text_deserialization_openfst() -> Result<()> {
                do_run!(test_vector_fst_text_deserialization, $fst_name);
                Ok(())
            }

            #[test]
            fn test_const_fst_aligned_bin_deserializer_as_vector_openfst() -> Result<()> {
                do_run!(test_const_fst_aligned_bin_deserializer_as_vector, $fst_name);
                Ok(())
            }
        }
    };
}

test_fst!(test_openfst_fst_000, "fst_000");
test_fst!(test_openfst_fst_001, "fst_001");
test_fst!(test_openfst_fst_002, "fst_002");
test_fst!(test_openfst_fst_003, "fst_003");
test_fst!(test_openfst_fst_004, "fst_004");
test_fst!(test_openfst_fst_005, "fst_005");
test_fst!(test_openfst_fst_006, "fst_006");
test_fst!(test_openfst_fst_007, "fst_007");
test_fst!(test_openfst_fst_008, "fst_008");
test_fst!(test_openfst_fst_009, "fst_009");
test_fst!(test_openfst_fst_010, "fst_010");
test_fst!(test_openfst_fst_011, "fst_011");
test_fst!(test_openfst_fst_012, "fst_012");
test_fst!(test_openfst_fst_013, "fst_013");
test_fst!(test_openfst_fst_014, "fst_014");
test_fst!(test_openfst_fst_015, "fst_015");
test_fst!(test_openfst_fst_016, "fst_016");
test_fst!(test_openfst_fst_017, "fst_017");
test_fst!(test_openfst_fst_018, "fst_018");
test_fst!(test_openfst_fst_019, "fst_019");
test_fst!(test_openfst_fst_020, "fst_020");
