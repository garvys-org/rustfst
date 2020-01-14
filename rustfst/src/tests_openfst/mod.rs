#![cfg(test)]

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;

use failure::{bail, Fail, Fallible};
use path_abs::PathAbs;
use path_abs::PathInfo;
use path_abs::PathMut;
use serde_derive::{Deserialize, Serialize};

use crate::fst_impls::VectorFst;
use crate::fst_properties::FstProperties;
use crate::fst_traits::TextParser;
use crate::semirings::{
    LogWeight, Semiring, StarSemiring, TropicalWeight, WeaklyDivisibleSemiring, WeightQuantize,
};
use crate::tests_openfst::algorithms::factor_weight_gallic::test_factor_weight_gallic;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicOperationResult;
use crate::tests_openfst::algorithms::factor_weight_gallic::FwGallicTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityOperationResult;
use crate::tests_openfst::algorithms::factor_weight_identity::FwIdentityTestData;
use crate::tests_openfst::algorithms::factor_weight_identity::{
    test_factor_weight_identity, test_factor_weight_identity_dynamic,
};
use crate::tests_openfst::algorithms::gallic_encode_decode::test_gallic_encode_decode;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicOperationResult;
use crate::tests_openfst::algorithms::gallic_encode_decode::GallicTestData;
use crate::tests_openfst::io::const_fst_bin_deserializer::{
    test_const_fst_aligned_bin_deserializer, test_const_fst_bin_deserializer,
};
use crate::tests_openfst::io::const_fst_bin_serializer::test_const_fst_bin_serializer;
use crate::tests_openfst::io::const_fst_text_serialization::test_const_fst_text_serialization;

use self::algorithms::{
    arc_map::{
        test_arc_map_identity, test_arc_map_input_epsilon, test_arc_map_invert,
        test_arc_map_output_epsilon, test_arc_map_plus, test_arc_map_quantize,
        test_arc_map_rmweight, test_arc_map_times,
    },
    arcsort::{test_arcsort_ilabel, test_arcsort_olabel},
    connect::test_connect,
    determinize::{test_determinize, DeterminizeOperationResult, DeterminizeTestData},
    encode::{test_encode, test_encode_decode, EncodeOperationResult, EncodeTestData},
    inverse::test_invert,
    minimize::{test_minimize, MinimizeOperationResult, MinimizeTestData},
    project::{test_project_input, test_project_output},
    properties::{parse_fst_properties, test_fst_properties},
    push::{test_push, PushOperationResult, PushTestData},
    replace::{test_replace, test_replace_dynamic, ReplaceOperationResult, ReplaceTestData},
    reverse::test_reverse,
    rm_epsilon::test_rmepsilon,
    shortest_distance::{
        test_shortest_distance, ShorestDistanceOperationResult, ShortestDistanceTestData,
    },
    shortest_path::{test_shortest_path, ShorestPathOperationResult, ShortestPathTestData},
    state_map::{test_state_map_arc_sum, test_state_map_arc_unique},
    topsort::test_topsort,
    union::{UnionOperationResult, UnionTestData},
    weight_pushing::{test_weight_pushing_final, test_weight_pushing_initial},
};
use self::fst_impls::const_fst::test_const_fst_convert_convert;
use self::io::vector_fst_bin_deserializer::test_vector_fst_bin_deserializer;
use self::io::vector_fst_bin_serializer::test_vector_fst_bin_serializer;
use self::io::vector_fst_text_serialization::test_vector_fst_text_serialization;
use self::misc::test_del_all_states;
use crate::tests_openfst::algorithms::closure::{
    test_closure_plus, test_closure_star, ClosureOperationResult, ClosureTestData,
};
use crate::tests_openfst::algorithms::concat::{
    test_concat, test_concat_dynamic, ConcatOperationResult, ConcatTestData,
};
use crate::tests_openfst::algorithms::union::{test_union, test_union_dynamic};

