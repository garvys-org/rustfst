#include <iostream>
#include <iomanip>

// Json library
#include "json.hpp"

// Fst lib
#include "fst/vector-fst.h"
#include "fst/script/print.h"
#include "fst/fst.h"
#include "fst/minimize.h"

#include "fst_000/fst_000.h"
#include "fst_001/fst_001.h"
#include "fst_002/fst_002.h"
#include "fst_003/fst_003.h"
#include "fst_004/fst_004.h"
#include "fst_005/fst_005.h"
#include "fst_006/fst_006.h"
#include "fst_007/fst_007.h"
#include "fst_008/fst_008.h"
#include "fst_009/fst_009.h"
#include "fst_010/fst_010.h"

#include "symt_000/symt_000.h"
#include "symt_001/symt_001.h"
#include "symt_002/symt_002.h"

using namespace std;
using json = nlohmann::json;

template<class A>
string fst_to_string(const fst::VectorFst<A>& a) {
    std::stringstream sstrm;
    const string sep = FLAGS_fst_field_separator.substr(0, 1);
    fst::FstPrinter<A> fstprinter(a, NULL, NULL, NULL, false, true, sep);
    fstprinter.Print(&sstrm, string("<rustfst>"));
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
    j["shortest_distance"] = {};
    std::vector<bool> v = {true, false};
    for(bool reverse: v) {
        std::vector<typename F::Weight> distance;
        fst::ShortestDistance(raw_fst, &distance, reverse);
        std::vector<string> distance_s;
        for(auto e: distance) {
            distance_s.push_back(std::to_string(e.Value()));
        }
        json j2;
        j2["reverse"] = reverse;
        j2["result"] = distance_s;
        j["shortest_distance"].push_back(j2);
    }
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

bool prop_to_bool(uint64 all_props, uint64 prop) {
    return (all_props & prop) == prop;
}

template<class F>
void compute_fst_determinization(const F& raw_fst, json& j, fst::DeterminizeType det_type, const string& name) {
    fst::DeterminizeOptions<typename F::Arc> opts;
    opts.type = det_type;
    F fst_out;
    fst::Determinize(raw_fst, &fst_out, opts);
    bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
    json j2;
    j2["det_type"] = name;
    j2["result"] = error? "error": fst_to_string(fst_out);
    j["determinize"].push_back(j2);
}

template<class F>
void compute_fst_determinization(const F& raw_fst, json& j) {
    j["determinize"] = {};

    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_FUNCTIONAL, "functional");
    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_NONFUNCTIONAL, "nonfunctional");
    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_DISAMBIGUATE, "disambiguate");
}

template<class F>
void compute_fst_topsort(const F& raw_fst, json& j) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcSort(&fst_out, fst::ILabelCompare<typename F::Arc>());
    fst::TopSort(&fst_out);
    bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
    j["topsort"]["result"] = error ? "error" : fst_to_string(fst_out);
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

template<class F>
void compute_fst_minimization(const F& raw_fst, json& j) {
    j["minimize"] = {};
    std::vector<bool> v = {true, false};
    for(bool allow_nondet: v) {
        auto fst_out = *raw_fst.Copy();
        fst::Minimize(&fst_out, (fst::VectorFst<typename F::Arc>*)nullptr, fst::kShortestDelta, allow_nondet);
        bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);

        json j2;
        j2["allow_nondet"] = allow_nondet;
        j2["result"] = error ? "error": fst_to_string(fst_out);

        j["minimize"].push_back(j2);
    }
}

template<class F>
void compute_fst_shortest_path(const F& raw_fst, json& j) {
    j["shortest_path"] = {};
    std::vector<bool> v = {true, false};
    for(int n = 1; n < 10; n++) {
        for(bool unique: v) {
            fst::VectorFst<typename F::Arc> fst_out;
            fst::ShortestPath(raw_fst, &fst_out, n, unique);
            bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
            json j2;

            j2["nshortest"] = n;
            j2["unique"] = unique;
            j2["result"] = error ? "error": fst_to_string(fst_out);

            j["shortest_path"].push_back(j2);
        }
    }
}

template<class F, fst::GallicType G>
void _compute_fst_gallic_encode_decode(const F& raw_fst, json& j, const string& gtype_s) {
    fst::ToGallicMapper<typename F::Arc, G> to_gallic;
    fst::FromGallicMapper<typename F::Arc, G> from_gallic(0);

    fst::VectorFst<fst::GallicArc<typename F::Arc, G>> fst_1;
    fst::ArcMap(raw_fst, &fst_1, &to_gallic);
    F fst_out;
    fst::ArcMap(fst_1, &fst_out, &from_gallic);

    json j2;
    j2["gallic_type"] = gtype_s;
    j2["result"] = fst_to_string(fst_out);
    j["gallic_encode_decode"].push_back(j2);
}

