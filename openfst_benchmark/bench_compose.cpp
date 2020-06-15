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
    const string path_in_1 = argv[3];
    const string path_in_2 = argv[4];
    const string path_out = argv[5];
    const string path_report_md = argv[6];

    cout << "Running benchmark for algorithm compose" << endl;
    BINARY_ALGO_BENCH(
        Compose(*fst_1, *fst_2, fst_out);
        delete fst_1;
        delete fst_2;
    )
}