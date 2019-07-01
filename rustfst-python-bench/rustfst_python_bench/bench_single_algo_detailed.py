import argparse
import os
import subprocess
import tempfile

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.constants import OPENFST_BINS, RUSTFST_CLI


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


def bench_algo(algo_name, path_in_fst, results_dir, path_report_md, warmup, runs, extra_args):

    if algo_name not in SupportedAlgorithms.get_suppported_algorithms():
        raise RuntimeError(f"Algorithm {algo_name} not supported."
                           f" Supported algorithms {set(SupportedAlgorithms.get_suppported_algorithms())}")
    algo = SupportedAlgorithms.get(algo_name)

    path_out_rustfst = os.path.join(results_dir, f'{algo_name}_rustfst.fst')

    # cmd_openfst = f"{openfst_cli} {extra_args} {path_in_fst} {path_out_openfst}"
    cmd_rustfst = f"{RUSTFST_CLI} {algo.rustfst_subcommand()} {extra_args} {path_in_fst} {path_out_rustfst} " \
                  f"--bench --export-markdown {path_report_md} --n_iters {runs} --n_warm_ups {warmup}"

    subprocess.check_call([cmd_rustfst], shell=True)


def main():
    with tempfile.TemporaryDirectory() as tmpdirname:
        args, extra_args = parse()
        bench_algo(args.algo_name, args.path_in_fst, tmpdirname, args.path_report_md, args.warmup, args.runs, extra_args)


if __name__ == '__main__':
    main()
