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