template<class F>
void compute_fst_gallic_encode_decode(const F& raw_fst, json& j) {
    // Encode and decode with a gallic mapper
    j["gallic_encode_decoder"] = {};
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_LEFT>(raw_fst, j, "gallic_left");
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_RIGHT>(raw_fst, j, "gallic_right");
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_RESTRICT>(raw_fst, j, "gallic_restrict");
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_MIN>(raw_fst, j, "gallic_min");
    _compute_fst_gallic_encode_decode<F, fst::GALLIC>(raw_fst, j, "gallic");
}

template<class F>
void compute_fst_factor_weight_identity(const F& raw_fst, json& j) {
    std::vector<bool> v = {false, true};
    j["factor_weight_identity"] = {};

    for(bool factor_arc_weights: v) {
        for(bool factor_final_weights: v) {
            uint32 mode;
            if (factor_arc_weights)
                mode |= fst::kFactorArcWeights;
            if (factor_final_weights)
                mode |= fst::kFactorFinalWeights;
            if (!factor_arc_weights && !factor_final_weights) {
                continue;
            }

            fst::FactorWeightOptions<typename F::Arc> opts(fst::kDelta, mode);
            fst::FactorWeightFst<
                typename F::Arc,
                typename fst::IdentityFactor<
                    typename F::Weight
                >
            > factored_fst(raw_fst, opts);
            fst::VectorFst<typename F::Arc> fst_out(factored_fst);

            json j2;
            j2["factor_final_weights"] = factor_final_weights;
            j2["factor_arc_weights"] = factor_arc_weights;
            j2["result"] = fst_to_string(fst_out);
            j["factor_weight_identity"].push_back(j2);
        }
    }
}

template<class F, fst::GallicType G>
void _compute_fst_factor_weight_gallic(const F& raw_fst, json& j, const string& gtype_s) {

    std::vector<bool> v = {true, false};

    for(bool factor_arc_weights: v) {
        for(bool factor_final_weights: v) {
            uint32 mode;
            if (factor_arc_weights)
                mode |= fst::kFactorArcWeights;
            if (factor_final_weights)
                mode |= fst::kFactorFinalWeights;
            if (!factor_arc_weights && !factor_final_weights) {
                continue;
            }
            fst::ToGallicMapper<typename F::Arc, G> to_gallic;
            fst::FromGallicMapper<typename F::Arc, G> from_gallic(0);

            /// To Gallic
            fst::VectorFst<fst::GallicArc<typename F::Arc, G>> fst_1;
            fst::ArcMap(raw_fst, &fst_1, &to_gallic);

            // Factor Weight
            fst::FactorWeightOptions<
                fst::GallicArc<typename F::Arc, G>
            > opts(fst::kDelta, mode);
            fst::FactorWeightFst<
                fst::GallicArc<typename F::Arc, G>,
                typename fst::GallicFactor<
                    int, typename F::Weight, G
                >
            > factored_fst(fst_1, opts);
            fst::VectorFst<fst::GallicArc<typename F::Arc, G>> fst_2(factored_fst);

            // From Gallic
            F fst_out;
            fst::ArcMap(fst_2, &fst_out, &from_gallic);

            json j2;
            j2["gallic_type"] = gtype_s;
            j2["factor_final_weights"] = factor_final_weights;
            j2["factor_arc_weights"] = factor_arc_weights;
            j2["result"] = fst_to_string(fst_out);
            j["factor_weight_gallic"].push_back(j2);
        }
    }
}

template<class F>
void compute_fst_factor_weight_gallic(const F& raw_fst, json& j) {
    // Encode and decode with a gallic mapper
    j["factor_weight_gallic"] = {};
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_LEFT>(raw_fst, j, "gallic_left");
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_RIGHT>(raw_fst, j, "gallic_right");
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_RESTRICT>(raw_fst, j, "gallic_restrict");
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_MIN>(raw_fst, j, "gallic_min");
    _compute_fst_factor_weight_gallic<F, fst::GALLIC>(raw_fst, j, "gallic");
}

