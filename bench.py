import argparse
import os
import subprocess


def main():
    parser = argparse.ArgumentParser(
        description="Script to build a HCLG with a new Language Model"
    )

    parser.add_argument(
        "path_in_fst",
        type=str,
        help="Path to input fst",
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

    OPENFST_BINS = './openfst-1.7.2/bin/'
    RUSTFST_CLI = './target/release/rustfst-cli'

    fstconnect = os.path.join(OPENFST_BINS, 'fstconnect')

    path_out_openfst = 'connect_openfst.fst'
    path_out_rustfst = 'connect_rustfst.fst'

    cmd_openfst = f"{fstconnect} {args.path_in_fst} {path_out_openfst}"
    cmd_rustfst = f"{RUSTFST_CLI} connect {args.path_in_fst} {path_out_rustfst}"

    cmd = f"hyperfine -w {args.warmup} -r {args.runs} '{cmd_openfst}' '{cmd_rustfst}'"
    subprocess.check_call([cmd], shell=True)

    fstequal = os.path.join(OPENFST_BINS, 'fstequal')
    subprocess.check_call([f"{fstequal} {path_out_openfst} {path_out_rustfst}"], shell=True)


if __name__ == '__main__':
    main()
