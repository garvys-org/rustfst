import argparse
import io
import os
import re
import subprocess
import tempfile

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.constants import OPENFST_BINS, RUSTFST_CLI
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


def bench_algo(algo_name, path_in_fst, results_dir, path_report_md, warmup, runs, algo):

    openfst_cli = os.path.join(OPENFST_BINS, algo.openfst_cli())

    path_out_openfst = os.path.join(results_dir, f'{algo_name}_openfst.fst')
    path_out_rustfst = os.path.join(results_dir, f'{algo_name}_rustfst.fst')

    cmd_openfst = f"{openfst_cli} {algo.get_cli_args()} {path_in_fst} {path_out_openfst}"
    cmd_rustfst = f"{RUSTFST_CLI} {algo.rustfst_subcommand()} {algo.get_cli_args()} {path_in_fst} {path_out_rustfst}"

    cmd = f"hyperfine -w {warmup} -r {runs} '{cmd_openfst}' '{cmd_rustfst}'" \
          f" --export-markdown {path_report_md} --show-output"
    subprocess.check_call([cmd], shell=True)

    algo.check_correctness(path_out_openfst, path_out_rustfst)


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
                        bench_algo(algoname, path_in_fst, tmpdirname, report_path_temp, warmup, runs, param)

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
