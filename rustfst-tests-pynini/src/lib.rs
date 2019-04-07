use std::collections::HashMap;
use std::fs::read_to_string;
use std::string::String;

use serde_derive::{Deserialize, Serialize};

use failure::{bail, format_err, Fail, Fallible, ResultExt};

use rustfst::algorithms::arc_compares::{ilabel_compare, olabel_compare};
use rustfst::algorithms::arc_mappers::{
    IdentityArcMapper, InputEpsilonMapper, InvertWeightMapper, OutputEpsilonMapper, PlusMapper,
    QuantizeMapper, RmWeightMapper, TimesMapper,
};
use rustfst::algorithms::state_mappers::{ArcSumMapper, ArcUniqueMapper};
use rustfst::algorithms::{
    arc_sort, connect, decode, determinize, encode, invert, isomorphic, project, push_weights,
    reverse, rm_epsilon, DeterminizeType, ProjectType, ReweightType,
};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_properties::FstProperties;
use rustfst::fst_traits::{MutableFst, TextParser};
use rustfst::semirings::{
    LogWeight, Semiring, StarSemiring, TropicalWeight, WeaklyDivisibleSemiring, WeightQuantize,
};

#[derive(Serialize, Deserialize, Debug)]
struct OperationResult {
    result: String,
}

