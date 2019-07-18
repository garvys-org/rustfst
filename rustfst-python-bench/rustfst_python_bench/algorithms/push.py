from rustfst_python_bench.utils import check_fst_equals


class PushAlgorithm:
    def __init__(self, reweight_to_final=False, push_weights=False, remove_total_weight=False):
        self.reweight_to_final = reweight_to_final
        self.push_weights = push_weights
        self.remove_total_weight = remove_total_weight

    @classmethod
    def openfst_cli(cls):
        return "fstpush"

    @classmethod
    def rustfst_subcommand(cls):
        return "push"

    def get_openfst_bench_cli(self):
        raise NotImplementedError

    def get_cli_args(self):
        r = []
        if self.push_weights:
            r.append("--push_weights")
        if self.remove_total_weight:
            r.append("--remove_total_weight")
        if self.reweight_to_final:
            r.append("--to_final")
        return " ".join(r)

    @classmethod
    def get_parameters(cls):
        return [cls(push_weights=True)]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
