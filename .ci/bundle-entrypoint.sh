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
FST_BENCH_RUSTFST=$CACHEDIR/rustfst/fsts_bench_rustfst_2

(cd $CACHEDIR/rustfst ; [ -d fsts_bench_rustfst_2 ] || tar zxf fsts_bench_rustfst_2.tgz)

touch metrics

fst_bench() {
    fst_name=$1
    op=$2
    shift 2

    $RUSTFST_CLI "$@" --bench --n_warm_ups 2 --n_iters 5 --export-markdown ./export-markdown
    evaltime=$(cut -d ' ' -f 6 < export-markdown)
    echo rustfst.fst_name.$fst_name.op.$op.runtime $evaltime >> metrics
}

fst_bench G_80MB project project $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB invert invert $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB connect connect $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB reverse reverse $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB tr_sort tr_sort $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB tr_unique map --map_type tr_unique $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB tr_sum map --map_type tr_sum $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB map_input_epsilon map --map_type input_epsilon $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output
fst_bench G_80MB map_rmweight map --map_type rmweight $FST_BENCH_RUSTFST/G_80MB.fst ./fst_output

fst_bench L_AND_G_35MB compose compose $FST_BENCH_RUSTFST/L.fst $FST_BENCH_RUSTFST/G_35MB.fst ./fst_output --compose_type default
fst_bench L_AND_G_35MB compose_lookahead compose $FST_BENCH_RUSTFST/L.fst $FST_BENCH_RUSTFST/G_35MB.fst ./fst_output --compose_type lookahead

# Super slow so deactivated for now
#fst_bench L_DIS_AND_G compose compose $FST_BENCH_RUSTFST/L_disambig.fst $FST_BENCH_RUSTFST/G.fst ./fst_output --compose_type default
#fst_bench L_DIS_AND_G compose_lookahead compose $FST_BENCH_RUSTFST/L_disambig.fst $FST_BENCH_RUSTFST/G.fst ./fst_output --compose_type lookahead

exit 0
