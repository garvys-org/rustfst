#include <string>
#include <iostream>

#include "fst/fstlib.h"
#include "./utils.h"

using namespace std;
using namespace fst;
using std::chrono::high_resolution_clock;

int main(int argc, char **argv) {
    auto n_warm_ups = stoi(argv[1]);
    auto n_iters = stoi(argv[2]);
    const string path_in = argv[3];
    const string path_out = argv[4];
    const string arc_cmp_olabel_s = argv[5];

    cout << "Running benchmark for algorithm arcsort" << endl;

    if (arc_cmp_olabel_s == "1") {
        UNARY_ALGO_BENCH(ArcSort(fst, fst::OLabelCompare<StdArc>());)
    } else {
        UNARY_ALGO_BENCH(ArcSort(fst, fst::ILabelCompare<StdArc>());)
    }
}