template<class F>
void compute_fst_push(const F& raw_fst, json& j) {
    std::vector<bool> v = {true, false};
    j["push"] = {};
    for(bool push_weights: v) {
        for(bool push_labels: v) {
            for(bool remove_total_weight: v) {
                for(bool remove_common_affix: v) {
                    for(bool reweight_to_final: v) {

                        uint32 pflags = 0;
                        if (push_weights) {
                            pflags |= fst::kPushWeights;
                        }
                        if (push_labels) {
                            pflags |= fst::kPushLabels;
                        }
                        if (remove_total_weight) {
                            pflags |= fst::kPushRemoveTotalWeight;
                        }
                        if (remove_common_affix) {
                            pflags |= fst::kPushRemoveCommonAffix;
                        }

//                        std::cerr << "push_weights = " << push_weights << std::endl;
//                        std::cerr << "push_labels = " << push_labels << std::endl;
//                        std::cerr << "remove_total_weight = " << remove_total_weight << std::endl;
//                        std::cerr << "remove_common_affix = " << remove_common_affix << std::endl;
//                        std::cerr << "reweight_to_final = " << reweight_to_final << std::endl;
                        F fst_out;
                        // Fixes crash in OpenFST on empty input
                        if (raw_fst.NumStates() > 0) {
                            if (reweight_to_final) {
                                fst::Push<typename F::Arc, fst::REWEIGHT_TO_FINAL>(raw_fst, &fst_out, pflags);
                            } else {
                                fst::Push<typename F::Arc, fst::REWEIGHT_TO_INITIAL>(raw_fst, &fst_out, pflags);
                            }
                        }

//                        std::cerr << "Done" << std::endl << std::endl;

                        json j2;
                        j2["push_weights"] = push_weights;
                        j2["push_labels"] = push_labels;
                        j2["remove_total_weight"] = remove_total_weight;
                        j2["remove_common_affix"] = remove_common_affix;
                        j2["reweight_to_final"] = reweight_to_final;
                        j2["result"] = fst_to_string(fst_out);
                        j["push"].push_back(j2);

                    }
                }
            }
        }
    }
}

template<class A>
void compute_fst_data(const fst::VectorFst<A>& raw_fst, const string fst_name) {
    std::cout << "FST :" << fst_name << std::endl;
    json data;

    data["name"] = fst_name;
    data["weight_type"] = A::Type();
    data["raw"]["result"] = fst_to_string(raw_fst);

    data["raw_vector_bin_path"] = "raw_vector.fst";
    raw_fst.Write(fst_name + "/raw_vector.fst");

    fst::FstWriteOptions write_opts("<unspecified>");
    fst::ConstFst<A> raw_const_fst(raw_fst);
    // Not aligned
    write_opts.align = false;
    data["raw_const_bin_path"] = "raw_const.fst";
    std::ofstream strm((fst_name + "/raw_const.fst").c_str(), std::ios_base::out | std::ios_base::binary);
    raw_const_fst.Write(strm, write_opts);

    // Aligned
    write_opts.align = true;
    data["raw_const_aligned_bin_path"] = "raw_const_aligned.fst";
    std::ofstream strm_aligned((fst_name + "/raw_const_aligned.fst").c_str(), std::ios_base::out | std::ios_base::binary);
    raw_const_fst.Write(strm_aligned, write_opts);

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
    compute_fst_compute_arc_map(raw_fst, data, "arc_map_invert", fst::InvertWeightMapper<A>());
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

    std::cout << "Minimization" << std::endl;
    compute_fst_minimization(raw_fst, data);

    std::cout << "ShortestPath" << std::endl;
    compute_fst_shortest_path(raw_fst, data);

    std::cout << "Gallic Encode Decode" << std::endl;
    compute_fst_gallic_encode_decode(raw_fst, data);

    std::cout << "Factor Weight Identity" << std::endl;
    compute_fst_factor_weight_identity(raw_fst, data);

    std::cout << "Factor Weight Gallic" << std::endl;
    compute_fst_factor_weight_gallic(raw_fst, data);

    std::cout << "Push" << std::endl;
    compute_fst_push(raw_fst, data);

    std::ofstream o(fst_name + "/metadata.json");
    o << std::setw(4) << data << std::endl;

    std::cout << std::endl;
}

void compute_symt_data(const fst::SymbolTable symt, const string symt_name) {
    std::cout << "Symt :" << symt_name << std::endl;
    json data;

    data["name"] = symt_name;

    data["num_symbols"] = symt.NumSymbols();

    data["symt_bin"] = "symt.bin";
    symt.Write(symt_name + "/symt.bin");

    data["symt_text"] = "symt.text";
    symt.WriteText(symt_name + "/symt.text");

    std::ofstream o(symt_name + "/metadata.json");
    o << std::setw(4) << data << std::endl;

    std::cout << std::endl;
}


int main() {
    FLAGS_fst_error_fatal = false;
    compute_fst_data(compute_fst_000(), "fst_000");
    compute_fst_data(compute_fst_001(), "fst_001");
    compute_fst_data(compute_fst_002(), "fst_002");
    compute_fst_data(compute_fst_003(), "fst_003");
    compute_fst_data(compute_fst_004(), "fst_004");
    compute_fst_data(compute_fst_005(), "fst_005");
    compute_fst_data(compute_fst_006(), "fst_006");
    compute_fst_data(compute_fst_007(), "fst_007");
    compute_fst_data(compute_fst_008(), "fst_008");
    compute_fst_data(compute_fst_009(), "fst_009");
    compute_fst_data(compute_fst_010(), "fst_010");

    compute_symt_data(compute_symt_000(), "symt_000");
    compute_symt_data(compute_symt_001(), "symt_001");
    compute_symt_data(compute_symt_002(), "symt_002");
}