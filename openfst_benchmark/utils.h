#define UNARY_ALGO_BENCH(code) { \
    std::cout.precision(6); \
    float avg_parsing_time = 0.0;\
    float avg_algo_time = 0.0;\
    float avg_serialization_time = 0.0;\
\
    for(int i = 0; i < (n_warm_ups+n_iters); i++) {\
\
        auto parsing_start = high_resolution_clock::now();\
        auto fst = VectorFst<StdArc>::Read(path_in);\
        auto parsing_end = high_resolution_clock::now();\
        auto parsing_duration = std::chrono::duration_cast<std::chrono::microseconds>(parsing_end - parsing_start).count() / 1000000.0;\
\
        auto algo_start = high_resolution_clock::now();\
        code\
        auto algo_end = high_resolution_clock::now();\
        auto algo_duration = std::chrono::duration_cast<std::chrono::microseconds>(algo_end - algo_start).count() / 1000000.0;\
\
        auto serialization_start = high_resolution_clock::now();\
        fst->Write(path_out);\
        auto serialization_end = high_resolution_clock::now();\
        auto serialization_duration = std::chrono::duration_cast<std::chrono::microseconds>(serialization_end - serialization_start).count() / 1000000.0;\
\
        if (i >= n_warm_ups) {\
            cout << "Run #" << i+1-n_warm_ups << "/" << n_iters << ": \t" << parsing_duration << "s\t" << algo_duration << "s\t" << serialization_duration << "s" << endl;\
            avg_parsing_time += parsing_duration;\
            avg_algo_time += algo_duration;\
            avg_serialization_time += serialization_duration;\
        } else {\
            cout << "Warmup #" << i+1 << "/" << n_warm_ups <<  ": \t" << parsing_duration << "s\t" << algo_duration << "s\t" << serialization_duration << "s" << endl;\
        }\
    }\
\
    cout << "Bench results (Warmups = " << n_warm_ups << ", Iterations = " << n_iters << ")" << endl;\
    cout << "\tMean parsing time : \t\t" << avg_parsing_time / n_iters << "s" << endl;\
    cout << "\tMean algorithm time : \t\t" << avg_algo_time / n_iters << "s" << endl;\
    cout << "\tMean serialization time : \t" << avg_serialization_time / n_iters << "s" << endl;\
    cout << "\tMean CLI time : \t\t" << (avg_parsing_time + avg_algo_time + avg_serialization_time) / n_iters << "s" << endl;\
}