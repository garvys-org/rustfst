#!/usr/bin/env bash
set -e

if [ ! -d openfst-1.7.2 ]; then
    wget http://www.openfst.org/twiki/pub/FST/FstDownload/openfst-1.7.2.tar.gz
    tar -zxvf openfst-1.7.2.tar.gz
fi

cd openfst-1.7.2
./configure --prefix=`pwd` --disable-bin --enable-static
make
make install