impl OperationResult {
    fn parse<F>(&self) -> F
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        F::from_text_string(self.result.as_str()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EncodeOperationResult {
    encode_labels: bool,
    encode_weights: bool,
    result: String,
}

impl EncodeOperationResult {
    fn parse<F>(&self) -> EncodeTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        EncodeTestData {
            encode_weights: self.encode_weights,
            encode_labels: self.encode_labels,
            result: F::from_text_string(self.result.as_str()).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DeterminizeOperationResult {
    det_type: String,
    result: String,
}

impl DeterminizeOperationResult {
    fn parse<F>(&self) -> DeterminizeTestData<F>
    where
        F: TextParser,
        F::W: Semiring<Type = f32>,
    {
        DeterminizeTestData {
            det_type: match self.det_type.as_str() {
                "functional" => DeterminizeType::DeterminizeFunctional,
                "nonfunctional" => DeterminizeType::DeterminizeNonFunctional,
                "disambiguate" => DeterminizeType::DeterminizeDisambiguate,
                _ => panic!("Unknown determinize type : {:?}", self.det_type),
            },
            result: match self.result.as_str() {
                "error" => Err(format_err!("lol")),
                _ => F::from_text_string(self.result.as_str()),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ParsedTestData {
    rmepsilon: OperationResult,
    name: String,
    invert: OperationResult,
    weight_type: String,
    raw: OperationResult,
    project_output: OperationResult,
    connect: OperationResult,
    weight_pushing_initial: OperationResult,
    weight_pushing_final: OperationResult,
    project_input: OperationResult,
    reverse: OperationResult,
    arc_map_identity: OperationResult,
    arc_map_rmweight: OperationResult,
    arc_map_invert: OperationResult,
    arc_map_input_epsilon: OperationResult,
    arc_map_output_epsilon: OperationResult,
    arc_map_plus: OperationResult,
    arc_map_times: OperationResult,
    arc_map_quantize: OperationResult,
    encode: Vec<EncodeOperationResult>,
    encode_decode: Vec<EncodeOperationResult>,
    state_map_arc_sum: OperationResult,
    state_map_arc_unique: OperationResult,
    determinize: Vec<DeterminizeOperationResult>,
    arcsort_ilabel: OperationResult,
    arcsort_olabel: OperationResult,
    fst_properties: HashMap<String, bool>,
}

struct EncodeTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    encode_labels: bool,
    encode_weights: bool,
    result: F,
}

struct DeterminizeTestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    det_type: DeterminizeType,
    result: Fallible<F>,
}

struct TestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    rmepsilon: F,
    #[allow(unused)]
    name: String,
    invert: F,
    raw: F,
    project_output: F,
    connect: F,
    weight_pushing_initial: F,
    weight_pushing_final: F,
    project_input: F,
    reverse: F,
    arc_map_identity: F,
    arc_map_rmweight: F,
    arc_map_invert: F,
    arc_map_input_epsilon: F,
    arc_map_output_epsilon: F,
    arc_map_plus: F,
    arc_map_times: F,
    arc_map_quantize: F,
    encode: Vec<EncodeTestData<F>>,
    encode_decode: Vec<EncodeTestData<F>>,
    state_map_arc_sum: F,
    state_map_arc_unique: F,
    determinize: Vec<DeterminizeTestData<F>>,
    arcsort_ilabel: F,
    arcsort_olabel: F,
    fst_properties: FstProperties,
}

fn parse_fst_properties(mapping: &HashMap<String, bool>) -> FstProperties {
    let mut props = FstProperties::empty();

    // 1
    if mapping["acceptor"] {
        props |= FstProperties::ACCEPTOR
    }
    // 2
    if mapping["not_acceptor"] {
        props |= FstProperties::NOT_ACCEPTOR
    }
    // 3
    if mapping["i_deterministic"] {
        props |= FstProperties::I_DETERMINISTIC
    }
    // 4
    if mapping["not_i_deterministic"] {
        props |= FstProperties::NOT_I_DETERMINISTIC
    }
    // 5
    if mapping["o_deterministic"] {
        props |= FstProperties::O_DETERMINISTIC
    }
    // 6
    if mapping["not_o_deterministic"] {
        props |= FstProperties::NOT_O_DETERMINISTIC
    }
    // 7
    if mapping["epsilons"] {
        props |= FstProperties::EPSILONS
    }
    // 8
    if mapping["no_epsilons"] {
        props |= FstProperties::NO_EPSILONS
    }
    // 9
    if mapping["i_epsilons"] {
        props |= FstProperties::I_EPSILONS
    }
    // 10
    if mapping["no_i_epsilons"] {
        props |= FstProperties::NO_I_EPSILONS
    }
    // 11
    if mapping["o_epsilons"] {
        props |= FstProperties::O_EPSILONS
    }
    // 12
    if mapping["no_o_epsilons"] {
        props |= FstProperties::NO_O_EPSILONS
    }
    // 13
    if mapping["i_label_sorted"] {
        props |= FstProperties::I_LABEL_SORTED
    }
    // 14
    if mapping["not_i_label_sorted"] {
        props |= FstProperties::NOT_I_LABEL_SORTED
    }
    // 15
    if mapping["o_label_sorted"] {
        props |= FstProperties::O_LABEL_SORTED
    }
    // 16
    if mapping["not_o_label_sorted"] {
        props |= FstProperties::NOT_O_LABEL_SORTED
    }
    // 17
    if mapping["weighted"] {
        props |= FstProperties::WEIGHTED
    }
    // 18
    if mapping["unweighted"] {
        props |= FstProperties::UNWEIGHTED
    }
    // 19
    if mapping["cyclic"] {
        props |= FstProperties::CYCLIC
    }
    // 20
    if mapping["acyclic"] {
        props |= FstProperties::ACYCLIC
    }
    // 21
    if mapping["initial_cyclic"] {
        props |= FstProperties::INITIAL_CYCLIC
    }
    // 22
    if mapping["initial_acyclic"] {
        props |= FstProperties::INITIAL_ACYCLIC
    }
    // 23
    if mapping["top_sorted"] {
        props |= FstProperties::TOP_SORTED
    }
    // 24
    if mapping["not_top_sorted"] {
        props |= FstProperties::NOT_TOP_SORTED
    }
    // 25
    if mapping["accessible"] {
        props |= FstProperties::ACCESSIBLE
    }
    // 26
    if mapping["not_accessible"] {
        props |= FstProperties::NOT_ACCESSIBLE
    }
    // 27
    if mapping["coaccessible"] {
        props |= FstProperties::COACCESSIBLE
    }
    // 28
    if mapping["not_coaccessible"] {
        props |= FstProperties::NOT_COACCESSIBLE
    }
    // 29
    if mapping["string"] {
        props |= FstProperties::STRING
    }
    // 30
    if mapping["not_string"] {
        props |= FstProperties::NOT_STRING
    }
    // 31
    if mapping["weighted_cycles"] {
        props |= FstProperties::WEIGHTED_CYCLES
    }
    // 32
    if mapping["unweighted_cycles"] {
        props |= FstProperties::UNWEIGHTED_CYCLES
    }

    props
}

impl<F> TestData<F>
where
    F: TextParser,
    F::W: Semiring<Type = f32>,
{
    pub fn new(data: &ParsedTestData) -> Self {
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
            arcsort_ilabel: data.arcsort_ilabel.parse(),
            arcsort_olabel: data.arcsort_olabel.parse(),
            fst_properties: parse_fst_properties(&data.fst_properties),
        }
    }
}

macro_rules! error_message_fst {
    ($fst_ref:expr, $fst:expr, $operation_name:expr) => {
        format!(
            "\nTest {} with pynini failing : \nREF = \n{}\nPRED = \n{}\n",
            $operation_name, $fst_ref, $fst
        )
    };
}

fn run_test_pynini(test_name: &str) -> Fallible<()> {
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push("..");
    absolute_path.push("rustfst-tests-data");
    absolute_path.push("rustfst_tests_data");
    absolute_path.push(test_name);
    absolute_path.push("metadata.json");

    let string = read_to_string(absolute_path).unwrap();
    let parsed_test_data: ParsedTestData = serde_json::from_str(&string).unwrap();

    match parsed_test_data.weight_type.as_str() {
        "tropical" => {
            let test_data: TestData<VectorFst<TropicalWeight>> = TestData::new(&parsed_test_data);
            do_run_test_pynini(&test_data)?;
        }
        "log" => {
            let test_data: TestData<VectorFst<LogWeight>> = TestData::new(&parsed_test_data);
            do_run_test_pynini(&test_data)?;
        }
        _ => bail!("Weight type unknown : {:?}", parsed_test_data.weight_type),
    };

    Ok(())
}

fn do_run_test_pynini<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + StarSemiring + WeaklyDivisibleSemiring + WeightQuantize,
{
    test_rmepsilon(&test_data)?;

    test_invert(&test_data)?;

    test_project_output(&test_data)?;

    test_project_input(&test_data)?;

    test_reverse(&test_data)?;

    test_connect(&test_data)?;

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

    test_fst_properties(&test_data)?;

    Ok(())
}

fn test_rmepsilon<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + WeaklyDivisibleSemiring + StarSemiring,
{
    // Remove epsilon
    let fst_rmepsilon: VectorFst<_> = rm_epsilon(&test_data.raw).unwrap();
    assert!(
        isomorphic(&fst_rmepsilon, &test_data.rmepsilon)?,
        "{}",
        error_message_fst!(test_data.rmepsilon, fst_rmepsilon, "RmEpsilon")
    );
    Ok(())
}

fn test_invert<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Invert
    let mut fst_invert = test_data.raw.clone();
    invert(&mut fst_invert);
    assert_eq!(
        test_data.invert,
        fst_invert,
        "{}",
        error_message_fst!(test_data.invert, fst_invert, "Invert")
    );
    Ok(())
}

fn test_project_output<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Project output
    let mut fst_project_output = test_data.raw.clone();
    project(&mut fst_project_output, ProjectType::ProjectOutput);
    assert_eq!(
        test_data.project_output,
        fst_project_output,
        "{}",
        error_message_fst!(
            test_data.project_output,
            fst_project_output,
            "Project Output"
        )
    );
    Ok(())
}

fn test_project_input<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Project input
    let mut fst_project_input = test_data.raw.clone();
    project(&mut fst_project_input, ProjectType::ProjectInput);
    assert_eq!(
        test_data.project_input,
        fst_project_input,
        "{}",
        error_message_fst!(test_data.project_input, fst_project_input, "Project Input")
    );
    Ok(())
}

fn test_reverse<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: 'static + Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Reverse
    let fst_reverse: VectorFst<_> = reverse(&test_data.raw).unwrap();
    assert!(
        isomorphic(&test_data.reverse, &fst_reverse)?,
        "{}",
        error_message_fst!(test_data.reverse, fst_reverse, "Reverse")
    );
    Ok(())
}

