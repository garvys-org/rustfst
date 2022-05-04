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
echo $PYTHON_VERSION
$PYTHON_VERSION --version

($PYTHON_VERSION -m pip freeze | grep black 1>/dev/null 2>&1) || $PYTHON_VERSION -m pip install black==21.7b0
$PYTHON_VERSION -m pip install pylint==2.6.0 pytest==6.2.5
$PYTHON_VERSION -m pip install -r rustfst-python/requirements-setup.txt

cd rustfst-python
$PYTHON_VERSION -m setup.py develop

# Check format
$PYTHON_VERSION -m black --check . || fail "Format your code by running black ." 1

# Run linting check
export ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
$PYTHON_VERSION -m pytest -vv -s --cache-clear --disable-warnings "$ROOT_DIR/linting/linting_test.py"

# Run rustfst python binding tests
$PYTHON_VERSION -m pytest -vv -s --cache-clear --disable-warnings ./tests

# Run benches on a small FST to check that the script is working fine.
cd ..
$PYTHON_VERSION -m pip install -e rustfst-python-bench
$PYTHON_VERSION rustfst-python-bench/rustfst_python_bench/bench_all.py rustfst-tests-data/fst_003/raw_vector.fst report.md
$PYTHON_VERSION rustfst-python-bench/rustfst_python_bench/bench_all_detailed.py rustfst-tests-data/fst_003/raw_vector.fst report2.md
