#!/bin/sh

set -ex

./build_openfst.sh
./run_openfst.sh

cargo build --all
cargo test --all
cargo check --benches --all # running benches on travis is useless
cargo doc --all

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz &&
tar xzf master.tar.gz &&
cd kcov-master &&
mkdir build &&
cd build &&
cmake .. &&
make &&
make install DESTDIR=../../kcov-build &&
cd ../.. &&
rm -rf kcov-master &&
for file in target/debug/rustfst-*; do [ -x "${file}" ] || continue; mkdir -p "target/cov/$(basename $file)"; ./kcov-build/usr/local/bin/kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file"; done &&
bash <(curl -s https://codecov.io/bash) &&
echo "Uploaded code coverage"

./build_bench.sh
python3 --version
virtualenv venv3 -p python3.6
. venv3/bin/activate
pip install -e rustfst-python-bench
# Run benches on a small FST to check that the script is working fine.
python rustfst-python-bench/rustfst_python_bench/bench_all.py rustfst-tests-data/fst_003/raw_vector.fst report.md
python rustfst-python-bench/rustfst_python_bench/bench_all_detailed.py rustfst-tests-data/fst_003/raw_vector.fst report2.md

