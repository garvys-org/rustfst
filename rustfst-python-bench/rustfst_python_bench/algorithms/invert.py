from rustfst_python_bench.utils import check_fst_equals


class InvertAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstinvert"

    @classmethod
    def rustfst_subcommand(cls):
        return "invert"

    def get_openfst_bench_cli(self):
        return "bench_invert", []

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)

