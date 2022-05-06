OPENFST_BINS = './openfst-1.7.2/bin/'
BENCH_OPENFST_BINS = './openfst_benchmark/'


def get_rusftfst_cli_dir(compilation_mode: str) -> str:
    return f"./target/{compilation_mode}/rustfst-cli"
