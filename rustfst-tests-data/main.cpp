#include <iostream>
#include <iomanip>

// Json library
#include "json.hpp"

#include "utils.h"

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
#include "fst_011/fst_011.h"
#include "fst_012/fst_012.h"
#include "fst_013/fst_013.h"
#include "fst_014/fst_014.h"
#include "fst_015/fst_015.h"
#include "fst_016/fst_016.h"
#include "fst_017/fst_017.h"
#include "fst_018/fst_018.h"
#include "fst_019/fst_019.h"
#include "fst_020/fst_020.h"

#include "symt_000/symt_000.h"
#include "symt_001/symt_001.h"
#include "symt_002/symt_002.h"

#include "../openfst_addon/optimize.cpp"

int ID_FST_NUM = 0;

using namespace std;
using json = nlohmann::json;

template<class F>
string dump_fst(const F& a, const string& dir_path) {
    auto name = "res_" + std::to_string(ID_FST_NUM) + ".fst";
    a.Write(dir_path + name);
    ID_FST_NUM++;
    return name;
}

template<class F>
string fst_to_string(const F& a) {
    using Arc = typename F::Arc;
    std::stringstream sstrm;
    const string sep = FLAGS_fst_field_separator.substr(0, 1);
    fst::FstPrinter<Arc> fstprinter(a, NULL, NULL, NULL, false, true, sep);
    fstprinter.Print(&sstrm, string("<rustfst>"));
    return sstrm.str();
}

template<class W>
string weight_to_string(const W& a) {
    std::stringstream ss;
    ss << a;
    return ss.str();
}

