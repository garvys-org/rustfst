import argparse
import os
import subprocess
import tempfile
import io

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.constants import OPENFST_BINS, RUSTFST_CLI, BENCH_OPENFST_BINS


def parse():
    parser = argparse.ArgumentParser(
        description="Script to bench CLIs of OpenFST and RustFST"
    )

    parser.add_argument(
        "algo_name",
        type=str,
        help="Name of the algorithm to benchmark"
    )

    parser.add_argument(
        "path_in_fst",
        type=str,
        help="Path to input fst",
    )

    parser.add_argument(
        "path_report_md",
        type=str,
        help="Path to use for the generated Markdown report"
    )

    parser.add_argument(
        "-w", "--warmup",
        type=int,
        help="Number of warmup rounds",
        default=3
    )

    parser.add_argument(
        "-r", "--runs",
        type=int,
        help="Number of bench runs",
        default=10
    )

    args, extra_args = parser.parse_known_args()

    extra_args = " ".join(extra_args)

    return args, extra_args


def bench_algo(algo_name, path_in_fst, results_dir, path_report_md, warmup, runs, algo):

    path_out_rustfst = os.path.join(results_dir, f'{algo_name}_rustfst.fst')

    cmd_rustfst = f"{RUSTFST_CLI} {algo.rustfst_subcommand()} {algo.get_cli_args()} {path_in_fst} {path_out_rustfst} " \
                  f"--bench --export-markdown {path_report_md} --n_iters {runs} --n_warm_ups {warmup}"

    subprocess.check_call([cmd_rustfst], shell=True)

    with io.open(path_report_md, mode="r") as f:
        data_rustfst = "| `rustfst` " + f.read()

    cli_name, xargs_cli = algo.get_openfst_bench_cli()
    path_out_openfst = os.path.join(results_dir, f'{algo_name}_openfst.fst')
    xargs_cli = " ".join(xargs_cli)

    path_cli = os.path.join(BENCH_OPENFST_BINS, cli_name)
    cmd_openfst = f"{path_cli} {warmup} {runs} {path_in_fst} {path_out_openfst} {path_report_md} {xargs_cli}"

    subprocess.check_call([cmd_openfst], shell=True)

    with io.open(path_report_md, mode="r") as f:
        data_openfst = "| `openfst` " + f.read()

    with io.open(path_report_md, mode="w") as f:
        f.write(data_openfst)
        f.write(data_rustfst)


def main():
    with tempfile.TemporaryDirectory() as tmpdirname:
        args, extra_args = parse()
        bench_algo(args.algo_name, args.path_in_fst, tmpdirname, args.path_report_md, args.warmup, args.runs, extra_args)


if __name__ == '__main__':
    main()
