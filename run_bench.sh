#!/usr/bin/env bash

set -e

if [ -d tmp_benchs ]; then
    rm -r tmp_benchs
fi

mkdir tmp_benchs

#echo "==================================================================="
#echo "BENCH INVERT"
#echo "C++ :"
#./openfst_benchmark/bench_invert 3 10 $1 tmp_benchs/invert_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli invert $1 tmp_benchs/invert_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH PROJECT"
#echo "C++ :"
#./openfst_benchmark/bench_project 3 10 $1 tmp_benchs/project_openfst.fst 0
#echo "RUST :"
#./target/release/rustfst-cli project $1 tmp_benchs/project_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH ARCSORT"
#echo "C++ :"
#./openfst_benchmark/bench_arcsort 3 10 $1 tmp_benchs/arcsort_openfst.fst 0
#echo "RUST :"
#./target/release/rustfst-cli arcsort $1 tmp_benchs/arcsort_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH CONNECT"
#echo "C++ :"
#./openfst_benchmark/bench_connect 3 10 $1 tmp_benchs/connect_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli connect $1 tmp_benchs/connect_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH REVERSE"
#echo "C++ :"
#./openfst_benchmark/bench_connect 3 10 $1 tmp_benchs/reverse_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli reverse $1 tmp_benchs/reverse_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"

#echo "==================================================================="
#echo "BENCH MAP - ARC UNIQUE"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_unique 3 10 $1 tmp_benchs/map_arc_unique_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type arc_unique $1 tmp_benchs/map_arc_unique_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH MAP - ARC SUM"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_sum 3 10 $1 tmp_benchs/map_arc_sum_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type arc_sum $1 tmp_benchs/map_arc_sum_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"

#echo "==================================================================="
#echo "BENCH MAP - IDENTITY"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_identity 3 10 $1 tmp_benchs/map_arc_identity_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type identity $1 tmp_benchs/map_arc_identity_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH MAP - INPUT EPSILON"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_input_epsilon 3 10 $1 tmp_benchs/map_arc_input_epsilon_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type input_epsilon $1 tmp_benchs/map_arc_input_epsilon_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"

#echo "==================================================================="
#echo "BENCH MAP - INVERT WEIGHT"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_invert_weight 3 10 $1 tmp_benchs/map_arc_invert_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type invert $1 tmp_benchs/map_arc_invert_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"
#
#echo "==================================================================="
#echo "BENCH MAP - OUTPUT EPSILON"
#echo "C++ :"
#./openfst_benchmark/bench_map_arc_output_epsilon 3 10 $1 tmp_benchs/map_arc_output_epsilon_openfst.fst
#echo "RUST :"
#./target/release/rustfst-cli map --map_type output_epsilon $1 tmp_benchs/map_arc_output_epsilon_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
#echo "==================================================================="
#echo -e "\n\n"

echo "==================================================================="
echo "BENCH MAP - RMWEIGHT"
echo "C++ :"
./openfst_benchmark/bench_map_arc_rmweight 3 10 $1 tmp_benchs/map_arc_rmweight_openfst.fst
echo "RUST :"
./target/release/rustfst-cli map --map_type rmweight $1 tmp_benchs/map_arc_rmweight_rustfst.fst --bench --n_warm_ups 3 --n_iters 10
echo "==================================================================="
echo -e "\n\n"

