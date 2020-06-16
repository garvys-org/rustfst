#!/usr/bin/env bash
set -e

if [[ ! -d openfst-1.7.2 || ! -d openfst-1.7.2/src ]]; then
    wget http://www.openfst.org/twiki/pub/FST/FstDownload/openfst-1.7.2.tar.gz
    tar -zxvf openfst-1.7.2.tar.gz

    # Default sort in c++ is unstable. This is to align with rust.
    perl -pi -e 's/std::sort/std::stable_sort/g' $(find openfst-1.7.2 -name "*.h" -o -name "*.cc")
fi

cd openfst-1.7.2
CXXFLAGS=-O3 CFLAGS=-O3 ./configure --prefix=`pwd` --enable-static
make -j 8
make install
