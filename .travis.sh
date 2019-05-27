#!/bin/sh

set -ex

./build_openfst.sh
./run_openfst.sh

cargo build --all
cargo test --all
cargo check --benches --all # running benches on travis is useless
cargo doc --all