template<class F>
void compute_fst_optimize(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    Optimize(&fst_out);
    j["optimize"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_invert(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Invert(&fst_out);
    j["invert"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_project_input(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Project(&fst_out, fst::ProjectType::PROJECT_INPUT);
    j["project_input"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_project_output(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Project(&fst_out, fst::ProjectType::PROJECT_OUTPUT);
    auto name = "fst_project_output.fst";
    j["project_output"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_reverse(const F& raw_fst, json& j, const string& dir_path) {
    F fst_out;
    fst::Reverse(raw_fst, &fst_out);
    j["reverse"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_remove_epsilon(const F& raw_fst, json& j, const string& dir_path) {
    using Arc = typename F::Arc;
    auto fst_out = *raw_fst.Copy();

    auto dyn_rmeps = fst::VectorFst<Arc>(fst::RmEpsilonFst<Arc>(raw_fst));

    fst::RmEpsilon(&fst_out);
    j["rmepsilon"]["result_static_path"] = dump_fst(fst_out, dir_path);
    j["rmepsilon"]["result_lazy_path"] = dump_fst(dyn_rmeps, dir_path);
}

template<class F>
void compute_fst_connect(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Connect(&fst_out);
    j["connect"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_condense(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_in = *raw_fst.Copy();
    std::vector<typename F::Arc::StateId> scc;
    F fst_out;
    fst::Condense(fst_in, &fst_out, &scc);
    std::vector<string> sccs;
    for (auto e: scc) {
        sccs.push_back(std::to_string(e));
    }
    j["condense"]["sccs"] = sccs;
    j["condense"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_state_reachable(const F& raw_fst, json& j) {
    using Arc = typename F::Arc;
    using Weight = typename F::Weight;
    auto fst_in = *raw_fst.Copy();
    fst::StateReachable<Arc> reachable(fst_in);

    j["state_reachable"]["result"] = {};
    auto error = reachable.Error();
    j["state_reachable"]["error"] = error;

    if (!error) {
        std::vector<int> final_states;
        for (int i = 0; i < fst_in.NumStates(); i++) {
            if (fst_in.Final(i) != Weight::Zero()) {
                final_states.push_back(i);
            }
        }

        for (int state = 0; state < fst_in.NumStates(); state++) {
            for(auto final_state: final_states) {
                json j2;
                j2["state"] = state;
                j2["final_state"] = final_state;

                reachable.SetState(state);
                auto res = reachable.Reach(final_state);
                auto error_reach = reachable.Error();

                j2["reachable"] = res;
                j2["error"] = error_reach;
                j["state_reachable"]["result"].push_back(j2);
                if (error_reach) {
                    return;
                }
            }
        }
    }

    if ((fst_in.NumStates() == 0) || error) {
        j["state_reachable"]["result"] = std::vector<int>();
    }
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
            distance_s.push_back(weight_to_string(e));
        }
        json j2;
        j2["reverse"] = reverse;
        j2["result"] = distance_s;
        j["shortest_distance"].push_back(j2);
    }
}

template<class F>
void compute_fst_compute_weight_pushing_initial(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Push(&fst_out, fst::ReweightType::REWEIGHT_TO_INITIAL);
    j["weight_pushing_initial"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_compute_weight_pushing_final(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::Push(&fst_out, fst::ReweightType::REWEIGHT_TO_FINAL);
    j["weight_pushing_final"]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F, class C>
void compute_fst_compute_tr_map(const F& raw_fst, json& j, const string& name, C mapper, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcMap(&fst_out, mapper);
    j[name]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_compute_tr_map_plus(const F& raw_fst, json& j, const typename F::Weight& weight, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    auto mapper = fst::PlusMapper<typename F::Arc>(weight);
    fst::ArcMap(&fst_out, mapper);
    auto name = "tr_map_plus";
    j[name]["weight"] = weight_to_string(weight);
    j[name]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_compute_tr_map_times(const F& raw_fst, json& j, const typename F::Weight& weight, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    auto mapper = fst::TimesMapper<typename F::Arc>(weight);
    fst::ArcMap(&fst_out, mapper);
    auto name = "tr_map_times";
    j[name]["weight"] = weight_to_string(weight);
    j[name]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F, class C>
void compute_fst_compute_tr_sort(const F& raw_fst, json& j, const string& name, C compare, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::ArcSort(&fst_out, compare);
    j[name]["result_path"] = dump_fst(fst_out, dir_path);
}

template<class F>
void compute_fst_encode(const F& raw_fst, json& j, const string& dir_path) {
    std::vector<bool> v = {true, false};
    j["encode"] = {};
    for(bool encode_labels: v) {
        for(bool encode_weights: v) {
            if (!encode_weights && !encode_labels) {
                continue;
            }
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
            j2["result_path"] = dump_fst(fst_out, dir_path);
            j["encode"].push_back(j2);
        }
    }
}

template<class F>
void compute_fst_encode_decode(const F& raw_fst, json& j, const string& dir_path) {
    std::vector<bool> v = {true, false};
    j["encode_decode"] = {};
    for(bool encode_labels: v) {
        for(bool encode_weights: v) {
            if (!encode_weights && !encode_labels) {
                continue;
            }
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
            j2["result_path"] = dump_fst(fst_out, dir_path);
            j["encode_decode"].push_back(j2);
        }
    }
}

template<class F, class C>
void compute_fst_state_map(const F& raw_fst, json& j, const string& name, C mapper, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::StateMap(&fst_out, mapper);
    j[name]["result_path"] = dump_fst(fst_out, dir_path);
}

bool prop_to_bool(uint64 all_props, uint64 prop) {
    return (all_props & prop) == prop;
}

template<class F>
void compute_fst_determinization(const F& raw_fst, json& j, fst::DeterminizeType det_type, const string& name, const string& dir_path) {
    fst::DeterminizeOptions<typename F::Arc> opts;
    opts.type = det_type;
    F fst_out;
    fst::Determinize(raw_fst, &fst_out, opts);
    bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
    json j2;
    j2["det_type"] = name;
    j2["result_path"] = error? "error": dump_fst(fst_out, dir_path);
    j["determinize"].push_back(j2);
}

template<class F>
void compute_fst_determinization(const F& raw_fst, json& j, const string& dir_path) {
    j["determinize"] = {};

    // To check if a FST is determinizable, let's try to disambiguate it.
    F raw_fst_disambiguated;
    fst::Disambiguate(raw_fst, &raw_fst_disambiguated);
    if (prop_to_bool(raw_fst_disambiguated.Properties(fst::kError, true), fst::kError)) {
        j["determinize"] = vector<int>();
        return;
    }

    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_FUNCTIONAL, "functional", dir_path);
    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_NONFUNCTIONAL, "nonfunctional", dir_path);
    compute_fst_determinization(raw_fst, j, fst::DeterminizeType::DETERMINIZE_DISAMBIGUATE, "disambiguate", dir_path);
}

template<class F>
void compute_fst_topsort(const F& raw_fst, json& j, const string& dir_path) {
    auto fst_out = *raw_fst.Copy();
    fst::TopSort(&fst_out);
    bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
    j["topsort"]["result_path"] = error ? "error" : dump_fst(fst_out, dir_path);
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
void compute_fst_minimization(const F& raw_fst, json& j, const string& dir_path) {
    j["minimize"] = {};
    auto delta = fst::kShortestDelta;
    std::vector<bool> v = {true, false};
    for(bool allow_nondet: v) {
        auto fst_out = *raw_fst.Copy();
        fst::Minimize(&fst_out, (fst::VectorFst<typename F::Arc>*)nullptr, delta, allow_nondet);
        bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);

        json j2;
        j2["delta"] = delta;
        j2["allow_nondet"] = allow_nondet;
        j2["result_path"] = error ? "error": dump_fst(fst_out, dir_path);

        j["minimize"].push_back(j2);
    }
}

template<class F>
void compute_fst_shortest_path(const F& raw_fst, json& j, const string& dir_path) {
    j["shortest_path"] = {};
    std::vector<bool> v = {true, false};
    for(int n = 1; n <= 5; n++) {
        for(bool unique: v) {
            fst::VectorFst<typename F::Arc> fst_out;
            fst::ShortestPath(raw_fst, &fst_out, n, unique);
            bool error = prop_to_bool(fst_out.Properties(fst::kError, true), fst::kError);
            json j2;

            j2["nshortest"] = n;
            j2["unique"] = unique;
            j2["result_path"] = error ? "error": dump_fst(fst_out, dir_path);

            j["shortest_path"].push_back(j2);
        }
    }
}

template<class F, fst::GallicType G>
void _compute_fst_gallic_encode_decode(const F& raw_fst, json& j, const string& gtype_s, const string& dir_path) {
    fst::ToGallicMapper<typename F::Arc, G> to_gallic;
    fst::FromGallicMapper<typename F::Arc, G> from_gallic(0);

    fst::VectorFst<fst::GallicArc<typename F::Arc, G>> fst_1;
    fst::ArcMap(raw_fst, &fst_1, &to_gallic);
    F fst_out;
    fst::ArcMap(fst_1, &fst_out, &from_gallic);

    json j2;
    j2["gallic_type"] = gtype_s;
    j2["result_path"] = dump_fst(fst_out, dir_path);
    j["gallic_encode_decode"].push_back(j2);
}

template<class F>
void compute_fst_gallic_encode_decode(const F& raw_fst, json& j, const string& dir_path) {
    // Encode and decode with a gallic mapper
    j["gallic_encode_decoder"] = {};
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_LEFT>(raw_fst, j, "gallic_left", dir_path);
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_RIGHT>(raw_fst, j, "gallic_right", dir_path);
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_RESTRICT>(raw_fst, j, "gallic_restrict", dir_path);
    _compute_fst_gallic_encode_decode<F, fst::GALLIC_MIN>(raw_fst, j, "gallic_min", dir_path);
    _compute_fst_gallic_encode_decode<F, fst::GALLIC>(raw_fst, j, "gallic", dir_path);
}

template<class F>
void compute_fst_factor_weight_identity(const F& raw_fst, json& j, const string& dir_path) {
    std::vector<bool> v = {false, true};
    j["factor_weight_identity"] = {};

    for(bool factor_tr_weights: v) {
        for(bool factor_final_weights: v) {
            uint32 mode;
            if (factor_tr_weights)
                mode |= fst::kFactorArcWeights;
            if (factor_final_weights)
                mode |= fst::kFactorFinalWeights;
            if (!factor_tr_weights && !factor_final_weights) {
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
            j2["factor_tr_weights"] = factor_tr_weights;
            j2["result_path"] = dump_fst(fst_out, dir_path);
            j["factor_weight_identity"].push_back(j2);
        }
    }
}

template<class F, fst::GallicType G>
void _compute_fst_factor_weight_gallic(const F& raw_fst, json& j, const string& gtype_s, const string& dir_path) {

    std::vector<bool> v = {true, false};

    for(bool factor_tr_weights: v) {
        for(bool factor_final_weights: v) {
            uint32 mode;
            if (factor_tr_weights)
                mode |= fst::kFactorArcWeights;
            if (factor_final_weights)
                mode |= fst::kFactorFinalWeights;
            if (!factor_tr_weights && !factor_final_weights) {
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
            j2["factor_tr_weights"] = factor_tr_weights;
            j2["result_path"] = dump_fst(fst_out, dir_path);
            j["factor_weight_gallic"].push_back(j2);
        }
    }
}

template<class F>
void compute_fst_factor_weight_gallic(const F& raw_fst, json& j, const string& dir_path) {
    // Encode and decode with a gallic mapper
    j["factor_weight_gallic"] = {};
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_LEFT>(raw_fst, j, "gallic_left", dir_path);
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_RIGHT>(raw_fst, j, "gallic_right", dir_path);
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_RESTRICT>(raw_fst, j, "gallic_restrict", dir_path);
    _compute_fst_factor_weight_gallic<F, fst::GALLIC_MIN>(raw_fst, j, "gallic_min", dir_path);
    _compute_fst_factor_weight_gallic<F, fst::GALLIC>(raw_fst, j, "gallic", dir_path);
}

template<class F>
void compute_fst_push(const F& raw_fst, json& j, const string& dir_path) {
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
                        j2["result_path"] = dump_fst(fst_out, dir_path);
                        j["push"].push_back(j2);

                    }
                }
            }
        }
    }
}

template<class F>
void do_compute_fst_replace(
        vector<pair<typename F::Arc::Label, const fst::Fst<typename F::Arc>* > > label_fst_pairs,
        int root,
        bool epsilon_on_replace,
        json& j,
        const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;
    fst::VectorFst<Arc> res;
    fst::Replace(label_fst_pairs, &res, root, epsilon_on_replace);

    vector<pair<typename Arc::Label, string > > label_fst_pairs_serialized;


    for (auto &e: label_fst_pairs) {
        auto label = e.first;
        auto fst = e.second;
        if (label == root) {
            continue;
        }
        label_fst_pairs_serialized.push_back(std::make_pair(label, dump_fst(*fst, dir_path)));
    }

    json j2;
    j2["label_fst_pairs_path"] = label_fst_pairs_serialized;
    j2["root"] = root;
    j2["epsilon_on_replace"] = epsilon_on_replace;
    j2["result_path"] = dump_fst(res, dir_path);
    j["replace"].push_back(j2);
}

template<class F>
void compute_fst_replace(const typename F::MyFst & raw_fst, json& j, const F& fst_test_data, const string& dir_path) {
    using MyFst = typename F::MyFst;
    using Weight = typename F::MyWeight;
    using Arc = typename F::MyArc;
    using StateId = typename F::MyArc::StateId;

    int N = 10;

    std::set<int> labels;
    for (fst::StateIterator<MyFst> siter(raw_fst); !siter.Done(); siter.Next()) {
        StateId state_id = siter.Value();
        for (fst::ArcIterator<MyFst> aiter(raw_fst, state_id); !aiter.Done(); aiter.Next()) {
            const Arc &tr = aiter.Value();
            labels.insert(tr.olabel);
        }
    }

    vector<int> labels_vec(labels.begin(), labels.end());

    j["replace"] = {};
    auto max_label = *std::max_element(labels.begin(), labels.end());
    auto root = max_label + 1;
    auto label_1 = max_label + 2;
    auto label_2 = max_label + 3;
    auto label_3 = max_label + 4;
    auto label_4 = max_label + 5;

    fst::VectorFst<Arc> fst_1;
    fst_1.AddState();
    fst_1.AddState();
    fst_1.AddState();
    fst_1.SetStart(0);
    fst_1.SetFinal(2, fst_test_data.random_weight());
    fst_1.AddArc(0, Arc(label_1, label_2, fst_test_data.random_weight(), 1));
    fst_1.AddArc(0, Arc(label_3, label_2, fst_test_data.random_weight(), 1));
    fst_1.AddArc(1, Arc(label_3, label_4, fst_test_data.random_weight(), 2));
    fst_1.AddArc(1, Arc(label_1, label_4, fst_test_data.random_weight(), 2));

    fst::VectorFst<Arc> fst_2;
    fst_2.AddState();
    fst_2.AddState();
    fst_2.AddState();
    fst_2.SetStart(0);
    fst_2.SetFinal(2, fst_test_data.random_weight());
    fst_2.AddArc(0, Arc(label_4, label_1, fst_test_data.random_weight(), 1));
    fst_2.AddArc(0, Arc(label_1, label_3, fst_test_data.random_weight(), 1));
    fst_2.AddArc(1, Arc(label_4, label_4, fst_test_data.random_weight(), 2));
    fst_2.AddArc(1, Arc(label_1, label_3, fst_test_data.random_weight(), 2));

    std::vector<bool> v = {true, false};

    std::random_shuffle(labels_vec.begin(), labels_vec.end());

    // Single replacement
    for (int i = 0; i < N; i++) {
        if (i < labels_vec.size()) {
            auto label = labels_vec[i];
            for (bool epsilon_on_replace: v) {
                vector<pair<typename Arc::Label, const fst::Fst<Arc>* > > label_fst_pairs;
                label_fst_pairs.push_back(std::make_pair(root, new fst::VectorFst<Arc>(raw_fst)));
                label_fst_pairs.push_back(std::make_pair(label, &fst_1));
                do_compute_fst_replace<MyFst>(label_fst_pairs, root, epsilon_on_replace, j, dir_path);
            }
        }
    }

    std::random_shuffle(labels_vec.begin(), labels_vec.end());

    // Two replacements
    for (int i = 0; i < N; i++) {
        if ((i + 1) < labels_vec.size()) {
            auto label_fst_1 = labels_vec[i];
            auto label_fst_2 = labels_vec[i+1];
            if (label_fst_1 != label_fst_2) {
                for (bool epsilon_on_replace: v) {
                    vector<pair<typename Arc::Label, const fst::Fst<Arc>* > > label_fst_pairs;
                    label_fst_pairs.push_back(std::make_pair(root, new fst::VectorFst<Arc>(raw_fst)));
                    label_fst_pairs.push_back(std::make_pair(label_fst_1, &fst_1));
                    label_fst_pairs.push_back(std::make_pair(label_fst_2, &fst_2));
                    do_compute_fst_replace<MyFst>(label_fst_pairs, root, epsilon_on_replace, j, dir_path);
                }
            }
        }
    }

    auto label_5 = label_4 + 1;
    fst_1.AddArc(0, Arc(label_3, label_5, fst_test_data.random_weight(), 1));
    fst_1.AddArc(1, Arc(label_5, label_2, fst_test_data.random_weight(), 2));

    std::random_shuffle(labels_vec.begin(), labels_vec.end());

    // Two replacements + recursion
    for (int i = 0; i < N; i++) {
        if (i < labels_vec.size()) {
            auto label = labels_vec[i];
            for (bool epsilon_on_replace: v) {
                vector<pair<typename Arc::Label, const fst::Fst<Arc>* > > label_fst_pairs;
                label_fst_pairs.push_back(std::make_pair(root, new fst::VectorFst<Arc>(raw_fst)));
                label_fst_pairs.push_back(std::make_pair(label, &fst_1));
                label_fst_pairs.push_back(std::make_pair(label_5, &fst_2));
                do_compute_fst_replace<MyFst>(label_fst_pairs, root, epsilon_on_replace, j, dir_path);
            }
        }
    }

    if (labels.size() == 0) {
        vector<int> p;
        j["replace"] = p;
    }
}

template<class F>
void compute_fst_union(const F& raw_fst, json& j, const fst::VectorFst<typename F::Arc>& fst_2, const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;
    j["union"] = {};

    auto fst_1 = new fst::VectorFst<Arc>(raw_fst);

    auto res_lazy = fst::VectorFst<Arc>(fst::UnionFst<Arc>(*fst_1, fst_2));

    fst::Union(fst_1, fst_2);

    json j2;
    j2["fst_2_path"] = dump_fst(fst_2, dir_path);
    j2["result_static_path"] = dump_fst(*fst_1, dir_path);
    j2["result_lazy_path"] = dump_fst(res_lazy, dir_path);

    j["union"].push_back(j2);
}

template<class F>
void compute_fst_concat(const F& raw_fst, json& j, const fst::VectorFst<typename F::Arc>& fst_2, const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;
    j["concat"] = {};

    auto fst_1 = new fst::VectorFst<Arc>(raw_fst);

    auto res_lazy = fst::VectorFst<Arc>(fst::ConcatFst<Arc>(*fst_1, fst_2));

    fst::Concat(fst_1, fst_2);

    json j2;
    j2["fst_2_path"] = dump_fst(fst_2, dir_path);
    j2["result_static_path"] = dump_fst(*fst_1, dir_path);
    j2["result_lazy_path"] = dump_fst(res_lazy, dir_path);

    j["concat"].push_back(j2);
}

template<class F>
void compute_fst_closure_plus(const F& raw_fst, json& j, const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;

    j["closure_plus"] = {};

    auto static_fst = fst::VectorFst<Arc>(raw_fst);
    fst::Closure(&static_fst, fst::CLOSURE_PLUS);

    auto lazy_fst = fst::VectorFst<Arc>(fst::ClosureFst<Arc>(raw_fst, fst::CLOSURE_PLUS));

    j["closure_plus"]["result_static_path"] = dump_fst(static_fst, dir_path);
    j["closure_plus"]["result_lazy_path"] = dump_fst(lazy_fst, dir_path);
}

template<class F>
void compute_fst_closure_star(const F& raw_fst, json& j, const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;

    j["closure_star"] = {};

    auto static_fst = fst::VectorFst<Arc>(raw_fst);
    fst::Closure(&static_fst, fst::CLOSURE_STAR);

    auto lazy_fst = fst::VectorFst<Arc>(fst::ClosureFst<Arc>(raw_fst, fst::CLOSURE_STAR));

    j["closure_star"]["result_static_path"] = dump_fst(static_fst, dir_path);
    j["closure_star"]["result_lazy_path"] = dump_fst(lazy_fst, dir_path);
}

template<class F>
void compute_fst_matcher(const F& raw_fst, json& j) {
    fst::vector<fst::MatchType> match_types = {fst::MATCH_INPUT, fst::MATCH_OUTPUT};
    j["matcher"] = {};

    auto num_states = raw_fst.NumStates();

    if (num_states == 0) {
        j["matcher"] = std::vector<int>();
    }

    std::set<int> labels;
    labels.insert(0);
    for (int state = 0; state < num_states; state++) {
        for (fst::ArcIterator<F> aiter(raw_fst, state); !aiter.Done(); aiter.Next()) {
            const auto &tr = aiter.Value();
            labels.insert(tr.ilabel);
            labels.insert(tr.olabel);
        }
    }

    for (auto match_type: match_types) {
        auto fst_sorted = fst::VectorFst<typename F::Arc>(raw_fst);
        if (match_type == fst::MATCH_INPUT) {
            fst::ArcSort(&fst_sorted, fst::ILabelCompare<typename F::Arc>());
        } else if (match_type == fst::MATCH_OUTPUT) {
            fst::ArcSort(&fst_sorted, fst::OLabelCompare<typename F::Arc>());
        }
        fst::SortedMatcher<F> matcher(fst_sorted, match_type);
        for (int state = 0; state < num_states; state++) {
            matcher.SetState(state);
            for (int label: labels) {
                json j2 = {};
                if (matcher.Find(label)) {
                    for (; !matcher.Done(); matcher.Next()) {
                    auto &tr = matcher.Value();
                        json j3;
                        j3["ilabel"] = tr.ilabel;
                        j3["olabel"] = tr.olabel;
                        j3["weight"] = weight_to_string(tr.weight);
                        j3["nextstate"] = tr.nextstate;
                        j2.push_back(j3);
                    }
                } else {
                    j2 = vector<int>();
                }
                json j1;
                j1["state"] = state;
                j1["label"] = label;
                j1["trs"] = j2;
                j1["match_type"] = match_type;
                j["matcher"].push_back(j1);
            }
        }
    }
}

template<class F, class FILTER>
void do_compute_fst_compose(const F& raw_fst, json& j, const fst::VectorFst<typename F::Arc>& fst_2, bool connect, FILTER filter, string filter_name, const string& dir_path) {
    using Arc = typename F::Arc;

    auto opts = fst::ComposeOptions();
    opts.connect = connect;
    opts.filter_type = filter;

    // static
    fst::VectorFst<Arc> static_fst;
    fst::Compose(raw_fst, fst_2, &static_fst, opts);

    json j2;
    j2["fst_2_path"] = dump_fst(fst_2, dir_path);
    j2["result_path"] = dump_fst(static_fst, dir_path);
    j2["filter_name"] = filter_name;

    j["compose"].push_back(j2);
}

template<class F>
void do_compute_fst_compose_lookahead(const F& raw_fst, json& j, const fst::VectorFst<typename F::Arc>& fst_2, const string& dir_path) {
    using Arc = typename F::Arc;

    using MATCHER1 = fst::LabelLookAheadMatcher<fst::SortedMatcher<fst::Fst<Arc>>, fst::olabel_lookahead_flags>;
    using MATCHER2 = fst::SortedMatcher<fst::Fst<Arc>>;

    using SEQ_FILTER = fst::AltSequenceComposeFilter<MATCHER1, MATCHER2>;
    using LOOK_FILTER = fst::LookAheadComposeFilter<SEQ_FILTER, MATCHER1, MATCHER2, fst::MATCH_OUTPUT>;
    using PUSH_WEIGHTS_FILTER = fst::PushWeightsComposeFilter<LOOK_FILTER, MATCHER1, MATCHER2, fst::MATCH_OUTPUT>;

    using PUSH_LABELS_FILTER = fst::PushLabelsComposeFilter<PUSH_WEIGHTS_FILTER,
                                         MATCHER1,
                                         MATCHER2,
                                         fst::MATCH_OUTPUT>;
    using COMPOSE_FILTER = PUSH_LABELS_FILTER;

    fst::ConstFst<Arc> ifst1(raw_fst);
    fst::VectorFst<Arc> ifst2(fst_2);
    fst::CacheOptions cacheOptions;

    fst::MatcherFst<
      fst::ConstFst<Arc>,
      fst::LabelLookAheadMatcher<fst::SortedMatcher<fst::Fst<Arc>>, fst::olabel_lookahead_flags>,
      fst::olabel_lookahead_fst_type,
      fst::LabelLookAheadRelabeler<Arc>
    > graph1Look(ifst1);

    fst::LabelLookAheadRelabeler<Arc>::Relabel(
    &ifst2, graph1Look, true);

    fst::ArcSort(&ifst2, fst::ILabelCompare<Arc>());

    fst::ComposeFstImplOptions<MATCHER1, MATCHER2, COMPOSE_FILTER> composeOptions(
    cacheOptions);
    composeOptions.matcher1 = graph1Look.InitMatcher(fst::MATCH_OUTPUT);
    composeOptions.matcher2 = new MATCHER2(ifst2, fst::MATCH_INPUT);

    auto lol =  new fst::ComposeFst<Arc>(graph1Look, ifst2, composeOptions);

    auto res_lazy = fst::VectorFst<Arc>(*lol);

    json j2;
    j2["fst_2_path"] = dump_fst(fst_2, dir_path);
    j2["result_path"] = dump_fst(res_lazy, dir_path);
    j2["filter_name"] = "lookahead";

    j["compose"].push_back(j2);
}


template<class F>
void compute_fst_compose(const F& raw_fst, json& j, const fst::VectorFst<typename F::Arc>& fst_2, const string& dir_path) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;
    j["compose"] = {};

    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::AUTO_FILTER, "auto", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::NULL_FILTER, "null", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::TRIVIAL_FILTER, "trivial", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::SEQUENCE_FILTER, "sequence", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::ALT_SEQUENCE_FILTER, "alt_sequence", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::MATCH_FILTER, "match", dir_path);
    do_compute_fst_compose(raw_fst, j, fst_2, false, fst::NO_MATCH_FILTER, "no_match", dir_path);

    do_compute_fst_compose_lookahead(raw_fst, j, fst_2, dir_path);
}

template<class F>
void compute_fst_queue(const F& raw_fst, json& j) {
    using Weight = typename F::Weight;
    using Arc = typename F::Arc;
    using StateId = typename F::Arc::StateId;

    fst::AutoQueue<StateId> queue(raw_fst, NULL, fst::AnyArcFilter<Arc>());

    vector<bool> enqueued(raw_fst.NumStates());
    for (int i = 0; i < raw_fst.NumStates(); i++) {
        enqueued[i] = false;
    }

    json j2 = {};

    if (raw_fst.Start() != fst::kNoStateId) {
        queue.Enqueue(raw_fst.Start());
        enqueued[raw_fst.Start()] = true;

        {
            json j3;
            j3["op_type"] = "enqueue";
            j3["state"] = raw_fst.Start();
            j2.push_back(j3);
        }

        while (!queue.Empty()) {
            auto state = queue.Head();
            queue.Dequeue();

            {
                json j3;
                j3["op_type"] = "dequeue";
                j3["state"] = state;
                j2.push_back(j3);
            }


            for (fst::ArcIterator<F> aiter(raw_fst, state); !aiter.Done(); aiter.Next()) {
              const auto &arc = aiter.Value();
              if (!enqueued[arc.nextstate]) {
                enqueued[arc.nextstate] = true;
                queue.Enqueue(arc.nextstate);

                {
                    json j3;
                    j3["op_type"] = "enqueue";
                    j3["state"] = arc.nextstate;
                    j2.push_back(j3);
                }

              }
            }
        }
    } else {
        j2 = vector<int>();
    }

    j["queue"]["result"] = j2;

}

template<class F>
void compute_fst_data(const F& fst_test_data, const string fst_name) {
    std::cout << "FST :" << fst_name << std::endl;
    json data;
    auto dir_path = fst_name + "/";

    auto raw_fst = fst_test_data.get_fst();

    // Force the computation of all the properties.
    raw_fst.Properties(fst::kFstProperties, true);

    data["name"] = fst_name;
    data["weight_type"] = F::MyArc::Type();
    data["raw"]["result_path"] = "raw_vector.fst";
    data["raw_text"] = fst_to_string(raw_fst);

    data["raw_vector_bin_path"] = "raw_vector.fst";
    raw_fst.Write(dir_path + "raw_vector.fst");

    fst::SymbolTable isymt;
    isymt.AddSymbol("<eps>");
    isymt.AddSymbol("good");
    isymt.AddSymbol("day");

    fst::SymbolTable osymt;
    osymt.AddSymbol("<epsilon>");
    osymt.AddSymbol("knock");
    osymt.AddSymbol("world");
    osymt.AddSymbol("hello");

    fst::VectorFst<typename F::MyArc> fst_with_symt(raw_fst);

    fst_with_symt.SetInputSymbols(&isymt);
    fst_with_symt.SetOutputSymbols(&osymt);

    data["raw_vector_with_symt_bin_path"] = "raw_vector_with_symt.fst";
    fst_with_symt.Write(dir_path + "raw_vector_with_symt.fst");

    fst::FstWriteOptions write_opts("<unspecified>");
    fst::ConstFst<typename F::MyArc> raw_const_fst(raw_fst);
    // Not aligned
    write_opts.align = false;
    data["raw_const_bin_path"] = "raw_const.fst";
    std::ofstream strm((dir_path + "raw_const.fst").c_str(), std::ios_base::out | std::ios_base::binary);
    raw_const_fst.Write(strm, write_opts);

    // Aligned
    write_opts.align = true;
    data["raw_const_aligned_bin_path"] = "raw_const_aligned.fst";
    std::ofstream strm_aligned((dir_path + "raw_const_aligned.fst").c_str(), std::ios_base::out | std::ios_base::binary);
    raw_const_fst.Write(strm_aligned, write_opts);

    std::cout << "Invert" << std::endl;
    compute_fst_invert(raw_fst, data, dir_path);

    std::cout << "Project Input" << std::endl;
    compute_fst_project_input(raw_fst, data, dir_path);

    std::cout << "Project Output" << std::endl;
    compute_fst_project_output(raw_fst, data, dir_path);

    std::cout << "Reverse" << std::endl;
    compute_fst_reverse(raw_fst, data, dir_path);

    std::cout << "Remove epsilon" << std::endl;
    compute_fst_remove_epsilon(raw_fst, data, dir_path);

    std::cout << "Connect" << std::endl;
    compute_fst_connect(raw_fst, data, dir_path);

    std::cout << "Condense" << std::endl;
    compute_fst_condense(raw_fst, data, dir_path);

    std::cout << "Shortest distance" << std::endl;
    compute_fst_shortest_distance(raw_fst, data);

    std::cout << "Weight pushing initial" << std::endl;
    compute_fst_compute_weight_pushing_initial(raw_fst, data, dir_path);

    std::cout << "Weight pushing final" << std::endl;
    compute_fst_compute_weight_pushing_final(raw_fst, data, dir_path);

    std::cout << "ArcMap" << std::endl;
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_identity", fst::IdentityMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_rmweight", fst::RmWeightMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_invert", fst::InvertWeightMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_input_epsilon", fst::InputEpsilonMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_output_epsilon", fst::OutputEpsilonMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map(raw_fst, data, "tr_map_quantize", fst::QuantizeMapper<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_map_plus(raw_fst, data, fst_test_data.get_weight_plus_mapper(), dir_path);
    compute_fst_compute_tr_map_times(raw_fst, data, fst_test_data.get_weight_times_mapper(), dir_path);

    std::cout << "ArcSort" << std::endl;
    compute_fst_compute_tr_sort(raw_fst, data, "tr_sort_ilabel", fst::ILabelCompare<typename F::MyArc>(), dir_path);
    compute_fst_compute_tr_sort(raw_fst, data, "tr_sort_olabel", fst::OLabelCompare<typename F::MyArc>(), dir_path);

    std::cout << "Encode" << std::endl;
    compute_fst_encode(raw_fst, data, dir_path);

    std::cout << "Encode / Decode" << std::endl;
    compute_fst_encode_decode(raw_fst, data, dir_path);

    std::cout << "StateMap" << std::endl;
    compute_fst_state_map(raw_fst, data, "state_map_tr_sum", fst::ArcSumMapper<typename F::MyArc>(raw_fst), dir_path);
    compute_fst_state_map(raw_fst, data, "state_map_tr_unique", fst::ArcUniqueMapper<typename F::MyArc>(raw_fst), dir_path);

    std::cout << "Determinization" << std::endl;
    compute_fst_determinization(raw_fst, data, dir_path);

    std::cout << "TopSort" << std::endl;
    compute_fst_topsort(raw_fst, data, dir_path);

    std::cout << "Properties" << std::endl;
    compute_fst_properties(raw_fst, data);

    std::cout << "Minimization" << std::endl;
    compute_fst_minimization(raw_fst, data, dir_path);

    std::cout << "Gallic Encode Decode" << std::endl;
    compute_fst_gallic_encode_decode(raw_fst, data, dir_path);

    std::cout << "Factor Weight Identity" << std::endl;
    compute_fst_factor_weight_identity(raw_fst, data, dir_path);

    std::cout << "Factor Weight Gallic" << std::endl;
    compute_fst_factor_weight_gallic(raw_fst, data, dir_path);

    std::cout << "Push" << std::endl;
    compute_fst_push(raw_fst, data, dir_path);

   std::cout << "Replace" << std::endl;
   compute_fst_replace(raw_fst, data, fst_test_data, dir_path);

    std::cout << "Union" << std::endl;
    auto fst_union = fst_test_data.get_fst_union();
    fst_union.Properties(fst::kFstProperties, true);
    compute_fst_union(raw_fst, data, fst_union, dir_path);

    std::cout << "Concat" << std::endl;
    auto fst_concat = fst_test_data.get_fst_concat();
    fst_concat.Properties(fst::kFstProperties, true);
    compute_fst_concat(raw_fst, data, fst_concat, dir_path);

    std::cout << "Closure Plus" << std::endl;
    compute_fst_closure_plus(raw_fst, data, dir_path);

    std::cout << "Closure Star" << std::endl;
    compute_fst_closure_star(raw_fst, data, dir_path);

//    std::cout << "Matcher" << std::endl;
//    compute_fst_matcher(raw_fst, data);

    std::cout << "Compose" << std::endl;
    auto fst_compose = fst_test_data.get_fst_compose();
    fst_compose.Properties(fst::kFstProperties, true);
    compute_fst_compose(raw_fst, data, fst_compose, dir_path);

    std::cout << "State Reachable" << std::endl;
    compute_fst_state_reachable(raw_fst, data);

    std::cout << "ShortestPath" << std::endl;
    compute_fst_shortest_path(raw_fst, data, dir_path);

    std::cout << "Queue" << std::endl;
    compute_fst_queue(raw_fst, data);

    std::cout << "Optimize" << std::endl;
    compute_fst_optimize(raw_fst, data, dir_path);

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

template <class W>
void compute_weight_data(const W& w1, const W& w2, const string weight_name) {
    std::cout << "Weight :" << weight_name << std::endl;
    json data;

    data["name"] = weight_name;
    data["weight_type"] = W::Type();
    data["tr_type"] = fst::ArcTpl<W>::Type();
    data["one"] = weight_to_string(W::One());
    data["zero"] = weight_to_string(W::Zero());

    data["weight_1"] = weight_to_string(w1);
    data["weight_2"] = weight_to_string(w2);

    data["plus"] = weight_to_string(Plus(w1, w2));
    data["times"] = weight_to_string(Times(w1, w2));

    std::ofstream o("weights/" + weight_name + ".json");
    o << std::setw(4) << data << std::endl;
}


int main() {
    srand (time(NULL));
    FLAGS_fst_error_fatal = false;

    compute_symt_data(compute_symt_000(), "symt_000");
    compute_symt_data(compute_symt_001(), "symt_001");
    compute_symt_data(compute_symt_002(), "symt_002");

    compute_weight_data(fst::TropicalWeight(1.2), fst::TropicalWeight(3.2), "weight_001");
    compute_weight_data(fst::LogWeight(1.2), fst::LogWeight(3.2), "weight_002");
    compute_weight_data(
        fst::ProductWeight<fst::TropicalWeight, fst::LogWeight>(1.2, 3.2),
        fst::ProductWeight<fst::TropicalWeight, fst::LogWeight>(0.3, 0.1),
        "weight_003"
    );
    compute_weight_data(
        fst::ProductWeight<fst::LogWeight, fst::TropicalWeight>(1.2, 3.2),
        fst::ProductWeight<fst::LogWeight, fst::TropicalWeight>(0.3, 0.1),
        "weight_004"
    );
    compute_weight_data(
        fst::StringWeight<int, fst::STRING_LEFT>(1),
        fst::StringWeight<int, fst::STRING_LEFT>(3),
        "weight_005"
    );
    compute_weight_data(
        fst::StringWeight<int, fst::STRING_RIGHT>(1),
        fst::StringWeight<int, fst::STRING_RIGHT>(3),
        "weight_006"
    );
    compute_weight_data(
        fst::StringWeight<int, fst::STRING_RESTRICT>(1),
        fst::StringWeight<int, fst::STRING_RESTRICT>(1),
        "weight_007"
    );
    {
        using SW = fst::StringWeight<int, fst::STRING_LEFT>;
        using W = fst::TropicalWeight;
        using GW = fst::GallicWeight<int, W, fst::GALLIC_LEFT>;
        auto w1 = GW(SW(1), W(1.2));
        auto w2 = GW(SW(2), W(3.1));
        compute_weight_data(w1, w2, "weight_008");
    }
    {
        using SW = fst::StringWeight<int, fst::STRING_RIGHT>;
        using W = fst::TropicalWeight;
        using GW = fst::GallicWeight<int, W, fst::GALLIC_RIGHT>;
        auto w1 = GW(SW(1), W(1.2));
        auto w2 = GW(SW(2), W(3.1));
        compute_weight_data(w1, w2, "weight_009");
    }
    {
        using SW = fst::StringWeight<int, fst::STRING_RESTRICT>;
        using W = fst::TropicalWeight;
        using GW = fst::GallicWeight<int, W, fst::GALLIC_RESTRICT>;
        auto w1 = GW(SW(1), W(1.2));
        auto w2 = GW(SW(1), W(3.1));
        compute_weight_data(w1, w2, "weight_010");
    }
    {
        using SW = fst::StringWeight<int, fst::STRING_RESTRICT>;
        using W = fst::TropicalWeight;
        using GW = fst::GallicWeight<int, W, fst::GALLIC_MIN>;
        auto w1 = GW(SW(1), W(1.2));
        auto w2 = GW(SW(2), W(3.1));
        compute_weight_data(w1, w2, "weight_011");
    }
    {
        using SW = fst::StringWeight<int, fst::STRING_RESTRICT>;
        using W = fst::TropicalWeight;
        using GW = fst::GallicWeight<int, W, fst::GALLIC>;
        auto w1 = GW(SW(1), W(1.2));
        auto w2 = GW(SW(2), W(3.1));
        compute_weight_data(w1, w2, "weight_012");
    }

    compute_fst_data(FstTestData000(), "fst_000");
    compute_fst_data(FstTestData001(), "fst_001");
    compute_fst_data(FstTestData002(), "fst_002");
    compute_fst_data(FstTestData003(), "fst_003");
    compute_fst_data(FstTestData004(), "fst_004");
    compute_fst_data(FstTestData005(), "fst_005");
    compute_fst_data(FstTestData006(), "fst_006");
    compute_fst_data(FstTestData007(), "fst_007");
    compute_fst_data(FstTestData008(), "fst_008");
    compute_fst_data(FstTestData009(), "fst_009");
    compute_fst_data(FstTestData010(), "fst_010");
    compute_fst_data(FstTestData011(), "fst_011");
    compute_fst_data(FstTestData012(), "fst_012");
    compute_fst_data(FstTestData013(), "fst_013");
    compute_fst_data(FstTestData014(), "fst_014");
    compute_fst_data(FstTestData015(), "fst_015");
    compute_fst_data(FstTestData016(), "fst_016");
    compute_fst_data(FstTestData017(), "fst_017");
    compute_fst_data(FstTestData018(), "fst_018");
    compute_fst_data(FstTestData019(), "fst_019");
    compute_fst_data(FstTestData020(), "fst_020");
}
