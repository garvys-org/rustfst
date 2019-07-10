from rustfst_python_bench.utils import check_fst_equals


class ShortestPathAlgorithm:

    def __init__(self, nshortest=1, unique=False):
        self.nshortest = nshortest
        self.unique = unique

    @classmethod
    def openfst_cli(cls):
        return "fstshortestpath"

    @classmethod
    def rustfst_subcommand(cls):
        return "shortestpath"

    def get_openfst_bench_cli(self):
        unique_s = "1" if self.unique else "0"
        return "bench_shortestpath", [str(self.nshortest), unique_s]

    def get_cli_args(self):
        r = []
        if self.unique:
            r.append("--unique")
        r.append(f"--nshortest={self.nshortest}")
        return " ".join(r)

    @classmethod
    def get_parameters(cls):
        return [
            cls(nshortest=1, unique=False),
            cls(nshortest=10, unique=False),
            # cls(nshortest=10, unique=True)
        ]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        # check_fst_equals(path_res_openfst, path_res_rustfst)
        pass


