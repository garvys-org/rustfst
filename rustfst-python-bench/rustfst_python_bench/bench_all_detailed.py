import argparse
import io
import os
import subprocess
import tempfile

from rustfst_python_bench.algorithms.supported_algorithms import SupportedAlgorithms
from rustfst_python_bench.constants import get_rusftfst_cli_dir, BENCH_OPENFST_BINS
from rustfst_python_bench.utils import header_report


def parse():
    parser = argparse.ArgumentParser(
        description="Script to bench all CLIs of OpenFST and RustFST"
    )

    parser.add_argument(
        "compilation_mode",
        type=str,
        choices=["debug", "release"]
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


def bench_algo(algo_name, path_in_fst, results_dir, path_report_md, warmup, runs, algo, compilation_mode):

    path_out_rustfst = os.path.join(results_dir, f'{algo_name}_rustfst.fst')

    rustfst_cli = get_rusftfst_cli_dir(compilation_mode)

    cmd_rustfst = f"{rustfst_cli} {algo.rustfst_subcommand()} {algo.get_rustfst_cli_args()} {path_in_fst} {path_out_rustfst} " \
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

    algo.check_correctness(path_out_openfst, path_out_rustfst)


def bench(compilation_mode: str, path_in_fst: str, path_report_md: str, warmup, runs):

    with io.open(path_report_md, mode="w") as report_f:
        report_f.write("# Benchmark OpenFST C++ functions vs RustFST Rust functions\n")
        header_report(report_f, path_in_fst, warmup, runs)
        with tempfile.TemporaryDirectory() as tmpdirname:
            report_path_temp = os.path.join(tmpdirname, f"report_temp.md")

            for algoname in sorted(SupportedAlgorithms.get_suppported_algorithms()):
                algo_class = SupportedAlgorithms.get(algoname)
                params = algo_class.get_parameters()
                report_f.write(f"## {algoname.capitalize()}\n")
                for param in params:
                    bench_algo(algoname, path_in_fst, tmpdirname, report_path_temp, warmup, runs, param, compilation_mode)

                    with io.open(report_path_temp, mode="r") as f:

                        if len(params) > 1:
                            report_f.write(f"### CLI parameters : ` {param.get_cli_args()}`\n")
                        report_f.write('| Command | Parsing [s] | Algo [s] | Serialization [s] | All [s] | \n')
                        report_f.write('|:---|' + '---:|'*4 + '\n')
                        report_f.write(f.read())


def main():
    args = parse()
    bench(
        compilation_mode=args.compilation_mode,
        path_in_fst=args.path_in_fst,
        path_report_md=args.path_report_md,
        warmup=args.warmup,
        runs=args.runs
    )


if __name__ == '__main__':
    main()
