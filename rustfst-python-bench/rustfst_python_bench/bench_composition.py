import argparse
import io
import os
import subprocess
import tempfile

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.constants import RUSTFST_CLI, BENCH_OPENFST_BINS
from rustfst_python_bench.utils import header_report


def parse():
    parser = argparse.ArgumentParser(
        description="Script to bench composition between openfst and rustfst"
    )

    parser.add_argument(
        "path_in_fst_1",
        type=str,
        help="Path to the first fst"
    )

    parser.add_argument(
        "path_in_fst_2",
        type=str,
        help="Path to the second fst"
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
        default=2
    )

    parser.add_argument(
        "-r", "--runs",
        type=int,
        help="Number of bench runs",
        default=5
    )

    parser.add_argument(
        "-c", "--compose-type",
        type=str,
        help="Compose type",
        default="default"
    )

    args = parser.parse_args()

    return args


def bench(path_in_fst_1, path_in_fst_2, path_report_md, warmup, runs, compose_type):
    algo_name = "compose"
    algo_class = SupportedAlgorithms.get(algo_name)
    algo = algo_class(compose_type=compose_type)

    with tempfile.TemporaryDirectory() as tmpdirname:

        path_out_rustfst = os.path.join(tmpdirname, f'{algo_name}_rustfst.fst')

        cmd_rustfst = f"{RUSTFST_CLI} {algo.rustfst_subcommand()} {algo.get_cli_args()} {path_in_fst_1} {path_in_fst_2} {path_out_rustfst} " \
                  f"--bench --export-markdown {path_report_md} --n_iters {runs} --n_warm_ups {warmup}"

        subprocess.check_call([cmd_rustfst], shell=True)

        cli_name, xargs_cli = algo.get_openfst_bench_cli()
        path_out_openfst = os.path.join(tmpdirname, f'{algo_name}_openfst.fst')
        xargs_cli = " ".join(xargs_cli)

        path_cli = os.path.join(BENCH_OPENFST_BINS, cli_name)
        cmd_openfst = f"{path_cli} {warmup} {runs} {path_in_fst_1} {path_in_fst_2} {path_out_openfst} {path_report_md} {xargs_cli}"

        subprocess.check_call([cmd_openfst], shell=True)

        algo.check_correctness(path_out_openfst, path_out_rustfst)


def main():
    args = parse()
    bench(args.path_in_fst_1, args.path_in_fst_2, args.path_report_md, args.warmup, args.runs, args.compose_type)


if __name__ == '__main__':
    main()