#[macro_use]
mod macros;

mod algorithms;
mod fst_impls;
mod io;
mod misc;
mod test_symt;

#[derive(Serialize, Deserialize, Debug)]
struct FstOperationResult {
    result: String,
}

impl FstOperationResult {
    fn parse<F>(&self) -> F
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        F::from_text_string(self.result.as_str()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedFstTestData {
    rmepsilon: FstOperationResult,
    name: String,
    invert: FstOperationResult,
    weight_type: String,
    raw: FstOperationResult,
    project_output: FstOperationResult,
    connect: FstOperationResult,
    weight_pushing_initial: FstOperationResult,
    weight_pushing_final: FstOperationResult,
    project_input: FstOperationResult,
    reverse: FstOperationResult,
    arc_map_identity: FstOperationResult,
    arc_map_rmweight: FstOperationResult,
    arc_map_invert: FstOperationResult,
    arc_map_input_epsilon: FstOperationResult,
    arc_map_output_epsilon: FstOperationResult,
    arc_map_plus: FstOperationResult,
    arc_map_times: FstOperationResult,
    arc_map_quantize: FstOperationResult,
    encode: Vec<EncodeOperationResult>,
    encode_decode: Vec<EncodeOperationResult>,
    state_map_arc_sum: FstOperationResult,
    state_map_arc_unique: FstOperationResult,
    determinize: Vec<DeterminizeOperationResult>,
    minimize: Vec<MinimizeOperationResult>,
    arcsort_ilabel: FstOperationResult,
    arcsort_olabel: FstOperationResult,
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
    closure_plus: ClosureOperationResult,
    closure_star: ClosureOperationResult,
}

pub struct FstTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub rmepsilon: F,
    #[allow(unused)]
    pub name: String,
    pub invert: F,
    pub raw: F,
    pub project_output: F,
    pub connect: F,
    pub weight_pushing_initial: F,
    pub weight_pushing_final: F,
    pub project_input: F,
    pub reverse: F,
    pub arc_map_identity: F,
    pub arc_map_rmweight: F,
    pub arc_map_invert: F,
    pub arc_map_input_epsilon: F,
    pub arc_map_output_epsilon: F,
    pub arc_map_plus: F,
    pub arc_map_times: F,
    pub arc_map_quantize: F,
    pub encode: Vec<EncodeTestData<F>>,
    pub encode_decode: Vec<EncodeTestData<F>>,
    pub state_map_arc_sum: F,
    pub state_map_arc_unique: F,
    pub determinize: Vec<DeterminizeTestData<F>>,
    pub minimize: Vec<MinimizeTestData<F>>,
    pub arcsort_ilabel: F,
    pub arcsort_olabel: F,
    pub topsort: F,
    pub fst_properties: FstProperties,
    pub raw_vector_bin_path: PathBuf,
    pub raw_const_bin_path: PathBuf,
    pub raw_const_aligned_bin_path: PathBuf,
    pub shortest_distance: Vec<ShortestDistanceTestData<F::W>>,
    pub shortest_path: Vec<ShortestPathTestData<F>>,
    pub gallic_encode_decode: Vec<GallicTestData<F>>,
    pub factor_weight_identity: Vec<FwIdentityTestData<F>>,
    pub factor_weight_gallic: Vec<FwGallicTestData<F>>,
    pub push: Vec<PushTestData<F>>,
    pub replace: Vec<ReplaceTestData<F>>,
    pub union: Vec<UnionTestData<F>>,
    pub concat: Vec<ConcatTestData<F>>,
    pub closure_plus: ClosureTestData<F>,
    pub closure_star: ClosureTestData<F>,
}

impl<F> FstTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub fn new(data: &ParsedFstTestData, absolute_path_folder: &Path) -> Self {
        Self {
            rmepsilon: data.rmepsilon.parse(),
            name: data.name.clone(),
            invert: data.invert.parse(),
            raw: data.raw.parse(),
            project_output: data.project_output.parse(),
            connect: data.connect.parse(),
            weight_pushing_initial: data.weight_pushing_initial.parse(),
            weight_pushing_final: data.weight_pushing_final.parse(),
            project_input: data.project_input.parse(),
            reverse: data.reverse.parse(),
            arc_map_identity: data.arc_map_identity.parse(),
            arc_map_rmweight: data.arc_map_rmweight.parse(),
            arc_map_invert: data.arc_map_invert.parse(),
            arc_map_input_epsilon: data.arc_map_input_epsilon.parse(),
            arc_map_output_epsilon: data.arc_map_output_epsilon.parse(),
            arc_map_plus: data.arc_map_plus.parse(),
            arc_map_times: data.arc_map_times.parse(),
            arc_map_quantize: data.arc_map_quantize.parse(),
            encode: data.encode.iter().map(|v| v.parse()).collect(),
            encode_decode: data.encode_decode.iter().map(|v| v.parse()).collect(),
            state_map_arc_sum: data.state_map_arc_sum.parse(),
            state_map_arc_unique: data.state_map_arc_unique.parse(),
            determinize: data.determinize.iter().map(|v| v.parse()).collect(),
            minimize: data.minimize.iter().map(|v| v.parse()).collect(),
            arcsort_ilabel: data.arcsort_ilabel.parse(),
            arcsort_olabel: data.arcsort_olabel.parse(),
            topsort: data.topsort.parse(),
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
            shortest_path: data.shortest_path.iter().map(|v| v.parse()).collect(),
            gallic_encode_decode: data
                .gallic_encode_decode
                .iter()
                .map(|v| v.parse())
                .collect(),
            factor_weight_identity: data
                .factor_weight_identity
                .iter()
                .map(|v| v.parse())
                .collect(),
            factor_weight_gallic: data
                .factor_weight_gallic
                .iter()
                .map(|v| v.parse())
                .collect(),
            push: data.push.iter().map(|v| v.parse()).collect(),
            replace: data.replace.iter().map(|v| v.parse()).collect(),
            union: data.union.iter().map(|v| v.parse()).collect(),
            concat: data.concat.iter().map(|v| v.parse()).collect(),
            closure_plus: data.closure_plus.parse(),
            closure_star: data.closure_star.parse(),
        }
    }
}