fn test_connect<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    // Connect
    let mut fst_connect = test_data.raw.clone();
    connect(&mut fst_connect)?;
    assert_eq!(
        test_data.connect,
        fst_connect,
        "{}",
        error_message_fst!(test_data.connect, fst_connect, "Connect")
    );
    Ok(())
}

fn test_weight_pushing_initial<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Weight pushing initial
    let mut fst_weight_push_initial = test_data.raw.clone();
    push_weights(
        &mut fst_weight_push_initial,
        ReweightType::ReweightToInitial,
    )?;
    assert_eq!(
        test_data.weight_pushing_initial,
        fst_weight_push_initial,
        "{}",
        error_message_fst!(
            test_data.weight_pushing_initial,
            fst_weight_push_initial,
            "Weight Pushing initial"
        )
    );
    Ok(())
}

fn test_weight_pushing_final<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring,
{
    // Weight pushing final
    let mut fst_weight_push_final = test_data.raw.clone();
    push_weights(&mut fst_weight_push_final, ReweightType::ReweightToFinal)?;
    assert_eq!(
        test_data.weight_pushing_final,
        fst_weight_push_final,
        "{}",
        error_message_fst!(
            test_data.weight_pushing_final,
            fst_weight_push_final,
            "Weight Pushing final"
        )
    );
    Ok(())
}

