#!/usr/bin/env bash
set -e

cd openfst_benchmark

for file in `ls *.cpp`
do
    echo "Building $file"
    g++ -O3 -std=c++11 $file -I ../openfst-1.7.2/src/include/ ../openfst-1.7.2/lib/libfst.a -o "${file%.*}"
done

cd ..

cargo build --release -p rustfst-cli

