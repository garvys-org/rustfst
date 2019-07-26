from rustfst_python_bench.utils import check_fst_equals


class PushAlgorithm:
    def __init__(self, reweight_to_final=False, push_weights=False, push_labels=False, remove_total_weight=False,
                 remove_common_affix=False):
        self.reweight_to_final = reweight_to_final
        self.push_weights = push_weights
        self.push_labels = push_labels
        self.remove_total_weight = remove_total_weight
        self.remove_common_affix = remove_common_affix

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
        if self.push_labels:
            r.append("--push_labels")
        if self.remove_total_weight:
            r.append("--remove_total_weight")
        if self.remove_common_affix:
            r.append("--remove_common_affix")
        if self.reweight_to_final:
            r.append("--to_final")
        return " ".join(r)

    @classmethod
    def get_parameters(cls):
        return [
            cls(push_weights=True),
            cls(push_weights=True, remove_total_weight=True),
            cls(push_weights=True, push_labels=True),
            cls(push_weights=True, push_labels=True, remove_common_affix=True)
        ]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
