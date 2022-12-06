#!/usr/bin/env bash
set -e

cd rustfst-tests-data

if [ ! -f json.hpp ]; then
    wget https://github.com/nlohmann/json/releases/download/v3.6.1/json.hpp
fi

rm **/metadata.json fst_*/*.fst weights/*.json symt_*/symt.bin symt_*/symt.text || true
echo "Compiling..."
g++ -std=c++11 main.cpp -I ../openfst-1.7.2/src/include/ ../openfst-1.7.2/lib/libfst.a
echo "OK"
echo "Running..."
./a.out
echo "OK"
