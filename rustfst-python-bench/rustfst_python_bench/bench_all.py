import argparse
import os
import io
import tempfile
import re
import platform
import datetime

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.bench_single_algo import bench_algo
from rustfst_python_bench.utils import header_report


def parse():
    parser = argparse.ArgumentParser(
        description="Script to bench all CLIs of OpenFST and RustFST"
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

    args = parser.parse_args()

    return args


def bench(path_in_fst, path_report_md, warmup, runs):

    with io.open(path_report_md, mode="w") as report_f:
        report_f.write("# Benchmark OpenFST CLI vs RustFST CLI\n")
        header_report(report_f, path_in_fst, warmup, runs)
        with tempfile.TemporaryDirectory() as tmpdirname:
            report_path_temp = os.path.join(tmpdirname, f"report_temp.md")

            for algoname in sorted(SupportedAlgorithms.get_suppported_algorithms()):
                    algo = SupportedAlgorithms.get(algoname)
                    params = algo.get_parameters()
                    report_f.write(f"## {algoname.capitalize()}\n")
                    for param in params:
                        bench_algo(algoname, path_in_fst, tmpdirname, report_path_temp, warmup, runs, param.get_cli_args())

                        with io.open(report_path_temp, mode="r") as f:

                            if len(params) > 1:
                                report_f.write(f"### CLI parameters : ` {param.get_cli_args()}`\n")
                            data = f.read()
                            data = re.sub(r'`\./openfst.*`', f'`{algo.openfst_cli()}`', data)
                            data = re.sub(r'`\./target.*`', f'`rustfst-cli {algo.rustfst_subcommand()}`', data)
                            report_f.write(data)


def main():
    args = parse()
    bench(args.path_in_fst, args.path_report_md, args.warmup, args.runs)


if __name__ == '__main__':
    main()
