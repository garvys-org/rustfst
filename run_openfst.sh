#!/usr/bin/env bash
set -e

cd rustfst-tests-data
rm **/metadata.json || true
echo "Compiling..."
g++ -std=c++11 main.cpp -I ../openfst-1.7.2/src/include/ ../openfst-1.7.2/lib/libfst.a
echo "OK"
echo "Running..."
./a.out
echo "OK"
