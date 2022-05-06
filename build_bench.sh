#!/usr/bin/env bash
set -ex

mode=$1

case $mode in
  "release")
    CPP_FLAGS="-O3"
    RUST_FLAGS="--release"
    ;;

  "debug")
    CPP_FLAGS=""
    RUST_FLAGS=""
    ;;

  *)
    echo "$mode not supported. Expected debug or release" && exit 1
    ;;
esac

cd openfst_benchmark

for file in `ls *.cpp`
do
    echo "Building $file"
    g++ $CPP_FLAGS -std=c++11 $file -I ../openfst-1.7.2/src/include/ ../openfst-1.7.2/lib/libfst.a -o "${file%.*}"
done

cd ..

cargo build $RUST_FLAGS -p rustfst-cli

