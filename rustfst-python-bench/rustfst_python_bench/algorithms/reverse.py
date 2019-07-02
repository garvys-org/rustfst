class ReverseAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstreverse"

    @classmethod
    def rustfst_subcommand(cls):
        return "reverse"

    def get_openfst_bench_cli(self):
        return "bench_reverse", []

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]
