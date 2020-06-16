#!/bin/sh

set -ex

ROOT=`pwd`
CACHEDIR=${CACHEDIR:-$HOME/.cache}
if [ -f rustfst-cli ]
then
    RUSTFST_CLI=./rustfst-cli
elif [ -x ./target/release/rustfst-cli ]
then
    RUSTFST_CLI=./target/release/rustfst-cli
else
    RUSTFST_CLI="cargo run -p rustfst-cli -q --release --"
fi

[ -e vars ] && . ./vars

[ -d $CACHEDIR ] || mkdir $CACHEDIR
aws s3 sync s3://tract-ci-builds/model $CACHEDIR
FST_BENCH_RUSTFST=$CACHEDIR/rustfst/fsts_bench_rustfst

(cd $CACHEDIR/rustfst ; [ -d fsts_bench_rustfst ] || tar zxf fsts_bench_rustfst.tgz)

touch metrics

fst_bench() {
    fst_name=$1
    op=$2
    shift 2

    $RUSTFST_CLI "$@" --bench --export-markdown ./export-markdown
    evaltime=$(cut -d ' ' -f 6 < export-markdown)
    echo rustfst.fst_name.$fst_name.op.$op.runtime $evaltime >> metrics
}

fst_bench G_80MB project project $FST_BENCH_RUSTFST/G.fst ./fst_output

exit 0
