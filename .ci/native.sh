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
  sudo apt-get install python3.6
  wget https://github.com/sharkdp/hyperfine/releases/download/v1.6.0/hyperfine_1.6.0_amd64.deb
  sudo dpkg -i hyperfine_1.6.0_amd64.deb
  sudo apt-get install virtualenv
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
python3 --version

if which python3.6
then
    virtualenv venv3 -p python3.6
elif which python3.7
then
    virtualenv venv3 -p python3.7
else
    echo "No suitable python version found."
    exit 2
fi
. venv3/bin/activate

(pip freeze | grep black 1>/dev/null 2>&1) || pip install black==19.10b0
pip install pylint==2.6.0 pytest==6.2.5
pip install -r rustfst-python/requirements-setup.txt
python rustfst-python/setup.py develop

# Check format
black --check . || fail "Format your code by running black ." 1

# Run linting check
export ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
python -m pytest -vv -s --cache-clear --disable-warnings "$ROOT_DIR/rustfst-python/linting/linting_test.py"

# Run rustfst python binding tests
python -m pytest -vv -s --cache-clear --disable-warnings rustfst-python

# Run benches on a small FST to check that the script is working fine.
python rustfst-python-bench/rustfst_python_bench/bench_all.py rustfst-tests-data/fst_003/raw_vector.fst report.md
python rustfst-python-bench/rustfst_python_bench/bench_all_detailed.py rustfst-tests-data/fst_003/raw_vector.fst report2.md