fn test_arc_map_identity<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    // ArcMap IdentityMapper
    let mut fst_arc_map_identity = test_data.raw.clone();
    let mut identity_mapper = IdentityArcMapper {};
    fst_arc_map_identity.arc_map(&mut identity_mapper)?;
    assert_eq!(
        test_data.arc_map_identity,
        fst_arc_map_identity,
        "{}",
        error_message_fst!(
            test_data.arc_map_identity,
            fst_arc_map_identity,
            "ArcMap identity"
        )
    );
    Ok(())
}

fn test_arc_map_rmweight<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    // ArcMap RmWeightMapper
    let mut fst_arc_map_rmweight = test_data.raw.clone();
    let mut rmweight_mapper = RmWeightMapper {};
    fst_arc_map_rmweight.arc_map(&mut rmweight_mapper)?;
    assert_eq!(
        test_data.arc_map_rmweight,
        fst_arc_map_rmweight,
        "{}",
        error_message_fst!(
            test_data.arc_map_rmweight,
            fst_arc_map_rmweight,
            "ArcMap RmWeight"
        )
    );
    Ok(())
}

fn test_arc_map_invert<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize + WeaklyDivisibleSemiring,
{
    // ArcMap InvertWeightMapper
    let mut fst_arc_map_invert = test_data.raw.clone();
    let mut invertweight_mapper = InvertWeightMapper {};
    fst_arc_map_invert.arc_map(&mut invertweight_mapper)?;
    assert_eq!(
        test_data.arc_map_invert,
        fst_arc_map_invert,
        "{}",
        error_message_fst!(
            test_data.arc_map_invert,
            fst_arc_map_invert,
            "ArcMap InvertWeight"
        )
    );
    Ok(())
}

fn test_arc_map_input_epsilon<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = InputEpsilonMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_input_epsilon,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_input_epsilon,
            fst_arc_map,
            "ArcMap InputEpsilonMapper"
        )
    );
    Ok(())
}

fn test_arc_map_output_epsilon<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = OutputEpsilonMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_output_epsilon,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arc_map,
            "ArcMap OutputEpsilonMapper"
        )
    );
    Ok(())
}

fn test_arc_map_plus<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = PlusMapper::new(1.5);
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_plus,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_plus,
            fst_arc_map,
            "ArcMap PlusMapper (1.5)"
        )
    );
    Ok(())
}

fn test_arc_map_times<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = TimesMapper::new(1.5);
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_times,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_times,
            fst_arc_map,
            "ArcMap TimesMapper (1.5)"
        )
    );
    Ok(())
}

fn test_arc_map_quantize<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arc_map = test_data.raw.clone();
    let mut mapper = QuantizeMapper {};
    fst_arc_map.arc_map(&mut mapper)?;
    assert_eq!(
        test_data.arc_map_quantize,
        fst_arc_map,
        "{}",
        error_message_fst!(
            test_data.arc_map_quantize,
            fst_arc_map,
            "ArcMap QuantizeMapper"
        )
    );
    Ok(())
}

fn test_encode<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    for encode_test_data in &test_data.encode {
        let mut fst_encoded = test_data.raw.clone();
        encode(&mut fst_encoded, encode_test_data.encode_labels, encode_test_data.encode_weights)
            .with_context(|_| format_err!(
            "Error when running test_encode with parameters encode_labels={:?} and encode_weights={:?}.",
            encode_test_data.encode_labels, encode_test_data.encode_weights))?;
        assert_eq!(
            encode_test_data.result,
            fst_encoded,
            "{}",
            error_message_fst!(encode_test_data.result, fst_encoded, "Encode")
        );
    }
    Ok(())
}

fn test_encode_decode<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    for encode_test_data in &test_data.encode_decode {
        let mut fst_encoded = test_data.raw.clone();
        let encode_table = encode(&mut fst_encoded, encode_test_data.encode_labels, encode_test_data.encode_weights)
            .with_context(|_| format_err!(
            "Error when running test_encode_decode with parameters encode_labels={:?} and encode_weights={:?}.",
            encode_test_data.encode_labels, encode_test_data.encode_weights))?;
        decode(&mut fst_encoded, encode_table)?;
        assert_eq!(
            encode_test_data.result,
            fst_encoded,
            "{}",
            error_message_fst!(
                encode_test_data.result,
                fst_encoded,
                format!(
                    "Encode/Decode with encode_labels={:?} and encode_weights={:?}",
                    encode_test_data.encode_labels, encode_test_data.encode_weights
                )
            )
        );
    }
    Ok(())
}

