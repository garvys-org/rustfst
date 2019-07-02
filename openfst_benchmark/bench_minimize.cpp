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
    const string path_report_md = argv[5];
    const string allow_nondet_s = argv[6];
    bool allow_nondet = false;
    if (allow_nondet_s == "1") {
        allow_nondet = true;
    }

    cout << "Running benchmark for algorithm minimize" << endl;
    UNARY_ALGO_BENCH(Minimize(fst, (VectorFst<StdArc>*)nullptr, kShortestDelta, allow_nondet);)
}