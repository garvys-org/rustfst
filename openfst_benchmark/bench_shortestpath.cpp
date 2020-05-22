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
    const string nshortest_s = argv[6];
    const string unique_s = argv[7];

    int nshortest = stoi(unique_s);

    bool unique = false;
    if (unique_s == "1") {
        unique = true;
    }

    cout << "Running benchmark for algorithm shortestpath" << endl;
    UNARY_ALGO_BENCH({
        auto rfst = new VectorFst<StdArc>();
        ShortestPath(*fst, rfst, nshortest, unique);
        delete fst;
        fst = rfst;
    })
}