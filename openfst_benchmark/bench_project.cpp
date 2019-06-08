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
    const string project_output_s = argv[5];
    auto project_type = ProjectType::PROJECT_INPUT;
    if (project_output_s == "1") {
        project_type = ProjectType::PROJECT_OUTPUT;
    }

    cout << "Running benchmark for algorithm project" << endl;
    UNARY_ALGO_BENCH(Project(fst, project_type);)
}