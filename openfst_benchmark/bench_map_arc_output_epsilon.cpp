
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

    cout << "Running benchmark for algorithm map output epsilon" << endl;
    UNARY_ALGO_BENCH({
        OutputEpsilonMapper<StdArc> mapper;
        ArcMap(fst, &mapper);
    })
}