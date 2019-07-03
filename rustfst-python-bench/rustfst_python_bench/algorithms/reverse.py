from rustfst_python_bench.utils import check_fst_equals


class ReverseAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstreverse"

    @classmethod
    def rustfst_subcommand(cls):
        return "reverse"

    def get_openfst_bench_cli(self):
        return "bench_reverse", []

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
