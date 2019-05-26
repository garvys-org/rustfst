#include <iostream>
#include <iomanip>

// Json library
#include "json.hpp"

// Fst lib
#include "fst/vector-fst.h"
#include "fst/script/print.h"

#include "fst_000/fst_000.h"
#include "fst_001/fst_001.h"
#include "fst_002/fst_002.h"
#include "fst_003/fst_003.h"
#include "fst_004/fst_004.h"
#include "fst_005/fst_005.h"
#include "fst_006/fst_006.h"
#include "fst_007/fst_007.h"

using namespace std;
using json = nlohmann::json;

template<class A>
string fst_to_string(const fst::VectorFst<A> a) {
    std::stringstream sstrm;
    fst::script::PrintFst(a, sstrm, string("<rustfst>"), NULL, NULL, NULL);
    return sstrm.str();
}

template<class F>
void compute_fst_invert(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Invert(&fst_out);
    j["invert"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_project_input(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Project(&fst_out, fst::ProjectType::PROJECT_INPUT);
    j["project_input"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_project_output(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Project(&fst_out, fst::ProjectType::PROJECT_OUTPUT);
    j["project_output"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_reverse(const F& raw_fst, json& j) {
    F fst_out;
    fst::Reverse(raw_fst, &fst_out);
    j["reverse"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_remove_epsilon(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    // Connect = false
    fst::RmEpsilon(&fst_out, false);
    j["rmepsilon"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_connect(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Connect(&fst_out);
    j["connect"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_shortest_distance(const F& raw_fst, json& j) {
    std::vector<typename F::Weight> v;
    fst::ShortestDistance(raw_fst, &v);
    std::vector<float> vf(v.size());
    for(int i = 0; i < v.size(); i++) {
        // TODO: Probable issue with infinity
        vf[i] = v[i].Value();
    }
    j["shortest_distance"]["result"] = vf;
}

template<class F>
void compute_fst_compute_weight_pushing_initial(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Push(&fst_out, fst::ReweightType::REWEIGHT_TO_INITIAL);
    j["weight_pushing_initial"]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_compute_weight_pushing_final(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::Push(&fst_out, fst::ReweightType::REWEIGHT_TO_FINAL);
    j["weight_pushing_final"]["result"] = fst_to_string(fst_out);
}

template<class F, class C>
void compute_fst_compute_arc_map(const F& raw_fst, json& j, const string& name, C mapper) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcMap(&fst_out, mapper);
    j[name]["result"] = fst_to_string(fst_out);
}

template<class F, class C>
void compute_fst_compute_arcsort(const F& raw_fst, json& j, const string& name, C compare) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcSort(&fst_out, compare);
    j[name]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_encode(const F& raw_fst, json& j) {
    std::vector<bool> v = {true, false};
    j["encode"] = {};
    for(bool encode_labels: v) {
        for(bool encode_weights: v) {
            uint32 flags = 0;
            if (encode_labels) {
                flags |= fst::kEncodeLabels;
            }
            if (encode_weights) {
                flags |= fst::kEncodeWeights;
            }
            auto fst_out = *raw_fst.Copy();
            fst::EncodeMapper<typename F::Arc> mapper(flags, fst::EncodeType::ENCODE);
            fst::Encode(&fst_out, &mapper);

            json j2;
            j2["encode_labels"] = encode_labels;
            j2["encode_weights"] = encode_weights;
            j2["result"] = fst_to_string(fst_out);
            j["encode"].push_back(j2);
        }
    }
}

template<class F>
void compute_fst_encode_decode(const F& raw_fst, json& j) {
    std::vector<bool> v = {true, false};
    j["encode_decode"] = {};
    for(bool encode_labels: v) {
        for(bool encode_weights: v) {
            uint32 flags = 0;
            if (encode_labels) {
                flags |= fst::kEncodeLabels;
            }
            if (encode_weights) {
                flags |= fst::kEncodeWeights;
            }
            auto fst_out = *raw_fst.Copy();
            fst::EncodeMapper<typename F::Arc> mapper(flags, fst::EncodeType::ENCODE);
            fst::Encode(&fst_out, &mapper);
            fst::Decode(&fst_out, mapper);

            json j2;
            j2["encode_labels"] = encode_labels;
            j2["encode_weights"] = encode_weights;
            j2["result"] = fst_to_string(fst_out);
            j["encode_decode"].push_back(j2);
        }
    }
}

template<class F, class C>
void compute_fst_state_map(const F& raw_fst, json& j, const string& name, C mapper) {
    auto fst_out = *raw_fst.Copy();
    fst::StateMap(&fst_out, mapper);
    j[name]["result"] = fst_to_string(fst_out);
}

template<class F>
void compute_fst_determinization(const F& raw_fst, json& j) {
    j["determinize"] = {};

    {
        fst::DeterminizeOptions<typename F::Arc> opts;
        opts.type = fst::DeterminizeType::DETERMINIZE_FUNCTIONAL;
        F fst_out;
        fst::Determinize(raw_fst, &fst_out, opts);
        json j2;
        j2["det_type"] = "functional";
        j2["result"] = fst_to_string(fst_out);
        j["determinize"].push_back(j2);
    }

    {
        fst::DeterminizeOptions<typename F::Arc> opts;
        opts.type = fst::DeterminizeType::DETERMINIZE_NONFUNCTIONAL;
        F fst_out;
        fst::Determinize(raw_fst, &fst_out, opts);
        json j2;
        j2["det_type"] = "nonfunctional";
        j2["result"] = fst_to_string(fst_out);
        j["determinize"].push_back(j2);
    }

    {
        fst::DeterminizeOptions<typename F::Arc> opts;
        opts.type = fst::DeterminizeType::DETERMINIZE_DISAMBIGUATE;
        F fst_out;
        fst::Determinize(raw_fst, &fst_out, opts);
        json j2;
        j2["det_type"] = "disambiguate";
        j2["result"] = fst_to_string(fst_out);
        j["determinize"].push_back(j2);
    }
}

template<class F>
void compute_fst_topsort(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcSort(&fst_out, fst::ILabelCompare<typename F::Arc>());
    fst::TopSort(&fst_out);
    j["topsort"]["result"] = fst_to_string(fst_out);
}

bool prop_to_bool(uint64 all_props, uint64 prop) {
    return (all_props & prop) == prop;
}

template<class F>
void compute_fst_properties(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    auto a = fst_out.Properties(fst::kTrinaryProperties, true);
    j["fst_properties"]["acceptor"] = prop_to_bool(a, fst::kAcceptor);
    j["fst_properties"]["not_acceptor"] = prop_to_bool(a, fst::kNotAcceptor);
    j["fst_properties"]["i_deterministic"] = prop_to_bool(a, fst::kIDeterministic);
    j["fst_properties"]["not_i_deterministic"] = prop_to_bool(a, fst::kNonIDeterministic);
    j["fst_properties"]["o_deterministic"] = prop_to_bool(a, fst::kODeterministic);
    j["fst_properties"]["not_o_deterministic"] = prop_to_bool(a, fst::kNonODeterministic);
    j["fst_properties"]["epsilons"] = prop_to_bool(a, fst::kEpsilons);
    j["fst_properties"]["no_epsilons"] = prop_to_bool(a, fst::kNoEpsilons);
    j["fst_properties"]["i_epsilons"] = prop_to_bool(a, fst::kIEpsilons);
    j["fst_properties"]["no_i_epsilons"] = prop_to_bool(a, fst::kNoIEpsilons);
    j["fst_properties"]["o_epsilons"] = prop_to_bool(a, fst::kOEpsilons);
    j["fst_properties"]["no_o_epsilons"] = prop_to_bool(a, fst::kNoOEpsilons);
    j["fst_properties"]["i_label_sorted"] = prop_to_bool(a, fst::kILabelSorted);
    j["fst_properties"]["not_i_label_sorted"] = prop_to_bool(a, fst::kNotILabelSorted);
    j["fst_properties"]["o_label_sorted"] = prop_to_bool(a, fst::kOLabelSorted);
    j["fst_properties"]["not_o_label_sorted"] = prop_to_bool(a, fst::kNotOLabelSorted);
    j["fst_properties"]["weighted"] = prop_to_bool(a, fst::kWeighted);
    j["fst_properties"]["unweighted"] = prop_to_bool(a, fst::kUnweighted);
    j["fst_properties"]["cyclic"] = prop_to_bool(a, fst::kCyclic);
    j["fst_properties"]["acyclic"] = prop_to_bool(a, fst::kAcyclic);
    j["fst_properties"]["initial_cyclic"] = prop_to_bool(a, fst::kInitialCyclic);
    j["fst_properties"]["initial_acyclic"] = prop_to_bool(a, fst::kInitialAcyclic);
    j["fst_properties"]["top_sorted"] = prop_to_bool(a, fst::kTopSorted);
    j["fst_properties"]["not_top_sorted"] = prop_to_bool(a, fst::kNotTopSorted);
    j["fst_properties"]["accessible"] = prop_to_bool(a, fst::kAccessible);
    j["fst_properties"]["not_accessible"] = prop_to_bool(a, fst::kNotAccessible);
    j["fst_properties"]["coaccessible"] = prop_to_bool(a, fst::kCoAccessible);
    j["fst_properties"]["not_coaccessible"] = prop_to_bool(a, fst::kNotCoAccessible);
    j["fst_properties"]["string"] = prop_to_bool(a, fst::kString);
    j["fst_properties"]["not_string"] = prop_to_bool(a, fst::kNotString);
    j["fst_properties"]["weighted_cycles"] = prop_to_bool(a, fst::kWeightedCycles);
    j["fst_properties"]["unweighted_cycles"] = prop_to_bool(a, fst::kUnweightedCycles);

    assert(j["fst_properties"].size() == 32);
}

template<class A>
void compute_data(const fst::VectorFst<A>& raw_fst, const string fst_name) {
    json data;

    data["name"] = fst_name;
    data["weight_type"] = A::Type();
    data["raw"]["result"] = fst_to_string(raw_fst);

    std::cout << "Invert" << std::endl;
    compute_fst_invert(raw_fst, data);

    std::cout << "Project Input" << std::endl;
    compute_fst_project_input(raw_fst, data);

    std::cout << "Project Output" << std::endl;
    compute_fst_project_output(raw_fst, data);

    std::cout << "Reverse" << std::endl;
    compute_fst_reverse(raw_fst, data);

    std::cout << "Remove epsilon" << std::endl;
    compute_fst_remove_epsilon(raw_fst, data);

    std::cout << "Connect" << std::endl;
    compute_fst_connect(raw_fst, data);

    std::cout << "Shortest distance" << std::endl;
    compute_fst_shortest_distance(raw_fst, data);

    std::cout << "Weight pushing initial" << std::endl;
    compute_fst_compute_weight_pushing_initial(raw_fst, data);

    std::cout << "Weight pushing final" << std::endl;
    compute_fst_compute_weight_pushing_final(raw_fst, data);

    std::cout << "ArcMap" << std::endl;
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_identity", fst::IdentityMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_rmweight", fst::RmWeightMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_invert", fst::InvertMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_input_epsilon", fst::InputEpsilonMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_output_epsilon", fst::OutputEpsilonMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_quantize", fst::QuantizeMapper<A>());
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_plus", fst::PlusMapper<A>(1.5));
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_times", fst::TimesMapper<A>(1.5));

    std::cout << "ArcSort" << std::endl;
    compute_fst_compute_arcsort(raw_fst, data, "arcsort_ilabel", fst::ILabelCompare<A>());
    compute_fst_compute_arcsort(raw_fst, data, "arcsort_olabel", fst::OLabelCompare<A>());

    std::cout << "Encode" << std::endl;
    compute_fst_encode(raw_fst, data);

    std::cout << "Encode / Decode" << std::endl;
    compute_fst_encode_decode(raw_fst, data);

    std::cout << "StateMap" << std::endl;
    compute_fst_state_map(raw_fst, data, "state_map_arc_sum", fst::ArcSumMapper<A>(raw_fst));
    compute_fst_state_map(raw_fst, data, "state_map_arc_unique", fst::ArcUniqueMapper<A>(raw_fst));

    std::cout << "Determinization" << std::endl;
    compute_fst_determinization(raw_fst, data);

    std::cout << "TopSort" << std::endl;
    compute_fst_topsort(raw_fst, data);

    std::cout << "Properties" << std::endl;
    compute_fst_properties(raw_fst, data);

    std::ofstream o(fst_name + "/metadata.json");
    o << std::setw(4) << data << std::endl;
}

int main() {
    compute_data(compute_fst_000(), "fst_000");
    compute_data(compute_fst_001(), "fst_001");
    compute_data(compute_fst_002(), "fst_002");
    compute_data(compute_fst_003(), "fst_003");
    compute_data(compute_fst_004(), "fst_004");
    compute_data(compute_fst_005(), "fst_005");
    compute_data(compute_fst_006(), "fst_006");
    compute_data(compute_fst_007(), "fst_007");
}