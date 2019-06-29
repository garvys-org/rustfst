#!/usr/bin/env bash
set -e

if [ ! -d openfst-1.7.2 ]; then
    wget http://www.openfst.org/twiki/pub/FST/FstDownload/openfst-1.7.2.tar.gz
    tar -zxvf openfst-1.7.2.tar.gz
fi

cd openfst-1.7.2
CXXFLAGS=-O3 CFLAGS=-O3 ./configure --prefix=`pwd` --enable-static
make
make install