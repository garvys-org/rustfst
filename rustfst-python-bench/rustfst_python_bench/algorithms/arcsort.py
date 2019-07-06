from rustfst_python_bench.utils import check_property_set


class ArcSortAlgorithm:

    def __init__(self, sort_olabel=False):
        self.sort_olabel = sort_olabel

    @classmethod
    def openfst_cli(cls):
        return "fstarcsort"

    @classmethod
    def rustfst_subcommand(cls):
        return "arcsort"

    def get_openfst_bench_cli(self):
        if self.sort_olabel:
            return "bench_arcsort", ["1"]
        else:
            return "bench_arcsort", ["0"]

    def get_cli_args(self):
        if self.sort_olabel:
            return "--sort_type=olabel"
        else:
            return "--sort_type=ilabel"

    @classmethod
    def get_parameters(cls):
        return [cls(sort_olabel=False), cls(sort_olabel=True)]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        if self.sort_olabel:
            check_property_set(path_res_openfst, "output label sorted")
            check_property_set(path_res_rustfst, "output label sorted")
        else:
            check_property_set(path_res_openfst, "input label sorted")
            check_property_set(path_res_rustfst, "input label sorted")

