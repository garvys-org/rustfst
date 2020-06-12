from rustfst_python_bench.utils import check_fst_equals


class ComposeAlgorithm:

    def __init__(self, compose_type="default"):
        self.compose_type = compose_type

    @classmethod
    def openfst_cli(cls):
        return "fstcompose"

    @classmethod
    def rustfst_subcommand(cls):
        return "compose"

    def get_openfst_bench_cli(self):
        if self.compose_type == "default":
            return "bench_compose", []
        elif self.compose_type == "lookahead":
            return "bench_compose_lookahead", []
        else:
            raise RuntimeError(f"Unknown compose_type={self.compose_type}")

    def get_cli_args(self):
        return f"--compose_type={self.compose_type}"

    @classmethod
    def get_parameters(cls):
        compose_types = ["default", "lookahead"]
        return [cls(compose_type=m) for m in compose_types]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