pub(crate) fn get_path_folder(test_name: &str) -> Fallible<PathBuf> {
    let mut path_repo = PathAbs::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
    path_repo.append("rustfst-tests-data")?;
    path_repo.append(test_name)?;
    Ok(path_repo.as_path().to_path_buf())
}

fn run_test_openfst_fst(test_name: &str) -> Fallible<()> {
    let absolute_path_folder = get_path_folder(test_name)?;
    let mut path_metadata = absolute_path_folder.clone();
    path_metadata.push("metadata.json");

    let string = read_to_string(&path_metadata)
        .map_err(|_| format_err!("Can't open {:?}", &path_metadata))?;
    let parsed_test_data: ParsedFstTestData = serde_json::from_str(&string).unwrap();

    match parsed_test_data.weight_type.as_str() {
        "tropical" | "standard" => {
            let test_data: FstTestData<VectorFst<TropicalWeight>> =
                FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
            do_run_test_openfst(&test_data)?;
        }
        "log" => {
            let test_data: FstTestData<VectorFst<LogWeight>> =
                FstTestData::new(&parsed_test_data, absolute_path_folder.as_path());
            do_run_test_openfst(&test_data)?;
        }
        _ => bail!("Weight type unknown : {:?}", parsed_test_data.weight_type),
    };

    Ok(())
}

