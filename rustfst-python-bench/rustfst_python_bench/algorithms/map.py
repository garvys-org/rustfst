from rustfst_python_bench.utils import check_fst_equals


class MapAlgorithm:

    def __init__(self, map_type="identity"):
        self.map_type = map_type

    @classmethod
    def openfst_cli(cls):
        return "fstmap"

    @classmethod
    def rustfst_subcommand(cls):
        return "map"

    def get_openfst_bench_cli(self):
        if self.map_type == "arc_sum":
            return "bench_map_arc_sum", []
        elif self.map_type == "arc_unique":
            return "bench_map_arc_unique", []
        elif self.map_type == "identity":
            return "bench_map_arc_identity", []
        elif self.map_type == "input_epsilon":
            return "bench_map_arc_input_epsilon", []
        elif self.map_type == "invert":
            return "bench_map_arc_invert_weight", []
        elif self.map_type == "output_epsilon":
            return "bench_map_arc_output_epsilon", []
        elif self.map_type == "rmweight":
            return "bench_map_arc_rmweight", []
        else:
            raise RuntimeError(f"Unknown map_type={self.map_type}")

    def get_cli_args(self):
        return f"--map_type={self.map_type}"

    @classmethod
    def get_parameters(cls):
        map_types = ["arc_sum", "arc_unique", "identity", "input_epsilon", "invert", "output_epsilon", "rmweight"]
        return [cls(map_type=m) for m in map_types]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
