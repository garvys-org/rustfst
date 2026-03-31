#!/usr/bin/env bash
set -e

if [[ ! -d openfst-1.7.2 || ! -d openfst-1.7.2/src ]]; then
    curl -fL -o openfst-1.7.2.tar.gz \
        -A "Mozilla/5.0" \
        "http://www.openfst.org/twiki/pub/FST/FstDownload/openfst-1.7.2.tar.gz" \
    || curl -fL -o openfst-1.7.2.tar.gz \
        "https://github.com/mjansche/openfst/archive/refs/tags/1.7.2.tar.gz"
    tar -zxvf openfst-1.7.2.tar.gz

    # Default sort in c++ is unstable. This is to align with rust.
    perl -pi -e 's/std::sort/std::stable_sort/g' $(find openfst-1.7.2 -name "*.h" -o -name "*.cc")

    # Fix compilation error: 's_' member was renamed to 'selector_' (unique_ptr)
    perl -pi -e 's/\btable\.s_\b/*table.selector_/g' $(find openfst-1.7.2 -name "bi-table.h")
fi

cd openfst-1.7.2
CXXFLAGS=-O3 CFLAGS=-O3 ./configure --prefix=`pwd` --enable-static
make -j 8
make install
