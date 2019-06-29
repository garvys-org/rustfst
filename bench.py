import argparse
import os
import subprocess


class SupportedAlgorithms(object):
    ALGORITHMS = {}

    @classmethod
    def register(cls, algoname, algo):
        cls.ALGORITHMS[algoname] = algo

    @classmethod
    def get_suppported_algorithms(cls):
        return cls.ALGORITHMS.keys()

    @classmethod
    def get(cls, algoname):
        return cls.ALGORITHMS[algoname]


class ConnectAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstconnect"

    @classmethod
    def rustfst_subcommand(cls):
        return "connect"


class InvertAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstinvert"

    @classmethod
    def rustfst_subcommand(cls):
        return "invert"


class ProjectAlgorithm:
    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstproject"

    @classmethod
    def rustfst_subcommand(cls):
        return "project"


SupportedAlgorithms.register("connect", ConnectAlgorithm)
SupportedAlgorithms.register("invert", InvertAlgorithm)
SupportedAlgorithms.register("project", ProjectAlgorithm)


def main():
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

    OPENFST_BINS = './openfst-1.7.2/bin/'
    RUSTFST_CLI = './target/release/rustfst-cli'

    if args.algo_name not in SupportedAlgorithms.get_suppported_algorithms():
        raise RuntimeError(f"Algorithm {args.algo_name} not supported. Supported algorithms {set(SupportedAlgorithms.get_suppported_algorithms())}")
    algo = SupportedAlgorithms.get(args.algo_name)

    openfst_cli = os.path.join(OPENFST_BINS, algo.openfst_cli())

    path_out_openfst = f'{args.algo_name}_openfst.fst'
    path_out_rustfst = f'{args.algo_name}_rustfst.fst'

    cmd_openfst = f"{openfst_cli} {extra_args} {args.path_in_fst} {path_out_openfst}"
    cmd_rustfst = f"{RUSTFST_CLI} {algo.rustfst_subcommand()} {extra_args} {args.path_in_fst} {path_out_rustfst}"

    cmd = f"hyperfine -w {args.warmup} -r {args.runs} '{cmd_openfst}' '{cmd_rustfst}'" \
        f" --export-markdown {args.path_report_md}"
    subprocess.check_call([cmd], shell=True)

    fstequal = os.path.join(OPENFST_BINS, 'fstequal')

    # Check correctness
    subprocess.check_call([f"{fstequal} {path_out_openfst} {path_out_rustfst}"], shell=True)


if __name__ == '__main__':
    main()