fn do_run_test_openfst<W>(test_data: &FstTestData<VectorFst<W>>) -> Fallible<()>
where
    W: 'static + Semiring<Type = f32> + StarSemiring + WeaklyDivisibleSemiring + WeightQuantize,
    <W as Semiring>::ReverseWeight: WeaklyDivisibleSemiring + WeightQuantize + StarSemiring,
    W: Into<<W as Semiring>::ReverseWeight> + From<<W as Semiring>::ReverseWeight>,
{
    test_rmepsilon(&test_data)?;

    test_invert(&test_data)?;

    test_project_input(&test_data)?;

    test_project_output(&test_data)?;

    test_reverse(&test_data)?;

    test_connect(&test_data)?;

    test_shortest_distance(&test_data)?;

    test_weight_pushing_initial(&test_data)?;

    test_weight_pushing_final(&test_data)?;

    test_arc_map_identity(&test_data)?;

    test_arc_map_rmweight(&test_data)?;

    test_arc_map_invert(&test_data)?;

    test_arc_map_input_epsilon(&test_data)?;

    test_arc_map_output_epsilon(&test_data)?;

    test_arc_map_plus(&test_data)?;

    test_arc_map_times(&test_data)?;

    test_arc_map_quantize(&test_data)?;

    test_encode(&test_data)?;

    test_encode_decode(&test_data)?;

    test_state_map_arc_sum(&test_data)?;

    test_state_map_arc_unique(&test_data)?;

    test_determinize(&test_data)?;

    test_arcsort_ilabel(&test_data)?;

    test_arcsort_olabel(&test_data)?;

    test_topsort(&test_data)?;

    test_fst_properties(&test_data)?;

    test_vector_fst_bin_deserializer(&test_data)?;

    test_vector_fst_bin_serializer(&test_data)?;

    test_shortest_path(&test_data)?;

    test_gallic_encode_decode(&test_data)?;

    test_factor_weight_identity(&test_data)?;

    test_factor_weight_identity_dynamic(&test_data)?;

    test_factor_weight_gallic(&test_data)?;

    test_minimize(&test_data)?;

    test_push(&test_data)?;

    test_const_fst_convert_convert(&test_data)?;

    test_vector_fst_text_serialization(&test_data)?;

    test_const_fst_text_serialization(&test_data)?;

    test_const_fst_bin_deserializer(&test_data)?;

    test_const_fst_aligned_bin_deserializer(&test_data)?;

    test_const_fst_bin_serializer(&test_data)?;

    test_del_all_states(&test_data)?;

    test_replace(&test_data)?;

    test_replace_dynamic(&test_data)?;

    test_union(&test_data)?;

    test_union_dynamic(&test_data)?;

    test_concat(&test_data)?;

    test_concat_dynamic(&test_data)?;

    test_closure_plus(&test_data)?;

    test_closure_star(&test_data)?;

    Ok(())
}

pub(crate) struct ExitFailure(failure::Error);

/// Prints a list of causes for this Error, along with any backtrace
/// information collected by the Error (if RUST_BACKTRACE=1).
impl std::fmt::Debug for ExitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fail = self.0.as_fail();

        writeln!(f, "{}", &fail)?;

        let mut x: &dyn Fail = fail;
        while let Some(cause) = x.cause() {
            writeln!(f, " -> caused by: {}", &cause)?;
            x = cause;
        }
        if let Some(backtrace) = x.backtrace() {
            writeln!(f, "{:?}", backtrace)?;
        }

        Ok(())
    }
}

impl<T: Into<failure::Error>> From<T> for ExitFailure {
    fn from(t: T) -> Self {
        ExitFailure(t.into())
    }
}

#[test]
fn test_openfst_fst_000() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_000").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_001() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_001").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_002() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_002").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_003() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_003").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_004() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_004").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_005() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_005").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_006() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_006").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_007() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_007").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_008() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_008").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_009() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_009").map_err(|v| v.into())
}

#[test]
fn test_openfst_fst_010() -> Result<(), ExitFailure> {
    run_test_openfst_fst("fst_010").map_err(|v| v.into())
}
