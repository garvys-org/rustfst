#include <string>
#include <iostream>

#include "fst/fstlib.h"
#include "./utils.h"

using namespace std;
using namespace fst;
using std::chrono::high_resolution_clock;

using MATCHER1 = fst::LabelLookAheadMatcher<fst::SortedMatcher<fst::Fst<StdArc>>, fst::olabel_lookahead_flags>;
using MATCHER2 = fst::SortedMatcher<fst::Fst<StdArc>>;

using SEQ_FILTER = fst::AltSequenceComposeFilter<MATCHER1, MATCHER2>;
using LOOK_FILTER = fst::LookAheadComposeFilter<SEQ_FILTER, MATCHER1, MATCHER2, fst::MATCH_OUTPUT>;
using PUSH_WEIGHTS_FILTER = fst::PushWeightsComposeFilter<LOOK_FILTER, MATCHER1, MATCHER2, fst::MATCH_OUTPUT>;

using PUSH_LABELS_FILTER = fst::PushLabelsComposeFilter<PUSH_WEIGHTS_FILTER,
                                     MATCHER1,
                                     MATCHER2,
                                     fst::MATCH_OUTPUT>;
using COMPOSE_FILTER = PUSH_LABELS_FILTER;
using MATCHER = fst::MatcherFst<
                          fst::ConstFst<StdArc>,
                          fst::LabelLookAheadMatcher<fst::SortedMatcher<fst::Fst<StdArc>>,fst::olabel_lookahead_flags>,
                          fst::olabel_lookahead_fst_type,
                          fst::LabelLookAheadRelabeler<StdArc>
                        >;
using COMPOSE_OPTIONS = fst::ComposeFstImplOptions<MATCHER1, MATCHER2, COMPOSE_FILTER>;

int main(int argc, char **argv) {
    auto n_warm_ups = stoi(argv[1]);
    auto n_iters = stoi(argv[2]);
    const string path_in_1 = argv[3];
    const string path_in_2 = argv[4];
    const string path_out = argv[5];
    const string path_report_md = argv[6];

    cout << "Running benchmark for algorithm compose lookahead" << endl;
    BINARY_ALGO_BENCH(
        {
        fst::CacheOptions cacheOptions;

        auto casting_start = high_resolution_clock::now();
        fst::ConstFst<StdArc> ifst1(*fst_1);
        auto casting_end = high_resolution_clock::now();
        auto casting_duration = std::chrono::duration_cast<std::chrono::microseconds>(casting_end - casting_start).count() / 1000000.0;
        cout << "Caating duration " << casting_duration << "s" << endl;

        MATCHER graph1Look(ifst1);

        fst::LabelLookAheadRelabeler<StdArc>::Relabel(
        fst_2, graph1Look, true);

        fst::ArcSort(fst_2, fst::ILabelCompare<StdArc>());

        COMPOSE_OPTIONS composeOptions(
        cacheOptions);
        composeOptions.matcher1 = graph1Look.InitMatcher(fst::MATCH_OUTPUT);
        composeOptions.matcher2 = new MATCHER2(*fst_2, fst::MATCH_INPUT);

        auto lol =  new fst::ComposeFst<StdArc>(graph1Look, *fst_2, composeOptions);

        *fst_out = *lol;
        }

    )
}