class MinimizeAlgorithm:

    def __init__(self, allow_nondet=False):
        self.allow_nondet = allow_nondet

    @classmethod
    def openfst_cli(cls):
        return "fstminimize"

    @classmethod
    def rustfst_subcommand(cls):
        return "minimize"

    def get_openfst_bench_cli(self):
        if self.allow_nondet:
            return "bench_minimize", ["1"]
        else:
            return "bench_minimize", ["0"]

    def get_cli_args(self):
        if self.allow_nondet:
            return "--allow_nondet=true"
        else:
            return ""

    @classmethod
    def get_parameters(cls):
        return [cls(allow_nondet=True)]