fn test_state_map_arc_sum<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    let mut fst_state_map = test_data.raw.clone();
    let mut mapper = ArcSumMapper {};
    fst_state_map.state_map(&mut mapper)?;

    assert_eq!(
        test_data.state_map_arc_sum,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_arc_sum,
            fst_state_map,
            "StateMap : ArcSum"
        )
    );

    Ok(())
}

fn test_state_map_arc_unique<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32>,
{
    let mut fst_state_map = test_data.raw.clone();
    let mut mapper = ArcUniqueMapper {};
    fst_state_map.state_map(&mut mapper)?;

    assert_eq!(
        test_data.state_map_arc_unique,
        fst_state_map,
        "{}",
        error_message_fst!(
            test_data.state_map_arc_unique,
            fst_state_map,
            "StateMap : ArcUnique"
        )
    );

    Ok(())
}

fn test_determinize<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeaklyDivisibleSemiring + WeightQuantize + 'static,
{
    for determinize_data in &test_data.determinize {
        println!("det_type = {:?}", determinize_data.det_type);
        let fst_raw = test_data.raw.clone();
        let fst_res: Fallible<F> = determinize(&fst_raw, determinize_data.det_type.clone());

        match (&determinize_data.result, fst_res) {
            (Ok(fst_expected), Ok(ref fst_determinized)) => {
                let a = isomorphic(fst_expected, fst_determinized)?;
                assert!(
                    a,
                    "{}",
                    error_message_fst!(
                        fst_expected,
                        fst_determinized,
                        format!(
                            "Determinize fail for det_type = {:?} ",
                            determinize_data.det_type
                        )
                    )
                );
            }
            (Ok(_fst_expected), Err(_)) => panic!(
                "Determinize fail for det_type {:?}. Got Err. Expected Ok",
                determinize_data.det_type
            ),
            (Err(_), Ok(_fst_determinized)) => panic!(
                "Determinize fail for det_type {:?}. Got Ok. Expected Err, \n{}",
                determinize_data.det_type, _fst_determinized
            ),
            (Err(_), Err(_)) => {
                // Ok
            }
        };
    }
    Ok(())
}

fn test_arcsort_ilabel<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, ilabel_compare)?;
    assert_eq!(
        test_data.arcsort_ilabel,
        fst_arcsort,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arcsort,
            "ArcSort ilabel"
        )
    );
    Ok(())
}

fn test_arcsort_olabel<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let mut fst_arcsort = test_data.raw.clone();
    arc_sort(&mut fst_arcsort, olabel_compare)?;
    assert_eq!(
        test_data.arcsort_olabel,
        fst_arcsort,
        "{}",
        error_message_fst!(
            test_data.arc_map_output_epsilon,
            fst_arcsort,
            "ArcSort olabel"
        )
    );
    Ok(())
}

fn test_fst_properties<F>(test_data: &TestData<F>) -> Fallible<()>
where
    F: TextParser + MutableFst,
    F::W: Semiring<Type = f32> + WeightQuantize,
{
    let ref_props = test_data.fst_properties;
    let props = test_data.raw.properties()?;

    assert_eq!(ref_props, props);

    Ok(())
}

pub struct ExitFailure(failure::Error);

/// Prints a list of causes for this Error, along with any backtrace
/// information collected by the Error (if RUST_BACKTRACE=1).
impl std::fmt::Debug for ExitFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fail = self.0.as_fail();

        writeln!(f, "{}", &fail)?;

        let mut x: &Fail = fail;
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
fn test_pynini_fst_000() -> Result<(), ExitFailure> {
    run_test_pynini("fst_000").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_001() -> Result<(), ExitFailure> {
    run_test_pynini("fst_001").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_002() -> Result<(), ExitFailure> {
    run_test_pynini("fst_002").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_003() -> Result<(), ExitFailure> {
    run_test_pynini("fst_003").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_004() -> Result<(), ExitFailure> {
    run_test_pynini("fst_004").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_005() -> Result<(), ExitFailure> {
    run_test_pynini("fst_005").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_006() -> Result<(), ExitFailure> {
    run_test_pynini("fst_006").map_err(|v| v.into())
}

#[test]
fn test_pynini_fst_007() -> Result<(), ExitFailure> {
    run_test_pynini("fst_007").map_err(|v| v.into())
}
