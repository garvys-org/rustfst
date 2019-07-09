from rustfst_python_bench.utils import check_fst_equals, check_property_set


class ConnectAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstconnect"

    @classmethod
    def rustfst_subcommand(cls):
        return "connect"

    def get_openfst_bench_cli(self):
        return "bench_connect", []

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_property_set(path_res_openfst, "accessible")
        check_property_set(path_res_openfst, "coaccessible")
        check_property_set(path_res_rustfst, "accessible")
        check_property_set(path_res_rustfst, "coaccessible")
        check_fst_equals(path_res_openfst, path_res_rustfst)

