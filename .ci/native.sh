#!/bin/sh

set -ex

which rustup || curl https://sh.rustup.rs -sSf | sh -s -- -y

. $HOME/.cargo/env

: "${RUST_VERSION:=stable}"
rustup toolchain add $RUST_VERSION
rustup default $RUST_VERSION

rustc --version

if [ `uname` = "Linux" ]
then
  sudo apt-get update
  sudo apt-get install software-properties-common
  sudo add-apt-repository -y ppa:deadsnakes/ppa
  sudo apt-get update
  wget https://github.com/sharkdp/hyperfine/releases/download/v1.6.0/hyperfine_1.6.0_amd64.deb
  sudo dpkg -i hyperfine_1.6.0_amd64.deb
fi

if [ `uname` ]
then
  brew install hyperfine
fi

./build_openfst.sh
./run_openfst.sh

cargo clean -p rustfst
cargo clean -p rustfst-cli

cargo build --manifest-path rustfst/Cargo.toml --features "state-label-u32"
cargo test --manifest-path rustfst/Cargo.toml  --features "state-label-u32"
cargo build --all
cargo test --all
cargo check --benches --all # running benches on travis is useless
cargo doc --all --no-deps

./build_bench.sh
cd rustfst-python

# Check format
uv tool run ruff check . || fail "Format your code by running 'uv tool run ruff check .' " 1

# Run rustfst python binding tests
uv sync --extra dev && uv run pytest -vv -s --cache-clear --disable-warnings ./tests

# Run benches on a small FST to check that the script is working fine.
cd ..
uv sync
uv run rustfst-python-bench/rustfst_python_bench/bench_all.py rustfst-tests-data/fst_003/raw_vector.fst report.md
uv run rustfst-python-bench/rustfst_python_bench/bench_all_detailed.py rustfst-tests-data/fst_003/raw_vector.fst report2.md
